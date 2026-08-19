# Accessibility contract

Type: grilling
Status: resolved
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

## Carried in from ticket 08

The announcement mechanism is settled — one `announce(text)` function: set the label of one
dedicated message `StaticText`, fire
`NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED, hwnd, OBJID_CLIENT, CHILDID_SELF)` on it; nothing
else in the app fires accessibility events ([research/08](../research/08-announcements.md)). Three
consequences land in this contract:

- **The `MessageDialog` trap.** NVDA announced a wxdragon `MessageDialog`'s title and OK button but
  **never its message body**. Every dialog in the contract needs a measured plan: a focusable text
  control carrying the message, an `announce()` on open, or an explicit re-test showing that dialog
  variant speaks. A dialog's body text must never be the only carrier of information.
- **Audio-only vs visible.** `announce()` speaks even when nothing visual changed (measured with the
  banner hidden). The contract must say, per message, whether it is audio-only or also has a visual
  home (banner, status bar) — the mechanism no longer forces that choice.
- **Repeats are safe.** Identical text announced twice speaks twice — no serial numbers or
  text-toggling tricks needed in product wording.

Also confirmed here: the status bar stays command-only even with a `NAMECHANGE` fired on its HWND —
routing any must-hear message there hides it, now as a measurement rather than an inference.

## Answer

Resolved 2026-08-19 by a grilling session. The contract in one sentence: **everything must-hear rides a
channel NVDA is measured to speak — the Status column, visible labels, dialog titles, and one `announce()`
function — and nothing else speaks at all.**

### D1. The Status column is the per-entry carrier

- Status text is **issue types only, no severity prefix** — NVDA already prepends the column header
  ("Status:"), and a "Warning:" on every arrow key is noise.
- Several Issues on one Entry: **all of them, comma-joined, in a fixed severity order** — never a
  single "winner". Five types maximum keeps the line short, and a winner would hide information a
  screen-reader user has nowhere else to get.
- A healthy Entry has an **empty** Status column — silence is the "all clear" signal, and NVDA skips
  empty sub-items (measured in the ticket-02 baseline). Never "OK".
- Exact per-type wording, rules, and the severity order are **owned by ticket 13** (carried in there).

### D2. Zero `set_accessibility_*` calls in v0.1.0

The PRD's `AccessibleName`/`AccessibleDescription`-on-everything is rewritten in wx terms: **every
interactive element has a visible text label read by the native comctl32 path; the only accessibility
call in the application is `announce()`.** Ticket 01 showed the first `set_accessibility_*` call on a
widget swaps its plumbing with unknown effect on the free row reading; ticket 02 showed the free path
already covers buttons, tabs, menus, dialog fields. Any future label beyond the visible one is a
re-measure against the baseline, not an assumed improvement. ADR:
[docs/adr/0003](../../docs/adr/0003-no-accessibility-calls-except-announce.md).

### D3. Announcements: a closed catalogue, never audio-only

Every `announce()` message also has a visible home — the Banner (the dedicated `StaticText` whose label
is set before the event fires). No audio-only messages: sighted and screen-reader users get the same
information. The status bar stays command-only (`NVDA+End`); nothing must-hear goes there.

The **closed catalogue** for v0.1.0 — nothing else is announced:

1. Scope tab activation and Refresh — "User PATH: {n} entries" / "User PATH: no entries" (also closes
   the empty-list baseline gap; no placeholder rows in the list — a fake row would impersonate an Entry).
2. Apply succeeded — "User PATH applied".
3. Apply failed (registry, backup, external change) — every failure announces; exact texts owned by
   tickets 12/13/14.
