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

## Input from ticket 06 (2026-08-26)

The Search contract is settled, and it hands this ticket one fact and one trap.

**The fact.** The Search field is a permanent native `TextCtrl`, one per Scope tab, holding that
Scope's own text. Filling it from the tree is therefore a per-Scope act, and it must behave exactly
as typing does: membership recomputes live, and the count speaks through the same debounced path
(catalogue item 9). Ctrl+F focuses-and-selects; ESC clears and returns to the list.

**The trap, which this ticket owes an answer to.** 06 decided Search matches the **currently
displayed** rendering, and 05 decided every Run starts in **raw** mode. The PRD builds the tree over
**expanded** paths. So Enter on a node whose text is `C:\jdk21` fills the Search bar with a string
that matches **nothing** in a raw-mode list showing `%JAVA_HOME%\bin` — the feature would deliver
the user an empty list and a spoken "No matching entries" as its normal result. Options to weigh:
the tree fills the field with raw text, or Enter also switches Expansion Mode to expanded, or the
search coupling is dropped in favour of selecting the row directly (which the ticket already asks
about as the simpler model).
