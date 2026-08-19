# 17 — Elevation: Restart as Administrator

**Spec:** [spec §9 (FR-uac-elevation)](../../pathmaster-v0-1-0/spec.md) · ADR-0005

**What to build:** The user edits the System PATH by relaunching the whole app elevated: one menu command, a guarded discard of unsaved work, the UAC prompt, and the elevated instance opening on the tab they left — with a graceful dialog when they decline. Elevation is never a write helper.

**Blocked by:** 08 (detection already wired; UI shell), 15 (the command runs through the close-confirm flow).

**Status:** ready-for-agent

- [ ] One entry point: Tools → "Restart as Administrator", disabled when already elevated
- [ ] When anything is dirty the command runs through the close-confirm flow with the dedicated title "Discard unsaved User changes and restart as administrator?", buttons [Discard and Restart] [Cancel]
- [ ] Relaunch via `ShellExecuteEx("runas", <current exe>, "--tab <active>")`; on success the original instance exits; the elevated instance honours `--tab` and opens on that tab
- [ ] On `ERROR_CANCELLED` (1223): dialog titled "Elevation was cancelled — still running without administrator rights", [OK], focus returns to where it was; the app keeps running
- [ ] The elevated window title is "Administrator: PathMaster"; elevated, the System Session is writable and the command is disabled
- [ ] Unelevated, the System tab names its reason but never grows a second elevation offer; no single-instance lock interferes (two instances are a designed state)
- [ ] All strings in the Catalogue with Ukrainian translations
