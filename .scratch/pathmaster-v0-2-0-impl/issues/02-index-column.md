# 02 — The `#` column returns to the main list

**Spec:** [delta-spec §2.1, §14 (header msgid)](../../pathmaster-v0-2-0/spec.md)

**What to build:** The main list on both Scope tabs becomes three columns — `#` / Path / Status — and NVDA's per-row reading gains the leading position number: "{#}; {path}; Status: {types}". The `#` cell carries the Entry's 1-based position in the Working Copy and never renumbers; the column is permanent (the window never reflows under the user), so it is there before any narrowing exists to need it.

**Blocked by:** 01 (retrofit lands first).

**Status:** done — one measured deviation from §2.1's predicted NVDA string, see Comments

- [x] Both Scope lists show `#` / Path / Status; `#` is the Entry's 1-based Working-Copy position, recomputed with the list contents on every Working-Copy change (Move, Delete, Add, Undo, Redo, Restore renumber the *data*; the column always shows current original positions) (`Row::position`, filled from `enumerate()` inside `Row::compose`, so every rebuild recounts; live: Alt+Down moved entry 1 and it read back as `2`, Up read `1` as the *other* entry, Del renumbered the rest up, Ctrl+Z put both back)
- [x] The `#` column is a deliberate pixel constant — one more explicit `FromDIP()` call beside the Status column's; Path still takes all remaining width (`INDEX_COLUMN_DIP = 48`; live at 100% scaling: 48 + 571 + 220 = 839 = the list's client width)
- [x] Column header `#` is a Catalogue msgid, shipped in both languages, gated (`msgids::COLUMN_INDEX`, in `REGISTRY`, in `en.po` and `uk.po`; both catalogue gates green, and the shell-strings test names it)
- [x] NVDA reads a row as **"{#}; Path: {path}; Status: {types}"** on the free native path — verified live on this machine. One word off §2.1's prediction, and not fixable from this side: see Comments
- [x] The count compensation does **not** return: entry counts still come from Announcements, and NVDA's row-position setting stays uncompensated (nothing in `announce.rs` or the Announcement path was touched; NVDA spoke no "n of 45" at any point in the live run)
- [x] Existing tests green; no other layout change rides along (`just ci` exit 0; the diff is `msgids.rs`, its two catalogues, its test, `scope_page.rs`, and one stale sentence in `backups_page.rs`)

## Comments

**2026-08-27 (implementation)** — `Row` gained a `position: usize` — a number, not the digits the cell
shows, because nothing about it is language and the Fix Issues dialog (ticket 09) wants the same value.
`Row::compose` fills it from `enumerate()`, so the column is recounted by the one function every rebuild
already goes through; there is no separate renumbering step that could be forgotten at a call site.
`render` writes all three cells, `render_status` writes only column 2 — the `#` and Path cells it skips
are right by construction, since only a Working-Copy change can move an Entry and every such change goes
through `render`.

`#` is left-aligned, and cannot be otherwise: comctl32 forces `LVCFMT_LEFT` on the leftmost report column
and silently drops any other format there. Said out loud in the code so it is not "fixed" later.

The uk catalogue repeats the English `#`. §14 lists this header twice (main list, and the Fix Issues
dialog) and both times gives no Ukrainian, where every neighbouring string in that section carries one —
so the sign standing as the source writes it is the spec's own reading, not an omission on this side.
«№» is the form a Ukrainian reader might expect; it is one catalogue edit away and no code change, which
is the whole reason the header is a msgid at all. Both `.po` files carry a translator note saying so.

**The one deviation — NVDA's actual string.** §2.1 predicts "{#}; {path}; Status: {types}". Measured
against real NVDA on this machine (`tools/nvda-drive.ps1`, staged copy, both languages):

```
Speaking ['1; Path: C:\scoop\apps\vscode\current\bin']
Speaking ['7; Path: C:\scoop\persist\uv\tools\shims; Status: Missing']
Speaking ['19; Path: C:\Program Files (x86)\VMware\VMware Player\bin\; Status: Duplicate']
```

— i.e. **"{#}; Path: {path}; Status: {types}"**. The rule NVDA is following is the same one v0.1.0's
baseline measured (ticket 02 of v0.1.0: "both columns and the second column's header name"): the leftmost
report column is the item's *name* and is read bare; every other column is read as "Header: value", and an
empty cell is skipped. v0.1.0 had Path in column 0, so Path was bare. Putting `#` in front of it demotes
Path to a header-prefixed column — that is not a choice this ticket made, it is what column 0 means.

Nothing can be done about it from this side that is worth doing. Blanking the Path header would buy the
bare reading and cost the visible header §14 requires; any `set_accessibility_*` call moves the widget off
the free comctl32 path entirely, which v0.1.0's baseline warns is a re-measure, not an addition. Nothing is
lost either way — position and path are both spoken, first keystroke, no Announcement — so the recommended
resolution is to correct §2.1's sentence to the measured string. **Left for the user: the delta-spec is
locked, and amending it is not an implementation ticket's call.**

Live verification (staged copy under its own `data\`, machine `PATH` read but never applied): both Scope
lists report 3 columns through `LVM_GETHEADER`/`HDM_GETITEMCOUNT`, headers `#` / Path / Status and
`#` / Шлях / Стан, widths 48 / 571 / 220 against a client width of 839; the Backups tab is untouched at
its three autosized columns. Renumbering driven by accelerator: Alt+Down, Up, Ctrl+Z, Del, Ctrl+Z, each
read back above. `just ci` exit 0.
