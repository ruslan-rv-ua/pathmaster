# Tree View browser contract

Type: grilling
Status: open
Blocked by: 01, 06

## Question

FR-tree-browser: a modal dialog showing all PATH entries as a filesystem-shaped tree; selecting a
node navigates the main window. The widget facts (01) and the Search contract (06) are in — the
PRD's Enter-on-any-node **fills the Search bar**, so search had to be settled first. Specify:

- Does the PRD's interaction model survive contact with the settled Search contract? Enter on an
  inner node → search text + close; Enter on a leaf → also scroll/select the row. Or is there a
  simpler model (leaf-only selection, no search coupling) that serves a screen-reader user better?
  Weigh what the feature is *for* against a long PATH: is it navigation, or comprehension?
- Tree construction: built over **expanded** paths (PRD) with raw shown in leaf detail — confirm,
  and decide how an undefined `%VAR%` entry and a Relative entry are placed (they have no clean
  filesystem position).
- Modal + diagnostics: the v0.1.0 borrow hazard is a Timer ticking inside a modal's event loop —
  this dialog must state its relationship to the diagnostic pass (ticket 14's discipline decision
  is about the mechanism; this ticket owes only the dialog's behaviour: live Issue labels or a
  snapshot taken at open?).
- Keyboard and NVDA: navigation keys, what each TreeItem's accessible name is (01 says what's
  possible), how Issue status is carried on a leaf (label suffix — the Status-column words?), and
  what opening/closing announces.
- Command home and shortcut (Alt+T per PRD — subject to 02's model), and the dialog's title and
  buttons through the Catalogue.
- If wxdragon's tree binding turns out unusable (01), this ticket decides the fallback: raw-handle
  work, a different presentation (grouped list?), or the feature dying with a recorded reason —
  the D&D ticket's right-to-die applies here too.
