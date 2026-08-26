# Research: making the borrow discipline structural — types over doc comments

Supporting ticket [14-structural-borrow-discipline](../issues/14-structural-borrow-discipline.md).
Researched 2026-08-26, per the map's standing directive 7 (research before grilling).

The question: the app's invariant — *"no borrow is ever held across a call that can run someone
else's code"* — is currently enforced by a doc comment on `App` in
`crates/pathmaster/src/ui/mod.rs` that enumerates the four kinds of re-entrant call and checks
every call site by hand. Which mechanism could enforce it structurally? Four candidates were
researched against primary sources; a synthesis closes. No decision is made here.

## 0. The hazard, stated from primary sources

Every piece of the hazard is documented behavior, not speculation:

- **`RefCell` panics on conflict, by design.** "Panics if the value is currently mutably
  borrowed" (`borrow`); "Panics if the value is currently borrowed" (`borrow_mut`). The
  non-panicking variants are `try_borrow`/`try_borrow_mut`
  ([RefCell docs](https://doc.rust-lang.org/std/cell/struct.RefCell.html)).
- **`ShowModal` runs a nested event loop.** "Note that this function creates a temporary event
  loop which takes precedence over the application's main event loop … and which is destroyed
  when the dialog is dismissed. This also results in a call to wxApp::ProcessPendingEvents()"
  ([wxDialog::ShowModal doc comment, interface/wx/dialog.h](https://github.com/wxWidgets/wxWidgets/blob/master/interface/wx/dialog.h)).
  "There can be more than one event loop at any given moment, e.g. an event handler called from
  the main loop can show a modal dialog, which starts its own loop resulting in two nested
  loops, with the modal dialog being the active one"
  ([wxEventLoopBase, interface/wx/evtloop.h](https://github.com/wxWidgets/wxWidgets/blob/master/interface/wx/evtloop.h)).
- **Timer events are delivered inside that nested loop.** On MSW, `WM_TIMER` "is posted by the
  GetMessage or PeekMessage function … only when no other higher-priority messages are in the
  thread's message queue" ([WM_TIMER, Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-timer))
  — i.e. *any* loop that pumps the queue dispatches timers, and the modal loop pumps the queue.
  wxWidgets' own bug tracker confirms it from the failure side: a busy timer saturates the modal
  loop — "the wxModalEventLoop never actually exits unless the event queue empties out for a
  moment" ([wxWidgets#11273, "ShowModal doesn't return if system isn't idle"](https://github.com/wxWidgets/wxWidgets/issues/11273);
  same story in the forum thread ["ShowModal() doesn't exit when wxTimer in use"](https://forums.wxwidgets.org/viewtopic.php?t=22212)).
  So the doc comment's premise — the diagnostic Timer's `collect_pass` can run while
  `ask_for_entry` or `question::ask` is open — is toolkit fact, not paranoia.
- **The whole class of API is considered a mistake by a peer toolkit.** GTK 4 deleted
  `gtk_dialog_run()` outright: "Nested main loops present re-entrancy issues and other hard to
  debug issues when coupled with other event sources … that are not under the toolkit or the
  application developer's control. Additionally, 'stop-the-world' functions do not fit the
  event-driven programming model of GTK"
  ([GTK 3→4 migration guide](https://docs.gtk.org/gtk4/migrating-3to4.html)). wxWidgets keeps
  `ShowModal`, so a wx app must live with what GTK removed.

## 1. Candidate: copy state out on every read

Snapshot the working state out of the cell before acting; write back atomically. Real projects
that made this *the* architecture:

- **Druid** built its whole data model on it: app state must implement `Data`, whose contract is
  "cheap to compare and cheap to clone" — expensive collections go behind `Arc` or the `im`
  crate's persistent structures ([druid `Data` docs](https://docs.rs/druid/latest/druid/trait.Data.html)).
  Widgets then receive the data by value each cycle; there is no `Rc<RefCell<AppState>>` for a
  callback to collide with. The cost Druid paid is a trait bound on *every* piece of app state
  and pervasive `Arc`/persistent-collection plumbing.
- **Elm itself** (the reference model) makes state immutable; every update produces a new model.
  The Rust ports (§3) keep the single owner but use `&mut` instead of copies.
- **This repo already does it locally**: `restore` "copies what it needs out of the page before
  it touches one" and `convert_or_keep` "drops the Session before it asks" (the `App` doc
  comment, `crates/pathmaster/src/ui/mod.rs`) — candidate 1 is the current discipline's manual
  escape hatch, generalized.

Costs, observed: clone-per-read is only viable when clones are cheap (Druid's `Data` bound
exists precisely to force that); write-back is not atomic against re-entrancy unless the
write-back itself is a single `borrow_mut` statement — a timer tick between snapshot and
write-back sees (and may overwrite) stale state. Copying narrows the panic window; it does not
close the *staleness* window. That failure mode is the "state tearing" Raph Levien describes for
architectures that mix old and new state in one cycle
([Advice for the next dozen Rust GUIs](https://raphlinus.github.io/rust/gui/2022/07/15/next-dozen-guis.html)).

## 2. Candidate: a borrow-scope wrapper type

Make holding a borrow across dispatch unrepresentable (or at least loud) in the API.

- **Closure-scoped access** — `state.with(|s| …)` / `state.with_mut(|s| …)` where the guard is
  created and dropped inside the wrapper, so no `Ref`/`RefMut` value ever escapes into a scope
  that can call `ShowModal`. This is a *containment* pattern, not a compile-time proof: the
  closure body can still open a dialog, and then the timer tick re-enters `with_mut` and panics
  exactly as before. What the wrapper buys is (a) a single choke point where `try_borrow_mut`
  can turn the panic into a defined fallback, and (b) grep-ability — the invariant becomes "no
  dialog call inside a `with_mut` closure", one rule about one function instead of a list of
  call sites. The std docs implicitly endorse the short-scope style (use `borrow_mut` at the
  point of mutation; keep guards minimal —
  [RefCell docs](https://doc.rust-lang.org/std/cell/struct.RefCell.html)).
- **`try_borrow` discipline** — the documented non-panicking variant
  ([RefCell docs](https://doc.rust-lang.org/std/cell/struct.RefCell.html)). Real projects reach
  for it after the panic ships (leptos hit "already borrowed: BorrowMutError" when a panic
  inside a `set_interval` callback left the guard alive —
  [leptos#3072](https://github.com/leptos-rs/leptos/issues/3072)). Note what `try_borrow` in a
  *timer tick* means semantically: the tick silently skips a beat when a dialog handler holds
  the state. For this app's Pump (poll for a finished pass every 100 ms —
  `crates/pathmaster/src/pump.rs`) a skipped tick is harmless *only if* the Timer keeps running
  to retry; that interacts with `take()` stopping the Timer when nothing is outstanding.
- **The branded-lifetime family (GhostCell/qcell)** — GhostCell "enables the user to safely
  synchronize access to a collection of data via a single permission", moving the aliasing check
  to compile time via branded lifetimes ([GhostCell paper page, MPI-SWS](http://plv.mpi-sws.org/rustbelt/ghostcell/)).
  Structurally this is the *strongest* form: with the permission token threaded as `&mut` through
  event handlers, holding it across a dispatch that also needs it would be a compile error, not
  a panic. But it presumes a single owner who can lend the token into each callback — and a
  retained-mode C++ toolkit's callbacks are `'static` closures entered by FFI with no way to be
  handed a `&mut` token from a caller. GhostCell fits candidate 3's architecture (one dispatch
  point that owns the token), not the current one-closure-per-widget shape.
- **Precedent that this hazard class gets *linted*, not just documented**: clippy's
  `await_holding_refcell_ref` — "Holding a RefCell ref across an await suspension point risks
  panics from a mutable ref shared while other refs are outstanding"
  ([clippy lint index](https://rust-lang.github.io/rust-clippy/master/index.html#await_holding_refcell_ref)).
  An `await` point and a `ShowModal` call are the same shape of hazard (a suspension where other
  code runs); no off-the-shelf lint exists for the sync-reentrancy version, which is why GUI
  projects reach for architecture instead.

## 3. Candidate: single dispatch point (Elm-style `update(msg)`)

Route every mutation through one function that is the only borrower. What the Rust GUI
ecosystem actually built:

| Framework | Where state lives | Mutation point | Evidence |
|---|---|---|---|
| **iced** | Owned by the runtime | `update(&mut self, message)`; "The runtime is in charge of every part of the loop: initializing the **state**, producing **messages**, executing the **update logic**, and running our **view logic**" | [iced book, "The Runtime"](https://book.iced.rs/the-runtime.html) |
| **relm4** (gtk4-rs) | Owned by the component | `fn update(&mut self, msg: Self::Input, sender: …)` is the exclusive mutation point; Elm-inspired by declaration | [relm4 SimpleComponent docs](https://docs.rs/relm4/latest/relm4/trait.SimpleComponent.html), [relm4 book](https://relm4.org/book/stable/) |
| **Xilem** | Owned by the app; `&mut` threaded to the handler | "calls to the `event` method on the `View` trait also contain a mutable borrow of the app state"; the observer-with-shared-state alternative "requires *shared mutable* access to that state, which is clunky at best in Rust" | [Levien, "Xilem: an architecture for UI in Rust"](https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html) |
| **egui/eframe** | Owned by the `App` impl | the framework calls the app's update/ui method with `&mut self` each frame | [eframe `App` docs](https://docs.rs/eframe/latest/eframe/trait.App.html) |
| **slint** | In the component's properties | callbacks capture `Weak` handles and upgrade per-invocation; "a strong reference should not be captured by the closures given to a callback" | [slint `Weak` docs](https://docs.slint.dev/latest/docs/rust/slint/struct.Weak) |

The load-bearing observation: **every one of these frameworks controls its own event loop**, so
"only the dispatcher borrows" is enforceable because nothing re-enters mid-`update`. How they
keep that true for exactly the two things that break it here:

- **Modal dialogs**: GTK4 (relm4's substrate) removed the blocking `run()`; dialogs are shown
  non-modally-in-control-flow and answer via a `response` signal handler
  ([GTK migration guide](https://docs.gtk.org/gtk4/migrating-3to4.html)). iced has no nested
  loop at all — a "modal" is just a view state rendered by the same `view()`. Nobody in this
  family lets a nested loop run while `update` holds the state, because nobody has a nested loop.
- **Timers**: a timer is a message source (iced subscriptions, relm4 commands, glib timeouts
  posting to the main loop — [gtk4-rs book, main event loop](https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html)).
  The tick becomes a `Msg::Tick` handled by the same single-borrower `update`, serialized with
  every other message by construction.

Applied to wxdragon this pattern is implementable *except* at `ShowModal`: an Elm-style
`dispatch(msg)` that does `let mut s = state.borrow_mut()` is exactly the single borrower —
until a message handler calls `ShowModal` and the Timer tick tries to `dispatch` inside the
nested loop. So candidate 3 on wx **requires** either (a) dialogs leave the dispatcher first
(collect inputs → drop borrow → `ShowModal` → feed the answer back in as a new message — the
GTK4 shape, done by hand), or (b) candidate 4 to make re-entry impossible. Levien also notes
the general cost of threading one mutable context everywhere: "not especially ergonomic … but
perhaps more seriously effectively enforces the app logic running on a single thread"
([Advice for the next dozen Rust GUIs](https://raphlinus.github.io/rust/gui/2022/07/15/next-dozen-guis.html))
— no cost here, the app is single-threaded by design.

## 4. Candidate: make the timer inert while a modal is open

Narrow the hazard instead of the borrows.

- **The fact base** (§0): timer events *are* delivered in `ShowModal`'s loop on MSW; this is
  queue mechanics ([WM_TIMER](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-timer)),
  and wx's tracker shows it biting ([wxWidgets#11273](https://github.com/wxWidgets/wxWidgets/issues/11273)).
- **Loop-identity gating is supported API**: `wxEventLoopBase::IsMain()` "Returns true if this
  is the main loop executed by wxApp::OnRun()", and `GetActive()` returns the currently running
  loop ([interface/wx/evtloop.h](https://github.com/wxWidgets/wxWidgets/blob/master/interface/wx/evtloop.h);
  mirrored at [wxPython wx.EventLoopBase](https://docs.wxpython.org/wx.EventLoopBase.html)).
  A tick handler beginning `if !EventLoop::get_active().is_main() { return; }` is the
  toolkit-blessed way to detect "I am running inside somebody's modal loop". (Whether wxdragon
  exposes `wxEventLoopBase` needs checking; if not, a hand-rolled depth counter around every
  `ShowModal`/message-box call is the same gate with the same coverage problem as the doc
  comment — every dialog call site must increment it.)
- **wx ships a reentrancy-flag helper**, which is precedent for the flag pattern being the
  sanctioned answer in wx-land: "wxRecursionGuard is a very simple class which can be used to
  prevent reentrancy problems in a function", flag reset guaranteed by the destructor
  ([interface/wx/recguard.h](https://github.com/wxWidgets/wxWidgets/blob/master/interface/wx/recguard.h)).
  The Rust equivalent is a `Cell<bool>`/depth counter plus a drop guard.
- **The sibling mechanism in wx itself**: `YieldFor(eventsToProcess)` exists because wx already
  concedes that pumping *all* events at a re-entrant point is dangerous — it "allows the caller
  to specify a mask of the wxEventCategory values which indicates which events should be
  processed and which should instead be 'delayed' (i.e. processed by the main loop later)"
  ([interface/wx/evtloop.h](https://github.com/wxWidgets/wxWidgets/blob/master/interface/wx/evtloop.h)).
  `ShowModal` offers no such mask, so the filtering has to live in the handler.
- **Community precedent** is stop-the-timer-around-the-dialog or one-shot timers restarted after
  each render (the workaround discussed for the saturation bug —
  [forum t=22212](https://forums.wxwidgets.org/viewtopic.php?t=22212),
  [wxWidgets#11273](https://github.com/wxWidgets/wxWidgets/issues/11273)). For this app the
  stop/restart variant has a wrinkle: `Pump::request` deliberately restarts the Timer
  unconditionally to be self-healing (`crates/pathmaster/src/pump.rs`), and a pass can land
  *while* a dialog is open; gating the *handler* (skip the tick, let the Timer keep firing)
  preserves the self-healing property, stopping the *Timer* does not.
- **The honest limit of candidate 4**: it neutralizes only the Timer. The doc comment's other
  re-entrancy sources — `render` firing list events synchronously, `BackupsPage::show` under a
  live `on_item_focused`, `Notebook::set_selection` running the page-changed handler
  synchronously (`crates/pathmaster/src/ui/mod.rs`) — are same-loop synchronous callbacks a
  modal gate never sees. Candidate 4 shrinks the doc comment; it cannot delete it.

## 5. Panics in the wild, and what those projects did

The strongest evidence for which discipline survives contact with reality:

- **tao (Tauri's windowing layer)**: opening a native file dialog on Linux panicked with
  "already borrowed: BorrowMutError" in `event_loop.rs` — `gtk_native_dialog_run` pumped the
  GTK loop while tao's event-loop `RefCell` was mutably borrowed
  ([tauri-apps/tao#60](https://github.com/tauri-apps/tao/issues/60)). Fix direction: stop
  holding the borrow across the dialog / avoid the nested-loop-under-borrow path — the same
  restructuring this ticket is about, performed post-crash.
- **gtk3-rs `Dialog::run()`**: the documented blocking API ("enters a recursive main loop" —
  [gtk3-rs Dialog docs](https://gtk-rs.org/gtk3-rs/git/docs/gtk/struct.Dialog.html)) was a
  recurring borrow-panic source in app code; GTK4 removed the API rather than teach everyone
  the discipline ([migration guide](https://docs.gtk.org/gtk4/migrating-3to4.html)). That is a
  toolkit vendor concluding the hand-checked discipline does not scale.
- **leptos**: a panic inside a timer callback left a `RefCell` borrowed, so the *next* tick hit
  "already borrowed" ([leptos#3072](https://github.com/leptos-rs/leptos/issues/3072)) — a
  reminder that guards held at unwind time make one panic cascade into a persistent one; this
  app aborts on panic in release via its panic hook, but debug runs unwind.
- **async Rust** got the mechanical check (clippy `await_holding_refcell_ref`,
  [lint index](https://rust-lang.github.io/rust-clippy/master/index.html#await_holding_refcell_ref));
  sync GUI re-entrancy has no equivalent lint, which is why the frameworks in §3 solved it by
  making the hazard unrepresentable instead of detectable.

## 6. What the ecosystem converged on

Reading §1–§5 together, three convergences worth naming:

1. **Every post-2020 Rust GUI framework converged on a single owner with one mutation point**
   (iced, relm4, Xilem, egui — §3), and the one toolkit family that had a blocking modal API in
   this world (GTK) deleted it, naming nested-loop re-entrancy as the reason. Nobody's answer
   was "document which call sites are dangerous"; nobody's answer was GhostCell either — the
   branded-token machinery appears in data-structure papers, not in any shipping GUI framework
   found here.
2. **The candidates are not rivals; the shipping systems compose 1+3, and a wx host needs 4.**
   Elm-style dispatch (3) is the skeleton; cheap-clone/copy-out (1) is how data crosses the
   dispatch boundary (Druid's `Data`, messages carrying values not references); and on a
   toolkit that still has `ShowModal`, a loop gate (4) — or the GTK4 move of pushing every
   dialog *outside* the dispatcher and feeding its answer back as a message — is what keeps the
   "single borrower" claim true. A closure-scoped wrapper (2) is the cheapest step from the
   current code toward that: it collapses "a list of checked call sites" into "one function
   whose body must not dispatch", which is a rule a reviewer — or a grep in CI — can check
   structurally.
3. **The timer facts are settled and local.** wxTimer events reach handlers inside
   `ShowModal`'s loop on MSW (queue mechanics + wx's own tracker, §0); `IsMain()` is the
   supported way to detect it; wx itself ships `wxRecursionGuard` for exactly this flag
   pattern; and gating the tick *handler* preserves `Pump`'s self-healing restart where
   stopping the Timer around each dialog would not. Whatever the structural mechanism chosen,
   this narrow fact means the Timer path can be closed independently and first.

Open questions for the discussion this file feeds: does wxdragon expose
`wxEventLoopBase::GetActive()`/`IsMain()`; whether a dispatcher refactor (3) is proportionate
for an app this size versus wrapper-plus-gate (2+4); and what a skipped tick under a modal
should do once the dialog closes (the current `ProcessPendingEvents` on modal exit — §0 — may
already cover it).
