# F1 and Help → Documentation

Type: grilling
Status: open
Blocked by: —

## Question

The first v0.2.0 candidate, raised 2026-08-25: `F1` — Windows' own help key — does nothing, and the
standing rule says every shortcut has a menu home, so the fix is a Help → Documentation item. Small
item, one real question:

- **The offline question.** The README lives online, so a shell-open of its URL does nothing on a
  machine with no network — a real caveat for an exe carried on a stick. Options to weigh: online
  URL with a named failure story; shipping a document beside the exe (ends "one portable file");
  embedding help text in the exe (a dialog or generated local file — the Catalogue question of a
  *long* text, and does it get translated?); or F1 opening the existing About/shortcuts surface
  instead. Decide the shape and its failure story.
- Menu home wording and both-language Catalogue entries; whether the existing Help menu's one-item
  structure (About) grows to two or more.
- What F1 does in *dialogs* (nothing? same target?) — say it, don't leave it to chance.
- The menu-structure steps of the Release Checklist (31, B12, the mnemonic gate) are voided by any
  menu change — note for the assembly ticket, which re-runs them once, for all of v0.2.0's menu
  growth together.
