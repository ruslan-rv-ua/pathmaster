# Map: PathMaster v0.1.0

Wayfinder map. Tickets are the files in `issues/`; the frontier is every ticket that is `open`, unclaimed,
and whose `Blocked by` list is fully `resolved`.

## Destination

A **locked, technically de-risked specification for PathMaster v0.1.0** — `spec.md` in this directory —
in which every decision needed to start building is settled, and every mechanism the product's
accessibility depends on has been proven against real NVDA rather than assumed.

Reaching it means: no open question stands between the spec and an implementation effort.
Building v0.1.0 is **not** part of this map; prototypes here exist to kill or confirm a decision and are thrown away.

## Notes

**Domain.** A portable Windows desktop app that reads, edits and diagnoses the `PATH` environment variable.
Source PRD (Ukrainian, verbatim, input only): [spec-input.md](spec-input.md). Where the PRD and a resolved
ticket disagree, the ticket wins.

**Stack (fixed, not up for reconsideration).** Rust + [wxdragon](https://github.com/AllenDang/wxdragon).
The user ruled this a hard constraint at charting. The accessibility question is therefore *how* to get
NVDA working inside wxdragon, never *whether* to switch toolkits.

**Settled at charting** (standing constraints for every session; not ticket decisions, so not listed below):

1. Destination is a spec, not a build. Wayfinder's plan-don't-do default holds.
2. Stack is a hard constraint (above).
3. NFR priority when they collide: **accessibility > portability (single exe, no runtime install) > exe size**.
   NFR-exe-size stays 🟡 and its budget is relaxed from 20 MB to **≤ 40 MB**.
4. v0.1.0 scope = **🔴 must only**, plus StatusBar, `settings.json`, and minimal logging.
   All other 🟡 features are deferred to v0.2.0 (see Out of scope).
5. NVDA verification is done **by the user personally** — prototype tickets are HITL and end in a real verdict,
   never in an inspector-tool guess.
6. i18n: language change takes effect **after restart**; `maxBackups` applies immediately.
7. Similar-path / typo detection is **cut** from v0.1.0; five diagnostic types remained at charting —
   later amended to **six** by ticket 13 (`Quoted` added on measured evidence).
8. The `theme` setting is **removed** — system colours always, High Contrast is a Windows mode, not an app choice.
9. Logging stays in v0.1.0, minimal.
10. Distribution: GitHub Releases + GitHub Actions, **unsigned** in v0.1.0 (SmartScreen accepted, documented in
    the README). Scoop via an **own bucket** with `persist: data`; winget submitted to `microsoft/winget-pkgs`
    as `InstallerType: portable`.
11. Portability: `data/` sits next to the exe; NFR-no-registry-writes is reworded from "nothing in AppData"
    to "**nothing outside the app's own directory**" — with **one named exception** decided in ticket 10:
    ComDlg32 MRU writes from the Browse folder picker (`wxDirDialog`), accepted and documented.
12. All artifacts (map, tickets, research, spec) are written in **English**; conversation with the user is Ukrainian.
13. Before asking the user questions in any HITL session, **research best practices on the internet first**
    (user directive, 2026-08-19) — bring the user informed options with evidence, not open-ended questions.

**Skills every session should consult.** `/grilling` and `/domain-modeling` for every grilling ticket;
`/research` for research tickets; `/prototype` for prototype tickets. Domain terms resolved by a ticket go
into `CONTEXT.md` at the repo root; a decision that is hard to reverse, surprising, and a genuine trade-off
earns an ADR under `docs/adr/`.

**A fact worth carrying into every session.** wxMSW does not draw its own controls — it wraps native Win32
comctl32 ones. A `wxListCtrl` *is* a `SysListView32`, a notebook *is* a `SysTabControl32`. NVDA therefore reads
them for free, including column text. That makes "announce the issue type" a **design** problem (put status in a
real column) rather than an accessibility-API problem. What native controls will *not* do for free is announce
transient, non-focus messages — which is why that has its own ticket.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Charting-time constraints live in Notes, not here. -->

- [wxdragon accessibility surface](issues/01-wxdragon-accessibility-surface.md) — **wxdragon already binds
  `wxAccessible` in full** (`AccessibleImpl`, `Accessible::notify_event`, five `set_accessibility_*` setters,
  Windows-only cfg), so forking is off the table. `wxUSE_ACCESSIBILITY` is derived ON and, once ticket 04
  produced a build, **observed as 1** — so the C layer's silent no-op `#else` branches are dead code. `get_handle()`
  exposes the `HWND` on every widget, so the direct `NotifyWinEvent` route is open too. There are no prebuilt
  binaries: wxWidgets 3.3.3 is compiled from pinned source, statically, and `crt-static` propagates into that
  C++ build. Pin wxdragon **≥ 0.9.17** (earlier `AccRole` discriminants are mis-ordered; the MSAA fixes came
  from a core NVDA developer). Details: [research/01](research/01-wxdragon-accessibility-surface.md).
- [PATH registry I/O semantics](issues/05-registry-io-semantics.md) — read and write raw through
  `winreg::get_raw_value` / `set_raw_value`, which preserve bytes **and** value type; never
  `set_value::<String>` (it writes `REG_SZ` unconditionally — the .NET bug that put `REG_SZ` `Path` on real
  machines in the first place). Preserve the existing type, never normalise. Missing value is a distinct state
  (`ERROR_FILE_NOT_FOUND`). 32767 is the per-variable limit, and the combined check must run on the
  **expanded merged** string. The `WM_SETTINGCHANGE` timeout is **per top-level window and multiplies**, so the
  spec's 5000 ms is a theoretical 18.8-minute freeze — use 1000–2000 ms off the UI thread. Detect external
  edits by re-reading `(vtype, bytes)`, never by the key's timestamp. 15 hazards catalogued, all of which
  produce a *successful* write with wrong content: [research/05](research/05-registry-io.md).
- [wxdragon widget inventory vs PathMaster UI](issues/03-widget-inventory.md) — wxdragon 0.9.18 over
  wxWidgets 3.3.3 can express the UI, and carries an ungated accessibility API (`set_accessibility_*`,
  `AccessibleImpl`, `Accessible::notify_event`) plus embedded-memory `.mo` translations. Seven requirements
  need rewriting: no veto on label-edit, no `wxInfoBar`, no system-colour access, no `wxAcceleratorTable`,
  no sub-item icons, no status-field styling, no `CallAfter`/`QueueEvent`. Widgets are auto-`Send` but
  resolve through a thread-local registry, so calling one off the UI thread silently no-ops.
  Full inventory: [research/03](research/03-widget-inventory.md).
- [Single-exe build profile](issues/04-single-exe-build-profile.md) — **every 🔴 must is reachable, none
  close to its limit.** `RUSTFLAGS=-C target-feature=+crt-static` satisfies NFR-portable: verified on the
  linked binary, the import table drops 32 → 19 DLLs, losing `VCRUNTIME140*`, `MSVCP140` and all eleven
  `api-ms-win-crt-*`. **7.22 MB against a 40 MB budget** with `lto=true, codegen-units=1, panic=abort`
  (`strip` moves *zero* bytes on MSVC — debug info is in a 52 MB PDB beside the exe, so "single exe" means
  not shipping it; `opt-level=3` and `"z"` both make it *bigger*). **Cold start 79.6 ms** vs a 2 s budget.
  Icon and `VERSIONINFO` demonstrated via `llvm-rc` — no new crate — but the **running window still has no
  icon** without an explicit `Frame::set_icon()`. **LLVM/libclang and Ninja are newly load-bearing CI pins.**
  Two traps: a deep checkout path breaks the build via MAX_PATH while blaming the C++ compiler, and
  `RUSTFLAGS` silently overrides `.cargo/config.toml` — so release CI must gate on the artifact's imports,
  never on the build config. Details: [research/04](research/04-build-profile.md).
- [Editing session model](issues/06-editing-session-model.md) — **two independent Editing Sessions, one per
  Scope**, each a Working Copy over a Baseline; the Backups tab is not a Scope, and a Session never survives a
  process boundary. **`dirty` is a comparison** (content vs Baseline), never a flag — so an edit and its exact
  reversal leave the session clean, and one predicate drives Apply, Cancel and close-confirm. Undo is a stack of
  whole-copy **Checkpoints**, not invertible commands ([ADR-0001](../../docs/adr/0001-checkpoint-based-undo.md)):
  one per user-visible operation, each carrying a focus hint, batches free, and **Cancel itself undoable**.
  **Apply is a barrier, not a stack flush** — Ctrl+Z after Apply moves the working copy only, never the registry.
  Apply's order is fixed: re-read → compare `(vtype, bytes)` → dialog → **back up what was just re-read, not the
  Baseline** → write → move Baseline. Refresh and "discard my changes" **clear the stack**; Apply and Cancel are
  **disabled while clean** (rewrites FR-cancel). An Entry is the **raw substring** plus an opaque id; the Working
  Copy owns the **Value Type**, and `%VAR%` typed into a `REG_SZ` scope raises an explicit convert-or-keep dialog
  — the single exception to never changing the type. Absent scopes are created `REG_EXPAND_SZ`; an empty value is
  **zero Entries, not one empty one**. Ubiquitous language: [CONTEXT.md](../../CONTEXT.md) — note **Snapshot**
  stays the backup file, so the undo step is a **Checkpoint**.

- [Portable data directory contract](issues/07-portable-data-directory.md) — the **Data Directory** is `data\`
  beside the exe, located by **resolving** `current_exe()`'s reparse points and stripping the resulting `\\?\`
  — not by trusting the launch path. Measured: `current_exe()` reports the **junction**, not its target, so the
  naive rule would put `data\` in `WinGet\Links\`, **shared with every other portable package**. When that
  directory cannot be written the app starts in **Read-only Data** — reads, diagnoses and lists, every Editing
  Session non-writable — and **never relocates**, because remembering a location outside its own directory
  requires writing outside its own directory ([ADR-0002](../../docs/adr/0002-resolved-data-directory-never-relocated.md)).
  **No single-instance lock**: elevation-by-relaunch makes two instances a designed state, so instead every
  replacement write is temp+rename, rotation tolerates missing files, and the log appends one line per record
  with **rotation only at open**. The **ACL fear was measured away** — inherited DACLs, not ownership, govern
  access, so an unelevated run still rotates what an elevated one wrote. NFR-no-registry-writes is rewritten as
  a claim about **the process** (Windows itself writes Amcache/Prefetch regardless), with a derived constraint:
  **no native file dialogs** in v0.1.0, since ComDlg32 MRU writes land under our process — and that is code
  discipline, invisible to the import table. `winget upgrade` keeps `data\`; `winget uninstall` deletes it,
  backups included. Rule for the whole startup path: **startup predicts, Apply verifies.**

- [NVDA baseline for a stock wxdragon shell](issues/02-nvda-baseline-stock-shell.md) — **the free ride is
  wide.** With no accessibility code at all, comctl32 announces list rows with **both columns and the second
  column's header name** (`'C:\scoop\shims; Status: Warning: Duplicate'`), plus window title, tab labels and
  selection, button names + roles + access keys, menu names, `'недоступно'` on a disabled item, `'позначено'`
  on a check-item, a focus order with no traps, and a status bar that answers `NVDA+End` with both fields.
  **Four gaps only**: an empty list says just `'список'` with no count; the status bar is **command-only**
  (absent from the Tab order, `F6` silent), so routing a message there hides it; row position is never spoken
  on this user's config; and anything not tied to a focus change is still ticket 08's problem. Two things must
  survive any refactor: `\t` in menu labels is *why* accelerators are spoken, and the first
  `set_accessibility_*` call moves a widget off this comctl32 path — so re-measure, never assume it only added.
  A measurement was lost to a state where NVDA treated the list as a leaf and announced nothing (now
  **[ticket 18](issues/18-nvda-deaf-on-listctrl.md)**); every pass now checks `NVDA+Tab` on a row answers
  `'елемент списку'` first. Harness: [tools/nvda-drive.ps1](tools/nvda-drive.ps1).
  Details: [research/02](research/02-nvda-baseline.md).

- [Live announcement mechanism](issues/08-live-announcement-mechanism.md) — **one event speaks, and it is
  enough**: `NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED, hwnd, OBJID_CLIENT, CHILDID_SELF)` on a dedicated
  message `StaticText` whose label was just set — verbatim, every time, repeating identical text, with focus
  anywhere, even while the widget is hidden. Every other candidate is dead: `NAMECHANGE`/`ALERT`/`SHOW` silent,
  `UiaRaiseNotificationEvent` **succeeds and is ignored**, the wx route (`notify_event` + Alert role) silent
  despite `wxUSE_ACCESSIBILITY=1`, the status bar unhearable even with an event fired at it. The rule: one
  `announce(text)` function, and nothing else fires accessibility events. Trap found on the design-away rung:
  a `MessageDialog` speaks its title and buttons but **never its message body** (→ ticket 09). Measured on
  NVDA 2025.3.3 / Win11 25H2, every pass gated on the ticket-18 sanity check.
  Details: [research/08](research/08-announcements.md).

- [Accessibility contract](issues/09-accessibility-contract.md) — **everything must-hear rides a measured
  channel, and nothing else speaks.** The Status column carries per-entry Issues: types only, no severity
  prefix, all of them comma-joined in a severity order (owned by ticket 13), empty for healthy — never "OK".
  **Zero `set_accessibility_*` calls in v0.1.0** — visible labels on the native comctl32 path plus one
  `announce()` are the whole strategy ([ADR-0003](../../docs/adr/0003-no-accessibility-calls-except-announce.md));
  any future label is a re-measure, not an improvement. Announcements are a **closed seven-item catalogue**
  (tab/Refresh entry counts, Apply success/failure, undo/redo with "unsaved changes" after crossing the Apply
  barrier, Cancel, Read-only Data at startup), each with a visible home on the Banner — no audio-only messages,
  and the empty-list gap is closed by the count announcement. Dialog bodies stay unheard **by discipline**: all
  critical dialog information lives in the title and buttons. No F6, no position announcements; focus never
  jumps without a reason (rules per Apply/Refresh/dialog-close). WCAG 4.5:1 inverts into a prohibition: the app
  never sets a colour. Verification is a 17-step NVDA checklist with expected speech per step, gated on the
  ticket-18 sanity check. New terms **Announcement** and **Banner**: [CONTEXT.md](../../CONTEXT.md).

- [Release and package manifests](issues/15-release-and-manifests.md) — **bare exe + `.sha256` sidecar, no
  zip; the name is free everywhere.** `PackageIdentifier` **`RuslanIskov.PathMaster`** (confirmed by the
  user; VERSIONINFO `CompanyName` must match), winget schema 1.12.0, three-file manifest. From
  winget's own source: `Commands: ["pathmaster"]` names the Links symlink **and renames the installed exe**,
  and winget writes an `HKCU\…\Uninstall\<ProductCode>` ARP key (16 values) plus the Links dir on the user
  PATH — README material. scoop: bare-exe URL with `#/PathMaster.exe` rename, `persist: "data"` junction
  (compatible with ticket 07's resolve rule), `checkver: "github"` + autoupdate off the sidecar, bucket from
  BucketTemplate whose excavator does the per-release bump; **GUI shim needs nothing** — scoop patches the
  shim's PE subsystem, no console flash. Workflow: tag → `windows-2025` (now the **VS2026 image** — drifted
  since ticket 04, CMake/Ninja now match the dev pins) → three-way version gate → build with
  `CARGO_TARGET_DIR=C:\t` (MAX_PATH) → **dumpbin gate on `VCRUNTIME|MSVCP|api-ms-win-crt`** → release via
  `gh`; PDB to CI artifacts only. Open: license (required field, user decision), repo URL, clean-VM run.
  Drafts ready to lift: [research/15-packaging/](research/15-packaging/); details:
  [research/15](research/15-packaging.md).

