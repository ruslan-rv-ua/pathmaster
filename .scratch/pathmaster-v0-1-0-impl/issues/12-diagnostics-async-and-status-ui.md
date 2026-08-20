# 12 — Async diagnostics pass, Status column, StatusBar fields

**Spec:** [spec §7 (FR-diag-async, FR-diag-status), §12 (StatusBar)](../../pathmaster-v0-1-0/spec.md)

**What to build:** Diagnostics come alive in the UI: a worker thread runs the ticket-09 rulebook over the Working Copies after load and after every change, the Status column fills with translated Issue-type words, and both StatusBar fields stay current — including the always-visible merged-length field. NVDA reads "{path}; Status: {types}" for free on every arrow key.

**Blocked by:** 08 (UI + Sessions), 09 (the rules).

**Status:** resolved

- [x] One worker thread runs a pass over the Working Copies (never the process environment, never the registry); results reach the UI via an `mpsc` channel drained by a wx Timer (~100 ms, running only while a pass is outstanding); widgets never called off the UI thread
- [x] A pass runs at load and after every Working Copy change (edit, undo/redo, Refresh, Restore); a System edit recomputes User's Issues too (cross-scope duplicates); Issues never enter Checkpoints
- [x] Status column carries the flagged types' words, comma-joined most-severe-first (Missing > Relative > Quoted > Duplicate > Empty; uk: Відсутній, Відносний, У лапках, Дублікат, Порожній); an empty column is the only healthy state — never "OK", no severity prefix, no icons
- [x] StatusBar field 0: "User PATH: {n} entries ({m} issues) | System PATH: {n} entries ({m} issues)", updated after every pass and Apply
- [x] StatusBar field 1 always shows "Merged PATH: {n} chars", appending " — exceeds 8,191 (cmd.exe limit)" past that threshold; over-length never appears in the Status column and is never an Announcement
- [x] Budget: full pass < 1 s for ≤ 200 entries
- [x] Issue-type words and StatusBar texts are in the Catalogue with Ukrainian translations

## Comments

Implemented 2026-08-20 on `feature/diagnostics-async-and-status-ui`. The rulebook ticket 09 built now
runs: the Status column fills, both StatusBar fields track it, and every edit re-diagnoses both
Scopes without the UI thread doing any of the work. Verified live against this machine's real PATH —
42 User entries, 19 System — and nothing was written to the registry, because Apply is ticket 13.

**Two adapters and a worker, in `pathmaster-platform::diagnostics`** — ticket 09's hand-off, taken up
as it was left:

- `ProcessEnvironment` over `GetEnvironmentVariableW`, whose lookup ignores case, so `%systemroot%`
  and `%SystemRoot%` are one variable. An undefined name answers `None`, never an empty string: the
  rules turn `None` into "left literal and reported unresolved", and an empty value would expand the
  reference silently away.
- `LocalFilesystem` over `GetDriveTypeW` (root) and `GetFileAttributesW` (probe).
- `Worker`, which owns the thread, the two `mpsc` channels, and — the part that matters — the
  generation counter.

**The staleness rule lives in the worker, not in its caller.** A pass is asynchronous, so an edit can
land while one is in flight, and the answer it brings back describes rows that have since moved.
`Worker::take()` therefore returns only the pass whose generation matches the last one asked for and
drops the rest; `outstanding()` is what the Timer runs on, and it stops the moment nothing is in
flight. The worker also **coalesces**: a burst of edits queues one request each, and running them in
turn would spend the whole budget on states the user has already left, so the queue is drained to its
last element before a pass starts. Both rules are asserted rather than timed — the tests drive a
filesystem that blocks inside `probe` until the test lets it through, so "the overtaken pass never
reached the UI" is a sequence, not a sleep.

**The Status column reads the last pass by Entry id, not by row.** This is the ticket's one real
design decision and it is an accessibility one. Rebuilding the column from the live `ScopeDiagnosis`
after an edit would show the previous pass's words against the *new* row order — a stale "Missing" on
the path the user has just corrected, in the exact window where focus is landing on that row and NVDA
is reading it aloud. Blanking the whole column instead is no better: an empty column is the healthy
state, so blanking says "clean" about rows nobody has looked at yet. So `Findings` (in the core, where
tests reach it) keeps each Entry's Issues **beside its id and the text they were computed from**, and
shows them only when both still match. A row that merely moved keeps its words; a row whose text
changed carries none until the next pass, which is ~one Timer tick away. Measured live: deleting the
first Entry left every Status word on its own path, one row higher.

**A pass landing must not move the user**, so `render_status` writes column 1 and touches nothing
else. The full `render` rebuilds the list — `delete_all_items` clears the selected-and-focused row —
and a pass finishes on its own schedule, including mid-arrow-key. It is also how a System edit
reaches the User tab, whose rows did not change but whose duplicates did.

