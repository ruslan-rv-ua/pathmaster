# PathMaster v0.1.0 — Locked Specification

**Status: locked.** Assembled 2026-08-19 by ticket [16](issues/16-locked-spec.md) from the resolved
wayfinder map ([map.md](map.md)). The source PRD is [spec-input.md](spec-input.md) (Ukrainian,
verbatim); where the PRD and this document disagree, **this document wins** — every deviation is
listed in [§21 PRD deviation notes](#21-prd-deviation-notes). Each requirement names the ticket that
settled it; ticket answers and the research files behind them are the authority on detail this
document only gists.

Decisions marked **[assembly]** were fixed by ticket 16 itself, under the delegations the resolved
tickets left it (exact Catalogue English — ticket 12 D10; the shortcut table — ticket 09 D5;
StatusBar wording — ticket 17 D10). Everything else traces to a resolved ticket.

## 0. Scope

**v0.1.0 = every 🔴 must requirement, plus the StatusBar, `settings.json`, and minimal logging.**
All other 🟡 should features are deferred to v0.2.0: Drag & Drop reorder, `%VAR%` expansion toggle,
Search bar, Filter bar, Tree View browser, Fix Issues dialog, Ctrl+C copy entry. Cut outright (not
deferred): similar-path/typo diagnostics, the `theme` setting. See [§20](#20-cut-and-deferred).

**NFR priority when they collide: accessibility > portability (single exe, no runtime install) >
exe size.** The exe-size budget is relaxed from the PRD's 20 MB to **≤ 40 MB** (measured build:
7.22 MB — ticket 04).

## 1. Product and stack

**PathMaster** is a portable Windows desktop application that reads, edits and diagnoses the `PATH`
environment variable, built for a screen-reader (NVDA) user first. Single `.exe`, no installer, all
data in `data\` beside the executable.

- **Stack (fixed, not reconsiderable):** Rust + [wxdragon](https://github.com/AllenDang/wxdragon)
  **≥ 0.9.17** (earlier `AccRole` discriminants are mis-ordered — ticket 01), decided **on**
  0.9.18 for its wxWidgets 3.3.3, compiled from pinned source, statically, `crt-static` propagating
  into the C++ build. There are no prebuilt wxWidgets binaries.
  *Amended by impl ticket 12: this read "decided against 0.9.18 over wxWidgets 3.3.3", which says
  the opposite of what was decided. research/01 researched 0.9.18 and found it the version that
  fetches wxWidgets 3.3.3 from pinned, SHA256-verified source; research/04's pin list says
  "wxdragon 0.9.18 — pin exactly and commit `Cargo.lock`"; every measured number in this document
  was taken on it; the manifest floor is `"0.9.17"` and `Cargo.lock` resolves 0.9.18. Impl ticket
  01 spotted the garble and recorded the correction in its own Comments, but left the sentence
  standing — where it went on to mislead ticket 12's reviewer into reporting the resolved version
  as a spec violation. Fixed at the source this time.*
- **Target OS:** Windows 10 21H2+ and Windows 11, x64 only. 32-bit is unsupported.
  **[assembly]** winget `MinimumOSVersion` is pinned to `10.0.19044.0` (= Windows 10 21H2,
  matching NFR-compatibility; ticket 15 left the floor unpinned).
- **Screen reader:** NVDA is the only one in scope. JAWS/Narrator must not be deliberately broken
  but are not targeted or tested.
- **Domain language:** [CONTEXT.md](../../CONTEXT.md) is canonical for every term this document
  capitalises — Scope, Entry, Value Type, Absent, Normalisation, Editing Session, Working Copy,
  Baseline, Dirty, Checkpoint, Apply, Issue, Snapshot, Corrupted, Announcement, Banner, Catalogue,
  Interface Language, Release Checklist, Sanity Check, Data Directory, Read-only Data.
- **ADRs:** [0001 Checkpoint-based undo](../../docs/adr/0001-checkpoint-based-undo.md) ·
  [0002 Data Directory never relocated](../../docs/adr/0002-resolved-data-directory-never-relocated.md) ·
  [0003 No accessibility calls except announce](../../docs/adr/0003-no-accessibility-calls-except-announce.md) ·
  [0004 Catalogue text is load-bearing](../../docs/adr/0004-catalogue-text-is-load-bearing.md) ·
  [0005 Elevation by whole-app relaunch](../../docs/adr/0005-elevation-by-whole-app-relaunch.md) ·
  [0006 Snapshot schema is decoded, not raw](../../docs/adr/0006-snapshot-schema-is-decoded-not-raw.md) ·
  [0007 Crate boundary is the test boundary](../../docs/adr/0007-crate-boundary-is-the-test-boundary.md)

**A load-bearing fact for every implementer:** wxMSW wraps native Win32 comctl32 controls — a
`wxListCtrl` *is* a `SysListView32`. NVDA reads them for free, columns included. The accessibility
strategy rides that free path everywhere and adds exactly one function (§10).

## 2. Requirement disposition

Every US / FR / NFR / TC from the PRD, explicitly **kept**, **rewritten**, or **cut**. "Rewritten"
means the intent survives with changed acceptance criteria; the section column holds the rewrite.

| PRD id | Disposition | Settled by | Where |
|---|---|---|---|
| US-view-path | rewritten — two columns (no index column), Status carries Issue-type words, empty = healthy | 09, 13, 17 | §7, §10, §12 |
| US-diagnose | rewritten — six types (typos cut, `Quoted` added), async mechanism named | 13 | §7 |
| US-edit | rewritten — modal dialog, not inline; DnD deferred | 10 | §6 |
| US-admin-elevation | rewritten — menu command, no InlineAlert | 12 | §9 |
| US-backup | rewritten — per-Scope rotation, Scope in filename, schema carries Value Type | 14 | §8 |
| US-restore | rewritten — Restore loads the Working Copy, never writes the registry | 14 | §8 |
| US-accessibility | rewritten — replaced by criteria naming the spoken text | 09 | §10, [Release Checklist](../../docs/release-checklist.md) |
| US-i18n | rewritten — restart-effect; one Catalogue; third-language premise rewritten | 11 | §11 |
| US-settings | rewritten — `language` + `maxBackups` only; `theme` cut | 11, 20 | §13 |
| US-high-contrast | rewritten — the app never sets a colour | 09 | §10 |
| FR-view-tabs | rewritten — kept + entry-count Announcement on tab activation | 09 | §10, §12 |
| FR-listview-columns | rewritten — no icons, no OK/Warning/Error severity labels | 09, 13 | §7 |
| FR-auto-diagnose | rewritten — worker thread → mpsc → Timer drain; runs on every Working Copy change | 13 | §7 |
| FR-diag-duplicates | rewritten — Normalisation defined; first copy canonical; cross-scope | 13 | §7 |
| FR-diag-nonexistent | rewritten as FR-diag-missing — directories only, local roots only | 13 | §7 |
| FR-diag-length | rewritten as FR-diag-overlength — scope-level, 8,191/32,767, StatusBar + Apply dialog | 13 | §7 |
| FR-diag-relative | rewritten — "not fully qualified"; skips existence check | 13 | §7 |
| FR-diag-empty | rewritten — whitespace-only included; zero-Entries edge fixed | 13 | §7 |
| FR-edit-f2 | rewritten — modal Edit dialog; validation set + commit sequence | 10 | §6 |
| FR-add-delete | rewritten — dialog-first Add; Delete loses its confirm | 10 | §6 |
| FR-reorder-keyboard | kept (Move Up / Move Down, one Checkpoint each) | 06 | §5, §15 |
| FR-reorder-dnd | **cut → v0.2.0** | charting | §20 |
| FR-undo-redo | rewritten — Checkpoints; Apply is a barrier, never a stack flush | 06 | §5 |
| FR-apply | rewritten — fixed internal order; backup of the re-read value | 06, 12 | §5 |
| FR-cancel | rewritten — disabled while clean; Cancel is itself a Checkpoint | 06 | §5 |
| FR-close-confirm | rewritten — names the dirty Scopes; partial failure aborts the close | 06 | §5 |
| FR-backup-auto | rewritten — filename carries Scope; schema per ADR-0006 | 14 | §8 |
| FR-backup-rotation | rewritten — per-Scope budget; `maxBackups` ≥ 1 | 14, 20 | §8 |
| FR-backup-ui | rewritten — Corrupted as passive list text; Restore into the Working Copy, no confirm | 14 | §8 |
| FR-settings-file | rewritten (**PRD overridden**) — set aside as `.bad`, per-field tolerance | 20 | §13 |
| FR-i18n-runtime | rewritten — after restart; notice rides the selector's label | 11 | §11 |
| FR-refresh | rewritten — active Scope only; clears the Undo stack; announces the entry count | 06, 09 | §5 |
| FR-copy-entry | **cut → v0.2.0** | charting | §20 |
| FR-browse-folder | rewritten — lives in the Edit dialog; `wxDirDialog`; MRU exception documented | 10 | §6 |
| FR-var-expansion-toggle | **cut → v0.2.0** | charting | §20 |
| FR-menubar | rewritten — reduced structure; accelerators belong to the code | 10, 11, 12 | §15 |
| FR-statusbar | rewritten — two fields, command-only (`NVDA+End`), no field styling | 02, 13, 17 | §12 |
| FR-search | **cut → v0.2.0** | charting | §20 |
| FR-tree-browser | **cut → v0.2.0** | charting | §20 |
| FR-filter-bar | **cut → v0.2.0** | charting | §20 |
| FR-fix-issues | **cut → v0.2.0** | charting | §20 |
| FR-uac-elevation | rewritten — one menu command; `ShellExecuteEx("runas")`; declined UAC → dialog | 12 | §9 |
| NFR-portable | kept — verified: `crt-static`, import table 19 DLLs, no VC++ runtime | 04 | §16 |
| NFR-no-registry-writes | rewritten — a claim about **the process**, one named exception | 07, 10 | §3 |
| NFR-startup-time | kept — ≤ 2 s on SSD (measured 79.6 ms) | 04 | §16 |
| NFR-exe-size | rewritten — ≤ **40 MB**, CI-gated (measured 7.22 MB) | charting, 04, 19 | §16 |
| NFR-compatibility | kept — Win10 21H2+ / Win11, x64 | PRD | §1 |
| NFR-accessibility-wcag | rewritten — inverted into "the application never sets a colour" | 09 | §10 |
| NFR-no-color-only | kept — satisfied by the text-only Status column | 09, 13 | §7, §10 |
| NFR-window-sizing | rewritten — 900×650 DIP default, min 800×600, geometry persisted + clamped | 17 | §12 |
| NFR-logging | rewritten — format fixed, 1 MB rotate-at-open, two PII prohibitions | 21 | §14 |
| TC-file-structure | rewritten — Data Directory contract; `.old`, `.bad`, temporaries named | 07, 20, 21 | §3 |
| TC-registry-keys | rewritten — raw read/write, Value Type preserved, Absent distinct | 05 | §4 |
| TC-wm-settingchange | rewritten (**PRD spec bug**) — 1000–2000 ms, off the UI thread | 05, 12 | §4 |
| OS-other-env-vars, OS-sync, OS-plugins, OS-web-cli, OS-auto-update | kept — out of scope | PRD | §20 |
| Additional: scoop + winget | kept — detailed manifests and pipeline | 15 | §16 |

## 3. Data Directory and portability

Settled by ticket [07](issues/07-portable-data-directory.md);
[ADR-0002](../../docs/adr/0002-resolved-data-directory-never-relocated.md).

**The Data Directory rule.** `data\` beside the executable, where "beside" is resolved
deliberately: `current_exe()` → resolve reparse points (`fs::canonicalize`) → strip `\\?\` (and
`\\?\UNC\` → `\\`) → parent → append `data`. Measured: `current_exe()` reports a junction, not its
target — the naive rule would put `data\` in winget's shared `Links\` directory. Fallbacks:
resolution failure → use the unresolved path; `current_exe()` failure → Read-only Data.

**Read-only Data.** When `data\` cannot be created or written (pid-unique probe file at startup),
the app **starts anyway**: reads, diagnoses, lists Snapshots; every Editing Session is non-writable
(which disables *every* editing action, not Apply alone), Settings and Restore are disabled and read
as disabled. The mode **never relocates** the directory and offers no prompt — remembering a
location outside the app's directory would require writing outside it. Exactly three reason strings
exist: *own location unknown*, *data directory cannot be created*, *data directory is not writable*.

**Startup decision tree:** locate → create (`create_dir_all`) → probe → open log (a logging failure
degrades logging only, never the mode) → read `settings.json` (read in both modes; created only in
Writable Data) → **mode is decided once and governs the UI only; Apply never consults it** — rule:
*startup predicts, Apply verifies*.

**Two instances are a designed state** (elevation is a relaunch): no single-instance lock. Every
replacement write is atomic (temp file in the same directory + `MoveFileExW(MOVEFILE_REPLACE_EXISTING)`);
Snapshots use the same temp+rename; rotation tolerates already-deleted files; the log appends one
line per record with share read/write. ACLs are a measured non-problem: inherited DACLs, not
ownership, govern access, so an unelevated run rotates what an elevated one wrote.

> **NFR-no-registry-writes** (must, rewritten) — The PathMaster process creates and modifies
> nothing outside its own Data Directory, apart from the two target PATH registry values written by
> Apply, **and one named exception**: ComDlg32 MRU registry writes (HKCU, under our process) caused
> by the Browse folder picker (`wxDirDialog`) — accepted by user decision (ticket 10 D2) and
> documented in the README.
>
> Acceptance: Process Monitor, **filtered to the `PathMaster.exe` process**, records no file or
> registry write outside `<exe dir>\data\`, the two PATH values, and (after Browse is used) the
> ComDlg32 MRU keys. No other native file dialog is ever opened — code discipline, unverifiable
> from the import table (`COMDLG32` is linked unconditionally by wxWidgets).

> **TC-file-structure** (must, rewritten) — `PathMaster.exe`; `data\settings.json` (+
> `data\settings.json.bad`, single set-aside copy — §13); `data\backups\*.json` Snapshots (+
> transient `*.tmp`); `data\pathmaster.log` + `data\pathmaster.log.old` (§14); transient pid-unique
> write probe. Nothing else, anywhere. The claim does **not** extend to the exe's own directory
> being exclusively ours — under winget it demonstrably is not.

`app.manifest`: comctl32 v6, `PerMonitorV2`, `longPathAware` (not relied upon), no `trustInfo`
(linker contributes `asInvoker`).

## 4. Registry I/O

Settled by ticket [05](issues/05-registry-io-semantics.md);
details [research/05](research/05-registry-io.md) (15 hazards, all producing a *successful* write
with wrong content).

> **TC-registry-keys** (must, rewritten) — User: `HKCU\Environment`, value `Path`. System:
> `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`, value `Path`.
>
> - Read and write **raw**: `winreg::get_raw_value` / `set_raw_value`, preserving bytes **and**
>   value type. Never `set_value::<String>` (writes `REG_SZ` unconditionally — the .NET bug).
> - The existing Value Type is preserved, never normalised (§5 names the single user-triggered
>   exception). A missing value is the distinct state **Absent** (`ERROR_FILE_NOT_FOUND`).
> - First Apply over an Absent Scope **creates** the value as `REG_EXPAND_SZ`. Apply with zero
>   Entries over a Present Scope writes an empty string — never deletes the value.
> - External edits are detected by re-reading `(vtype, bytes)` — never the key's timestamp.
> - 32,767 is the per-variable limit; the combined check runs on the **expanded merged** string (§7).

> **TC-wm-settingchange** (must, rewritten — the PRD's 5000 ms is a spec bug: the timeout applies
> per top-level window and multiplies; 226 windows × 5000 ms ≈ 18.8 min) —
> `SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, L"Environment", SMTO_ABORTIFHUNG,
> 1000–2000, …)`, run **off the UI thread**; the `lParam` string UTF-16LE, NUL-terminated, outliving
> the call. A `0` return / timeout is **not** an Apply failure: logged `WARN`, never surfaced as an
> error (already-open shells never see the change regardless; only newly launched processes do).

## 5. Editing model

Settled by ticket [06](issues/06-editing-session-model.md);
[ADR-0001](../../docs/adr/0001-checkpoint-based-undo.md).

**Two independent Editing Sessions, one per Scope** (the Backups tab is not a Scope). Each is a
Working Copy over a Baseline with its own Undo/Redo stack and `writable` flag (User: always;
System: only when elevated; Read-only Data: neither). A non-writable Session disables every editing
action. A Session never survives a process boundary.

- **Entry** = raw substring between `;` separators, byte-for-byte; opaque id surviving Move/Edit
  (for focus restoration). Round-trip invariant: split-then-join reproduces the decoded value
  exactly. An empty value decodes to **zero Entries**, not one empty Entry.
- **Dirty is a comparison** (Working Copy vs Baseline: order, raw strings, Value Type), never a
  flag. One predicate drives Apply, Cancel, close-confirm, and the Refresh/Restore warnings.
  **Apply and Cancel are disabled while clean** and read as disabled (rewrites FR-cancel).
- **Value Type** belongs to the Working Copy, the Baseline, the dirty comparison, and every
  Checkpoint. Committing `%VAR%` into a `REG_SZ` Scope raises the convert-or-keep dialog (§6) —
  the single exception to "never change the type"; never automatic.

> **FR-undo-redo** (must, rewritten) — Undo is a stack of whole-copy **Checkpoints** (Entries with
> ids, Value Type, focus hint), one per user-visible operation: Add, Delete, Move Up, Move Down,
> one confirmed Edit, one type change, one Cancel, one Restore. Ctrl+Z / Ctrl+Y; a new operation
> truncates the Redo stack; batches are one Checkpoint. Focus moves to the Checkpoint's hinted
> Entry, and the undo/redo Announcement fires (§10). **Apply is a barrier, not a flush**: Ctrl+Z
> after Apply moves the Working Copy only, never the registry, re-dirtying the Session with the
> ", unsaved changes" Announcement suffix.

> **FR-apply** (must, rewritten) — order fixed: **re-read → compare `(vtype, bytes)` → (external-
> change dialog) → back up what was just re-read, never the Baseline → write → move Baseline →
> re-run diagnostics.** The external-change dialog: title "PATH was modified externally since last
> refresh", buttons **[Overwrite]** (proceed; Undo stack survives) / **[Refresh and discard my
> changes]** (Working Copy and Baseline both become the new value; Undo stack cleared; nothing
> written, no backup) / **[Cancel]** (nothing happens; Session stays dirty and knowingly stale).
> Detection lives only in Apply — no watcher, no polling. Failure taxonomy: §9.

> **FR-cancel** (must, rewritten) — acts on the active Scope's Session; confirmation
> ("Discard changes?" [Yes] [No]) only while dirty; disabled while clean. **Cancel is itself a
> Checkpoint** — Ctrl+Z restores the discarded work. Announces "Changes discarded".

> **FR-refresh** (must, rewritten) — F5 / menu; re-reads **the active Scope only**; confirmation
> while dirty; **clears that Session's Undo/Redo stack**; re-runs diagnostics; announces the entry
> count ("{Scope}: {n} entries" — supersedes the PRD's "PATH refreshed"). Focus stays on the Entry
> with the same id if it survived, else its nearest neighbour by index, else the list.
>
> *Filled in by impl ticket 11.* The confirmation's title, which is the whole of it (§10): **"Refresh
> discards your unsaved changes and the undo history — continue?"** [Yes] [No] — unlike Cancel's it
> names the undo history, because Refresh is the one discard Ctrl+Z cannot take back. **A re-read
> that fails leaves the Session exactly as it was** and announces nothing: an unreadable value is
> not an Absent one (§4), and the Announcement catalogue is closed at seven. The §9 taxonomy that
> will name it arrives with Apply.

> **FR-close-confirm** (must, rewritten) — one dialog for the application, the title naming the
> dirty Scopes: **"Unsaved changes in: User PATH, System PATH — save before closing?"**, buttons
> [Save] [Discard] [Cancel]. Save applies each dirty Session in turn, **User first**, each through
> the full Apply path. **Partial failure aborts the close**: window stays open, focus moves to the
> failed tab, the reason is announced. Clean Sessions close with no dialog.

> *Filled in by impl ticket 15.* Four things the requirement leaves to the implementation:
>
> - **One list of dirty Scopes, read once.** It is what the title names and what the Apply Run is
>   handed, in tab order — which is User first. A second reading, or a second ordering rule in the
>   Catalogue, could only ever promise an order the sequence does not keep.
> - **[Cancel] is the default, the initial focus and the Escape key**, like every other dialog in
>   the application (§10's dialog discipline): it is the answer that changes least.
> - **A [Cancel] inside the run stops the close too, and moves nothing.** The external-change
>   dialog's [Cancel] and either over-length gate leave the run incomplete, so the window stays
>   open — but the user chose that, so there is nothing to announce and no failed tab to be sent
>   to. "Did the run complete" and "which Scope failed" are therefore two questions with two
>   answers, not one negated.
> - **The failed tab is activated before the outcome is applied.** Activating a tab speaks its
>   entry count (§10.1 item 1) and the failure speaks Announcement 3; last spoken is what is
>   heard, and the reason is what the user needs.
>
> Every route out — the title bar's [X], Alt+F4, File → Exit, the taskbar's Close — arrives as one
> close event, so the dialog is asked once and in one place. **A Windows log-off or shutdown is
> not one of them**: that is `wxEVT_QUERY_END_SESSION` on the application object, which nothing
> implements, so the Working Copies die unasked. Accepted for v0.1.0 — nothing of the machine is
> lost, only edits that had never reached it — and named here rather than left to be discovered.

## 6. Entry editing interaction

Settled by ticket [10](issues/10-entry-editing-interaction.md).

> **FR-edit-f2** (must, rewritten — "inline" is dropped deliberately) — F2, Enter, and double-click
> on a row all open the same **modal dialog**: title ("Edit entry" / "Add entry"), one labelled
> path field, Browse, OK, Cancel. `ListCtrlStyle::EditLabels` is not used. Validation on OK blocks
> `< > | "`, the separator `;`, and length-zero only; **whitespace-only commits verbatim** (the
> editor never trims or normalises). A failed validation shows a `MessageDialog` whose **title is
> the message** (e.g. "The entry contains a forbidden character: <" **[assembly]**), single OK; on
> dismiss focus returns to the field with the text intact. The OK sequence is fixed:
> **(1) validate → (2) `%VAR%`-into-`REG_SZ` dialog** — title "This entry uses %VAR%, but the
> value type (REG_SZ) does not expand variables" **[assembly]**, buttons [Change type to
> REG_EXPAND_SZ] / [Keep as literal text], each outcome one Checkpoint → **(3) commit as one
> Checkpoint**. A rejected edit leaves no Checkpoint — Ctrl+Z into an invalid state is impossible
> by construction.

