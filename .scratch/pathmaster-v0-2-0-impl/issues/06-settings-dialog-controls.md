# 06 — Settings-dialog controls for the three view-state fields

**Spec:** [delta-spec §15](../../pathmaster-v0-2-0/spec.md)

**What to build:** The Settings dialog grows three controls for the fields ticket 03 introduced, so the primary user can tune the narrowing behaviour without hand-editing `settings.json`: whether filtered counts speak, the debounce delay, and whether ESC returns focus to the list.

**Blocked by:** 03 (the fields and the behaviour they gate).

**Status:** ready-for-agent

- [ ] Three controls with the assembly labels (amendable at implementation like v0.1.0's dialog strings were): "Speak filtered entry counts" («Озвучувати кількість відфільтрованих записів»), "Delay before speaking the count (ms)" («Затримка перед озвученням кількості (мс)»), "Escape returns focus to the list" («Escape повертає фокус до списку»)
- [ ] The dialog's existing rules extend unchanged: only changed settings are written, domains are one rule read twice (0–5000 for the delay, 0 legal), Read-only Data disables the controls and OK
- [ ] Changing the delay in the dialog demonstrably changes when the count speaks; turning `speakFilteredCount` off demonstrably silences items 9/10/11 without touching anything else
- [ ] New dialog msgids shipped in both languages, i18n gate green; NVDA reads the three controls and their states on the free native path
