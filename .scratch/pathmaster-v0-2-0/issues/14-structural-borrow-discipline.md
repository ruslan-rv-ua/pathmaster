# Making the UI's borrow discipline structural

Type: grilling
Status: resolved (2026-08-26)
Blocked by: —

## Question

Deferred by the 2026-08-20 architecture review and pulled into scope at charting because v0.2.0
adds dialogs and UI surfaces (Tree View, Fix Issues, a search field, possibly a filter row) — and
the hazard grows with exactly that: "no borrow is ever held across a call that can run someone
else's code" is enforced today by a doc comment on `App` listing hand-checked call sites, while the
diagnostic Timer ticks inside modal dialogs' own event loops. The project's standing preference is
structural over disciplinary (ADR-0007, v0.1.0 ticket 11). Decide the mechanism **before** the new
surfaces are specified in code:

- Consult `/codebase-design`. Candidates to weigh: copy Working Copy state out on every read (the
  review's known cost: changes how every command is written); a borrow-scope wrapper type that makes
  holding-across-dispatch unrepresentable; confining all Session access to a single dispatch point;
  or keeping the discipline but making the Timer inert while any modal is open (narrows the hazard
  instead of the borrows).
- The decision criterion set at deferral: "revisit if it bites, or if the call-site list outgrows a
  comment." v0.2.0's new dialogs are the second condition approaching — the ticket should count the
  call sites the new features add under each candidate.
- Whatever is chosen earns an ADR if it's hard to reverse, surprising, and a real trade-off — this
  one likely is.
- Out of scope here: collapsing `ScopeDiagnosis` into `Findings` (map's Out of scope; the review's
  other deferral).

## Resolution (2026-08-26)

Researched first: [research/14-borrow-discipline-best-practices.md](../research/14-borrow-discipline-best-practices.md),
per the map's standing directive 7. The hazard is toolkit fact (`ShowModal` nests an event loop;
`WM_TIMER` is dispatched by whatever loop pumps the queue; wxWidgets#11273 shows it biting), GTK 4
deleted its blocking dialog API over exactly this, and every post-2020 Rust GUI framework converged
on a single mutation point. Decisions, each accepted on recommendation:

1. **Mechanism: scoped-access wrapper + modal door** (candidates 2+4 composed), decided in
   **[ADR-0011](../../../docs/adr/0011-borrow-discipline-is-structural.md)** — the full statement
   lives there. A full Elm dispatcher (candidate 3) was rejected as disproportionate — it rewrites
   every handler and on wx still needs the door; copy-out (candidate 1) closes the panic window but
   not the staleness window and rewrites every command; GhostCell cannot receive a branded `&mut`
   through wxdragon's FFI `'static` closures.
2. **Coverage is a rule, not a roster**: behind the wrapper goes any cell reached by more than one
   kind of call (command / Timer tick / synchronous toolkit callback) — today both Sessions, both
   `findings`, the Backups page's file cell; `last_read`, `settings` and the `Pump`'s worker cell
   stay plain `RefCell`s. A future cell classifies itself.
3. **The door**: one module owns a modal-depth `Cell` (Drop-guarded — a dialog panic cannot jam it)
   and the single function every `show_modal`/message box goes through; the tick handler returns
   while depth > 0. The Timer keeps firing — a pass landing mid-dialog is collected ≤100 ms after
   close; `Pump::request`'s self-healing restart is untouched. A source-scan `#[test]` (the
   heading-parity gate's genre) fails the build if `show_modal` appears outside the door module.
4. **Retrofit is total and lands first**: all ~47 existing borrow sites and every dialog call move
   to the mechanism in the first implementation ticket, before any new surface is coded — the
   `App` doc comment is then deleted. No two-regime transition period.
5. **ADR-0011 written now** (this session), not deferred to implementation: all three ADR criteria
   hold (hard to reverse, surprising without context, real trade-off).

**Call-site count the ticket asked for** (new v0.2.0 surfaces under the status quo): two new modal
dialogs (Tree View, Fix Issues), Filtered-View rebuild behind all 8 `after_edit` paths, seven
Filter radio handlers, Copy's clipboard write (OLE can pump), Search's text handler and debounce
tick — ≈25–30 new borrow sites and 3–4 new re-entrancy kinds atop v0.1.0's 47 sites and four
kinds. The deferral criterion ("the call-site list outgrows a comment") is met; under the chosen
mechanism the same additions cost zero hand-checked entries.

**Facts handed to ticket 15 / implementation**: the Search debounce timer must be owned by a
non-Frame widget — wxdragon 0.9.18 binds `on_tick` on the timer's owner with no id filter, so two
timers on one owner fire each other's handlers (verified in sources; pump.rs already warns of
this). No new menu item, no new Announcement, no settings field, no CONTEXT.md term — the
vocabulary is architectural and lives in ADR-0011.
