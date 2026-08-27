# 08 — Copy entry

**Spec:** [delta-spec §8, §13 (items 13, 14)](../../pathmaster-v0-2-0/spec.md)

**What to build:** Edit → Copy (Ctrl+C) puts the focused visible Entry's currently displayed rendering on the clipboard — raw in raw mode, expanded in expanded mode — and NVDA hears "Copied to clipboard", or the failure message when the clipboard write fails.

**Blocked by:** 04 (copy-what-is-shown needs Expansion Mode to exist).

**Status:** in review — built, gated, and verified against live NVDA in both languages; **§8's clipboard mechanism is one measured deviation** (wx's own error dialog), see Comments

- [x] Menu home Edit → Copy `\tCtrl+C`, joining the per-Entry group after Delete Entry; `session: None` disables it on the Backups tab exactly as Edit/Delete; msgid "Copy" («Копіювати») shipped
- [x] Copies the focused visible Entry's currently displayed rendering with exact text fidelity — no quotes added, an Entry's own quotes are content; always exactly one Entry (single-select reaffirmed)
- [x] Scoping is the platform's own: no focus-checking handler, no dynamic accelerator tables — with focus in the Search field or a dialog field, Ctrl+C copies that field's text, not the Entry (wxMSW text entries claim it before accelerator translation); the command is otherwise frame-wide
- [x] Success speaks Announcement 13 "Copied to clipboard" — fixed text, no echo of the payload; failure speaks Announcement 14 "Could not copy to clipboard" immediately on a failed write, no retry; both pairs shipped in both languages
- [x] No selection = silent no-op (the edit/delete precedent) — silence only ever means "nothing was selected"
- [x] The copy demonstrably outlives the Run (close the app, paste still works) — **without** a `flush()`, which the mechanism this ticket landed on does not need
- [x] No Ctrl+Insert twin, no settings field, no new NVDA obligation

## Comments

**2026-08-27 (implementation)** — Three msgids and two Announcements. `MENU_COPY` is **"Co&py"**,
not the `&Copy` every other Windows Edit menu has: C is already `&Cancel Changes`' in this menu,
and the per-menu mnemonic gate is a gate, not a preference. Ukrainian keeps the Latin letter in
parentheses as ADR-0004 requires — «Копіювати(&P)». `COPIED_TO_CLIPBOARD` and `COPY_FAILED` are
two **separate** `Announcement` variants rather than one carrying a `bool`: ADR-0009's amendment
says one variant per item of the closed set, and a bare `true` at the call site would say nothing
about which sentence it picked. The closed-set test grows to twelve variants over items
`[1,2,3,4,6,7,8,9,10,11,13,14]`.

`Command::Copy` sits after Delete in `ALL`, which is where §12 puts it in the Edit menu, and it is
the **one Entry command that answers above the writability line** in `Command::over`. §8 names only
`session: None` as what closes it, and the codebase's own rule is that `!session.writable()` closes
what *edits* — Copy reads. So an unelevated System tab and a Read-only Data run both keep Copy
while Add/Edit/Delete/Move grey out, which the live menu dump confirms; what it still asks is
Edit and Delete's own question, a focused **visible** Entry, so an empty result set closes it too.
The handler is `edit`'s shape without the dialog: raw text owned out of the scoped access, then
`Rendering::render` — the same call `Row::compose_visible` makes, so "what is shown" is one
expression and not two that could disagree. No Checkpoint, no `after_edit`, no diagnostic pass.