- [Entry editing interaction](issues/10-entry-editing-interaction.md) — **editing is a modal dialog, not
  inline**: F2/Enter/double-click open one dialog (labelled path field + Browse + OK/Cancel), which kills the
  un-vetoable-edit problem and the `edit_label`-vs-F2 spike in one move; the menu gains "Edit Entry…\tF2".
  Browse **survives as a named exception** to "no native file dialogs" (user override of ticket 07's derived
  constraint): `wxDirDialog`'s ComDlg32 MRU writes are accepted and go in the README. Add is dialog-first —
  OK appends at the end as one Checkpoint, Cancel leaves no trace, closing the Add-then-Escape question.
  Delete **loses its confirm dialog** — undo is the safety net. Validation on OK blocks `< > | "`, the
  separator `;`, and length-zero only (whitespace-only commits verbatim → ticket 13); errors are a
  MessageDialog with the message **in the title**; the OK sequence is validate → `%VAR%`-into-`REG_SZ`
  convert-or-keep → one Checkpoint, so a rejected edit leaves nothing to undo into. Duplicates and
  not-yet-existing paths commit legally; diagnostics flags them asynchronously. A **quoted-entry** question
  surfaced for Normalisation (→ ticket 13 comment).

- [i18n mechanism](issues/11-i18n-mechanism.md) — **`wxTranslations` with `.mo` embedded through a custom
  `TranslationsLoader`**; no Rust i18n crate, so visible labels and Announcements share one `translate()`
  and "one Catalogue" becomes structural rather than disciplinary. `add_std_catalog()` is **never called** —
  every dialog whose button text carries meaning already uses our own buttons, since `MessageDialog`
  cannot relabel its own.
  msgids are **English source text**, and because **`msgctxt` is not bound at any level**, that
  English becomes an API surface: where two strings mean different things their English must differ
  (Cancel-the-command vs Cancel-the-button, a collision that already exists). Symbolic keys were rejected —
  a miss returns the msgid, so a key would be **spoken aloud**. Placeholders are named braces `{n}`, not
  `%d`, which in this domain would be indistinguishable from `%VAR%`. **`.po` is committed and `.mo`
  generated at build time by `polib`** (pure Rust — no `msgfmt` CI pin), which keeps ticket 04's *gate the
  artifact, not the build config* and makes `.po`/`.mo` drift structurally impossible. The gate is a plain
  `#[test]` over a **registry of msgid constants** — presence via `get_string(…).is_some()` (never
  `translate(s) != s`), placeholder integrity, mnemonic uniqueness, and a self-sensitivity check; gettext
  excludes fuzzy entries from `.mo`, so they read as missing for free. **Accelerators belong to the code,
  never the Catalogue** — `wxAcceleratorTable` is absent, so the label string *is* the binding and a
  translated `"\tCtrl+Я"` would delete the shortcut outright; Ukrainian mnemonics stay **Latin in
  parentheses** (`Файл(&F)`) because this application edits Latin paths and its user sits in a Latin layout
  ([ADR-0004](../../docs/adr/0004-catalogue-text-is-load-bearing.md)). Interface Language resolves by a
  two-way branch (`Ukrainian → uk`, everything else → `en`); `settings.json` takes `auto|en|uk` and records
  the **choice, not its outcome**; in Read-only Data the selector is disabled and reads as disabled. The
  restart notice rides the selector's own label, so ticket 09's Announcement catalogue **stays closed at
  seven**. Composed strings constrain the English: operation names translate as verbal nouns and must
  differ from the buttons that perform them. The ticket's "add a third language without touching the code"
  premise is **rewritten, not satisfied** — an embedded catalogue always needs a rebuild; the real workflow
  is dropping `xx.po` into `i18n/` plus one mapping arm. New terms **Catalogue** and **Interface Language**:
  [CONTEXT.md](../../CONTEXT.md).

