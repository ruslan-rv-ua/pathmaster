# Locked v0.1.0 spec

Type: task
Status: resolved
Blocked by: 09, 10, 11, 12, 13, 14, 15, 17, 19, 20, 21, 22, 23, 24

## Question

Assemble every resolved decision into `spec.md` — the destination artifact of this map.

Rewrite the source PRD in English with all decisions folded in:

- Every US / FR / NFR / TC is explicitly **kept, rewritten, or cut**. Cut items stay listed with a one-line
  reason, so nobody re-adds them next quarter by accident.
- WinUI vocabulary (`InlineAlert`, `InlineBanner`, `TabPages`) replaced with the real wx widgets chosen in
  ticket 03; `os.Stat()` replaced with its Rust equivalent.
- The scope line restated at the top: v0.1.0 = 🔴 must + StatusBar + `settings.json` + minimal logging;
  the deferred 🟡 set named as v0.2.0.
- Acceptance criteria that can actually be tested — the accessibility ones must name **the text NVDA speaks**,
  not assert that something "is accessible".
- Traceability: each requirement points at the ticket that settled it.

When this ticket closes, the map is complete and the effort leaves wayfinding for an implementation effort.

## Answer

Resolved 2026-08-19. **The spec is locked: [spec.md](../spec.md).** Three artifacts were produced:

1. **[spec.md](../spec.md)** — the destination. Every PRD US/FR/NFR/TC is dispositioned
   kept/rewritten/cut in one traceability table (§2), each pointing at its settling ticket; the
   rewritten contracts fill §3–§19; cuts carry one-line reasons (§20); outright PRD overrides are
   enumerated (§21, thirteen items — FR-settings-file, the WM_SETTINGCHANGE 5000 ms bug, inline
   editing, the dropped confirm dialogs, the six-types amendment, and the rest). WinUI vocabulary
   is replaced throughout (`InlineAlert`/`InlineBanner`/`TabPages` → the Banner, menu-command
   elevation, wx notebook; `os.Stat()` → `std::fs::metadata` semantics). Accessibility acceptance
   criteria name the exact text NVDA speaks. The ticket-18 anomaly is recorded as a documented open
   risk (§19), per the decision not to block on it.
2. **[docs/release-checklist.md](../../docs/release-checklist.md)** — the canonical Release
   Checklist: gate-zero Sanity Check, the 17 D8 steps with the ticket-13 wording filled in, the
   ticket-10 dialog steps, the elevated-instance section (installed-NVDA requirement), the
   cross-DPI drag step, and the non-NVDA release checks (README.uk sync, Process Monitor, clean-VM).
3. **Tooling promoted**: `nvda-drive.ps1` and its README moved (`git mv`) from
   `.scratch/pathmaster-v0-1-0/tools/` to the permanent repo-root `tools/`, links updated.

Assembly-level decisions the tickets delegated here are marked **[assembly]** in the spec and were
fixed as: the exact Catalogue English for all Announcements, dialog titles and failure texts
(ticket 12 D10's delegation); the menu structure and shortcut table (ticket 09 D5's delegation —
Ctrl+S Apply, Del Delete, Alt+Up/Down Move); the StatusBar field wording (ticket 17 D10's
delegation); operation names "Change value type" and "Restore snapshot" added to the undo set;
Restore's confirm dialog dropped and post-Restore focus fixed (extending tickets 10 D4 / 14);
winget `MinimumOSVersion` pinned to 10.0.19044.0 (= Win10 21H2, closing ticket 15's open floor).
The log-rotation conflict between tickets 07 (5 MB/`.log.1`) and 21 (1 MB/`.old`) resolves to
ticket 21 by recency.

**The map is complete.** No open question stands between the spec and an implementation effort;
still owed at release time (actions, not decisions): the clean-VM run, one live winget install
observation, and the repo URL in the manifest drafts.

## Comments

**2026-08-19, from ticket 19 (test and verification strategy):** two additions to the assembly.
(1) Create `docs/release-checklist.md` as part of this ticket — the canonical Release Checklist:
ticket 09's 17 D8 steps, the elevated-instance section (ticket 12), the cross-DPI window-drag step
(ticket 17), every NVDA step gated on the ticket-18 sanity check. (2) This ticket is deliberately
**not** blocked on ticket 18: the anomaly does not reproduce, and the spec must not stall on it.
Instead the spec records it as a **documented open risk** — the sanity-check precondition on every
NVDA pass as interim detection, restart-NVDA as the user-facing workaround — and links the open
ticket. Blocked-by extended with the four tickets graduated from the fog on the same day (20–23).

**2026-08-19, from ticket 18 (NVDA deaf-list anomaly, resolved):** the risk note this ticket
records is now sharper than "does not reproduce, restart NVDA": cause narrowed to winEvent
delivery loss for the app instance (plausible, unreported upstream), a **detectable signature**
exists (`WM_GETOBJECT` silence after a focus change — what v0.1.0 does with it is ticket 24, now
also blocking this spec), the support ladder is Alt+Tab → restart app → restart NVDA, and the note
must warn that **`announce()` is very likely silent in the deaf state too**. Details:
[ticket 18's Answer](18-nvda-deaf-on-listctrl.md).

**2026-08-19, from ticket 20 (failure taxonomy):** FR-settings-file is **rewritten, not kept** —
the PRD's "overwrite the corrupted file with a valid version + StatusBar warning" is overridden
(set-aside as `settings.json.bad`, startup dialog with the message in the title, per-field
tolerance with raw values preserved). List it in the PRD-deviation notes with ticket 20 as the
settling ticket. `maxBackups` acceptance gains a testable bound: valid domain ≥ 1, default 50,
invalid → default in memory with the raw value preserved in the file.

**2026-08-19, from ticket 23 (crate layout, resolved):** the last blocker is closed — this ticket is
now the frontier, and the whole map's remaining work. What 23 hands the assembly: the spec gains a
short **repository layout** section (three-crate workspace, dependency direction bin → platform →
core, bin-only GUI crate, `[[bin]] name = "PathMaster"`, flat `crates/` layout with the ticket-04
profile in the virtual root — details in [ADR-0007](../../../docs/adr/0007-crate-boundary-is-the-test-boundary.md));
ticket 19's test-strategy section can now name *where* each tier lives (core unit tests + the three
proptest properties in `core/tests/properties.rs`; registry integration tests `#[cfg(windows)]` in
platform; the GUI tier is the Release Checklist only); ticket 11's msgid gate is restated as split
(polib integrity check in core, one `get_string` smoke test in the bin); and the Release Checklist
tooling reference points at the permanent repo-root `tools/` (nvda-drive.ps1 promoted out of
`.scratch/`), which this ticket should perform as part of assembly.