**One measured deviation from §8, and it is the clipboard mechanism itself.** §8 names
`Clipboard::set_text` and then `flush()`. Built that way first, and the failure path measured
badly: with the clipboard held open by another process, Ctrl+C speaks Announcement 14 **and then
wx raises its own modal box** — «Pathmaster Error» / "Failed to put data on the clipboard" — from
the `wxLogSysError` inside `wxClipboard::AddData`, which a GUI app's default log target turns into
a `MessageBox`. It is untranslated, outside the closed Catalogue, it steals focus, and it even
misspells the product name (wx capitalises it, so §16's proper noun comes back as "Pathmaster").
`wxClipboard::Flush` reports failure the same way, so the spec's best-effort flush could raise one
too. The wxWidgets answer is `wxLogNull` around the call — precisely
[what KiCad did](https://gitlab.com/kicad/code/kicad/-/merge_requests/654) for this exact message —
and **wxdragon binds no `wxLog` at any level**, 0.9.20 included, so it is unreachable from Rust
here at any pin.

So the write goes through `clipboard::copy` (`crates/pathmaster/src/clipboard.rs`): the plain Win32
road `OpenClipboard` → `EmptyClipboard` → `SetClipboardData(CF_UNICODETEXT, HGLOBAL)` →
`CloseClipboard`, which is the road
[`clipboard-win`](https://docs.rs/clipboard-win/latest/clipboard_win/struct.Clipboard.html) takes
and which says nothing to anyone — the `bool` is the whole report, which is exactly what §8 asks
for. The owner window is the frame and must be: opened with a null owner, `EmptyClipboard` sets the
owner to null and the very next `SetClipboardData`
[fails](https://learn.microsoft.com/en-us/windows/win32/dataxchg/using-the-clipboard). **It needs
no flush**: a real `HGLOBAL` is handed to the system, which owns it from then on — only *delayed
rendering* (a null handle) dies with the process, and only OLE's live data object needs
`OleFlushClipboard`. So "the copy outlives the Run" becomes a property of the mechanism instead of
a second call whose result §8 then has to say is never announced, and wx touches the clipboard
never (its destructor only clears what its own `m_lastDataObject` holds). **§8's last two bullets
are superseded by this; every other sentence of §8 stands unchanged**, and ticket 12's catalogue
and Checklist audit should read them that way.

No log record on failure, deliberately: the write answers `bool` and nothing more, so there is no
raw error code for a line to carry, and §14's records are built from derived facts rather than
from the fact that something went wrong. The Announcement is the channel, which is §8's whole
argument.

**Verified against live NVDA on this machine** (`tools/nvda-drive.ps1`, staged copy): Ctrl+C on a
focused row speaks `Copied to clipboard` **and nothing else** — no payload echo, no row re-read.
With the clipboard held open from another process, it speaks `Could not copy to clipboard` **and
nothing else** — that second line is the deviation's proof: on the `set_text` build the same run
spoke `Pathmaster Error / діалог` and `OK / кнопка` after it, and it is now gone. Ctrl+F into the
Search field, Ctrl+A, Ctrl+C: NVDA logs the input and **no Announcement follows** — wxMSW's
text-entry preprocessing kept Ctrl+C for the field, so the Entry command never fired, and the
scoping §8 predicted needs no code at all. ESC back to the list and Ctrl+C speaks
`Copied to clipboard` again. On a list arrived at but never arrowed through — no focused row —
Ctrl+C speaks **nothing**.

Cross-process probes additionally confirmed, in **both** languages: the Edit menu carries
`Co&py <TAB> Ctrl+C` / «Копіювати(&P)  Ctrl+C» at id 6004, in position 3, right after Delete
Entry; it reads **live** on an unelevated System tab where every editing item is greyed, and
**greyed** on the Backups tab, where the command posted anyway does nothing at all (banner
unchanged, clipboard sentinel intact). Over a Snapshot restored into the User Working Copy and
never applied — the only way to get an Entry carrying its own quotes, since `"` is a forbidden
character in a *typed* Entry — raw mode copies `"C:\Quoted Path\bin"`, `%SystemRoot%\plain-var`
and `"%SystemRoot%\quoted and expanded"` byte for byte, and Ctrl+E then copies the same three rows
as `"C:\Quoted Path\bin"`, `C:\WINDOWS\plain-var` and `"C:\WINDOWS\quoted and expanded"` — the
variable resolved, the quotes still content, nothing added. A copy made and then followed by a
clean Exit is still on the clipboard afterwards, with no `flush()` anywhere. Releasing the held
clipboard lets the very next Ctrl+C succeed normally: the failure is transient and nothing about
it is remembered.

The README's keyboard table and the Release Checklist's Copy steps are ticket 12's by its own
checklist, and are left to it.