- [Elevation and System PATH writes](issues/12-elevation-model.md) — **elevation is a whole-app relaunch,
  never a write helper** ([ADR-0005](../../docs/adr/0005-elevation-by-whole-app-relaunch.md)): the single-exe
  helper is an EoP surface, prompts per write, and no neighbouring tool does it. Detection is
  `GetTokenInformation(TokenElevation)` — never `TokenElevationType`, which misreads built-in-admin/UAC-off.
  **One entry point**: a "Restart as Administrator" menu command, disabled when elevated; Read-only Data and
  the System tab name reasons but never grow a second offer, and a System-Snapshot Restore unelevated is a
  disabled control. The command runs *through* the close-confirm flow (title names what is lost: "Discard
  unsaved User changes and restart as administrator?"), spawns with **one argument — the active tab**, and the
  original instance **exits** on success; a declined UAC prompt is `ERROR_CANCELLED` and answers with a
  **dialog** (title-only message), keeping the Announcement catalogue closed at seven. Apply failures: a
  four-row taxonomy (snapshot-write, registry-write, external-edit, broadcast-is-not-a-failure) with two
  invariants — no failure mutates the Working Copy, none moves the Baseline. Elevated title:
  "Administrator: PathMaster". **Exported risk**: portable NVDA is deaf to elevated windows — the ticket-19
  checklist must test the elevated instance explicitly.

- [Diagnostics contract](issues/13-diagnostics-contract.md) — **six Issue types, one-word labels,
  and every rule testable.** Splitting is naive (every `;` separates — matching the OS itself, not
  cmd's quote-aware rule); Normalisation strips one pair of quotes, expands `%VAR%` (process
  environment; unknown names stay literal → `Missing`), `/`→`\`, trims trailing `\` (root-safe),
  folds case — and **never touches the filesystem** (no 8.3, no symlinks; no surveyed tool does).
  Duplicates: first copy in runtime order (System → User, left to right) is canonical, **every
  later copy flags**, cross-scope included (User copy carries it) — so a System edit recomputes
  User's Issues. Existence: local roots only (`GetDriveTypeW`, no network round trip), **network
  entries are never probed in v0.1.0** (a dead UNC blocks 20–60 s and cannot be cancelled);
  exists-but-a-file flags `Missing`, access-denied does not. **`Quoted` is the sixth type** — the
  quoted spelling is measured-dead for `CreateProcessW`/PowerShell/`where`/Python, alive for
  cmd/CRT/Rust/Node: a silent breakage with a trivial fix. **Over-length left the column
  entirely**: scope-level, a passive StatusBar length field plus an Apply-time dialog at the honest
  thresholds — 8,191 (cmd drops the variable, KB 830473; may proceed) and 32,767 (hard cap; no
  proceed) — 2,047 is folklore. Async: worker → `mpsc` → **Timer drain** (idle-handler trap
  avoided). Words, spoken on every arrow: `Missing > Relative > Quoted > Duplicate > Empty`;
  Empty is exclusive, Relative skips existence. All research:
  [research/13](research/13-diagnostics-facts.md).

- [Backup and restore contract](issues/14-backup-restore-contract.md) — **rotation is per-Scope**, never a
  single pooled count. Filename is `YYYY-MM-DDTHH-MM-SS-<Scope>.json`, local time, numeric suffix on same-second
  collision — Scope in the name is load-bearing for per-Scope rotation and listing without parsing content. A
  Snapshot now carries `valueType` alongside `entries` (or an explicit `absent: true`), closing ticket 05's H15
  and satisfying ticket 06's Absent/zero-Entries requirement, while staying human-readable JSON
  ([ADR-0006](../../docs/adr/0006-snapshot-schema-is-decoded-not-raw.md)). **Restore loads into the Working
  Copy and never writes the registry directly** — one ordinary Checkpoint, reusing the Apply path and its
  pre-Apply backup. **Corrupted** is schema-validity, two-layer (parse, then shape), all-or-nothing, surfaced as
  passive Backups-list text — never a new Announcement, keeping ticket 09's catalogue closed at seven — and a
  Corrupted file still counts toward its Scope's rotation budget. Foreign files are silently invisible; atomic
  temp files get a `.tmp` extension so both listing and rotation skip them by extension alone. New term
  **Corrupted**: [CONTEXT.md](../../CONTEXT.md).

- [Window layout, sizing and iconography](issues/17-window-layout-and-iconography.md) — **one vertical
  sizer: always-visible fixed-height Banner above the notebook** (empty at rest — the layout never reflows
  under the user), notebook `proportion=1`, native status bar outside the sizer with **length field right,
  messages left**. First run 900×650 DIP, minimum 800×600; **Path column takes all remaining width, Status
  is the app's single deliberate pixel constant** — and the single explicit `FromDIP()` call. Geometry
  persists in `settings.json` on clean shutdown, restored **clamped to the connected monitors' work area**,
  falling back to default-centred-on-primary when fully off-screen. The implicit FFI `FromDIP` is accepted
  as structural; the cross-monitor DPI-change hazard is a documented risk with a checklist line in ticket 19.
  Icon: **a stylised path**, one embedded SVG through `BitmapBundle::from_svg_data` for the frame, the
  ticket-04 `.ico` route (16/24/32/48/256) for Explorer/taskbar — two assets, one source design. **No other
  in-app iconography**: the Banner stays purely textual, and nothing anywhere sets a colour.

- [Test and verification strategy](issues/19-test-and-verification-strategy.md) — **functional core,
  imperative shell**: every pure rule (splitting, Normalisation, diagnostics, the Session model, Snapshot
  schema, rotation, thresholds) is unit-tested in Rust; the GUI shell is covered by the **Release Checklist**
  only. The registry adapter takes its **key path as a constructor parameter** and its integration tests run
  against a temporary key on the *live* `HKCU` — mocks rejected because ticket 05's hazards are precisely
  about real API behaviour. `nvda-drive.ps1` **stays a measurement tool, never a CI gate** (ticket 18: a deaf
  NVDA is indistinguishable from a regression, and a flaky gate gets ignored); UI automation is out of scope.
  Release CI gates on the artifact: version gate, `cargo test`, dumpbin imports, **exe ≤ 40 MB**; push CI runs
  `cargo test` + clippy on every push to `develop`. The Checklist is canonical in `docs/release-checklist.md`
  (D8's 17 steps + elevated-instance + DPI-drag, all NVDA steps gated on the ticket-18 sanity check) and each
  release attaches a **filled copy** naming the NVDA used; a failed step blocks the release. Cadence: size
  every release, thresholds unit-tested + once against the real registry, cold start only when the startup
  path changes, clean-VM once per packaging change. A **`proptest` layer of exactly three properties**:
  split→join byte-identity, Snapshot round-trip, Normalisation idempotence. Ticket 16 deliberately **not**
  blocked on ticket 18 — the spec records the anomaly as a documented open risk instead. New term
  **Release Checklist**: [CONTEXT.md](../../CONTEXT.md).

- [Failure taxonomy: settings load and log write](issues/20-failure-taxonomy-remainder.md) — **the
  taxonomy closes without an eighth Announcement.** Settings validation mirrors ticket 14's two
  layers: unparsable JSON or a non-object root → full defaults, the bad file **set aside as
  `settings.json.bad`** (never overwritten in place — overriding FR-settings-file's silent reset,
  StatusBar warning included), and **one startup dialog** with the message in its title, the
  ticket-12 catalogue-preserving move. Bad values of known fields fall back **per-field in memory
  while the file keeps the raw value** until the user changes that setting in the UI — ticket 11's
  "choice, not outcome" extended so a v0.2 value survives a v0.1 run; unknown fields are ignored
  and preserved the same way. `maxBackups` is valid at **≥ 1** (default 50) — clamping rejected,
  `0` outlawed because rotation at zero silently deletes the pre-Apply safety net. Per-field
  fallbacks are log-only. **No logging failure touches the app**: every record an independent
  attempt, failures silently dropped but counted, one `N log records were lost` line on recovery;
  an unopenable log at startup means a run without a log, never Read-only Data. Absent
  `settings.json` is a first run, not a failure.

- [NVDA went deaf on the list — unexplained](issues/18-nvda-deaf-on-listctrl.md) — **cause narrowed,
  signature found, no reliable in-app cure.** Zero focus winEvents from the list survived to NVDA's
  object stage for the whole silent window; NVDA's own SysListView32 focus branch descends via live
  `accFocus` (which was healthy), so one processed event would have healed it. Source-refuted:
  stale-object cache, injection-instance, event-flood starvation. Plausible survivor: OS-side winEvent
  delivery loss for that app instance — unreported on NVDA's tracker. **Signature:** focus change with
  no `WM_GETOBJECT (OBJID_CLIENT)` within ~1 s, observable via `SetWindowSubclass` with zero
  accessibility code — this reopens 19 D3's condition, re-posed as
  [ticket 24](issues/24-deaf-state-detection-decision.md) (blocks the spec). Support ladder:
  Alt+Tab (triggers NVDA's `_fakeFocus` direct-MSAA rebuild; needs live test) → restart app (observed
  to clear) → restart NVDA (guaranteed). **Warning:** `announce()` rides the same pipeline — likely
  silent in this state. Details: [research/18](research/18-nvda-deaf-on-listctrl.md).

- [README and user-facing docs](issues/22-readme-and-user-docs.md) — **one `README.md`, mirrored in
  full by `README.uk.md`** (English canonical, cross-linked at the top; commands untranslated), no
  split docs, no badges. Accessibility is the headline section right after a non-technical opening
  (NVDA tested, JAWS/Narrator untargeted, installed-NVDA-for-elevated, ticket-18 workaround); Install
  couples the SmartScreen explanation with `Get-FileHash` verification against the `.sha256` sidecar;
  a ~10-row keyboard reference table; the portability section names everything written to the machine
  (ComDlg32 exception, winget's ARP key/Links/rename, scoop's shim/junction, uninstall-deletes-`data\`);
  `settings.json` + `.bad` documented; by-design cuts get one-line reasons but **no v0.2.0 promise
  list**; the Release Checklist is user-visible trust documentation (filled copy per release). One
  screenshot with full alt text at release time. **License: MIT** — closing ticket 15's open field.
  Drift guard: one new non-NVDA Checklist step — `README.uk.md` in sync or README untouched.

## Not yet specified

In scope, but not yet sharp enough to ticket. Graduates as the frontier advances.

- Nothing at present — every remaining question is a live ticket. (Error/failure taxonomy remainder,
  log format, README, and repo/crate layout graduated to tickets 20–23 on 2026-08-19, once ticket 19
  fixed the seams that shape them.)

## Out of scope

Ruled beyond this destination. Does not graduate.

- **Building v0.1.0.** This map produces the spec; implementation is a separate effort.
- **All 🟡 should features**, deferred to v0.2.0: Drag & Drop reorder, `%VAR%` expansion toggle, Search bar,
  Filter bar, Tree View browser, Fix Issues dialog, Ctrl+C copy entry.
- **Similar-path / typo diagnostics** — cut at charting; a false-positive generator (`C:\Python312` vs
  `C:\Python313` are both legitimate) and trust in the diagnostics matters more than their breadth.
- **The `theme` setting** — cut at charting; system colours always.
- **A `--data-dir` switch** — considered and left out of v0.1.0 by
  [Portable data directory contract](issues/07-portable-data-directory.md); it survives the no-relocation
  principle (it carries the location per launch rather than remembering it), so it is a v0.2.0 candidate.
- **Code signing** — deferred until there are real users; v0.1.0 ships unsigned by decision, not by oversight.
- **UI automation** (FlaUI, WinAppDriver or successors) — ruled out by
  [Test and verification strategy](issues/19-test-and-verification-strategy.md): WinAppDriver is dead, FlaUI
  drags .NET into a Rust repo, and the Release Checklist already walks every critical flow. Revisit only if
  the app outgrows a one-person manual pass.
- **Everything in PRD §10**: other environment variables, cross-machine sync, plugins, web/CLI front ends,
  auto-update.
- **Non-Windows platforms, 32-bit Windows, and screen readers other than NVDA** (JAWS/Narrator must not be
  deliberately broken, but are not targeted or tested).
