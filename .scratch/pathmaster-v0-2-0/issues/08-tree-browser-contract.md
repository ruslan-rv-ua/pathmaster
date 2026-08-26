# Tree View browser contract

Type: grilling
Status: resolved (2026-08-26)
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

## Resolution (2026-08-26)

Researched first: [research/08-tree-browser-best-practices.md](../research/08-tree-browser-best-practices.md),
per the map's standing directive 7. Decisions:

1. **The Search coupling dies — a recorded PRD deviation.** No surveyed tool (RapidEE, the
   Explorer/Regedit master–detail canon, `SHBrowseForFolder`) couples tree activation to a search
   field, and 06's trap (expanded tree text vs raw-mode list) would make an empty list the feature's
   normal result. Instead: **Enter on a leaf selects that Entry's row in the main list — by Entry
   identity, never by text — and closes the dialog; Enter on an inner node expands/collapses it**
   (the native default action per the ARIA APG). The tree is a comprehension surface; the leaf-jump
   is its exit ramp. "Show me everything under X" is a future Filter feature, not this dialog's job.
2. **Per-Scope, like Search and the Filter**: the dialog opens over the active Scope tab, its title
   names the Scope, and the command is **disabled on Backups** (like Ctrl+F and the Filter submenu).
   Cross-Scope duplicate hunting stays diagnostics' job, not the tree's.
3. **Built over the Filtered View, snapshotted at open** — the Entries currently visible, which in
   the unnarrowed state is the whole Working Copy. Every leaf is therefore navigable, and the dialog
   never touches the narrowing criteria (03's contract: only the user's own narrowing actions change
   them). Wanting the whole PATH's shape = clear the narrowing first, then open the tree.
4. **Base is always the expanded reading** — Normalisation's own, undefined `%VAR%` staying literal —
   **independent of Expansion Mode**. The filesystem shape is the feature's nature, not a rendering.
5. **Entries with no filesystem position get top-level group nodes, never exclusion**:
   "Unresolved variables" (undefined-`%VAR%` Entries as literal leaves) and "Relative entries".
   Precedents: Device Manager's "Other devices", WinDirStat's `<Free Space>`/`<Unknown>`, VS
   "Miscellaneous Files". Empty groups are not shown; no artificial "PATH" super-root; groups sort
   after the drive roots at the end of the top level. Group names are Catalogue strings.
6. **Single-child chains compress into one node with the joined label**
   ("Program Files\Java\jdk-21") — VS Code `compactFolders`' default, and what keeps a long PATH
   inside the UX guide's four-level budget. **Siblings sort alphabetically, case-insensitive**;
   PATH order belongs to the main list and cannot survive prefix-merging anyway.
7. **The leaf label is the whole audible payload** (NVDA spends `accValue` on the level; a native
   tree item has no columns or description), so it carries up to three parts:
   segment/joined chain + **raw form in parentheses only when it differs** from the expansion +
   **Issue suffix in the exact Status-column words only when an Issue exists**, several joined as in
   the column: `bin (%JAVA_HOME%\bin) — Missing path`. Inner nodes and groups carry no suffixes —
   status belongs to the Entry, not the prefix. Tooltips rejected: NVDA demonstrably cannot reach
   them (nvda#3314/#8118/#10320).
8. **One leaf per Entry.** Two Entries with the same expanded path (the classic duplicate) are two
   sibling leaves — a leaf must lead to exactly one row; the raw parenthetical and the Duplicate
   suffix tell them apart, and native trees allow same-label siblings.
9. **Snapshot, no live diagnostics, no refresh affordance** — reopening is the refresh. Label
   changes in a native tree are silent to NVDA without bespoke live-region work, a diagnostics-driven
   rebuild under focus is 04's hazard class, and a snapshot keeps every timer out of the modal's
   event loop — fully decoupling this dialog from ticket 14's mechanism decision. The main list
   stays the authority on current Issue labels, and the leaf-jump lands the user there anyway.
10. **Buttons: "Go to entry" (default) + Cancel; Esc closes; no OK, no Close.** "Go to entry" is
    disabled while an inner node or group is selected — the visible, NVDA-readable statement of the
    leaf-only commit rule — and exists despite Enter because the UX guide demands a redundant
    visible form of the double-click/Enter action. No Enter conflict: wxMSW's tree eats unmodified
    Enter and raises `ITEM_ACTIVATED` (confirmed in `msw/treectrl.cpp`), so the activation handler
    is the single home of the commit logic and the button calls it too. Tab order: tree → Go to
    entry → Cancel; initial focus on the tree's first top-level node.
11. **Command home: View menu, "PATH tree…" with an ellipsis, disabled on Backups. Alt+T is
    rejected as a recorded PRD deviation** — the UX guide forbids Alt+letter shortcuts (access-key
    conflicts; it would shadow any future T mnemonic). Proposed key **Ctrl+T** (in the UX guide's
    recommended conflict-free set); the final key belongs to assembly (15), like Ctrl+I.
12. **No new Announcements — the closed set does not grow.** NVDA natively speaks the modal title
    and focused node on open, the landed row (all columns) after Go to entry, and the restored focus
    after Cancel/Esc. Title, buttons, menu item and the two group names are ordinary Catalogue
    strings, not Announcements. **No `settings.json` fields**: the dialog remembers nothing —
    snapshot per open, expansion state not preserved.
13. **The fallback branch closes unused**: 01 confirmed wxTreeCtrl is bound and native
    `SysTreeView32`; the right-to-die is not activated.
14. **Tree View** becomes a `CONTEXT.md` term: a modal, per-Scope comprehension surface — the
    Scope's Filtered View snapshotted at open, shaped as the filesystem; derived view state that
    reads and never changes.

Downstream: three NVDA obligations (joined compressed labels, three-part leaf labels, focus landing
after Go to entry) → ticket 16; menu item, final accelerator, dialog title/button/group-name wording
and Catalogue numbering → assembly (15); `CONTEXT.md` gains **Tree View**.
