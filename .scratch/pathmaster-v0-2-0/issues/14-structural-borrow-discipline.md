# Making the UI's borrow discipline structural

Type: grilling
Status: open
Blocked by: —

## Question

Deferred by the 2026-08-20 architecture review and pulled into scope at charting because v0.2.0
adds dialogs and UI surfaces (Tree View, Fix Issues, a search field, possibly a filter row) — and
the hazard grows with exactly that: "no borrow is ever held across a call that can run someone
else's code" is enforced today by a doc comment on `App` listing hand-checked call sites, while the
diagnostic Timer ticks inside modal dialogs' own event loops. The project's standing preference is
structural over disciplinary (ADR-0007, v0.1.0 ticket 11). Decide the mechanism **before** the new
surfaces are specified in code:

- Consult `/codebase-design`. Candidates to weigh: copy Working Copy state out on every read (the
  review's known cost: changes how every command is written); a borrow-scope wrapper type that makes
  holding-across-dispatch unrepresentable; confining all Session access to a single dispatch point;
  or keeping the discipline but making the Timer inert while any modal is open (narrows the hazard
  instead of the borrows).
- The decision criterion set at deferral: "revisit if it bites, or if the call-site list outgrows a
  comment." v0.2.0's new dialogs are the second condition approaching — the ticket should count the
  call sites the new features add under each candidate.
- Whatever is chosen earns an ADR if it's hard to reverse, surprising, and a real trade-off — this
  one likely is.
- Out of scope here: collapsing `ScopeDiagnosis` into `Findings` (map's Out of scope; the review's
  other deferral).
