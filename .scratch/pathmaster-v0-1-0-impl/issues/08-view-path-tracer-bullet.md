# 08 — View PATH tracer bullet: read, list, announce

**Spec:** [spec §5 (Sessions), §10, §10.1 items 1 and 7, §12 (StatusBar field 0)](../../pathmaster-v0-1-0/spec.md) · ADR-0003

**What to build:** The first end-to-end demoable slice: launch PathMaster and NVDA speaks the real machine. Startup reads both Scopes through the registry adapter into their Editing Sessions, the Scope tabs list the real Entries (Status column empty for now), activating a tab announces its entry count through the new `announce()` + Banner mechanism, and a Read-only Data run announces its reason. Session writability is wired: System unelevated and Read-only Data read as non-writable.

**Blocked by:** 01 (shell), 02 (Sessions), 03 (registry adapter), 04 (data mode + elevation), 06 (Announcement text), 07 (startup order and language).

**Status:** ready-for-agent

- [ ] Startup populates two Sessions (User, System) from raw registry reads, Baselines set; Absent decodes to zero Entries; the lists render each Entry's raw text in the Path column
- [ ] `announce(text)` exists and is the only thing that fires accessibility events: set the Banner `StaticText` label, then `NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED, hwnd, OBJID_CLIENT, CHILDID_SELF)`; the Banner is always visible, fixed height, layout never reflows
- [ ] Announcement 1 fires on Scope tab activation: "User PATH: {n} entries" / "System PATH: {n} entries"; the zero case is its own msgid ("User PATH: no entries"); no placeholder rows; Ukrainian strings shipped
- [ ] Announcement 7 fires once at startup in Read-only Data: "Read-only: {reason}" with the three §3 reason texts; StatusBar field 0 names the mode and reason in that state
- [ ] Session writable flags: User writable, System writable only when elevated, neither in Read-only Data; a non-writable Session's state is queryable by later tickets (controls to disable arrive with them)
- [ ] StatusBar field 0 shows the general status (entry counts; issue counts join in ticket 12); the status bar is command-only — absent from the Tab order, answered by `NVDA+End`, no field styling
- [ ] Focus and traversal per the contract: tabs → list → (buttons later); focus never jumps without a reason
- [ ] Startup log line records version, elevation, data state, language
