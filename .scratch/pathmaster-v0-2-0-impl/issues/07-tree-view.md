# 07 — Tree View

**Spec:** [delta-spec §6, §14 (strings)](../../pathmaster-v0-2-0/spec.md)

**What to build:** View → "PATH Tree…" (Ctrl+T) opens a modal, per-Scope comprehension surface: the active Scope's Filtered View snapshotted at open, merged by the expanded reading into a prefix tree with compressed chains and audible three-part leaf labels. Enter on a leaf (or "Go to entry") selects that Entry's row in the main list — by identity, never by text — and closes.

**Blocked by:** 03 (the Filtered View the snapshot is taken of).

**Status:** done — verified against live NVDA by the user at the keyboard; **§6 amended twice from the toolkit** (the handler performs the toggle, and must consume the activation), see Comments

- [x] Menu item View → "PATH Tree…" with Ctrl+T, disabled on the Backups tab; dialog title names the Scope ("PATH Tree — User PATH" / "… — System PATH"), both languages
- [x] Content: the active Scope's Filtered View snapshotted at open (whole Working Copy when unnarrowed); the dialog never touches the narrowing criteria; snapshot — no live diagnostics, no refresh affordance, no timer in the modal's event loop; reopening is the refresh
- [x] The merge algorithm lives in `pathmaster-core` and is unit-tested: Entries merged by the expanded reading (Normalisation's own, undefined `%VAR%` literal, independent of Expansion Mode) into a prefix tree; single-child chains compress into one node with the joined label; siblings sort alphabetically case-insensitive; "Unresolved variables" and "Relative entries" top-level groups sort after the drive roots and hide when empty; no artificial super-root; one leaf per Entry — duplicates are sibling leaves
- [x] Leaf label is the whole audible payload: segment/joined chain + raw form in parentheses only when it differs + Issue suffix in the exact Status-column words only when an Issue exists (`bin (%JAVA_HOME%\bin) — Missing`); inner nodes and groups carry no suffixes
- [x] Interaction: Enter on a leaf selects that Entry's row in the main list by Entry identity and closes; Enter on an inner node expands/collapses — **performed by the `ITEM_ACTIVATED` handler rather than by a native default action, which does not exist for Enter**; that handler is the single home of the commit logic; the landed row speaks in full and Cancel speaks the restored focus
- [x] Buttons "Go to entry" (default; disabled while an inner node or group is selected) + Cancel; Esc closes; no OK, no Close; tab order tree → Go to entry → Cancel; initial focus on the first top-level node
- [x] Widget: wxdragon `TreeCtrl` — the native `SysTreeView32`
- [x] No new Announcements, no `settings.json` fields; the dialog remembers nothing — expansion state not preserved
- [x] Catalogue strings shipped in both languages: the two titles, "Go to entry", the two group names; Cancel reuses the existing msgid; i18n gate green

## Comments

**2026-08-27 (implementation)** — The shape is a new pure module, `pathmaster_core::tree`, and it takes
**no Expansion Mode and cannot be given one** — which is the whole of §6's "independent of Expansion
Mode", said structurally rather than in a comment. `Tree::of` takes `(EntryId, &str, &[Issue])` per
visible Entry plus an injected `Environment`, and hands back `Node`s the dialog builds items from.
31 tests fix every rule §6 states.

**Four readings §6 left to implementation, each with its reason.**

1. **Quotes are read past.** §6 says "the expanded reading (Normalisation's own)", and Normalisation's
   own first two steps are `strip_quotes` then `expand` — the same pair `diagnose_entry` takes. Reading
   `expand(raw)` alone (Expansion Mode's rendering) would give `"C:\tools"` a first segment of `"C:`,
   inventing a drive root nobody has, and `is_fully_qualified` would then exile the Entry to
   "Relative entries" — which is a lie about a Quoted entry. The raw parenthetical still shows the
   quotes and the `Quoted` suffix still explains them.
2. **Merging is case- and slash-folded, by `Normalised::of_expanded` over one segment.** Merging asks
   "are these the same directory?", which is the one question `CONTEXT.md` says Normalisation exists to
   answer, so `C:\Tools\bin` and `C:\tools\lib` are one node (the **first** spelling wins the label).
   The same fold orders siblings, so two spellings of one directory can never be separated by the
   alphabet — one fold, both questions.
3. **Only the segments *above* an Entry's last are merged; the leaf is always pushed.** That is what
   makes §6's "one leaf per Entry — duplicates are sibling leaves" true, and it also keeps an Entry
   that happens to be another's prefix reachable: `C:\tools` and `C:\tools\bin` give a leaf `tools`
   beside a leaf `tools\bin` rather than a `tools` node with the first Entry swallowed into it.
   Verified live against the real 45-entry PATH, where `scoop\apps\nodejs` holds exactly that pair.
4. **An Empty Entry is grouped with "Relative entries".** §6 closes the group set at two and says
   "never exclusion", and an Entry with no usable path text is neither unresolved nor placeable — so
   by elimination it goes to the unqualified group, where its own `Empty` suffix corrects the group's
   name immediately. A third group would be a spec change; dropping it would make a leaf-per-Entry
   promise the dialog does not keep. Recorded as the honest reading of a closed set rather than as
   a preference. Live: a whitespace-only Entry reads `    — Empty` there.

**Identity is a position path, not a handle.** wxdragon's `TreeItemId` has no equality, its
`Into<u64>` is the address of the Rust wrapper rather than of the item, and its custom-data
round trip goes through that address — so none of the three is an identity a caller may keep.
What the dialog keeps instead is the item's **place among its siblings, level by level**, walked
with `GetItemParent`/`GetPrevSibling` and handed back to `Tree::at`. It terminates on the hidden
root, which is the one item wxMSW answers "no parent" for (`msw/treectrl.cpp`, `IS_VIRTUAL_ROOT`),
and it is exact because the tree is a snapshot nothing rebuilds. `TR_HIDE_ROOT` is also how "no
artificial super-root" is spelled to wx: the drive roots and the groups become real native roots.

**Two measured deviations from §6's letter.**

*The first is real behaviour and it changed a line of the ticket.* §6 says Enter on an inner node
expands/collapses "via the native default action". **There is no such action**: native
`SysTreeView32` does nothing at all with Enter (it toggles on Left/Right and `+`/`-`), and wxMSW
discards the `ITEM_ACTIVATED` it raises from `TVN_KEYDOWN` (`(void)HandleTreeEvent`). So the toggle
is performed by the handler, which leaves that handler the single home of both halves of the
activation gesture — commit on a leaf, toggle on anything else — which is what §6 was protecting.

*And that handler must consume the event.* Built without it first, and the double-click measured
wrong: wxdragon calls `event.Skip(true)` **before every handler** (`WxdEventHandler::DispatchEvent`),
so a handler that says nothing leaves the event skipped; wxMSW reads that as "the user did not handle
the activation" and lets the tree act on the double-click as well (`*result = processed`, `NM_DBLCLK`).
Our toggle and comctl32's then both fired and a double-click on a folder visibly did nothing.
`event.event.skip(false)` is the fix, and it is load-bearing rather than tidiness. The keyboard path
was never affected — wxMSW ignores the result there — which is exactly why only a mouse probe found it.

*The second deviation was a measurement gap, not a behaviour, and the user closed it — see the NVDA
paragraph below.* **Escape could not be measured from the probe.** wx's `SetEscapeId` rides
`wxEVT_CHAR_HOOK`, which wxMSW generates from a `WH_KEYBOARD` hook
(`msw/window.cpp`, `wxKeyboardHook`); a posted `WM_KEYDOWN` does not reach it, and `SendInput` from
this session reaches nothing at all — with the dialog confirmed foreground and `SendInput` reporting
two events accepted, Escape closed neither the Tree View **nor the Add/Edit dialog**, whose Escape is
v0.1.0-shipped and Release-Checklist-verified. So the finding is the probe's, per the standing rule to
confirm against a known-good dialog first. `dialog.set_escape_id(ID_ABANDON)` is character-for-character
what the four existing dialogs do; the Release Checklist is where it is proven.

**A second measured fix, invisible in the spec.** The dialog first shipped with `HasButtons` alone and
the whole top level rendered as a flat list of drive roots with no expander at all: comctl32 draws no
expand button on a *root* item unless `TVS_LINESATROOT` sits beside `TVS_HASBUTTONS`. The style is now
`Default | Single | HideRoot`, `Default` being wx's own `HasButtons | LinesAtRoot` pair.

**The fill is all-or-nothing.** A widget item's only identity is its position, so a level built with
one item missing would name different Entries than the snapshot's positions do — and "Go to entry"
would then select, and offer for editing, a row the user did not choose. `append` therefore fails the
whole fill rather than skipping a node, and an unbuildable tree opens empty with the button disabled.
The branch is unreachable in practice (wx refuses an item only for a control already destroyed); it is
answered rather than assumed away because the failure mode is wrong data, not a missing feature.

**Nothing about the modal is live, by construction rather than by discipline.** The dialog is handed a
`Tree` and has no route back to a `Session`, a `Findings` or a `Criteria`, so no pass, no edit and no
narrowing can reach it — and the two Timers that do run under a modal loop (the diagnostic Pump's and
the Search debounce) were already gated on `door::modal_open()`, which this dialog goes through like
every other. The Issues on a leaf are the last completed pass's, read by Entry id exactly as the
Status column reads them, so a leaf says what its row says.

**Live verification** (staged copies with private Data Directories, `%NOPE%\bin`, `..\relative` and a
whitespace-only Entry added to the Working Copy and never applied; menus and widgets read
cross-process, labels read through a buffer allocated in the target with `TVM_GETITEMW`):

- **English** — menu `PATH &Tree…\tCtrl+T`, id 6022, last in View; title `PATH Tree — User PATH`;
  buttons `Go to entry` / `Cancel`; focus at open on the tree; top level `C:`, `Unresolved variables`,
  `Relative entries` in that order, groups after the drive root; `%NOPE%\bin — Missing`,
  `    — Empty`, `..\relative — Relative`.
- **Shape, against the machine's own 45-entry PATH** — chains compressed
  (`Program Files` → `Git\cmd`, `Java\jdk-21\bin`), siblings sorted case-insensitively, three sibling
  leaves for the three `C:\Windows` Entries with `— Duplicate` on the two the pass flagged, and
  `%SystemRoot%\System32\drivers` merged under `C:\Windows\System32` by its expansion.
- **Interaction** — "Go to entry" disabled on an inner node and on a group, enabled on a leaf; Enter on
  an inner node expands then collapses and never commits; Enter on a leaf closes the dialog and lands
  on `#48 '..\relative'` with focus on the list; the button route lands the same way; a double-click
  expands a folder exactly once and commits on a leaf; tab order (walked by `GetNextDlgTabItem` with
  the button live) is tree → Go to entry → Cancel; the item is greyed on the Backups tab.
- **Ukrainian** — `Дерево PATH(&T)…\tCtrl+T` (Latin mnemonic in parentheses, accelerator appended by
  code); `Дерево PATH — PATH користувача` and `… — PATH системи`; `Перейти до запису` / `Скасувати`;
  `Нерозв'язані змінні` / `Відносні записи`; leaf suffixes reuse the Status column's own words
  (`— Відсутній`, `— Відносний`). View mnemonics S, F, I, E, T unique in both languages, gated.

**No new Announcements and no `settings.json` field**, as §6 requires: NVDA speaks the modal title on
open, the focused node as the tree is walked, the landed row after Go to entry and the restored focus
after Cancel, all natively. The dialog holds no state at all — a fresh `Tree` and a fresh `TreeCtrl`
per open, so "expansion state not preserved" is a property of the code rather than a rule kept by hand.

**Verified against live NVDA on this machine (2026-08-27), the user at the keyboard** — rounds one
and two's provenance, not round three's harness, and so recorded as **round four** in delta-spec §19.
Seven readings, all as designed: Ctrl+T speaks the Scope-named dialog title and the first top-level
node; arrowing the tree speaks a compressed node's **joined** label in full; a three-part leaf speaks
all three parts — segment, raw form in parentheses, Status word; Enter on an inner node speaks the
expanded and then the collapsed state and does **not** commit; Enter on a leaf closes the dialog and
the landed row speaks in full in the main list; **Esc closes and returns the focus to where Ctrl+T was
given from** — the one reading the probe could not reach, now measured by ear; and Tab walks
tree → "Go to entry" → Cancel with the button reading as unavailable on a folder node and available on
a leaf.

That discharges on the built application the three obligations ticket 16 took against a prototype
(joined compressed labels, three-part leaves, the focus landing after Go to entry). **Nothing NVDA
said amended a contract.** The two §6 amendments above came from the toolkit, not from the reading,
and both were already found by the cross-process mouse probe before this session.
