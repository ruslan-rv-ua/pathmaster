# PathMaster — proposed features and where they might go

Candidates raised in the brainstorm of **2026-08-28**, each with a suggested version. Written
down so nothing has to be remembered, and so a suggestion can be argued with rather than
re-invented.

## What this document is not

- **Not a promise.** Nothing here is committed until that release's effort is charted and its
  spec is locked under `.scratch/<slug>/`. The README's rule is unchanged: what is merely not
  here yet lives in the tracker, not in the README.
- **Not the tracker.** A ticket is a question with an owner and a resolution. A line here is a
  candidate with a guess at its place.
- **Not a re-litigation.** Everything the v0.1.0 and v0.2.0 specs cut, deferred or declined
  (§20 of each) stays that way. The last section names them so they are not proposed twice.

## How the versions were chosen

Each version is **one theme**, not a bag of features, and the grouping follows the machinery:
items that need the same new infrastructure ship together, so that infrastructure is designed
once for all of them rather than three times.

Where an item has no version, it is because a decision is missing — not because it is small.

## At a glance

| Feature | Version |
|---|---|
| Effective PATH view | v0.3.0 |
| Which entry wins? (command resolver) | v0.3.0 |
| Shadowed (Issue type) | v0.3.0 |
| Not a folder (Issue type) | v0.3.0 |
| Unreadable folder (Issue type) | v0.3.0 |
| Multi-select and bulk operations | v0.4.0 |
| Edit as text | v0.4.0 |
| Move to top / bottom / to position | v0.4.0 |
| Paste several entries | v0.4.0 |
| Open the entry's folder | v0.4.0 |
| Review changes before Apply | v0.5.0 |
| Snapshot on demand, with a note | v0.5.0 |
| Compare a Snapshot with the current value | v0.5.0 |
| Export and import | v0.5.0 |
| Collapse an entry to a variable | unplaced — needs a decision |
| "Everything under X" prefix filter | unplaced — needs a decision |
| Empty folder (Issue type) | unplaced — needs a decision |
| Other list-shaped variables | its own effort, not a version slot |
| A command-line mode | its own effort, not a version slot |
| A second screen reader | verification work, any version |
| Code signing | a channel decision, any version |
| Disabled entries | argued against |
| Auto-update | argued against |

---

## v0.3.0 — What the PATH actually does

The theme: v0.1.0 and v0.2.0 diagnose the **shape** of a PATH — entries that do not exist, are
quoted, are duplicated. None of them answers the question a PATH exists to answer, which is
*which program runs when I type a name*. That question is the release.

All five items share one new piece of machinery — **listing a directory's contents**, which the
application has never done (today it only `stat`s a path). Designing that once is the reason
they are one release: off the UI thread, cached, `PATHEXT`-aware, and skipping UNC roots under
the standing network rule.

| Feature | What it is | Why here | Cost and risk |
|---|---|---|---|
| **Effective PATH view** | A third, read-only surface: the machine's value followed by the user's, in real resolution order. | The other items are about resolution order, and no surface shows it. Today the boundary between Scopes has to be held in the head. | A tab that is not a Scope breaks the "tab = Scope" reading the whole UI rests on. Search and Filter are per-Scope — what they mean here is the design question. No Editing Session, no Apply, no Undo. |
| **Which entry wins?** | Type a command name, get every entry that provides it, in order, with the winner named. `where.exe`, in the application's own terms. | The everyday failure is never "the folder is missing" — it is "the wrong python runs". | First directory *listing* the app has ever done: off the UI thread, cached, UNC skipped. A new dialog and new Announcements — and the Announcement set is closed by decision, so each one is a change to it. |
| **Shadowed** | An entry provides an executable an earlier entry already provides, so it is unreachable for that name. | Same scan as the resolver: its answer, turned into a standing diagnostic. | A seventh Issue type: a Filter radio item, a Catalogue string, a Release Checklist step. The Status column holds one word, so *what it says* about "shadowed by entry 4" is a real design question. Shadowing also crosses Scopes, where today only Duplicate does. |
| **Not a folder** | The entry exists but is a file. | The `metadata` call that answers Missing already knows this and throws it away. | Cheap in itself; the new-Issue-type cost above applies. |
| **Unreadable folder** | The entry exists but its contents cannot be read. | Same call again: a permission error is today indistinguishable from Missing, and the two need different repairs. | As above. Three new Issue types in one release is three new Filter items — which is the growth ticket 07's exclusive-state model was chosen to allow. |