**Everything under `\\` is Network and goes unprobed**, the device namespace included. `\\?\UNC\…` is
a UNC path wearing a prefix and `\\?\C:\…` is not, and the cheap way to be sure is not to ask: a
`\\?\C:` Entry therefore never flags Missing. That is a false negative on a spelling all but unheard
of in a `PATH`, traded against the 20-60 second uncancellable block a dead UNC costs — which is the
entire reason the question exists. Text with *no* root — an Entry whose leading `%VAR%` this run does
not define — is Local and must be, because failing at the probe is how it flags Missing (§7 D10).

**Two hazards the worker thread guards, both about not being the UI thread.** `SetThreadErrorMode`
(`SEM_FAILCRITICALERRORS`) is set once at thread start: probing `A:\…` on a machine with no medium in
the drive raises the OS's own "no disk" dialog, from a thread that owns no window. And nothing here
touches a widget — wx's event tables are thread-local, so a widget call from the worker would not
crash, it would silently do nothing, which is worse.

**Access denied is a branch the machine will not demonstrate, and that is written down.** The test
that tried to provoke `ERROR_ACCESS_DENIED` from `GetFileAttributesW` measured the opposite:
`FILE_READ_ATTRIBUTES` is granted implicitly to anyone who may traverse the parent, so a deny-ACL'd
directory reads its attributes back, and so does `C:\System Volume Information` on an ordinary
account. The branch stays for the hardened tokens without that implicit grant; the rule it feeds —
access denied is never Missing — is held in `core/tests/diagnostics.rs` where a fake can say so, and
the platform test now pins what the machine *can* show: a directory the user cannot read is still a
directory, never NotFound.

**Eight msgids**, both `.po` files, gate passing. The five Issue words are single words by spec
(«Відсутній», «Відносний», «У лапках», «Дублікат», «Порожній») and there is deliberately no word for
a healthy Entry — "OK" is a string this Catalogue must not be able to say. The two StatusBar strings
are **suffixes** rather than one string each, for the reason Announcement 5's ", unsaved changes" is
one: a gettext lookup selects its plural form on one number, and field 0's line carries two. The
8,191 in the over-length suffix is literal text, not a placeholder — a measured OS constant, and the
one fact the sentence exists to carry must not be droppable by a translation. «проблема» for Issue
follows the user's own Ukrainian in `spec-input.md`.

**Budget, measured**: a 200-entry pass over real local paths finishes in under 10 ms — two orders
inside §7's one second. The test that pins it is not a performance gate at that headroom; it is a
*network* gate, because the one way this pass can take seconds is by probing a root it was told never
to touch.

### What the live run showed

Driven through the running window (`PostMessage(WM_COMMAND, …)` for menu commands, `WM_SETTEXT` +
the dialog's own button id for the Add dialog):

- All five Issue words render in Ukrainian on the right rows, healthy rows empty.
- StatusBar field 0: «PATH користувача: 42 записи (20 проблем) | PATH системи: 19 записів (9 проблем)»
  — and its two numbers answer to different clocks, on purpose. Delete an Entry and the count is 41
  immediately while the issue count still reads the last pass's, catching up one tick later. The
  entry count is the screen's; the issue count is the pass's, which is what "updated after every
  pass" means.
- StatusBar field 1 tracked the edit exactly: 2229 → 2198 on deleting a 30-character Entry, which is
  the Entry plus its separator.
- The over-length suffix fires: a 6,203-character Entry took the field to
  «Об'єднаний PATH: 8433 символи — перевищує 8 191 (ліміт cmd.exe)», with no Announcement and nothing
  new in that row's Status column, which read «Відсутній» alone.
- **Field 1's text is wider than field 1 at the default window size**, and the tail clips visually.
  `SB_GETTEXTLENGTH` says the control holds all 63 characters, so `NVDA+End` — which reads the
  control's text, not its pixels — speaks the whole sentence. Left alone deliberately: no split of a
  900 px bar fits both fields' longest texts, spec §12 makes the Status column the app's *single*
  pixel constant so a second one is out, and the field is command-only by design. Recorded here so
  the next person measures it rather than rediscovering it.

One method note for whoever drives this window next: `LVM_SETITEMSTATE` and `SB_GETTEXT` pass
pointers, which are not marshalled across processes — sending them from a probe **crashes comctl32 in
the target** with an access violation. Click with `WM_LBUTTONDOWN`/`UP` (coordinates are values) and
read lengths with `SB_GETTEXTLENGTH` instead. The crash that cost twenty minutes here was the probe's,
not the application's.

The GUI itself stays Release-Checklist territory (ADR-0007); steps 2–4 and 16 already name what this
ticket delivers, so the Checklist gains nothing new.
