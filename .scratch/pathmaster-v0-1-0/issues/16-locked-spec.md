# Locked v0.1.0 spec

Type: task
Status: open
Blocked by: 09, 10, 11, 12, 13, 14, 15, 17, 19

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