## v0.4.0 — Editing at scale

The theme: every editing command acts on exactly one entry. On a forty-entry PATH that is not
editing, it is arithmetic. Nothing here needs new I/O; it is all Working Copy operations and
one Checkpoint each.

| Feature | What it is | Why here | Cost and risk |
|---|---|---|---|
| **Multi-select and bulk operations** | Native list multi-selection; Delete, Move and Copy act on the selection. | The foundation the rest of this release stands on. NVDA reads native multi-selection without help. | Every command that today says "the focused Entry" has to learn to say "the selection", and the Announcements need plurals. One Checkpoint per operation is already the rule. |
| **Edit as text** | The whole PATH in one multi-line field, one entry per line; OK is a single Checkpoint. | The escape hatch for every operation not in the menus — and a plain text field is the most navigable widget a screen reader has. | Round-trip fidelity is the risk: leading and trailing whitespace are part of an Entry and must survive, while a line-based editor makes them invisible. An Entry cannot contain a newline, so the encoding is at least total. |
| **Move to top / bottom / to position** | Three commands beside the existing Move Up and Move Down. | `Alt+↑` thirty times is the problem multi-select does not solve. | Three accelerators and three menu items; `OPERATION_MOVE` already exists to name them in undo. |
| **Paste several entries** | Clipboard text, split on newlines — or on `;` for a whole pasted PATH — added as one Checkpoint. | Copy already exists; this is its other half, and how a PATH arrives from a colleague or a wiki. | The split rule needs deciding: newlines and `;` mean different things. The Add dialog's rejection of forbidden characters applies unchanged. |
| **Open the entry's folder** | Hand the entry to the shell. | Cheapest item in this document: `ShellExecuteW` is already in `platform/shell.rs` with a directory caller. | Only the edge cases need deciding: what it does for a Missing, a Relative or a UNC entry. |

## v0.5.0 — Seeing the change

The theme: "nothing is irreversible" currently rests entirely on undo *after* the fact. Three of
these four are the same machinery used three ways — a diff over two lists of Entries that tells
**moved** apart from **removed and added again**. That algorithm is the release; the dialogs
around it are ordinary.

| Feature | What it is | Why here | Cost and risk |
|---|---|---|---|
| **Review changes before Apply** | Added, removed, moved, edited — between the Working Copy and its Baseline, before the registry is written. | The check that belongs *before* the irreversible-looking act, not after it. | The matching algorithm is the whole feature. Whether it is a step inside Apply or a separate command is a design question: an extra confirmation on `Ctrl+S` is a cost, not a gift. |
| **Snapshot on demand, with a note** | "Take a copy now, call it *before installing Node*." | Today a Snapshot only ever happens as a side effect of Apply — never before something *else* changes the PATH. | The Snapshot schema (ADR-0006) gains a field, and old files must still load. It counts toward rotation like any other. |
| **Compare a Snapshot with the current value** | The diff above, run between a saved copy and now. | "What did that installer do to my PATH last Tuesday" — and the reason this belongs beside the diff rather than in its own release. | Almost free once the diff exists. |
| **Export and import** | The value out to a file or the clipboard, and back. | Moving to a new machine, and attaching a PATH to a bug report. | Import lands as one Checkpoint and never writes the registry directly — the rule Restore already follows. Plain text is enough; a `.reg` export invites a double-click that writes the registry behind the application's back. |

## Unplaced — a decision is missing, not a version