> **FR-add-delete** (must, rewritten) — **Add is dialog-first**: same dialog, empty field; OK
> appends at the end (lowest search precedence) as one Checkpoint, focus on the new row;
> Cancel/Escape leaves nothing (no Entry, no Checkpoint, no Issue). **Delete has no confirm
> dialog** — undo is the safety net; focus stays at the same index, clamped to the new last row.

> **FR-browse-folder** (must, rewritten) — Browse lives in the Edit/Add dialog and opens
> **`wxDirDialog`** (the named exception to "no native file dialogs" — ComDlg32 MRU writes accepted
> and documented, §3). Seeds from the field text when it names an existing directory; the chosen
> folder replaces the field text; focus returns to the field. Implementation: call `.destroy()` on
> the `DirDialog` — its `Drop` leaks.

Duplicates and not-yet-existing paths commit legally — diagnostics flags them asynchronously; they
are never blocked at commit and never warned about during typing.

*Filled in by impl ticket 11.* The dialog's own strings: the field is labelled with the `Path`
column header — one label for one thing — and its buttons are **[Browse…] [OK] [Cancel]**, ours
rather than stock, because `add_std_catalog()` is never called (§11). The **length-zero** rejection
has its own title, "The entry cannot be empty"; `wxDirDialog` is given a title of ours,
**"Choose a folder"**, or it would speak wx's built-in English in a Ukrainian run. Every two-button
dialog in the application gives the **negative button the default, the initial focus and Escape**:
Windows hands Enter to the focused button, so a default the focus does not sit on is not the answer
Enter gives. In the confirmations that button is the one that changes nothing; in convert-or-keep
*both* outcomes commit by design ("each outcome one Checkpoint"), so Escape means [Keep as literal
text] — the answer that at least leaves the Value Type alone. Browse seeds from the field text
**literally**: a
`%VAR%` is not expanded to find a seed, because expansion is diagnostics' pass over the process
environment (§7) and a folder picker is not the place to start a second one.

