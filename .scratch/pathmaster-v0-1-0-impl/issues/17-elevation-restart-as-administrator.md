# 17 — Elevation: Restart as Administrator

**Spec:** [spec §9 (FR-uac-elevation)](../../pathmaster-v0-1-0/spec.md) · ADR-0005

**What to build:** The user edits the System PATH by relaunching the whole app elevated: one menu command, a guarded discard of unsaved work, the UAC prompt, and the elevated instance opening on the tab they left — with a graceful dialog when they decline. Elevation is never a write helper.

**Blocked by:** 08 (detection already wired; UI shell), 15 (the command runs through the close-confirm flow).

**Status:** resolved

- [x] One entry point: Tools → "Restart as Administrator", disabled when already elevated
- [x] When anything is dirty the command runs through the close-confirm flow with the dedicated title "Discard unsaved User changes and restart as administrator?", buttons [Discard and Restart] [Cancel]
- [x] Relaunch via `ShellExecuteEx("runas", <current exe>, "--tab <active>")`; on success the original instance exits; the elevated instance honours `--tab` and opens on that tab
- [x] On `ERROR_CANCELLED` (1223): dialog titled "Elevation was cancelled — still running without administrator rights", [OK], focus returns to where it was; the app keeps running
- [x] The elevated window title is "Administrator: PathMaster"; elevated, the System Session is writable and the command is disabled
- [x] Unelevated, the System tab names its reason but never grows a second elevation offer; no single-instance lock interferes (two instances are a designed state)
- [x] All strings in the Catalogue with Ukrainian translations

### Heard, not only seen

The steps this ticket added were run on real NVDA by the user on 2026-08-24 and reported as
passing: **C8–C10** — the dedicated dialog whose title names the User changes about to be lost and
whose two buttons are its two outcomes, [Cancel] holding the default, the focus and Escape; the
relaunch itself, the original instance leaving through the ordinary close path and the elevated one
arriving on the tab it was left on, titled "Administrator: PathMaster"; and the Tools menu of that
elevated instance reading its own restart item as unavailable.

**This ticket is also what makes section C reachable at all.** C1–C7 have existed since ticket 12
and describe an elevated instance the application had no way to produce — until now they could only
be reached by starting the executable as administrator by hand, which is not the path any user
takes. They were run through the command this ticket built, which is the first time the section has
been exercised as written.

The measured half of this ticket stops well short of that. A staged copy under a probe answered for
the argument that crosses the boundary (`--tab user|system|backups` landing on tabs 0, 1 and 2, an
unrecognised value reading as a plain launch), for the Tools menu carrying three items with the new
one enabled, and for the dirty-Session route raising the dedicated dialog whose [Cancel] leaves the
application running. **Everything past the UAC prompt was beyond it**: the prompt is drawn on the
secure desktop, which no synthesised input can reach, and the elevated instance it produces is a
window a medium-integrity probe may not drive. So the exit-on-success, the declined-prompt dialog
and every elevated reading rest on the manual pass alone — which is the arrangement ADR-0005
predicted when it named the elevated instance a separate accessibility surface.
