# Accessibility contract

Type: grilling
Status: open
Blocked by: 02, 08

## Question

What is the app's accessibility contract — what must be announced, when, and how verbosely?

With the baseline known (ticket 02) and the announcement mechanism chosen (ticket 08), turn US-accessibility
from an aspiration into testable criteria.

- **Per-entry announcement.** What exactly does NVDA say when focus lands on an entry with an issue? Does the
  text come from the Status column, a suffix on the Path text, or an event? What happens when one entry has
  several issues — is all of it spoken, or a winner?
- **Vocabulary.** The PRD demands `AccessibleName` and `AccessibleDescription` on every interactive element;
  that is WinForms language with no wx counterpart. Replace it with what wx actually offers, and identify the
  few elements that genuinely need more than their own visible label.
- **Verbosity policy.** What is announced always, and what only on demand. Over-announcing is as real a defect
  as silence — a list that re-reads the full status on every arrow key becomes unusable.
- **Keyboard map.** Every command reachable without a mouse; the F6 / Tab / arrow boundaries between panes;
  no focus traps; what happens to focus after Apply, after Refresh, and after a dialog closes.
- **Verification.** The concrete NVDA test script the user runs before a release — a checklist short enough to
  actually be run, listing the expected spoken text for each step.

Output: the accessibility section of the locked spec, and rewritten US-accessibility acceptance criteria that
name what is spoken rather than asserting that something is accessible.
