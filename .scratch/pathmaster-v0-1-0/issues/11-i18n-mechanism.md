# i18n mechanism

Type: grilling
Status: open
Blocked by: 03

## Question

How are translations stored, embedded, and selected?

Settled at charting: language changes take effect **after restart**; `maxBackups` applies immediately;
`settings.json` holds `language` and `maxBackups` only (`theme` is cut). FR-settings-file and FR-i18n-runtime
must both be rewritten to stop contradicting each other.

Open:

- **Mechanism.** wx `.mo` catalogs through `wxLocale` (only if ticket 03 found it bound) versus a Rust-side
  crate (`fluent`, `rust-i18n`) with the catalog embedded in the exe. Which, and why — including how it
  survives the single-exe constraint.
- **One catalog, not two.** Every NVDA-facing string must come from the same catalog as the visible UI, or
  translations will silently diverge from what is spoken. Confirm nothing is announced from a hard-coded
  literal.
- **Default from system locale**: which API, and how `uk-UA`, `uk`, and an unrecognised locale each resolve.
- **Where the catalog lives** in the repo, and the workflow for adding a third language later without touching
  the code.
- **What is deliberately not translated**: registry paths, file names, log lines, and the exact
  `WM_SETTINGCHANGE` payload.
- Do any announced strings need plural forms or interpolation ("3 items", "N of M entries")? That constrains
  the mechanism choice, so decide it here rather than discovering it in v0.2.0.

## Carried in from ticket 09

Announcement texts are translation strings like any other UI text: canonical English in the spec,
Ukrainian shipped as translations. The closed catalogue of Announcements (ticket 09, D3) defines which
strings exist; this ticket owns only how they are stored, embedded and selected.