## 7. Diagnostics

Settled by ticket [13](issues/13-diagnostics-contract.md);
facts [research/13](research/13-diagnostics-facts.md). **Six Issue types** (charting's five amended
on measured evidence). Issues are a derived view of the Working Copies — never part of them,
excluded from Checkpoints, recomputed after any edit, undo/redo, Refresh or Restore.

> **FR-diag-split** (must, new) — the raw value splits on **every** `;`; quotes never protect a
> separator (matches `CreateProcessW`/`SearchPathW`, PowerShell, Python).

> **FR-diag-normalise** (must, new) — Normalisation is comparison-only, never stored, never
> written, never touches the filesystem (no 8.3, no symlinks, no canonical casing): strip one pair
> of surrounding `"` → expand `%VAR%` (`ExpandEnvironmentStringsW`, process environment; unknown
> names stay literal) → `/`→`\` → trim trailing `\` unless that leaves a bare root (`C:\` stays) →
> compare ordinal case-insensitively.

> **FR-diag-duplicate** (must, rewritten) — equal Normalisations are duplicates. Evaluation order
> is the runtime order: System Working Copy first, then User, left to right; the first occurrence
> is canonical and clean, **every later copy flags `Duplicate`**, cross-scope included (the User
> copy carries it) — so a System edit recomputes User's Issues too.

> **FR-diag-missing** (must, rewritten from FR-diag-nonexistent; `os.Stat()` → Rust `std::fs::metadata`
> on the normalised text) — local-rooted Entries only (root classified via `GetDriveTypeW` / UNC
> prefix — no network round trip): flag `Missing` when the quote-stripped expanded path does not
> name an existing **directory** — not-found and exists-but-is-a-file both flag; `ERROR_ACCESS_DENIED`
> does **not**. **Network-rooted Entries are never probed in v0.1.0** and never flag (a dead UNC
> blocks 20–60 s uncancellably; documented in the README). An undefined `%VAR%` flags Missing
> naturally.

> **FR-diag-relative** (must, rewritten) — flag `Relative` on any Entry not fully qualified.
> Qualified: `X:\…`, `\\server\share…`, `\\?\…`. Flagged: `.`, `..`, bare names, rooted `\foo`,
> drive-relative `C:foo`. Relative Entries skip the existence check.

> **FR-diag-empty** (must, rewritten) — flag `Empty` on a zero-length or whitespace-only Entry. An
> Absent or empty Scope decodes to zero Entries and reports nothing; a trailing `;` produces a
> genuine empty Entry and does flag.

> **FR-diag-quoted** (must, new) — flag `Quoted` on any Entry containing `"`. Measured: the quoted
> spelling is dead for `CreateProcessW`/`SearchPathW`, PowerShell, `where`, Python; alive for
> cmd/CRT/Rust/Node — silent breakage, trivial fix.

> **FR-diag-overlength** (must, rewritten from FR-diag-length) — scope-level, never per-entry, never
> in the Status column, never an Announcement. The merged length is
> `len(expand(System WC) + ";" + expand(User WC))` in UTF-16 code units, shown always in the
> StatusBar's right field (§12). At Apply, if the post-write merged length exceeds **8,191**:
> warning dialog, title "cmd.exe will ignore a PATH longer than 8,191 characters ({n} after this
> Apply)" **[assembly]**, buttons [Apply Anyway] [Cancel] (KB 830473; proceeding is legal). At
> **≥ 32,767**: title "PATH cannot exceed 32,767 characters ({n} after this Apply)" **[assembly]**,
> single [Cancel] — no proceed (hard cap). No 2,047 warning (folklore).

> **FR-diag-async** (must, rewritten from FR-auto-diagnose) — a pass runs on one worker thread over
> the Working Copies (never the process environment, never the registry); results reach the UI via
> an `mpsc` channel drained by a wx **Timer** (~100 ms, running only while a pass is outstanding).
> Widgets are never called off the UI thread (they would silently no-op — thread-local registry).
> Runs at load and after every Working Copy change. Budget: full pass < 1 s for ≤ 200 entries.

> **FR-diag-status** (must, rewritten from FR-listview-columns) — the Status column carries the
> flagged types' words, comma-joined most-severe-first: **`Missing` > `Relative` > `Quoted` >
> `Duplicate` > `Empty`** (uk: Відсутній, Відносний, У лапках, Дублікат, Порожній). Coexistence:
> Empty is exclusive; Relative and Missing never co-occur; Quoted co-occurs freely. An empty column
> is the only healthy state — never "OK", no severity prefix, no icons. NVDA reads
> "{path}; Status: {types}" for free on every arrow key.

## 8. Backups and restore

Settled by ticket [14](issues/14-backup-restore-contract.md);
[ADR-0006](../../docs/adr/0006-snapshot-schema-is-decoded-not-raw.md).

> **FR-backup-auto** (must, rewritten) — Apply backs up **the value just re-read from the registry**
> (never the Baseline) before writing, into
> `data\backups\YYYY-MM-DDTHH-MM-SS-<Scope>.json` — **local time, Scope in the name**, numeric
> suffix on same-second collision (`…-System-1.json`). Written temp+rename with a `.tmp` extension
> mid-write. Schema (human-readable JSON, exactly one of `entries`/`absent`):
>
> ```json
> { "timestamp": "2026-08-19T14-32-07", "scope": "System", "valueType": "REG_EXPAND_SZ",
>   "entries": ["C:\\Windows", "%JAVA_HOME%\\bin"] }
> ```
> ```json
> { "timestamp": "2026-08-19T14-32-07", "scope": "System", "absent": true }
> ```

**Amended by impl ticket 10** — four rules the implementation had to fix, none of them widening
what is written, all of them about what is *accepted* when read back:

- **`valueType` belongs to the `entries` shape**, as the two schemas print: an Absent Scope has no
  value and so no Value Type, and requiring one would make the second schema's own file Corrupted.
- **A field this version does not know is ignored on read, never Corrupted** — a v0.2 field must
  not make today's Snapshots unrestorable in the version that wrote them.
- **A UTF-8 BOM is stripped before parsing**, the same reading `settings.json` gets (§13): several
  Windows editors add one, and a backup lost to an invisible character would be unexplainable.
- **The Scope and the extension in a file name are matched case-insensitively.** Windows names one
  file either way, so `…-SYSTEM.JSON` is that Snapshot, not a foreign file. The `scope` *field*
  inside the file stays exact — that is JSON content, where `system` is simply a different string.

