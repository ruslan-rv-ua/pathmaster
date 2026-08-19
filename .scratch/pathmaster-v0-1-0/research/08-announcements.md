# Ticket 08 — Live announcement mechanism: findings

Measured 2026-08-19 against real NVDA on the machine the app is being built for.
NVDA **2025.3.3 x86**, Windows **11 25H2 (10.0.26200.9168)**, prototype
[`../prototypes/08-announcements/`](../prototypes/08-announcements/src/main.rs) (wxdragon 0.9.18,
wxWidgets 3.3.3 static, `windows` 0.62), driven by
[`../tools/nvda-drive.ps1`](../tools/nvda-drive.ps1). Every pass opened with the ticket-18 sanity
check — `NVDA+Tab` on a list row answered `'елемент списку'` — so no result below comes from the
deaf-list state. Unless stated otherwise, focus sat **inside the populated list** when the trigger
fired, which is where a real user's focus lives.

## The verdict table

Each trigger set a message (serial-numbered, so deduplication can never fake a positive) and fired
one candidate mechanism. "Spoke" means a `Speaking [...]` line with exactly that message appeared
in NVDA's log; "silent" means the keystroke's `Input:` line is in the log and no speech followed.

| # | Mechanism | Result |
|---|---|---|
| T0 | Control: banner `Panel` shown + `StaticText` label set, **no accessibility call** | **Silent.** Confirms the ticket's premise and that the harness detects silence. |
| T1 | Raw `NotifyWinEvent(EVENT_OBJECT_NAMECHANGE, static, OBJID_CLIENT, CHILDID_SELF)` after `set_label` | Silent. |
| T2 | Raw `NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED, static, OBJID_CLIENT, CHILDID_SELF)` after `set_label` | **Spoke, verbatim, every time**: `Speaking ['T2 #4: Copied to clipboard — liveregion']`. |
| T3 | Raw `NotifyWinEvent(EVENT_SYSTEM_ALERT, static, OBJID_CLIENT, CHILDID_SELF)` | Silent. |
| T4 | UIA `UiaRaiseNotificationEvent` on `UiaHostProviderFromHwnd(static)` | **Call succeeded, NVDA ignored it.** The `Result` was `Ok` (a failure would have rewritten the probe-readable label to `FAILED`), yet nothing was spoken. NVDA does not act on UIA notifications from this MSAA-served window. |
| T5 | `set_status_text` + raw `NAMECHANGE` on the status bar's own HWND | Silent. The status bar stays command-only (`NVDA+End`), exactly as ticket 02 found. |
| T6 | wx route: `set_accessibility_role(AccRole::Alert)` + `Accessible::notify_event(EVENT_SYSTEM_ALERT, …, AccObjectType::Alert, 0)` | Silent. The in-toolkit route (research/01's rung 4) does not reach NVDA even with `wxUSE_ACCESSIBILITY=1` confirmed compiled in. |
| T7 | Design-away: move focus to a read-only `TextCtrl` holding the message | Spoke — but wrapped in chrome: `['редактор', 'лише для читання', …, 'T7 #10: PATH refreshed — focus moved here']`, and it costs the user their place in the list. |
| T8 | Design-away: modal `MessageDialog` | **Trap.** NVDA announced `['PathMaster', 'діалог']` and `['OK', 'кнопка']` — **the message body was never spoken.** A wxdragon `MessageDialog` cannot be assumed to read its own text. |
| T9 | Raw `NotifyWinEvent(EVENT_OBJECT_SHOW, static, OBJID_CLIENT, CHILDID_SELF)` as the banner is shown | Silent. |

## Properties of the winner (T2, `EVENT_OBJECT_LIVEREGIONCHANGED`)

All measured, not assumed:

- **Repeats identical text.** Firing the event again *without changing the label* spoke the same
  message again, twice in a row (`T2 #1` three times). "Copied to clipboard" twice in a row is
  safe; no serial-number or text-toggling tricks are needed in the product.
- **Independent of focus position.** Spoke with focus on a list row and with focus on a button.
- **Independent of the widget's visibility.** Spoke while the banner `Panel` was hidden
  (`banner.show(false)`). Double-edged: pure audio toasts need no visual change, and nothing warns
  you that an invisible control is talking — visual presentation is a separate, deliberate decision.
- **Survives the wx accessibility path.** After `set_accessibility_role` flipped a *sibling* widget
  onto the wx-mediated `WM_GETOBJECT` path, T2 (and T1's silence) re-measured unchanged.
- **Costs nothing.** One `user32` call on a `HWND` wxdragon already exposes via `get_handle()`;
  no wx code involved, no `CallAfter` timing, works regardless of `wxUSE_ACCESSIBILITY`.

## The rule

> **Every transient message reaches NVDA through exactly one function**: `announce(text)` — set the
> label of one dedicated message `StaticText`, then
> `NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED, its_hwnd, OBJID_CLIENT.0, CHILDID_SELF)`.
> Nothing else in the app fires accessibility events. Focus moves and dialogs are navigation, not
> announcement — and a `MessageDialog`'s body text must never be the only carrier of information
> (T8: NVDA does not read it).

Where a message also has a visual home (the length-warning banner, the status bar), updating that
home is a separate concern of the same code path, not a separate announcement mechanism. Which
messages get which visual home is ticket 09 / ticket 17 territory.

## Carried out to other tickets

- **→ ticket 09 (accessibility contract):** the T8 trap — a wxdragon `MessageDialog` announces its
  title and buttons but **not its message body**. Every dialog in the contract needs either a
  focusable text control carrying the message, or an `announce()` on open, or a measured exception.
- **→ ticket 09:** `announce()` speaks even when nothing visual changed (hidden-static measurement)
  — the contract should say which messages are audio-only and which must also be visible.
- **→ ticket 17 (window layout):** the banner needs a dedicated message `StaticText` as its first
  child; the announce mechanism is settled and imposes no other constraint on the banner's design.
  (Standing rule from ticket 04 unchanged: no hardcoded background colour.)

## Measurement notes

- NVDA treats a Win32 application in focus mode; there is no browse-mode document here. Browse mode
  as a distinct state was therefore not measurable in this app shape — the closest real-world
  variant covered is "focus parked on different control kinds", which made no difference.
- The `-Probe` cross-check (reading `LVM_GETNEXTITEM` and MSAA `accFocus` straight from the
  control) separated "silent" from "didn't happen" throughout; T4's success/failure was made
  probe-readable by rewriting the label on error.
- Two harness fixes landed during this ticket: digit keys added to `nvda-drive.ps1`'s VK table, and
  a latent bug fixed where `A`–`Z` were stored as `char` keys that no string lookup could ever
  match (documented, never exercised until now).
