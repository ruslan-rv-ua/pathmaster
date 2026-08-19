# Locked v0.1.0 spec

Type: task
Status: open
Blocked by: 09, 10, 11, 12, 13, 14, 15, 17, 19, 20, 21, 22, 23

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

## Comments

**2026-08-19, from ticket 19 (test and verification strategy):** two additions to the assembly.
(1) Create `docs/release-checklist.md` as part of this ticket — the canonical Release Checklist:
ticket 09's 17 D8 steps, the elevated-instance section (ticket 12), the cross-DPI window-drag step
(ticket 17), every NVDA step gated on the ticket-18 sanity check. (2) This ticket is deliberately
**not** blocked on ticket 18: the anomaly does not reproduce, and the spec must not stall on it.
Instead the spec records it as a **documented open risk** — the sanity-check precondition on every
NVDA pass as interim detection, restart-NVDA as the user-facing workaround — and links the open
ticket. Blocked-by extended with the four tickets graduated from the fog on the same day (20–23).

**2026-08-19, from ticket 20 (failure taxonomy):** FR-settings-file is **rewritten, not kept** —
the PRD's "overwrite the corrupted file with a valid version + StatusBar warning" is overridden
(set-aside as `settings.json.bad`, startup dialog with the message in the title, per-field
tolerance with raw values preserved). List it in the PRD-deviation notes with ticket 20 as the
settling ticket. `maxBackups` acceptance gains a testable bound: valid domain ≥ 1, default 50,
invalid → default in memory with the raw value preserved in the file.