Suffixes only climb: a suffix rotation has freed is never reissued, because within one second the
suffix **is** the age, and handing a fresh Snapshot an old name would have the rotation that
follows the write delete the backup that write just took.

> **FR-backup-rotation** (must, rewritten) — `maxBackups` (default 50, valid domain **≥ 1** — §13)
> is an **independent per-Scope budget**, identified from the filename alone; the oldest of that
> Scope is deleted on overflow. Corrupted files count toward their Scope's budget and rotate like
> valid ones. Rotation tolerates files already deleted by another instance.

> **FR-backup-ui** (must, rewritten) — the Backups tab lists Snapshots: date/time, Scope, entry
> count; files failing two-layer validation (parse; then shape: `timestamp` string, `scope`
> `System|User`, exactly one of `entries` (string array, with `valueType` `REG_SZ|REG_EXPAND_SZ`
> beside it) / `absent: true`) show **`[Corrupted]`** as passive list text — never an Announcement — with
> Restore disabled per-row. Foreign files (wrong name pattern, `.tmp`) are silently invisible.
> **Restore loads the chosen Snapshot's Entries and Value Type into the target Scope's Working
> Copy as one ordinary Checkpoint — it never writes the registry directly** and therefore inherits
> Apply's pre-write backup. The PRD's confirm dialog is dropped — Restore is undoable
> (**[assembly]**, same reasoning as Delete). **[assembly]** After Restore the target Scope's tab
> is activated with focus on the restored list, so the operation is heard through focus. Restore
> to a non-writable Session (System unelevated; Read-only Data) is a disabled control.

**Amended by impl ticket 14** — four rules the tab had to settle, none of them changing what a
Snapshot is or what Restore does:

- **`[Corrupted]` stands in the entry-count column**, where a count would. It answers the same
  question — how many Entries restoring this file would load — for a file that cannot be read;
  a fourth column would be a fourth heading to say the same thing.
- **A Snapshot that cannot be *read* is Corrupted too**, not invisible. Unparsable JSON, a
  mistyped field and a file the OS will not open are one thing to the person looking at the row:
  nothing can be restored from it. Invisible would make the list disagree with the directory,
  where the file still occupies its Scope's rotation budget.
- **The list is newest first** — the reverse of the order rotation reads the directory in.
  Rotation wants the oldest; someone restoring wants the backup they took last.
- **Restoring a Snapshot of an Absent Scope loads no Entries, typed `REG_EXPAND_SZ`** — the type
  the first Apply will create (§4), because an Absent Scope recorded no Value Type and a Working
  Copy has no Absent state to restore into. Its entry-count column therefore reads `0`.
  **That round trip is lossy, and knowingly so**: `absent: true` keeps Absent and
  present-but-empty distinct *in the file*, which is what ADR-0006 added it for and what a person
  reading the file still gets — but restoring one and applying it creates a present, empty value
  rather than deleting the Scope's. It is the same loss reading an Absent Scope already takes (§4:
  the Working Copy is empty and typed `REG_EXPAND_SZ`), and closing it would mean an Absent state
  in the Working Copy and a delete path in Apply, neither of which v0.1.0 has. Recorded here rather
  than left for someone to rediscover from the code.

Apply-time backup failure (Announcement): "Apply failed — could not write a backup, no changes
were made." `winget uninstall` deletes `data\`, Snapshots included; `winget upgrade` keeps it —
README material.

## 9. Elevation and failure taxonomy

Settled by ticket [12](issues/12-elevation-model.md);
[ADR-0005](../../docs/adr/0005-elevation-by-whole-app-relaunch.md).