4. Undo/Redo — "Undone: {operation}" / "Redone: {operation}", where {operation} is the short Checkpoint
   operation name ("Add entry", "Edit entry", "Delete entry", "Move entry", "Cancel"). No full path in the
   announcement — focus moves to the row (the Checkpoint's focus hint) and NVDA reads it there.
5. Undo across the Apply barrier — same undo announcement with the suffix ", unsaved changes" (ticket 06:
   the user must learn a saved Session is dirty again).
6. Cancel — "Changes discarded" (Cancel is itself undoable → then item 4 fires).
7. Read-only Data startup — the reason, once, at start.

Canonical texts are English; Ukrainian ships as translation strings (mechanism owned by ticket 11).

### D4. Row position is NVDA's business, not ours

"3 of 12" is NVDA's "report object position information" setting; compensating would double-speak for
users who have it on. The contract does not announce position; entry counts come from D3 item 1.

### D5. Keyboard: Tab order is the whole map — no F6

Tabs → list → buttons, full Tab traversal with no traps (a tested criterion), Ctrl+Tab switches Scope
tabs. **F6 is unused by decision, not oversight** — it would be a third way to do the same thing.
Global focus invariants (the rest of the shortcut table assembles in ticket 16 from tickets 10/13/14):

- **Focus never jumps without a reason.**
- After Apply — stays on the current Entry.
- After Refresh — the Entry with the same id if it survived re-read, else its nearest neighbour by
  index, else the list itself.
- After any dialog closes — the control that opened it.
- Apply and Cancel while a Session is clean are **disabled and read as disabled** (via the menu, where
  NVDA speaks the unavailable state — measured in baseline).

### D6. Dialogs: title discipline, bodies stay unheard

Decision: **do nothing mechanically** about the measured MessageDialog trap (body never spoken). The
compensating rule: **all critical information in a dialog is carried by its title and buttons** — the
body is detail for sighted users only. Example: title "Unsaved changes: User PATH, System PATH", not
"Warning" + body. This keeps ticket 08's invariant (a dialog body is never the sole carrier) as a
wording discipline with zero code.

### D7. High contrast / WCAG 4.5:1 inverts into a prohibition

NFR-accessibility-wcag is rewritten: **the application never sets a colour** (Banner included — a set
background would punch a hard-coded rectangle through High Contrast). Contrast is a property of the
system theme; 4.5:1 is untestable-by-us. The testable criteria: no colour-setting call anywhere in the
code, and no information whose only carrier is colour.

### D8. The NVDA verification checklist

Run before each release, by the user, on real NVDA. **Gate zero every session** (ticket 18): focus a
list row, `NVDA+Tab` must answer "елемент списку" — if not, the session is in the ticket-18 anomaly
state and no measurement counts.

| # | Step | Expected speech |
|---|------|-----------------|
| 1 | Launch the app | Window title "PathMaster" |
| 2 | Arrow to a healthy Entry | Path text only — no "Status:" |
| 3 | Arrow to an Entry with one Issue | "{path}; Status: {type}" |
| 4 | Arrow to an Entry with several Issues | All types, comma-joined, severity order |
| 5 | Ctrl+Tab to the other Scope | Tab label, then "System PATH: {n} entries" |
| 6 | Activate an empty Scope | "...: no entries" |
| 7 | Refresh (F5) | "{scope}: {n} entries"; `NVDA+Tab` confirms focus kept the Entry |
| 8 | Edit an entry, Ctrl+Z | "Undone: Edit entry"; focus lands on the row |
| 9 | Ctrl+Y | "Redone: Edit entry" |
| 10 | Apply | "User PATH applied" |
| 11 | Ctrl+Z after Apply | "Undone: Edit entry, unsaved changes" |
| 12 | Cancel | "Changes discarded" |
| 13 | Close with a dirty Session | Dialog title names the dirty Scopes; title + buttons spoken |
| 14 | Menu with a clean Session | Apply/Cancel items read as unavailable ("недоступно") |
| 15 | Full Tab cycle | Every control reached and spoken; cycle returns to start, no trap |
| 16 | `NVDA+End` | Status bar fields spoken on demand |
| 17 | Start with an unwritable `data\` | Read-only Data announcement at startup, reason named |

Steps 3, 4 and the failure texts of D3 item 3 gain exact wording when tickets 13/12/14 resolve; the
checklist rows are placeholders until then, the steps themselves are fixed.

### Requirement rewrites

- **US-accessibility** acceptance criteria are replaced by D8 — criteria that name the spoken text.
- **NFR-accessibility-wcag** → D7 prohibition wording.
- **US-high-contrast** → satisfied by D7 (never set a colour), not by detection.
- **NFR-no-color-only** → satisfied by D1 (text-only Status column).
- **PRD `AccessibleName`/`AccessibleDescription`** → D2 rewording.