| Feature | What it is | What has to be decided first |
|---|---|---|
| **Collapse an entry to a variable** | `C:\jdk21\bin` becomes `%JAVA_HOME%\bin` when `JAVA_HOME` is exactly `C:\jdk21` — the mirror of Expansion Mode, and the only answer to Over-length that is not "delete something". | Which variables are candidates: every defined one, or a curated set? And what happens when two of them match the same prefix. Over-length is a Scope-level Issue with "its own surface" (v0.2.0 §20) — this would be that surface, which does not exist yet. |
| **"Everything under X" prefix filter** | Narrow the list to entries under a chosen prefix, reachable from the Tree View. | Already named by ticket 08 as a future *Filter* feature rather than the Tree's job. What names the prefix, and whether it composes with the exclusive Filter state or sits beside it. |
| **Empty folder** | An entry whose folder contains nothing. | PATH is searched for DLLs as well as executables, so "no executables" is not "dead weight" — a false positive of exactly the kind the typo diagnostic was cut for. Flagging only a *completely* empty folder may be the whole answer, or the answer may be no. |

## Their own effort, not a version slot

- **Other list-shaped variables** — `PATHEXT`, `PSModulePath`, `PYTHONPATH`, `CLASSPATH`,
  `GOPATH`, `INCLUDE`, `LIB`: the same `;`-separated shape, the same problems, the same UI and
  the same diagnostics. This is **not** the general environment-variable editor the PRD declined
  (OS-other-env-vars); it is only variables of this one shape. The cost is that Scope stops
  being "User or Machine" and becomes "(Variable, Scope)" — a change to the domain model, which
  is a charting exercise, not a line in a release.
- **A command-line mode** — `--check` with a non-zero exit code when a PATH has Issues, and
  `--list`. `pathmaster-core` is already a separate crate, so this is a second surface on
  finished machinery, and the only way the tool ever reaches somebody's CI. One real obstacle:
  the executable is `#![windows_subsystem = "windows"]`, so it has no console to print to —
  `AttachConsole(ATTACH_PARENT_PROCESS)` is the usual answer, and it needs proving before it is
  promised.
- **A second screen reader** — JAWS or Narrator are today "not deliberately broken, not tested".
  This is verification work, not feature work: the Release Checklist names NVDA's exact expected
  speech, so a second reader means a second column of expected speech and a maintainer who runs
  it. It probably widens the audience more than any feature above.
- **Code signing** — deferred "until there are real users", and priced under the old
  EV-certificate model. Azure Trusted Signing changed that price materially (**verify the
  current figure before relying on it**). SmartScreen's "Windows protected your PC" is a
  first-run barrier for exactly the people who have not yet decided to trust the project.

## Raised and argued against

Recorded so they are rejected on purpose rather than forgotten.

- **Disabled entries** — park an entry without deleting it, to test whether it is the culprit.
  Genuinely useful, and it keeps state outside the registry that nothing in the Baseline /
  Working Copy model can account for. The model is worth more.
- **Auto-update** — contradicts portability, and OS-auto-update was already out of scope in the
  PRD.

## Already decided against — do not propose again

Each of these carries its reason in the spec that killed it; this list exists only to stop the
same idea arriving twice.

- **v0.2.0 §20** — drag & drop reorder (cut 2026-08-26); in-app deaf-state detection; the
  network-path deadline prober; the winget submission; collapsing `ScopeDiagnosis` into
  `Findings`; multi-select *Filter* states (unrelated to multi-select in the list, above); a
  `Ctrl+Insert` copy twin; generating the User Guide's keyboard table from source. Declined
  within v0.2.0: the Search-bar coupling from the tree; severity classes; an Issue type for an
  undefined `%VAR%`; a repair for Relative; F1 inside dialogs; persisting Expansion Mode, Filter
  or Search text.
- **v0.1.0 §20** — similar-path and typo diagnostics; the `theme` setting; UI automation; the
  PRD's other-variables, sync, plugins, web-CLI and auto-update items; non-Windows platforms;
  32-bit.