> **FR-uac-elevation** (must, rewritten — no InlineAlert; the wx widget vocabulary replaces WinUI's) —
> elevation is a **whole-app relaunch**, never a write helper. Detection:
> `GetTokenInformation(TokenElevation)` once at startup (never `TokenElevationType`). **One entry
> point**: the menu command **"Restart as Administrator"**, disabled when elevated. The command
> runs through the close-confirm flow when anything is dirty — title "Discard unsaved User changes
> and restart as administrator?", buttons [Discard and Restart] [Cancel] — then
> `ShellExecuteEx("runas", <current exe>, "--tab <active>")`; on success the original instance
> **exits**; on `ERROR_CANCELLED` (1223) a dialog, title "Elevation was cancelled — still running
> without administrator rights", [OK], focus returns. The elevated window title is
> **"Administrator: PathMaster"**. Unelevated, the System Session is non-writable (every editing
> action disabled — §5); the System tab and Read-only Data name reasons but never grow a second
> elevation offer.

*Amended by impl ticket 17, which built the relaunch.* The argument is spelled
`--tab user|system|backups` — the Backups tab is a tab the user can leave, so it crosses too — and
a value the parser does not recognise reads as a plain launch (the User tab), never a guess; the
writer and the reader are one type in `pathmaster-platform`, so the two instances cannot drift
apart about the spelling. The dedicated discard dialog deliberately offers **no [Save]**: it is
the relaunch's own, its two buttons are its two outcomes, and the standard close-confirm keeps its
three. On success the original instance exits **through the ordinary close path** — geometry
written, `shutdown: clean` logged — with the already-answered question not asked twice. A spawn
failure that is not `ERROR_CANCELLED` keeps the application running and earns one
`ERROR elevation:` log line with the raw code; the on-screen reporting stays with
`ShellExecuteEx`'s own error UI, which is deliberately not suppressed — and on the one path that
fails before that call can show anything, a process that cannot name its own executable, the log
line is the only witness. Only the declined prompt is ours to answer, and the dialog above answers
it. The elevated instance honours `--tab` before the
notebook's handlers exist, so opening on the tab the user left announces nothing — the same
silence a plain launch opens with.

**Apply failure taxonomy** (the five rows; invariants: **no failure mutates the Working Copy, none
moves the Baseline**, every failure lands one log record with the raw error code):

| Failure | User sees (exact text **[assembly]**) |
|---|---|
| Re-read fails at Apply | Announcement: "Apply failed — {cause}" — the registry-write row's own text, because nothing was written either way |
| Snapshot write fails at Apply | Announcement: "Apply failed — could not write a backup, no changes were made." |
| Registry write fails (access denied / key unopenable / locked) | Announcement: "Apply failed — {cause}", e.g. "Apply failed — access denied." |
| External edit detected at re-read | the §5 three-button dialog |
| Broadcast returns 0 / times out | **nothing** — not a failure; `WARN` log line only |

> *Fifth row filled in by impl ticket 13* ([ADR-0008](../../docs/adr/0008-apply-sequence-lives-in-platform.md)).
> A re-read that fails was named by neither this section nor the ticket, and it is the first step of
> FR-apply's fixed order — so it needed a row before Apply could be written. Proceeding without the
> comparison was rejected outright: it is precisely the case where an external change is overwritten
> with no dialog, which is the hazard the whole order exists for. This is also the row §5's FR-refresh
> was waiting on: Refresh has failed silently since impl ticket 11 for want of a name, and moves onto
> this text once it exists.

## 10. Accessibility contract

Settled by tickets [09](issues/09-accessibility-contract.md) (contract),
[08](issues/08-live-announcement-mechanism.md) (mechanism),
[02](issues/02-nvda-baseline-stock-shell.md) (baseline), [01](issues/01-wxdragon-accessibility-surface.md)
(binding surface); [ADR-0003](../../docs/adr/0003-no-accessibility-calls-except-announce.md).
The contract in one sentence: **everything must-hear rides a channel NVDA is measured to speak —
the Status column, visible labels, dialog titles, and one `announce()` function — and nothing else
speaks at all.**

- **Zero `set_accessibility_*` calls in v0.1.0.** Every interactive element has a visible text
  label read by the native comctl32 path (the PRD's `AccessibleName`/`AccessibleDescription` is
  WinForms vocabulary with no wx counterpart — rewritten to this). The first such call on a widget
  swaps its plumbing with unknown effect; any future label is a **re-measure against the ticket-02
  baseline**, never an assumed improvement.
- **One `announce(text)` function** and nothing else fires accessibility events: set the label of
  the Banner's dedicated `StaticText`, then
  `NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED, hwnd, OBJID_CLIENT, CHILDID_SELF)`. Measured: it
  speaks verbatim, every time, repeats included, focus anywhere. Every other candidate is measured
  dead (`NAMECHANGE`/`ALERT`/`SHOW`, `UiaRaiseNotificationEvent`, the wx route, the status bar).
- **Announcements are a closed seven-item catalogue** (§10.1), each with a visible home on the
  Banner — no audio-only messages. The status bar stays command-only (`NVDA+End`); nothing
  must-hear goes there.
- **Dialog discipline**: NVDA never speaks a `MessageDialog` body — all critical information lives
  in the **title and buttons**. Every dialog text in this spec obeys it.
- **Keyboard**: the Tab order is the whole map — tabs → list → buttons, full traversal, no traps;
  Ctrl+Tab switches Scope tabs; **no F6** (by decision). Focus never jumps without a reason: after
  Apply — stays on the current Entry; after Refresh — same id / nearest neighbour / list; after
  any dialog — the control that opened it. Disabled Apply/Cancel read as disabled via the menu.
- **Row position** ("3 of 12") is NVDA's setting, never compensated for; entry counts come from
  Announcement 1. The PRD's index column is dropped — the list has two columns, Path and Status.
- **Colour**: **the application never sets a colour, anywhere** (Banner included). This satisfies
  US-high-contrast (native controls inherit the system theme) and rewrites NFR-accessibility-wcag —
  4.5:1 is a property of the system theme, untestable-by-us; the testable criteria are *no
  colour-setting call in the code* and *no information whose only carrier is colour*.

### 10.1 The Announcement catalogue — exact English **[assembly]**

Closed at seven. Canonical English below (msgids); Ukrainian ships in the Catalogue.

1. **Scope tab activation and Refresh** — "User PATH: {n} entries" / "System PATH: {n} entries";
   zero case is its own msgid: "User PATH: no entries" (closes the empty-list baseline gap; no
   placeholder rows).
2. **Apply succeeded** — "User PATH applied" / "System PATH applied".
3. **Apply failed** — the §9 taxonomy texts.
4. **Undo/Redo** — "Undone: {operation}" / "Redone: {operation}". Operation names (distinct English
   from the buttons that perform them — a ticket 11 D14 requirement): "Add entry", "Edit entry",
   "Delete entry", "Move entry", "Discard changes", "Change value type" **[assembly]**, "Restore
   snapshot" **[assembly]**. No path text in the announcement — focus lands on the row and NVDA
   reads it.
   *Amended by impl ticket 11: the Cancel operation was listed as "Cancel", which the same
   sentence's own rule forbids — three meanings need three English strings, and the third is
   the dialog button "Cancel"/«Скасувати» that every modal carries (ADR-0004). The command keeps
   §15's name one word longer ("Cancel Changes"/«Відхилити зміни») and the operation becomes the
   verbal noun D14 asks for ("Discard changes"/«відхилення змін»). "Add entry" and "Edit entry"
   double as the §6 dialog titles: identical English for one meaning, as ADR-0004 requires.*
5. **Undo across the Apply barrier** — item 4's text with the suffix ", unsaved changes".
6. **Cancel** — "Changes discarded".
7. **Read-only Data at startup** — "Read-only: {reason}", with the three §3 reasons **[assembly]**:
   "the application's own location is unknown" / "the data directory cannot be created" / "the data
   directory is not writable".

### 10.2 Verification

US-accessibility's acceptance criteria are replaced by the
**[Release Checklist](../../docs/release-checklist.md)** (canonical there; summarized: ticket 09's
17 steps naming expected speech, the ticket-10 dialog steps, the elevated-instance section, the
DPI-drag step, every NVDA step gated on the Sanity Check). Run personally by the user on real NVDA
before every release; a filled copy naming the NVDA used is attached to each release; a failed step
blocks the release.

## 11. Internationalisation

Settled by ticket [11](issues/11-i18n-mechanism.md);
[ADR-0004](../../docs/adr/0004-catalogue-text-is-load-bearing.md).

- **Mechanism**: `wxTranslations` with `.mo` embedded through a custom `TranslationsLoader`
  (`include_bytes!` from `OUT_DIR`); no Rust i18n crate. Visible labels and Announcements share one
  `translate()` — "one Catalogue" is structural. `add_std_catalog()` is **never called**; every
  dialog whose button text carries meaning uses our own buttons (a `MessageDialog` cannot relabel
  its own — the only stock dialog left is validation's single OK).
- **msgids are English source text** and that English is an API surface: where two strings mean
  different things their English must differ (Cancel-the-command vs Cancel-the-button). Symbolic
  keys rejected — a miss returns the msgid, and a key would be spoken aloud.
- **Placeholders are named braces** `{n}`, `{operation}` — `%d` is indistinguishable from `%VAR%`
  in this domain. Substitution is one explicit helper.
- **`.po` committed in `i18n/`; `.mo` generated at build time by `polib`** (pure Rust, no `msgfmt`
  pin); `build.rs` enumerates `i18n/*.po`. Adding a language: drop `xx.po` in, then name it in
  `pathmaster-core::language` (a variant with its code and endonym, its stored form, and the
  resolution arm), rebuild — the PRD's "without touching the code" is rewritten, not pretended
  satisfied. Ukrainian `.po` carries `Plural-Forms: nplurals=3`.
- **Interface Language** resolves by a two-way branch: system language `Ukrainian` → `uk`,
  everything else → `en` (English is the fallback, not the default). The system language comes
  from **Windows** (`GetUserDefaultUILanguage`, in `pathmaster-platform`), never from
  `Locale::get_system_language()`: wxdragon's `Language` enum mirrors wxWidgets 3.2 and the
  vendored 3.3.3 renumbered `wxLanguage`, so that call cannot answer `Ukrainian` (ticket 11 D3,
  amended by impl ticket 06). `settings.json` takes `"auto" | "en" | "uk"` and records the
  **choice, not its outcome**. Startup order: Data Directory → settings → translations → UI →
  writability → announce. In Read-only Data the selector is disabled and reads as disabled.
- **Accelerators belong to the code, never the Catalogue**: `wxAcceleratorTable` is absent, so the
  label string *is* the binding — the Catalogue holds `"&Undo"`, the code appends `"\tCtrl+Z"`.
  Ukrainian mnemonics keep the Latin letter in parentheses: `"Файл(&F)"`. Languages are listed by
  endonym ("English", "Українська").
- **The completeness gate** is a plain `#[test]` over a registry of msgid constants: presence via
  `get_string(…).is_some()` (never `translate(s) != s`), plural presence, placeholder integrity,
  per-menu mnemonic uniqueness, self-sensitivity. Fuzzy entries are excluded from `.mo` by gettext
  and so read as missing for free. Split across crates per §17.
- **Never translated**: registry paths, file names, the `WM_SETTINGCHANGE` payload, and **the
  entire log** (a diagnostic artifact — no `translate()` on any logging path). Issue-type words
  **are** translated (they are interface).
- **FR-i18n-runtime** (must, rewritten): language applies after restart; the restart notice rides
  the selector's own label — **"Language (takes effect after restart)"** — so the Announcement
  catalogue stays closed. `maxBackups` applies immediately.

*Amended by impl ticket 16, which built the selector.* It offers one item per value the file can
store, `auto` first — a choice the file accepts but the dialog cannot reach would be one the user
could leave and never come back to by hand. **The auto item is Catalogue text and the two languages
are not**: it names a rule (follow the system) rather than a language, while an endonym translated
into the current Interface Language would be the one item in this list a user who cannot read that
language could not find. The order is written down once, and the dialog reads its answer back by
position, so a label list that could differ in length or order from the choices it stands for would
be a selector answering with a language nobody picked. Nothing re-translates a running window:
committing a new language leaves the menu bar, the tabs and the Banner exactly as they were.

## 12. Window layout, sizing, iconography

Settled by ticket [17](issues/17-window-layout-and-iconography.md).

- **Layout**: one vertical `wxBoxSizer` — the **Banner above the notebook** (always visible, fixed
  height, its `StaticText` empty at rest; the layout never reflows under the user), notebook at
  `proportion=1, wxEXPAND`, native status bar attached to the frame outside the sizer. Tabs:
  "User PATH", "System PATH", "Backups"; "User PATH" active at startup (FR-view-tabs kept).
- **Sizing** (NFR-window-sizing rewritten): first run **900×650 DIP**, minimum 800×600; the list
  fills its tab; the **Status column is the app's single deliberate pixel constant** and the single
  explicit `FromDIP()` call; the Path column takes all remaining width. Maximize supported.
- **Geometry persistence**: position, size, maximised state in `settings.json`, written on clean
  shutdown only; restored clamped to the connected monitors' work area; fully off-screen → default
  size centred on primary. The implicit FFI `FromDIP` is accepted as structural; the cross-monitor
  DPI-drag hazard is a documented risk with a Release Checklist step.
- **StatusBar** (FR-statusbar rewritten; command-only, absent from the Tab order, answered by
  `NVDA+End`; a field cannot be styled — text carries everything): **field 0 (left)** — general
  status: "User PATH: {n} entries ({m} issues) | System PATH: {n} entries ({m} issues)"
  **[assembly]**, updated after every diagnostic pass and Apply; **field 1 (right)** — the passive
  merged-length field: "Merged PATH: {n} chars" **[assembly]**, with " — exceeds 8,191 (cmd.exe
  limit)" appended past that threshold **[assembly]**. In Read-only Data field 0 names the mode
  and reason.
- **Icon**: a stylised path motif; **two assets, one source design** — an embedded SVG via
  `BitmapBundle::from_svg_data` → `Frame::set_icon()` (window), and the `.ico` exe resource via
  `llvm-rc` (16/24/32/48/256, 256 PNG-compressed) for Explorer/taskbar. **No other in-app
  iconography**; the Banner is purely textual; nothing anywhere sets a colour.

**Amended by impl ticket 15**, which built the geometry persistence above — five rules the
requirement does not state:

- **The clamp is to one monitor, the one showing most of the window**, never to the union of them
  all. The union has holes: two monitors of different heights leave a region inside the bounding
  box and on no screen, and a window clamped into it would be exactly the unreachable window the
  clamp exists to prevent. Position moves first; the size is cut only where it does not fit at all.
- **Sharing an edge is being off-screen.** A remembered window whose rectangle meets a work area in
  zero pixels is nowhere, and takes the default place like any other window that is nowhere.
- **The work area, never the monitor's full rectangle** — a window restored under the taskbar has
  its own title bar out of reach.
- **A run that can see no monitors takes the default place**, which is also what a failed
  enumeration reads as. Nothing has gone wrong from where the user sits: the window opens where a
  first run's window opens.
- **A minimised window is not written at all.** Windows parks one far off every monitor and
  reports that as its position, and its maximised state reads `false` whatever it was — so
  recording it would replace a good remembered geometry with one the next start can only read as
  off-screen. The file keeps what it already said, which is the last place the user could see the
  window.
- **Physical pixels at both ends, and the maximised rectangle is recorded beside the flag.**
  wxdragon routes a *builder's* size through its implicit `FromDIP` and `set_size_with_pos`,
  `get_position` and `get_size` through nothing, so the round trip neither scales nor unscales, and
  the restore sets the geometry on the built frame rather than handing it to the builder. A window
  closed maximised records the geometry wx reports *while* maximised; restoring sets that and
  maximises over it, so an un-maximise afterwards lands on a window the size of the screen rather
  than on nothing at all.

**Amended by impl ticket 18**, which built the icon — two assets from one source design, and
three rules the requirement does not state:

- **The `.ico` is generated, never drawn.** `tools/make-icon.ps1` rasterises `icon.svg` into the
  five layers and assembles the file; the result is committed because the build reads it, but
  nothing hand-edits it. "One source design" is otherwise a rule someone has to keep, and the two
  assets would drift the first time one of them was touched.
- **The 256 layer costs a hand-written ICO writer.** ImageMagick refuses a 256 px layer outright
  (`InvalidDimensions`) and will write 255, which is not a size Windows looks for. The format is a
  6-byte directory, one 16-byte entry per layer and then either a headerless BMP or — for 256 — the
  PNG file verbatim with its width and height written as `0`, which is how 256 is spelled in a byte.
- **The two icon surfaces fail independently.** The exe resource governs Explorer, Alt+Tab from the
  file and pinned shortcuts; `Frame::set_icon` governs the title bar, the taskbar button and Alt+Tab
  from the window. A build can get either one right alone, and the Release Checklist looks at both
  (L8, L9) because the file's icon looking right is exactly what hides the other half being wrong.

## 13. Settings and its failure taxonomy

Settled by tickets [20](issues/20-failure-taxonomy-remainder.md) and [11](issues/11-i18n-mechanism.md).

> **FR-settings-file** (rewritten — **the PRD is overridden**: no silent in-place reset, no
> StatusBar-only warning) — `data\settings.json`, hand-editable, holding `language`
> (`"auto"|"en"|"uk"`, default `auto`), `maxBackups` (int **≥ 1**, default 50; `0` outlawed —
> rotation at zero deletes the pre-Apply safety net), and window geometry (§12). Absent file =
> first run: defaults, no dialog, no log line; created on first natural write.
>
> - **Parse layer, all-or-nothing**: unparsable JSON or a non-object root → the file is renamed to
>   `settings.json.bad` (temp+rename, single copy, next incident overwrites it; no rename in
>   Read-only Data), the run uses full defaults, and **one startup dialog** shows — title
>   "Settings could not be read — defaults are in use", [OK].
> - **Field layer, per-field**: an invalid value of a known field (`maxBackups: -3`,
>   `language: "fr"`) falls back to its default **in memory** while the file keeps the raw value
>   until the user changes that setting in the UI (the choice-not-outcome rule; a v0.2 value
>   survives a v0.1 run). Clamping rejected. Witnessed by one `WARN` log line each — no dialog, no
>   Announcement.
> - **Unknown fields are ignored and preserved** through every rewrite.

**Amended by impl ticket 07** — three rules the implementation had to fix and the requirement above
does not state:

- **The set-aside answers bad *contents*, never a file this run could not open.** A lock held by the
  other instance (two instances are a designed state, §3) or a denied ACL leaves the run on
  defaults and still shows the dialog, but the file is not renamed: moving a good `settings.json`
  onto the single `.bad` copy would destroy exactly what the set-aside exists to preserve. Bytes in
  hand that are not UTF-8 *are* bad contents, and are set aside like unparsable JSON.
- **One `WARN settings:` line records an unreadable file** — whether it was set aside or left in
  place. The dialog is the user's witness; without this line the log, which is the only diagnostic
  artifact a developer ever sees (§14), is silent about a file that moved on disk.
- **Geometry is one field.** `window` is a record of five members (`x`, `y`, `width`, `height`,
  `maximised`) and falls back as a unit under that one name, because half a position is not a place
  to put a window and the members have no individual defaults to fall back to. A non-positive
  `width`/`height` is invalid like any other out-of-domain value — and, like any other, is not
  clamped.

**Amended by impl ticket 15**, which added the file's first natural writer — the geometry written
on a clean shutdown. Two rules the taxonomy above, which is about *reading*, does not cover:

- **A write is one atomic replace of the whole document**, so the other instance (two are a
  designed state, §3) never reads a half-written file and a failed write leaves the previous one
  intact. It amends the document rather than serialising the settings, which is what carries a hand
  edit's unknown fields — nested ones included — and its key order through.
- **A write that fails earns one `WARN settings:` line and nothing else.** No dialog and no
  Announcement: nothing the user asked for failed, the previous file is untouched, and on the
  shutdown path the window is already going, so a dialog would outlive what it is about. The log is
  the only witness a setting that silently never persists can have.

**[assembly]** The Settings dialog (Tools → Settings…) holds the language selector (label
"Language (takes effect after restart)", endonym items, disabled in Read-only Data) and the
`maxBackups` field; our own OK/Cancel buttons.

**Amended by impl ticket 16**, which built that dialog — the file's second natural writer, and the
first one the user asks for. Four rules the taxonomy above, which is about *reading*, does not give:

- **Only the settings the user changed are written**, compared setting by setting. This is the
  choice-not-outcome rule at the one moment it can be lost: a `language` the file kept because this
  version could not read it stands in the document while the dialog shows the default that replaced
  it *in memory*, so an OK over an untouched selector must leave the raw value standing. Writing
  both settings on every OK would quietly downgrade a v0.2 file to whatever v0.1 happened to be
  doing, which is exactly what keeping the raw value exists to prevent — and it is per setting, so
  a run that changes the budget leaves an unreadable `language` alone and vice versa.
- **An OK that changes nothing writes nothing.** Dirty is a comparison and not a record that
  something happened (`CONTEXT.md`), and the same reading applies here: a user who retypes the
  value already in the field has changed no setting. So a hand-edited file is not reformatted by
  being looked at, and a first run does not gain a `{}` nobody asked for.
- **The run adopts what the file took, never the other way round.** The document is amended on a
  copy, written, and only then becomes what the window holds. Two things follow, and both matter.
  The file and the run can never disagree about what the settings are. And a write that failed
  leaves the difference standing, so the next OK is a change again and can be retried — where
  recording in memory first would leave the second attempt comparing equal and writing nothing,
  which is precisely the state a user whose setting did not persist must not be left in. The
  condition that fails this write is one §3 calls a designed state: the other instance holding the
  file.
- **A failed OK earns the `WARN settings:` line *and* a dialog** — "Settings could not be written —
  nothing was changed", the mirror of the unreadable-file dialog above, with the whole message in
  the title and the stock [OK]. This is the shutdown writer's rule **not** extended, and the
  difference is the whole reason: nobody asked for the geometry write and the window is already
  going, while here the user asked, is still sitting in front of the application, and — because
  nothing was adopted — gets nothing at all for having pressed OK. It is not an Announcement; that
  catalogue stays closed at seven.
- **The valid domain is one rule read twice.** What the field accepts and what the file accepts are
  the same predicate — whole, and ≥ 1 — so the dialog cannot come to accept a budget the file
  would reject: a value the user chose, saw written and lost at the next start, with a `WARN` line
  as its only trace. The field is validated on commit like a path, keeping its text and its focus
  on a rejection (§6), and **`0` is refused rather than clamped** here for the reason it is refused
  there.

Two things the dialog decides that the requirement leaves open. **In Read-only Data the OK button
is disabled too**, not only the two controls: that run has no write path, so OK is a button with
nothing to do, and what is left is a dialog the settings can still be read out of and a Cancel.
And **the budget's label says "Snapshots"** — `CONTEXT.md` reserves *Backup* for the act of taking
one and for the directory they live in, which is what the tab and the Tools item are named for,
while what a budget counts is files.

And one consequence of the first rule above, named here rather than left to be discovered: **a
value this version cannot read is cleared by choosing something else and choosing back.** A file
saying `"language": "fr"` shows the selector on the auto choice, because that is the fallback the
run is using — so a user who *wants* auto changes nothing by selecting it, `"fr"` stands, and the
`WARN` line recurs at every start, which is true: the file really does still say something this
version cannot read. Clearing it takes two trips through the dialog. The alternative is writing the
language on every OK, which is the rule above deleted.

## 14. Logging

Settled by ticket [21](issues/21-log-format.md) (supersedes ticket 07's 5 MB/`.log.1` sketch).

> **NFR-logging** (rewritten) — `data\pathmaster.log`, human-first plain text, one record per line:
> `<RFC 3339 local+offset> <LEVEL> <area>: <message>` (e.g.
> `2026-08-19T15:36:31+03:00 INFO  startup: PathMaster 0.1.0, elevated: no, data: writable, language: uk`).
> Exactly three levels, five-char padded: `INFO ` (healthy skeleton), `WARN ` (survived by the app),
> `ERROR` (a user-requested operation failed; panic). English always, outside the Catalogue.
>
> - **Healthy-run skeleton (3–5 lines, never an empty file)**: startup (version, elevation, data
>   state, language); one audit line per Apply
>   (`INFO apply: User scope written, 14 entries, 512 chars, REG_EXPAND_SZ`); clean shutdown
>   (`INFO shutdown: clean`).
> - **Two absolute prohibitions (PII)**: no Entry/PATH text in any record, and no absolute
>   filesystem paths in any record — only derived facts (counts, lengths, Value Type, Scope,
>   `data: writable` — never the location). Rejected settings values are logged (the only witness)
>   but truncated to ~100 chars with a marker.
> - **No logging failure touches the app**: every record an independent attempt (no latch); failed
>   writes silently dropped and counted; one `WARN log: N records were lost` on recovery. An
>   unopenable log at startup = a run without a log, never Read-only Data.
> - **Panics reach the log even under `panic=abort`**: a `std::panic::set_hook` hook appends one
>   `ERROR panic:` line (message + `file:line`, no backtrace — the PDB isn't shipped) directly past
>   the logger, best-effort, so it cannot recurse.
> - **Rotation only at open**: over **1 MB** → rename to `pathmaster.log.old` (single overwritten
>   generation); if the rename fails (another instance holds it), carry on appending.

## 15. Menus and keyboard **[assembly]**

Assembled per ticket 09 D5's delegation from the bindings tickets 10/12/13/14/17 fixed. The PRD's
five-menu structure shrinks with the v0.2.0 cuts (no View menu, no Fix Issues, no Restore Backup
item — the Backups tab covers it). Every label lives in the Catalogue; accelerators are appended by
code (§11); mnemonics per-menu unique (gated).

| Menu | Items |
|---|---|
| **File** | Apply `Ctrl+S` · Exit `Alt+F4` |
| **Edit** | Add Entry… · Edit Entry… `F2` · Delete Entry `Del` · Move Up `Alt+Up` · Move Down `Alt+Down` · Undo `Ctrl+Z` · Redo `Ctrl+Y` · Cancel · Refresh `F5` |
| **Tools** | Settings… · Open Backups Folder (a shell invocation, not a file dialog) · Restart as Administrator |
| **Help** | About |

Keyboard map (the README table mirrors it): Tab/Shift+Tab full traversal; Ctrl+Tab /
Ctrl+Shift+Tab between tabs; arrows in lists; F2 / Enter / double-click edit; Enter/Space activate;
`NVDA+End` reads the status bar. Apply/Cancel disabled while clean; every menu item's enabled state
reflects the active Session. Buttons per Scope tab: Add, Edit, Delete, Move Up, Move Down, Apply,
Cancel; Backups tab: Restore. No scenario requires a mouse.

*Amended by impl ticket 11, which built the Edit menu and the per-Scope buttons.* The table names
commands, not msgids; the shipped labels differ where ADR-0004 requires it. Menu items carry their
mnemonic ("&Add Entry…"), buttons carry none — the Tab order is the map and a button's `&` would
race the menu bar — and a `…` marks the two that open a dialog ("Add…", "Edit…"). The Cancel
command is **"Cancel Changes"** in both places, leaving "Cancel" to the dialog button that means
"do not commit". **A non-writable Session disables every Edit menu item except Refresh**, which
re-reads rather than edits: §5 disables "every editing action" and Refresh is not one, Read-only
Data "still reads, diagnoses and lists" (`CONTEXT.md`), and an unelevated System tab would
otherwise never see an external change without a restart.

*Amended by impl ticket 15, which added File → Exit and so completed the File menu.* Alt+F4 is
Windows' own gesture given a menu home: naming it on the item does not create the shortcut — the
system closes a window on Alt+F4 whatever any menu says — it makes the item **read** as the
shortcut it already is, which is the only way a screen-reader user learns of one (ADR-0004). The
item is **available in every state**, dirty Sessions and Read-only Data alike — which is this
section's own "every menu item's enabled state reflects the active Session" **overridden**, and
named as such. Exit is the second item that does not follow the active Session (Open Backups
Folder was the first, following the Run instead) and the only one that follows nothing at all: an
application a dirty Session could disable the way out of is one the user has to kill, and the
close-confirm is the whole of why they do not have to. The File menu's mnemonics are A and x.

*Amended by impl ticket 14, which built the Tools menu and the Backups tab's Restore button.*
Tools opened with **Open Backups Folder** alone; Restart as Administrator arrives with ticket 17,
and the mnemonics across the bar are F, E, T. The item shows `data\backups\`,
**creating it when this Run has not yet taken a Snapshot** and falling back to the Data Directory
when it cannot be created — a menu item that reads as available opens something. It is disabled in
the one run that has no Data Directory at all (§3, own location unknown). **Restore is a button and
nothing else**: §15 gives it no menu item and no accelerator, so it is not one of the commands the
menu bar carries, and its enabled state follows the focused Snapshot row rather than any Session
the notebook is showing.

*Amended by impl ticket 16, which added Tools → **Settings…** — first in the menu, as the table
above has it, and with no accelerator: §15 gives it none, and every shortcut in this application
must have a menu home rather than the other way round.* It is the third item that does not follow
the active Session, and like Exit it is **available in every state** — Read-only Data, the Backups
tab and a dirty Session alike. That is deliberate and not an oversight of "every menu item's
enabled state reflects the active Session": **the read-only state is answered inside the dialog**,
which disables its own controls (§13), because an item that read as unavailable would say the
settings cannot be looked at, and looking at them is exactly what that run can still do. The Tools
mnemonics are S and O.

*Amended by impl ticket 17, which added Tools → **Restart as Administrator** — last in the menu,
as the table above has it, with no accelerator and no `…`: what follows is the UAC prompt and a
relaunch, not a dialog of ours.* It is the fourth item that does not follow the active Session,
and the second to follow the Run (Open Backups Folder follows its Data Directory): it is enabled
exactly when the process is **not elevated** — every tab, Read-only Data and dirty Sessions
included, because the command is Read-only Data's standing remedy (§9 D7) and runs through the
close-confirm flow rather than around it. The Tools mnemonics are S, O and R.

*Amended by impl ticket 18, which added the **Help** menu and its one item, **About** — the
fourth menu and the last item on the bar, as the table above has it.* No accelerator and no `…`:
the `…` in this application marks the two items that open a dialog *asking* something, and About
states and is dismissed. It is the **fifth item that does not follow the active Session**, and the
first that follows nothing at all — not the Run, not a Scope: it names the build, which is true in
every state the application can be in, and a user checking what they are running is likeliest to do
it when something else has already gone wrong. The bar's mnemonics are F, E, T, H and the Help
menu's is A, alone in its menu and so unable to collide. The dialog is one sentence because it is
one title (§10 D6), carrying §16's three facts — name, version, licence — of which only the version
is filled in.

## 16. Build, packaging, release

Settled by tickets [04](issues/04-single-exe-build-profile.md) and
[15](issues/15-release-and-manifests.md); drafts ready to lift:
[research/15-packaging/](research/15-packaging/).

- **NFR-portable (kept, verified)**: `RUSTFLAGS=-C target-feature=+crt-static`; import table 19
  DLLs, no `VCRUNTIME140*`/`MSVCP140`/`api-ms-win-crt-*`. Profile: `lto=true, codegen-units=1,
  panic=abort`, `opt-level` default (3 and "z" both measured bigger); `strip` moves zero bytes on
  MSVC — the 52 MB PDB is simply not shipped (kept as a CI artifact for symbolication).
  **NFR-exe-size**: ≤ 40 MB, CI-gated (measured 7.22 MB). **NFR-startup-time (kept)**: ≤ 2 s on
  SSD (measured 79.6 ms cold).
- **Identity**: `VERSIONINFO` via `llvm-rc` (the only identity an unsigned binary carries);
  `CompanyName` "Ruslan Iskov" matching `PackageIdentifier` **`RuslanIskov.PathMaster`**;
  **License: MIT**. Unsigned by decision; SmartScreen documented in the README with `Get-FileHash`
  verification against the `.sha256` sidecar.
- **Release shape**: bare `PathMaster.exe` + `.sha256` sidecar (`<hex64> *<name>`), no zip.
- **winget**: schema 1.12.0, three-file manifest, `InstallerType: portable`,
  `Commands: ["pathmaster"]` (names the Links symlink and renames the installed exe), submitted to
  `microsoft/winget-pkgs`. winget writes an HKCU ARP key (16 values) and puts its Links dir on the
  user PATH; `upgrade` keeps `data\`, `uninstall` deletes it — README material, not our promise
  broken.
- **scoop**: own bucket from BucketTemplate (excavator auto-bumps), bare-exe URL with
  `#/PathMaster.exe` rename, `bin` + `shortcuts` + `persist: "data"` (junction — compatible with
  the §3 resolve rule), `checkver: "github"`, autoupdate hash from the sidecar; scoop patches the
  shim to GUI subsystem — no console flash.
- **Release workflow**: tag `v*` → `windows-2025` (VS2026 image; LLVM/libclang + Ninja are
  load-bearing pins, `LIBCLANG_PATH` set explicitly) → three-way version gate (tag / `Cargo.toml` /
  `.rc`) → **`cargo fmt --check` + `cargo test` + `cargo clippy`, the gate push CI applies to
  `develop`** → build with `CARGO_TARGET_DIR=C:\t` (MAX_PATH — a deep path breaks the wxWidgets
  build while blaming the compiler) → **dumpbin gate failing on `VCRUNTIME|MSVCP|api-ms-win-crt`**
  → exe-size gate ≤ 40 MB → release via `gh`; PDB to CI artifacts only. **Rule: gate the artifact,
  never the build config** (`RUSTFLAGS` silently overrides `.cargo/config.toml`).
- **Still owed before the first release** (release-time actions, not open decisions): one clean-VM
  run with no VC++ redistributable; one live winget install observing the symlink-resolve and
  uninstall behaviour; the repo URL filled into the manifest drafts.

**Amended by impl ticket 18**, which built the pipeline and the manifests — four rules §16 does
not state:

- **The gates run on the staged file, and the sidecar is written after them.** "Gate the artifact"
  is only true if the gated bytes are the published bytes, so the copy into `dist\` happens before
  the gates rather than after; and a run that fails a gate leaves **no** `.sha256` behind, because a
  published hash is an invitation to install what it names.
- **A fourth gate: the identity, read back out of the linked exe.** A `.res` that failed to link
  leaves a perfectly working binary with no `VERSIONINFO` at all — no company, no version, nothing.
  For a binary that is unsigned by decision that is the whole of its identity, so the workflow reads
  `CompanyName`, `ProductName` and both versions off `dist\`'s file rather than trusting the `.rc`
  it compiled.
- **Two of the three version legs are checked on every `cargo test`, not only on a tag.**
  `crates/pathmaster-core/tests/versioninfo.rs` compares `Cargo.toml`'s version with the `.rc`'s,
  which is where drift is actually introduced — a bump is a `Cargo.toml` edit and the `.rc` is the
  file it is easy to forget. It sits in `core` and not with the resource it reads, because the
  check is pure text and the binary's tests link wxWidgets: ADR-0009's "the only test that links
  wxWidgets" stays true of the msgid smoke test alone. The tag is the leg only a release can
  check, and the three-way gate keeps it.
- **The release URL is the repository's, and that fixes the manifests.** `ruslan-rv-ua/pathmaster2`
  is where `gh release create` publishes, so it is what both manifests download from; the scoop
  bucket is its own repository (`ruslan-rv-ua/scoop-bucket`) and the manifests live at
  `packaging/`, submitted from there rather than consumed by anything here. The licence field both
  of them left open is **MIT**, which is also the `LegalCopyright` line in the exe.

**Amended 2026-08-24**, closing the gap the ticket-18 review named: **the release workflow runs the
tests itself.** §18 puts push CI on `develop`, but a release branch starts from `develop` and
finishes into `main`, and the tag lands there — so without this the commit a release is cut from is
one no CI has ever seen, and anything that branch carried went past the gate unexamined. The step
runs `--release --target`, matching the build below it, so wxWidgets is compiled once for the job
rather than once per profile (measured: one shared `wxdragon-sys` build directory, and 46 s of
tests on a warm tree). It runs **all three** of push CI's commands rather than the tests alone: a
shorter list would be a second thing to keep in step, and formatting that reaches a tag is itself
evidence that code got there without passing through `develop`. This is the one gate in §16 that is
**not** about the artifact — the artifact rule governs everything downstream of the build.

## 17. Repository layout

Settled by ticket [23](issues/23-crate-and-module-layout.md);
[ADR-0007](../../docs/adr/0007-crate-boundary-is-the-test-boundary.md).

**Three-crate Cargo workspace, flat matklad layout** (`crates/`, virtual manifest root holding the
§16 release profile). Dependency direction fixed: **bin → platform → core, never reverse.** No test
ever links wxWidgets.

- **`crates/pathmaster-core`** — pure, no I/O, any-OS: `path` (split/join), `normalize`,
  `diagnostics`, `session`, `snapshot`, `rotation`, `backups` (the Backups list: a Snapshot's name
  married to what reading its file turned out to be — impl ticket 14), `thresholds`, `settings`
  (parse + per-field rules), `logfmt` (line shape, truncation, levels), `language` (the stored
  choice and the §11 branch), `msgids` (registry + `.po` integrity gate via polib), `catalogue`
  (the injected lookup, the Announcement type, and everything composed out of the msgids — impl
  ticket 19, ADR-0009).
  Module names indicative; the inter-crate seams are what this spec fixes hard.
- **`crates/pathmaster-platform`** — imperative shell, no wx: `registry` (adapter, **key path as a
  constructor parameter**), `datadir`, `diagnostics` (the pass's two adapters and its worker thread,
  §7), `elevation`, `locale` (the system language, §11), `logwriter`, `panic_hook` (writes past the
  logger; core supplies only the line format), `settings` (the file in the Data Directory: read in
  both modes, set aside and written only in Writable Data — core owns the parse), `broadcast`,
  `snapshots` (the Snapshot files: `data\backups\` spelled once, and the one listing both the next
  name and the rotation are answered from — impl ticket 13), `geometry` (where the window opens: the
  monitors' work areas, and the clamp onto them — impl ticket 15), `apply` (the Apply Run — impl
  ticket 13,
  [ADR-0008](../../docs/adr/0008-apply-sequence-lives-in-platform.md)), `startup` (everything a Run
  is, decided in one place — impl ticket 20,
  [ADR-0010](../../docs/adr/0010-run-properties-decided-in-one-place.md)).
- **`crates/pathmaster`** — **bin-only, no lib target**: `ui/*`, `announce`, `pump` (Timer drain),
  `catalog` (TranslationsLoader), `main.rs` (the composition root: the located directory,
  `startup::decide`, the Catalogue, the window),
  `build.rs` (polib → `.mo`; llvm-rc → icon/VERSIONINFO), `i18n/*.po`,
  `resources/` (`icon.svg` — the design, embedded for the frame; `app.ico` — generated from it;
  `app.rc` — the one ICON statement and the VERSIONINFO — impl ticket 18).
  `[[bin]] name = "PathMaster"` — no CI rename step.
  **The binary gains no module for the icon or the version**: the frame reads the SVG where it
  sets it and the About dialog reads `CARGO_PKG_VERSION` where it speaks it, and the gate that
  keeps the `.rc` and `Cargo.toml` agreeing is `core/tests/versioninfo.rs` — pure text over two
  files, kept out of the crate that links wxWidgets so that ADR-0009's "the only test that links
  wxWidgets" stays true of the msgid smoke test alone (impl ticket 18).

`tools/` at the repo root is permanent: `nvda-drive.ps1` (promoted out of `.scratch/` by this
ticket), `make-icon.ps1` (the icon generator — impl ticket 18) and, when built, the ticket-24
`WM_GETOBJECT` watcher. **`packaging/`** joins it as a permanent root directory: the winget and
scoop manifests, which are submitted elsewhere and consumed by nothing here (impl ticket 18).

## 18. Test and verification strategy

Settled by ticket [19](issues/19-test-and-verification-strategy.md). **Functional core, imperative
shell** — the tiers, placed by §17:

- **Unit tests (core, in-module)**: splitting, Normalisation, all §7 rules and the severity order,
  the Session model (dirty-as-comparison, Checkpoint semantics, the Apply barrier), Snapshot
  schema + two-layer Corrupted validation, per-Scope rotation, the 8,191/32,767 threshold logic,
  settings parse + per-field fallback, the log line format, the msgid registry gate (polib half).
- **Property tests**: exactly three (`proptest`, dev-dependency of core alone), each in the test
  file of the module it constrains — split→join byte-identity in `core/tests/path.rs`;
  Normalisation idempotence in `core/tests/normalize.rs`; Snapshot round-trip of
  `(valueType, entries|absent)` in `core/tests/snapshot.rs` (ticket 23, amended by impl tickets
  02/09: a shared `properties.rs` separates each property from the rules it is about and from the
  examples that share its fixtures; the cap of three is untouched).
- **Registry integration tests (platform)**: plain `#[cfg(windows)]`, no opt-in gate, against a
  temporary key under `HKCU\Software\PathMasterTest` on the **live** registry (mocks rejected —
  ticket 05's hazards are about real API behaviour): `(vtype, bytes)` preservation, type
  round-trips, Absent as a distinct state.
- **One `get_string` smoke test in the binary** (the wx half of the i18n gate), run in CI where wx
  is built anyway.
- **The GUI shell is covered by the [Release Checklist](../../docs/release-checklist.md) only.**
  `nvda-drive.ps1` stays a measurement tool, never a CI gate (a deaf NVDA is indistinguishable
  from a regression; a flaky gate gets ignored). No UI automation (WinAppDriver dead, FlaUI drags
  .NET in).
- **CI**: push CI (`cargo test` + clippy + `cargo fmt --check` on every push/PR to `develop` — the
  format gate added by impl ticket 09, after drift reached `develop` unnoticed); release CI per §16.
- **Cadence**: exe size every release (automated); thresholds unit-tested every release, confirmed
  once against the real registry; cold start re-measured only when the startup path changes;
  clean-VM once per packaging change.

## 19. Documented open risk: the NVDA deaf-list state

Tickets [18](issues/18-nvda-deaf-on-listctrl.md) / [24](issues/24-deaf-state-detection-decision.md);
details [research/18](research/18-nvda-deaf-on-listctrl.md). Once, unreproduced, NVDA treated the
list as a leaf and announced nothing. Cause narrowed to OS-side winEvent delivery loss for the app
instance (plausible, unreported upstream). **v0.1.0 ships zero deaf-state code.**

- **Signature** (measured): focus change with no `WM_GETOBJECT (OBJID_CLIENT)` within ~1 s
  (threshold: "about one second, tune in the harness"), observable via `SetWindowSubclass` with
  zero accessibility code. The watcher lives in the **measurement harness only**, backing — never
  replacing — the manual Sanity Check (`NVDA+Tab` on a focused row must answer with the row).
- **Support ladder** (README + risk note): Alt+Tab away and back → restart the app → restart NVDA
  (guaranteed).
- **Warning**: `announce()` rides the same pipeline and is very likely silent in this state.
- Every NVDA measurement anywhere is void — not failed, void — unless the Sanity Check passed
  first. In-app detection is a v0.2.0 candidate, parked on the map.

## 20. Cut and deferred

Nobody re-adds these by accident; each carries its reason.

**Deferred to v0.2.0** (live in the tracker, not promised in the README): FR-reorder-dnd (Drag &
Drop), FR-var-expansion-toggle, FR-search, FR-tree-browser, FR-filter-bar, FR-fix-issues,
FR-copy-entry — all 🟡, out of the must-only scope. Also parked: a `--data-dir` switch (survives
the no-relocation principle); in-app deaf-state detection (§19); a network-path deadline prober;
and a **Help → Documentation item giving `F1` a menu home** — the **first v0.2.0 candidate**,
raised 2026-08-24, small in itself but carrying one open question, since the README is online and
the item would do nothing offline.
Plus two architectural deferrals from the 2026-08-20 review, neither blocking anything: **collapsing
`ScopeDiagnosis` into `Findings`**, so a pass arrives already paired to the Working Copy it ran over
instead of the window re-pairing it; and **making the UI's borrow discipline structural** rather than
a doc comment listing hand-checked call sites. Reasons in the map's Out of scope.

**Cut, not deferred**: similar-path/typo diagnostics (a false-positive generator —
`C:\Python312` vs `C:\Python313` are both legitimate; trust in diagnostics beats breadth); the
`theme` setting (system colours always — High Contrast is a Windows mode, not an app choice); the
PRD's index column (position is NVDA's setting; §10); code signing (until there are real users);
UI automation (§18).

**Out of scope, kept from the PRD**: OS-other-env-vars (only `Path` is read or written), OS-sync,
OS-plugins, OS-web-cli, OS-auto-update. Plus: non-Windows platforms, 32-bit Windows, screen
readers other than NVDA (not deliberately broken, not tested).

## 21. PRD deviation notes

Where this spec **overrides** the PRD outright (rewrites that keep intent are in §2):

1. **FR-settings-file** — the PRD's "overwrite the corrupted file with a valid version + StatusBar
   warning" is overridden: set-aside as `settings.json.bad`, startup dialog (title carries the
   message), per-field tolerance with raw values preserved (ticket 20). The StatusBar is
   unhearable; overwriting a hand-edited file is data loss.
2. **TC-wm-settingchange** — the PRD's 5000 ms is a spec bug (multiplies per top-level window):
   1000–2000 ms off the UI thread (ticket 05).
3. **FR-cancel** — "with no changes, cancels immediately" becomes "Apply and Cancel are disabled
   while clean" (ticket 06): a no-op button gives a screen-reader user no signal.
4. **FR-refresh's announcement** — "PATH refreshed" is superseded by the entry-count Announcement
   (ticket 09's closed catalogue).
5. **Inline editing** (FR-edit-f2) — replaced by a modal dialog (ticket 10); the end-of-edit event
   cannot be vetoed and the inline editor cannot host Browse.
6. **Delete's confirm dialog** — dropped (ticket 10); undo is the safety net. **Restore's confirm
   dialog** — dropped (ticket 14 + assembly): Restore no longer overwrites anything, it loads the
   Working Copy as an undoable Checkpoint.
7. **Backup filename and schema** — Scope in the name, local time, collision suffix; `valueType` /
   `absent` fields added (tickets 14, 05 H15); rotation per-Scope, not pooled.
8. **US-diagnose's six types** — the PRD's six included typos; this spec's six are Missing,
   Relative, Quoted, Duplicate, Empty, Over-length(scope-level) — typos cut, Quoted added on
   measured evidence (ticket 13). Status values "OK / Warning / Error" and status icons are
   dropped: type words only, empty = healthy.
9. **US-i18n "without restart" vs FR-i18n-runtime "after restart"** — the PRD contradicts itself;
   after-restart wins (charting), and the "add a language without touching code" premise is
   rewritten (ticket 11).
10. **NFR-exe-size** — 20 MB → 40 MB, below accessibility and portability in priority (charting).
11. **NFR-no-registry-writes** — a machine-wide claim is unachievable (Windows writes Amcache etc.
    because the exe ran); rewritten as a process claim with one named ComDlg32 exception
    (tickets 07, 10).
12. **WinUI vocabulary replaced**: `InlineAlert` → the disabled-state + menu-command model (§9)
    and the Banner (§10); `InlineBanner` → the Banner; `TabPages` → wx notebook tabs;
    `os.Stat()` → `std::fs::metadata` semantics per §7; `FolderBrowserDialog` → `wxDirDialog`;
    `AccessibleName`/`AccessibleDescription` → visible labels on the native path (§10).
13. **NVDA acceptance criteria** — every "NVDA announces X" line in the PRD is replaced by the
    Release Checklist's steps naming exact expected speech; per-entry announcements come from the
    Status column read on focus, not from events.
