# 16 — Settings dialog

**Spec:** [spec §13 (dialog), §11 (FR-i18n-runtime)](../../pathmaster-v0-1-0/spec.md)

**What to build:** Tools → Settings… lets the user change the Interface Language and the backup budget: a modal dialog with the language selector (whose own label carries the restart notice, keeping the Announcement catalogue closed) and the `maxBackups` field, writing settings.json atomically on OK.

**Blocked by:** 07 (settings semantics), 08 (UI shell, Tools menu home).

**Status:** ready-for-agent

- [ ] Dialog holds the language selector labelled "Language (takes effect after restart)" with endonym items ("English", "Українська") plus the auto choice, and the `maxBackups` field (valid domain ≥ 1); our own OK/Cancel buttons, never stock ones
- [ ] The file records the choice, not its outcome (`"auto" | "en" | "uk"`); language applies after restart — no live re-translation, no extra Announcement
- [ ] `maxBackups` applies immediately (next rotation uses it)
- [ ] OK writes settings.json via the atomic-replace helper, preserving unknown fields; a field previously invalid in the file is replaced only when the user changes that setting
- [ ] In Read-only Data the dialog's controls are disabled and read as disabled (no write path)
- [ ] Escape/Cancel leaves the file untouched; focus returns to the control that opened the dialog
- [ ] All strings in the Catalogue with Ukrainian translations; the completeness gate passes
