# Ticket 18 — NVDA deaf on the list: findings

Reading investigation, 2026-08-19, against primary sources only — no live reproduction was possible
(the state does not reproduce; see ticket). Every claim below is pinned to the source that owns it.
The deaf window itself: `02-nvda-baseline.md`, section "The anomaly".

## Provenance

| | |
|---|---|
| NVDA source | tag [`release-2025.3.3`](https://github.com/nvaccess/nvda/tree/release-2025.3.3) — the exact version on the measurement machine |
| Files read | `source/IAccessibleHandler/__init__.py`, `internalWinEventHandler.py`, `orderedWinEventLimiter.py`, `utils.py`, `source/NVDAObjects/IAccessible/__init__.py`, `sysListView32.py`, `source/eventHandler.py`, `source/api.py`, `source/globalCommands.py`, `source/UIAHandler/__init__.py`, `nvdaHelper/remote/sysListView32.cpp`, `injection.cpp` |
| wxWidgets source | tag `v3.3.3`, `src/msw/listctrl.cpp` |
| Issue tracker | github.com/nvaccess/nvda — searched via web + GitHub search API (queries listed at the end) |

Line references are to those tags; permalinks are given the first time a file is cited.

## Headline

**The deaf state is NVDA-side (or OS-side), not app-side; nothing in wx or the app caused it.**
Reading the pipeline pins the failure to one narrow place: for ~7 minutes, **not a single
`EVENT_OBJECT_FOCUS` winEvent from the list survived to NVDA's object-building stage** — neither the
item events from arrowing nor the container event from the `Shift+Tab` re-entry. Everything NVDA
does after that stage was demonstrably healthy. Three of the ticket's hypotheses are refuted by
source; what survives is a delivery/early-drop failure that the app **can detect** (WM_GETOBJECT
silence) and **cannot reliably fix** — but both observed and guaranteed recoveries exist, and the
cheapest one is *restart the app*, not restart NVDA.

## 1. The pipeline a spoken row requires

For NVDA to announce a row in a SysListView32, all of this must happen, in order:

1. comctl32 fires `EVENT_OBJECT_FOCUS(hwnd, OBJID_CLIENT, 1-based row)` via `NotifyWinEvent` on item
   focus change. (comctl32 is closed source; this is its standard documented MSAA behaviour and is
   what NVDA's whole sysListView32 module is built against.)
2. NVDA's out-of-context hook receives it —
   [`internalWinEventHandler.py`](https://github.com/nvaccess/nvda/blob/release-2025.3.3/source/IAccessibleHandler/internalWinEventHandler.py)
   `winEventCallback` (L68), hooks registered per event type at L205-213.
3. The limiter passes it — [`orderedWinEventLimiter.py`](https://github.com/nvaccess/nvda/blob/release-2025.3.3/source/IAccessibleHandler/orderedWinEventLimiter.py):
   focus events go straight into a dedicated cache (L60-65) and are **exempt from the
   10-events-per-thread flood cap** (which applies only to generic events, L109-112); only the
   newest 4 focus events per pump are kept (L32, L116).
4. `pumpAll` accepts it — [`IAccessibleHandler/__init__.py`](https://github.com/nvaccess/nvda/blob/release-2025.3.3/source/IAccessibleHandler/__init__.py)
   L1010-1092, gated by `eventHandler.shouldAcceptEvent` (foreground-application checks,
   [`eventHandler.py`](https://github.com/nvaccess/nvda/blob/release-2025.3.3/source/eventHandler.py) L477-562).
5. `processFocusWinEvent` (L679) → `winEventToNVDAEvent(..., useCache=False)` (L728) → drop-gates:
   invalid window (L545), ghost/hung window (L552), **native-UIA window (L559)** → then
   `NVDAObjects.IAccessible.getNVDAObjectFromEvent` (L576) →
   `oleacc.AccessibleObjectFromEvent` (L347-357) — **which sends `WM_GETOBJECT` to the app**.
6. The SysListView32 special case (L732-749): for any focus event on a list — *even childID 0* —
   NVDA queries the control's live MSAA `accFocus` and, if it names a child, **builds the row object
   directly and focuses it**.
7. `processFocusNVDAEvent` (L753-774): duplicate check, then `shouldAllowIAccessibleFocusEvent`
   (object or ancestor must have `State.FOCUSED`,
   [`NVDAObjects/IAccessible/__init__.py`](https://github.com/nvaccess/nvda/blob/release-2025.3.3/source/NVDAObjects/IAccessible/__init__.py) L869-887)
   → queue `gainFocus` → speech.

Two facts about this pipeline decide the whole investigation:

- **Step 6 means a healthy `accFocus` heals everything.** The cross-check during the deaf window
  showed `accFocus` naming the correct row. So if *any* focus winEvent from the list hwnd — item or
  container — had reached step 6 during those 7 minutes, NVDA's focus would have landed on the row
  and spoken it. It never did (`NVDA+Tab` kept answering the List). Therefore **zero focus winEvents
  from that hwnd were processed for the whole window**, including the `Shift+Tab` re-entry. The
  failure is at steps 1-5, not in object interpretation.
- **Every drop in steps 4-5 is logged only when `config.conf["debugLog"]["MSAA"]` is on**
  ([`utils.py`](https://github.com/nvaccess/nvda/blob/release-2025.3.3/source/IAccessibleHandler/utils.py) L72-74;
  every drop site is wrapped in `isMSAADebugLoggingEnabled()`). At the Input/Output log level used
  for measurement, **all candidate failure modes produce exactly what was observed: nothing.**
  "No error in the log" discriminates between none of them.

## 2. What the captured evidence proves against the source

- **The benign `LVM_GETGROUPINFOBYINDEX failed` line is a fingerprint, not noise.** It is emitted in
  exactly one place, the in-process helper's group-info fetch
  ([`sysListView32.cpp`](https://github.com/nvaccess/nvda/blob/release-2025.3.3/nvdaHelper/remote/sysListView32.cpp) L43),
  reachable only from `List.getListGroupInfo`, called only from `List.event_gainFocus` **when the
  List is `api.getFocusObject()`**
  ([`sysListView32.py`](https://github.com/nvaccess/nvda/blob/release-2025.3.3/source/NVDAObjects/IAccessible/sysListView32.py) L182-224).
  So during the silent window a container `gainFocus` **was executed** — NVDA's machinery from the
  queue onward, plus the in-proc RPC channel both directions, was alive. The deafness began *after*
  that container focus and consisted of nothing further arriving.
- **`NVDA+Tab` answering row/column counts proves raw `SendMessage` access was healthy.** The
  gesture reads `api.getFocusObject()`
  ([`globalCommands.py`](https://github.com/nvaccess/nvda/blob/release-2025.3.3/source/globalCommands.py) L2816-2817),
  and the spoken 'з 11 рядків і 2 стовпців' is computed live via `LVM_GETITEMCOUNT` / header counts
  (`sysListView32.py` L237-250). NVDA could still interrogate the control; it just had the wrong
  object as focus and no events to correct it.
- **The `NVDA+End` failure is the same disease, second symptom.** `api.getStatusBar`
  ([`api.py`](https://github.com/nvaccess/nvda/blob/release-2025.3.3/source/api.py) L437-459) starts
  from the **cached** foreground object (L46-55: updated only by processed events) and then calls
  `getDesktopObject().objectFromPoint(...)` — a *fresh* out-of-process MSAA object creation via
  `AccessibleObjectFromPoint`. Either a stale foreground object or failing fresh-object creation
  breaks it. A single cause — "fresh MSAA event/object processing for this app instance is dead,
  existing objects and raw SendMessage still work" — explains both observed symptoms at once.
- **Not sleep mode** (the classic "NVDA silent in one app" cause): in sleep mode NVDA's own
  gestures don't execute and focus still *tracks* silently
  (`eventHandler.py` L333-353, L411-412 — `doPreGainFocus` runs even in sleep mode). Observed was
  the opposite on both counts: gestures spoke, focus did not track.

## 3. The hypotheses, judged

| Hypothesis (ticket) | Verdict | Why |
|---|---|---|
| 2. Stale cached object in a long-lived NVDA session | **REFUTED (source)** | The focus path builds a fresh object every time: `useCache=False` (`IAccessibleHandler/__init__.py` L728). The only object cache, `liveNVDAObjectTable`, is a `weakref.WeakValueDictionary` (L70) and is bypassed for focus. The UIA-window classification cache expires after **0.5 s** ([`UIAHandler/__init__.py`](https://github.com/nvaccess/nvda/blob/release-2025.3.3/source/UIAHandler/__init__.py) L1305-1318). Nothing on this path can stay stale for 7 minutes, let alone "until NVDA restarts". |
| 3. Injection specific to that process instance | **REFUTED as the silencing mechanism (source)** | Event delivery is entirely out-of-process (`SetWinEventHook`, `internalWinEventHandler.py` L205-213); the in-proc helper is used only for *reading* text/columns/groups, with an out-of-proc `VirtualAllocEx` fallback when injection is absent (`sysListView32.py` L270-317, L545-609). A broken injection degrades row *text*, it cannot silence focus. And this injection demonstrably worked — the group-info log line traveled its RPC channel. |
| 1. Race at window creation, degraded object cached | **REFUTED as stated; a fragment survives** | No cache (above). The surviving fragment: if the container focus event is processed at a moment when `accFocus` yields nothing (no item focused yet, or the query fails), NVDA correctly focuses the List (L736-749) — which is the state NVDA was *in*. But that state is self-healing the moment any later focus event is processed, so a creation race explains the starting point, **not** the 7-minute persistence. |
| Event flood starving focus (not in ticket, obvious suspect) | **REFUTED (source)** | Focus events are exempt from the per-thread flood cap and were sparse anyway (`orderedWinEventLimiter.py` L60-65, L109-116; the flood problem was fixed for the focused object in [PR #11520](https://github.com/nvaccess/nvda/pull/11520)). The foreground-defer loop is bounded at 2 cycles (`internalWinEventHandler.py` L29, L222-252). |
| UIA misclassification of the list hwnd | **UNLIKELY (source, doubly guarded)** | `winEventToNVDAEvent` does drop *all* MSAA events for windows classified native-UIA (L559-564) — the right *shape* of failure. But for `SysListView32` the classifier requires both a server-side UIA provider answer **and** `FrameworkId == WinForm` (`UIAHandler/__init__.py` L1286-1297, the [#15283](https://github.com/nvaccess/nvda/issues/15283) guard); a wx list satisfies neither, and the 0.5 s cache TTL means the misclassification would have to recur consistently for 7 minutes. |
| **Delivery loss** — item/container focus winEvents from that process instance never reached NVDA's hook thread (OS delivery, or comctl32 not firing) | **PLAUSIBLE — top-ranked** | The only class consistent with everything: zero events processed across 7 minutes and ~17 focus changes; no log entries at any level; raw SendMessage and existing COM proxies healthy; in-proc RPC healthy; state bound to the app *instance* (relaunching the app cleared it while the same NVDA process kept running — evidenced in `02`, pass 3). Cannot be confirmed by reading — the loss would be inside closed comctl32/user32 or the hook transport. |
| Deterministic early per-event drop inside NVDA (ghost-window check, `AccessibleObjectFromEvent` failing NVDA↔this-process while other MSAA clients succeed) | **PLAUSIBLE — second** | Same observable outcome, same absence of logging at Input/Output level. Discriminable from delivery loss only by live measurement (below). |

**No matching prior report exists on the NVDA tracker.** Searches (web + GitHub API) for silent
listviews, focus stuck on a list container, winEvents ceasing until restart, and SysListView32
regressions surfaced only different diseases: flood starvation ([#11520](https://github.com/nvaccess/nvda/pull/11520)),
a 2022.1 column regression with loud tracebacks ([#13735](https://github.com/nvaccess/nvda/issues/13735)),
injection lifecycle instability on NVDA restart/update ([#16933](https://github.com/nvaccess/nvda/issues/16933),
[#7563](https://github.com/nvaccess/nvda/issues/7563)), and WinForms lists lacking MSAA
([#15283](https://github.com/nvaccess/nvda/issues/15283)). Our anomaly is unreported; if it recurs
under the logging below, it is tracker-worthy.

## 4. wxWidgets verdict — the toolkit is passive here

`src/msw/listctrl.cpp` (wx `v3.3.3`) was searched for anything that could race or suppress the MSAA
event stream: there is **no HWND recreation** (no `Recreate`/`MSWRecreate` path; style changes go
through `SetWindowLong`), **no `WM_GETOBJECT` handling**, and **no `NotifyWinEvent` calls or
suppression** — wx merely consumes `LVN_ITEMCHANGED` notifications to synthesize its own
`wxEVT_LIST_ITEM_FOCUSED` (L2575-2596). The MSAA events are fired by comctl32 itself, untouched by
the toolkit. There is nothing on the app side to fix, and no app-side change could have caused this.

## 5. Detectable signature — YES, with one caveat

**The app can detect the deaf state from inside the process: WM_GETOBJECT silence.**

Every focus winEvent that NVDA actually processes forces `oleacc.AccessibleObjectFromEvent`
(`IAccessibleHandler/__init__.py` L347-357 via L576, fresh every time — `useCache=False`), and that
call sends **`WM_GETOBJECT` (lParam = `OBJID_CLIENT`) to the list's own window**. So:

> **Signature:** a screen reader is present (`nvdaHelperRemote.dll` loaded in-process, or
> `SPI_GETSCREENREADER`) **and** the list's focused item just changed (`EVT_LIST_ITEM_FOCUSED`)
> **and** no `WM_GETOBJECT` arrives at the list HWND within ~1 s. Observable by subclassing the
> list HWND (`SetWindowSubclass`) — pure message watching, no accessibility code, no comctl32-path
> change (the ticket-02 caveat about `set_accessibility_*` does not apply).

Coverage caveat, from the drop-site map in §1: the signature catches every failure class **up to and
including object creation** — delivery loss, ghost-window drop, UIA-window drop, and
`AccessibleObjectFromEvent` failure (partial). It does *not* catch a post-creation rejection
(`shouldAllowIAccessibleFocusEvent` false, duplicate drop), where `WM_GETOBJECT` arrives and NVDA
still says nothing. The evidence says our deaf window was a pre-creation failure (a processed event
would have healed focus via `accFocus`, §1), so the signature covers the observed disease.

**→ ticket 19:** this reopens D3 as promised — the no-NVDA-automation decision was conditioned on
"no detectable signature", and one exists at the cost of a subclass proc in the prototype/harness.

Second, NVDA-side, zero-code detection for the *measurement machine*: turn on NVDA's **MSAA debug
logging** (`debugLog → MSAA`, Advanced settings) and leave it on. Every drop site in §1 then logs
its reason verbatim ("Dropping winEvent…", "Could not instantiate an NVDAObject…", "IAccessible
focus event not allowed by…"). A recurrence with that flag on is fully diagnosable from
`%TEMP%\nvda.log`; without it, it never will be.

## 6. Mitigation and recovery — verdicts per candidate

| Candidate (ticket) | Verdict | Evidence |
|---|---|---|
| App re-fires `NotifyWinEvent(EVENT_OBJECT_FOCUS, list_hwnd, OBJID_CLIENT, row+1)` on each focus change | **Helps only if comctl32 failed to fire; harmful if unconditional** | It enters the identical pipeline, so it cannot outrun delivery loss or an NVDA-side gate. And it must be gated on the §5 signature: focus events with childID > 0 are **never** treated as duplicates (`NVDAObjects/IAccessible/__init__.py` L858-862), so under a healthy NVDA an unconditional re-fire speaks every row **twice**. |
| Toggling focus away and back | **Probably useless for the observed state** | It just generates more focus winEvents down the same pipe — and the observed window already contained a real `Shift+Tab` re-entry that changed nothing. Would only heal the refuted "container focused because accFocus was momentarily empty" fragment. |
| Recreating the control | **Not supported by source** | The only per-HWND state in NVDA is the weak object table and the 0.5 s UIA cache — neither persists (§3). If the fault is per-process delivery, a new HWND in the same process inherits it. Untested, low expected value. |
| **`Alt+Tab` away and back (user gesture)** | **Best self-heal candidate — needs live test** | The foreground/switch events originate outside the deaf app (shell/system), and NVDA's fallback for "switch happened but no valid focus event followed" is `_fakeFocus`: it rebuilds focus by **direct MSAA descent from the desktop** (`api.getDesktopObject().objectWithFocus()`), consuming no winEvents from the app at all (`IAccessibleHandler/__init__.py` L967-993, wired in `pumpAll` L1082-1092). Direct MSAA was healthy during the window, so this path should land on the row and speak it. |
| **Restart the app** | **Observed to work** | Pass 3 of ticket 02: same NVDA process, fresh app instance, fully healthy. Consistent with every surviving hypothesis (all remaining candidate state is per-process-instance or per-HWND). Cheaper than restarting NVDA and the first thing to try. |
| **Restart NVDA** | **Guaranteed-by-construction fallback** | Restart re-registers every `SetWinEventHook` (`internalWinEventHandler.py` L193-213 via `IAccessibleHandler.initialize` L1000-1006), rebuilds the UIA handler and its cache (`UIAHandler/__init__.py` L527), empties all object tables, and re-arms the injection hooks ([`injection.cpp`](https://github.com/nvaccess/nvda/blob/release-2025.3.3/nvdaHelper/remote/injection.cpp) L330-339, L300-307). Every candidate cause on the NVDA side of the boundary is rebuilt; only a hypothetical fault wholly inside comctl32's instance state would survive it — which app restart then clears. |

**Support answer for the field** (the ticket's ask): *if NVDA goes silent on the list but still
answers `NVDA+Tab`: first `Alt+Tab` away and back; if still silent, close and reopen PathMaster; if
still silent, restart NVDA (`NVDA+Q` → restart). The app is not at fault and no setting fixes it.*

## 7. A cross-ticket warning: the deaf state very likely silences `announce()` too

Ticket 08's winner, `EVENT_OBJECT_LIVEREGIONCHANGED`, is a winEvent riding **the same hook, limiter,
and `winEventToNVDAEvent` gates** as the focus events that went missing (mapped at
`internalWinEventHandler.py` L57, processed as a generic event through `processGenericWinEvent`
L600-676) — and generic events are additionally subject to the flood cap that focus events are
exempt from. If delivery for the process is dead, `announce()` is dead with it. The ticket-18
precondition (`NVDA+Tab` must answer 'елемент списку') already voids measurement passes in the deaf
state; the product-side consequence is that **no announcement mechanism the app can fire is a
workaround for this state** — which is why the recovery story in §6 matters. (An app-driven
`nvdaController_speakText` over the ncalrpc channel — the channel proven alive during the deaf
window, since the in-proc log line crossed it (`injection.cpp` L370-376) — is the one path likely to
still speak; noted as an option, not a recommendation.)

## 8. What remains live-measurement work

Reading cannot split the two surviving hypotheses; these tests can, in order of cheapness:

1. **Already-captured evidence, answerable today:** in the pass-2 log (16:24-16:31), did the
   *buttons and tabs* speak during the Tab traversal that preceded the silence? If yes, the
   deafness was scoped to the list HWND; if no, to the whole process. The raw log exists; the
   research file never recorded this.
2. **Standing config change:** enable NVDA's MSAA debug logging on the measurement machine (§5) so
   any recurrence self-documents its drop site.
3. **Signature probe:** add the `WM_GETOBJECT` subclass watcher to the ticket-02/08 prototype and
   log focus-change → WM_GETOBJECT latency per keystroke; a deaf recurrence then distinguishes
   "no WM_GETOBJECT" (pre-creation loss) from "WM_GETOBJECT but silent" (post-creation rejection).
4. **Second listener:** run a parallel `SetWinEventHook(EVENT_OBJECT_FOCUS)` listener in the
   harness; during a recurrence, it seeing events NVDA misses = NVDA-side drop; it seeing nothing =
   comctl32/OS-side loss.
5. **Recovery trials, only meaningful during a recurrence:** `Alt+Tab` out/in, focus toggle,
   app-fired `NotifyWinEvent`, control recreation — in that order, one at a time, `NVDA+Tab` after
   each.

## Tracker search queries used (for reproducibility)

GitHub web search and `api.github.com/search/issues` over `repo:nvaccess/nvda`: "SysListView32 list
items not announced focus stuck restart", "EVENT_OBJECT_FOCUS listview winEvent dropped", "stops
announcing focus changes until restart winEvent hook", "list items silent restart NVDA", "focus
stays on the list", "reports the list instead item", "winEvents stop being received", FileZilla/wx
variants. Best matches are cited in §3; none reproduces this anomaly.
