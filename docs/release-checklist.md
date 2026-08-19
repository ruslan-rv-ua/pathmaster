# PathMaster Release Checklist

The canonical, manual verification script that gates every release. Defined by tickets 09 (D8),
10 (D8), 12, 17 (D5), 18/24, 19 (D6) and 22 of the v0.1.0 wayfinder map; the spec
([.scratch/pathmaster-v0-1-0/spec.md](../.scratch/pathmaster-v0-1-0/spec.md), §10.2/§18) points here.

**How it is used.** Run personally, on real NVDA, before each release. Each release attaches a
**filled copy** of this document to the GitHub Release — every step marked pass / fail / void /
skipped-with-reason, plus the header below. **A failed step blocks the release.** A pass is a
record, not a ritual.

```
Release:        v_._._
Date:           ____-__-__
NVDA version:   _____
NVDA install:   installed / portable   (elevated-instance section REQUIRES installed NVDA)
Windows:        _____
Monitors:       _____ (count and scale factors, for step L1)
```

## Gate zero: the Sanity Check

Before any NVDA step, and again whenever a step's silence is suspicious: focus a list row and press
`NVDA+Tab`. NVDA must answer with the **row** ("елемент списку" / "list item"), not the list.

- If it does not, the session is in the ticket-18 deaf state and **every NVDA measurement is void —
  not failed, void**. Recovery ladder: Alt+Tab away and back → restart PathMaster → restart NVDA
  (guaranteed). Re-run the gate, then repeat the voided steps.
- The `WM_GETOBJECT` watcher in `tools/` (when built) **backs** this gate as a diagnostic — it
  never replaces the manual gesture, which stays canonical (the signature misses post-creation
  rejections).

## A. Main NVDA pass (unelevated instance)

Every step presumes gate zero passed. Expected speech is the canonical English; on a Ukrainian
interface expect the Catalogue's Ukrainian equivalents.

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| 1 | Launch the app | Window title "PathMaster" | |
| 2 | Arrow to a healthy Entry | Path text only — no "Status:" | |
| 3 | Arrow to an Entry with one Issue | "{path}; Status: {type}" — type one of Missing / Relative / Quoted / Duplicate / Empty | |
| 4 | Arrow to an Entry with several Issues | All types, comma-joined, in the order Missing > Relative > Quoted > Duplicate > Empty | |
| 5 | Ctrl+Tab to the other Scope | Tab label, then "System PATH: {n} entries" | |
| 6 | Activate an empty Scope | "…: no entries" | |
| 7 | Refresh (F5) | "{scope}: {n} entries"; `NVDA+Tab` confirms focus kept the Entry | |
| 8 | Edit an entry, Ctrl+Z | "Undone: Edit entry"; focus lands on the row | |
| 9 | Ctrl+Y | "Redone: Edit entry" | |
| 10 | Apply | "User PATH applied" | |
| 11 | Ctrl+Z after Apply | "Undone: Edit entry, unsaved changes" | |
| 12 | Cancel | "Changes discarded" | |
| 13 | Close with a dirty Session | Dialog title names the dirty Scopes ("Unsaved changes in: …"); title + buttons spoken | |
| 14 | Menu with a clean Session | Apply/Cancel items read as unavailable ("недоступно") | |
| 15 | Full Tab cycle | Every control reached and spoken; cycle returns to start, no trap | |
| 16 | `NVDA+End` | Both status bar fields spoken on demand (entry/issue counts; merged PATH length) | |
| 17 | Start with an unwritable `data\` | Read-only Data Announcement at startup, reason named | |

## B. Dialog steps (ticket 10)

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| B1 | F2 on a row | Edit dialog: title "Edit entry", labelled path field, buttons spoken | |
| B2 | OK on an entry containing `<` | Error dialog whose **title is the message**; OK; focus returns to the field, text intact | |
| B3 | Browse in the Edit dialog | Standard Windows folder picker, operable by keyboard; chosen folder replaces the field text, focus returns to the field | |

## C. Elevated instance (ticket 12)

**Requires installed NVDA** — a portable NVDA is deaf to elevated windows; record which was used in
the header. Elevate via Tools → Restart as Administrator.

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| C1 | Alt+Tab to the elevated instance | Title "Administrator: PathMaster" spoken first | |
| C2 | Gate zero, elevated | `NVDA+Tab` on a row answers with the row | |
| C3 | Arrow one list row, elevated | Row read with columns, as in step 3 | |
| C4 | One Announcement, elevated (e.g. F5) | "{scope}: {n} entries" | |
| C5 | Decline the UAC prompt (separate attempt) | Dialog title "Elevation was cancelled — still running without administrator rights"; original instance stays functional | |

## D. Layout and environment

| # | Step | Expected result | ✓ |
|---|------|-----------------|---|
| L1 | Drag the window between monitors with different DPI scale factors | Layout survives — no clipped or misplaced controls. Skippable with a note when only one monitor is available | |
| L2 | Resize to minimum 800×600 and maximise | List fills its tab; Path column takes remaining width; nothing clipped | |

## E. Non-NVDA release checks

| # | Step | Expected result | ✓ |
|---|------|-----------------|---|
| E1 | `README.uk.md` is in sync with `README.md`, or the release did not change the README | drift guard (ticket 22) | |
| E2 | Process Monitor, filtered to `PathMaster.exe`: normal session incl. one Browse use | No file/registry write outside `<exe dir>\data\`, the two PATH values, and ComDlg32 MRU after Browse | |
| E3 | Clean-VM run (no VC++ redistributable) | App starts and lists PATH. Required once for v0.1.0, then only when packaging changes; note "not repeated — packaging unchanged" otherwise | |

CI gates (version gate, `cargo test`, dumpbin imports, exe ≤ 40 MB) run automatically and are not
part of this manual pass.
