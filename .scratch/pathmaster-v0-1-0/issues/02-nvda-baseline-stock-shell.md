# NVDA baseline for a stock wxdragon shell

Type: prototype
Status: open
Blocked by: —

## Question

What does NVDA announce for a stock wxdragon UI, with **no accessibility code written at all**?

This is the most load-bearing fact in the effort: it decides how much of US-accessibility arrives free from
native Win32 controls and how much has to be engineered.

Build the smallest throwaway app (`/prototype`) that carries the app's real shape:

- A frame with a menubar — File / Edit / View / Tools / Help — with accelerators (`Ctrl+Z`, `F5`) and one
  deliberately disabled item.
- A notebook with three tabs: User PATH / System PATH / Backups.
- A `wxListCtrl` in report mode, columns `Path` and `Status`, ~10 rows, some with `Warning: Duplicate` /
  `Error: Path does not exist` text in the Status column, and one empty-list tab.
- Add / Delete / Move Up / Move Down buttons, and a status bar with two fields.

No accessibility work of any kind. Then the user runs NVDA and reports, **verbatim**, what is spoken for:

1. App launch and window title.
2. Switching tabs with Ctrl+Tab.
3. Arrowing through the list — is the Status column read along with Path? Are column headers announced? Is the
   row position ("3 of 12") announced?
4. Landing on the empty list.
5. Opening a menu, moving through items, hearing a shortcut and a disabled state.
6. Tab / Shift+Tab between panes and buttons; where focus goes and whether anything traps it.
7. Status bar — is it reachable and readable at all?

Record what is spoken **and what is silent**. Silence is the finding that matters.

Findings → `../research/02-nvda-baseline.md`, with the prototype's location noted.
