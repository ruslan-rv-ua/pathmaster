# Accessibility contract

Type: grilling
Status: open
Blocked by: 08

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

## Carried in from ticket 03

Three requirement rewrites land here, all forced by what the binding cannot do:

- **US-high-contrast inverts into a prohibition.** `wxSystemSettings::GetColour` is unbound and there is no
  High Contrast detection, so the app must simply **never set a colour** — native controls then inherit the
  system theme, High Contrast included. Decide what this means for NFR-accessibility-wcag's 4.5:1 criterion,
  which becomes untestable-by-us rather than satisfied-by-us. Note the hand-built banner (ticket 08) must not
  set a background colour either, or it punches a hard-coded rectangle through High Contrast.
- **Status becomes text-only.** A ListCtrl sub-item cannot carry an icon, so the Status column has no icon to
  pair with its text. NFR-no-color-only wanted text anyway, and NVDA reads the column for free.
- **The status bar cannot highlight a field**, so the over-limit state must be carried in the field's text.

## Carried in from ticket 01

**Adding accessibility labels is a change of plumbing, not a pure addition.** By default no wx control
overrides `CreateAccessible()`, so `WM_GETOBJECT` goes unhandled and comctl32's own IAccessible serves the
ListCtrl — that is where the free row reading comes from. The **first** `set_accessibility_*` call on a widget
moves it onto the wx-mediated path. Whether that degrades the native row reading is UNKNOWN: the
`NOT_IMPLEMENTED` → `CreateStdAccessibleObject` fallback suggests not, but that is a code reading, not a test.
So the contract must say **where labels are set and where they are deliberately not**, and any widget that
gains one has to be re-tested against the ticket-02 baseline rather than assumed improved.

## Carried in from ticket 06

- **Every Checkpoint carries a focus hint** — the id of the Entry the change concerned. Ctrl+Z and Ctrl+Y must
  move focus there and announce what changed; for a screen-reader user that is the only way to learn what was
  undone. This ticket owns the wording.
- **Ctrl+Z immediately after Apply is legal** and silently re-dirties the Session. That state change needs an
  announcement, or the user cannot tell that a previously saved session now has unsaved work.
- **Apply and Cancel are disabled while a Session is clean** (rather than being no-ops), so the disabled state
  itself is carrying information and must read as disabled.
- **The close-confirm dialog names the dirty Scopes explicitly** — "unsaved changes in: User PATH, System PATH" —
  so the user is not left hunting across tabs.
