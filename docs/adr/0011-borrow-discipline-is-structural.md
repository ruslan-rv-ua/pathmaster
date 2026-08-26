# The borrow discipline is structural: scoped access and one modal door

Since diagnostics arrived, the UI's safety invariant — *no borrow is ever held across a call that
can run someone else's code* — has been enforced by a doc comment on `App` listing every call site
checked by hand. The hazard it guards is toolkit fact, not caution: `ShowModal` runs a nested event
loop, `WM_TIMER` is dispatched by whatever loop pumps the queue, so the diagnostic Timer's
`collect_pass` runs *inside* an open dialog and takes the Sessions' and findings' borrows there.
v0.1.0 could carry the comment: four kinds of re-entrant call, roughly ten dialog sites. v0.2.0
adds two modal dialogs (Tree View, Fix Issues), a list rebuilt after every Working-Copy change,
seven Filter handlers, clipboard writes and a debounce timer — the checked list roughly doubles,
which is precisely the "revisit if the call-site list outgrows a comment" trigger the 2026-08-20
review set. The wider ecosystem's verdict is one-sided: every post-2020 Rust GUI framework routes
all mutation through a single owner, and GTK 4 deleted its blocking dialog API naming nested-loop
re-entrancy as the reason (evidence:
`.scratch/pathmaster-v0-2-0/research/14-borrow-discipline-best-practices.md`).

**Two mechanisms replace the comment.** First, **scoped access**: every state cell that more than
one kind of call reaches — a command, the Timer tick, or a synchronous handler the toolkit calls
back (today: both Sessions, both `findings`, the Backups page's file cell) — hides behind a wrapper
whose only access is `with(|s| …)` / `with_mut(|s| …)`. The guard is created and dropped inside the
wrapper, so no `Ref` can escape into a scope that opens a dialog; the invariant collapses from a
list of call sites to one reviewable rule — *a closure body must not dispatch* (no dialog, no
`set_selection`, no page rebuild inside it). Cells only one path touches (`last_read`, `settings`,
the `Pump`'s own worker cell behind its interface) stay plain `RefCell`s with local borrows; the
classification is the rule above, not a roster, so a future cell sorts itself. Second, **one modal
door**: a small module owns a modal-depth `Cell` and exposes the single function through which
every `show_modal` and message box in the application is called — increment, run, decrement by
`Drop` guard, so a panic inside a dialog cannot leave the door jammed shut. The Timer's tick
handler returns immediately while the depth is non-zero.

**The gate is on the handler, not the Timer.** The Timer keeps firing every 100 ms under an open
dialog and its ticks are simply skipped; a pass that lands mid-dialog is collected by the first
tick after the dialog closes. Stopping the Timer around each dialog was rejected because
`Pump::request` deliberately restarts unconditionally to be self-healing, and a stop/restart pair
at every door would re-create per-site discipline. Nothing needs to happen "on dialog close".

**Both rules are enforced by the build.** A source-scan `#[test]` — same genre as the User Guide's
heading-parity gate — fails if `show_modal` appears anywhere in the binary crate outside the door
module. The wrapper enforces its half at the type level: with no guard escaping, holding a borrow
across a dispatch is unrepresentable, and what remains ("does this closure body dispatch?") is a
one-question review of a short closure, not an audit of guard lifetimes.

**The alternatives were real and are rejected here.** A full Elm-style dispatcher — the ecosystem's
own shape, and `run(command)` is already half of one — was rejected as disproportionate: it
rewrites every handler rather than every access, and on a toolkit that keeps `ShowModal` it still
needs this ADR's door (or hand-built GTK4-style dialog extraction) to make "only the dispatcher
borrows" true. It remains the natural next step if this mechanism ever strains, and nothing here
obstructs it. Copy-state-out-on-every-read (Druid's architecture) closes the panic window but not
the staleness window — a pass landing between snapshot and write-back sees stale state — and
changes how every command is written, the exact cost the review flagged. GhostCell-style branded
tokens require a caller who lends `&mut` into each callback; wxdragon's handlers are `'static`
closures entered by FFI, so there is no such caller short of the dispatcher refactor just declined.

## Consequences

- **The retrofit is total and lands first.** All ~47 existing borrow sites move to scoped access
  and every dialog call moves through the door *before* any v0.2.0 surface is implemented; the
  new features are specified and written in the mechanism's terms from day one. Two coexisting
  regimes would leave the tick-vs-borrow collision possible for the whole transition, which is
  the state this ADR exists to end.
- **The doc comment on `App` is deleted**, replaced by a short pointer to the wrapper, the door,
  and this ADR. Its four-kind enumeration of re-entrant calls stops being load-bearing.
- **New dialogs cost zero discipline.** Tree View and Fix Issues open through the door like every
  other modal; nothing about them is hand-checked, and the scanner fails the build if one bypasses
  it.
- **Two timers may never share an owner.** wxdragon's `on_tick` binds `EventType::TIMER` on the
  timer's *owner* with no id filter, so co-owned timers fire each other's handlers (verified in
  0.9.18 sources). The Pump's Timer owns the Frame; the Search debounce timer (tickets 04/06)
  must be owned by a different widget — its own page's control, not the Frame.
- **A skipped tick is invisible by design**: the dialog has the screen, the pass's results write a
  Status column the user cannot see change mid-dialog, and the ≤100 ms catch-up after close is
  below anything NVDA would narrate.
- **No new domain terms.** "Door", "depth" and the wrapper are architecture vocabulary; they live
  here and in code, not in `CONTEXT.md`.
