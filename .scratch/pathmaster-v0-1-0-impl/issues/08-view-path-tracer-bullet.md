# 08 — View PATH tracer bullet: read, list, announce

**Spec:** [spec §5 (Sessions), §10, §10.1 items 1 and 7, §12 (StatusBar field 0)](../../pathmaster-v0-1-0/spec.md) · ADR-0003

**What to build:** The first end-to-end demoable slice: launch PathMaster and NVDA speaks the real machine. Startup reads both Scopes through the registry adapter into their Editing Sessions, the Scope tabs list the real Entries (Status column empty for now), activating a tab announces its entry count through the new `announce()` + Banner mechanism, and a Read-only Data run announces its reason. Session writability is wired: System unelevated and Read-only Data read as non-writable.

**Blocked by:** 01 (shell), 02 (Sessions), 03 (registry adapter), 04 (data mode + elevation), 06 (Announcement text), 07 (startup order and language).

**Status:** resolved

- [x] Startup populates two Sessions (User, System) from raw registry reads, Baselines set; Absent decodes to zero Entries; the lists render each Entry's raw text in the Path column
- [x] `announce(text)` exists and is the only thing that fires accessibility events: set the Banner `StaticText` label, then `NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED, hwnd, OBJID_CLIENT, CHILDID_SELF)`; the Banner is always visible, fixed height, layout never reflows
- [x] Announcement 1 fires on Scope tab activation: "User PATH: {n} entries" / "System PATH: {n} entries"; the zero case is its own msgid ("User PATH: no entries"); no placeholder rows; Ukrainian strings shipped
- [x] Announcement 7 fires once at startup in Read-only Data: "Read-only: {reason}" with the three §3 reason texts; StatusBar field 0 names the mode and reason in that state
- [x] Session writable flags: User writable, System writable only when elevated, neither in Read-only Data; a non-writable Session's state is queryable by later tickets (controls to disable arrive with them)
- [x] StatusBar field 0 shows the general status (entry counts; issue counts join in ticket 12); the status bar is command-only — absent from the Tab order, answered by `NVDA+End`, no field styling
- [x] Focus and traversal per the contract: tabs → list → (buttons later); focus never jumps without a reason
- [x] Startup log line records version, elevation, data state, language

## Comments

Implemented 2026-08-20. The first end-to-end slice: launch reads the real machine, the
lists show it, and everything spoken goes through the one voice.

- **`announce.rs`** (spec §17's module): `Announcer` wraps the Banner's `StaticText`;
  `announce(text)` sets the label, then fires
  `NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED, hwnd, OBJID_CLIENT, CHILDID_SELF)` — the
  ADR-0003 mechanism, and the only accessibility call in the application. `windows-sys`
  joins the bin crate for exactly that call.
- **Sessions**: `main.rs` loads both Scopes through `ScopeKey::{user,system}().read()`,
  decoded into `Session::new` (Baselines set there; Absent → zero Entries, tested in core
  since ticket 02). Writability: User = Writable Data; System = Writable Data ∧ elevated.
  The Sessions ride `Rc<RefCell<…>>` into the UI so tab activation reads live counts.
- **Announcement 1**: `on_page_changed` — the two Scope tabs announce their entry count
  (zero case its own msgid); the Backups tab is not a Scope and announces nothing.
- **Announcement 7 + StatusBar**: four new msgids ("Read-only: {reason}" + the three §3
  reasons), registered, shipped in both `.po` files, gated. `ReadOnlyReason::catalogue_msgid()`
  (platform, tested) names each reason's string beside the enum. Field 0 shows the two
  entry counts joined " | " in Writable Data, or the read-only text in that state; no
  styling, nothing else touches the status bar.
- **A failed startup read is not specced**; taken road: log
  `Record::scope_read_failed(scope, cause)` (WARN, raw OS error / vtype — new core
  constructor, tested) and load an empty **non-writable** Session — nothing may be written
  over a value that was never read. No dialog, no Announcement: the catalogue is closed.
- **Startup log line**: already landed with tickets 05/07; verified live — a real run
  writes `PathMaster 0.1.0, elevated: no, data: writable, language: uk` and no read-failure
  lines on this machine.

NVDA speech itself is Release-Checklist territory (ADR-0007): the mechanism is the
ticket-08 measured one, verbatim from the prototype.
