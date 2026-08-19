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
7. Similar-path / typo detection is **cut** from v0.1.0; five diagnostic types remain.
8. The `theme` setting is **removed** — system colours always, High Contrast is a Windows mode, not an app choice.
9. Logging stays in v0.1.0, minimal.
10. Distribution: GitHub Releases + GitHub Actions, **unsigned** in v0.1.0 (SmartScreen accepted, documented in
    the README). Scoop via an **own bucket** with `persist: data`; winget submitted to `microsoft/winget-pkgs`
    as `InstallerType: portable`.
11. Portability: `data/` sits next to the exe; NFR-no-registry-writes is reworded from "nothing in AppData"
    to "**nothing outside the app's own directory**".
12. All artifacts (map, tickets, research, spec) are written in **English**; conversation with the user is Ukrainian.

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
  zip; the name is free everywhere.** Recommended `PackageIdentifier` **`RuslanIskov.PathMaster`** (user
  still to confirm; VERSIONINFO `CompanyName` must match), winget schema 1.12.0, three-file manifest. From
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

## Not yet specified

In scope, but not yet sharp enough to ticket. Graduates as the frontier advances.

- **Error and failure taxonomy** — what the user is shown, what is logged, and what is announced when a registry
  write, a backup write, or a settings load fails. Waits on the registry, elevation and backup tickets.
- **Log format** — what a record contains and how it reads. **Rotation is settled** by ticket 07
  (one generation, at open only); the format waits on the failure taxonomy.
- **README and user-facing docs** — including the honest description of what winget/scoop themselves write to
  the machine. Waits on the packaging ticket.
- **Repository and crate layout for the implementation effort** — module seams, what is a library vs the GUI
  shell. Deliberately last: it is shaped by every decision above.

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
- **Everything in PRD §10**: other environment variables, cross-machine sync, plugins, web/CLI front ends,
  auto-update.
- **Non-Windows platforms, 32-bit Windows, and screen readers other than NVDA** (JAWS/Narrator must not be
  deliberately broken, but are not targeted or tested).
