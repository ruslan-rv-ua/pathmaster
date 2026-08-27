# 01 — Borrow-discipline retrofit: scoped access and the modal door

**Spec:** [delta-spec §11](../../pathmaster-v0-2-0/spec.md) · ADR-0011

**What to build:** The total retrofit the spec orders as the first implementation ticket, before any new surface is coded. No user-visible behaviour changes; what changes is that the UI's borrow discipline stops being a doc comment and becomes structure: every multi-entry-point cell sits behind a scoped-access wrapper, and every modal dialog passes through one guarded door that silences the Timer's tick while a dialog is up.

**Blocked by:** None — can start immediately.

**Status:** done

- [x] Every cell reached by more than one kind of call (command / Timer tick / synchronous toolkit callback) goes behind a `with`/`with_mut` wrapper whose guard cannot escape — today both Sessions, both `findings`, the Backups page's file cell; the classification rule is written down so a future cell classifies itself (`crates/pathmaster/src/scoped.rs`)
- [x] One module owns a Drop-guarded modal-depth `Cell` and the single function every `show_modal`/message box passes through; the Timer's tick handler is inert while depth > 0, and the Timer itself keeps firing (`Pump`'s self-healing preserved — a pass landing mid-dialog is collected ≤ 100 ms after close) (`crates/pathmaster/src/ui/door.rs`; the gate is in `bind`'s tick closure)
- [x] A source-scan `#[test]` fails the build if `show_modal` appears outside the door module (`crates/pathmaster-core/tests/modal_door.rs`, the versioninfo genre — token scan, comments included, with self-staleness guards)
- [x] All ~47 existing borrow sites and every dialog call are converted in this one ticket — no two-regime transition period survives it (remaining `.borrow` sites are the sanctioned plain cells: `last_read`, `settings`, the Pump's worker)
- [x] The `App` doc comment that carried the discipline as prose is deleted (replaced by a short pointer to `Scoped`, `door` and ADR-0011)
- [x] No behaviour change: the full existing test gate stays green (`just ci` exit 0), and a manual run shows dialogs, ticks and commands working as before (staged-copy probe: Add dialog opened through the door and held across ticks, commit landed, the pass was collected — StatusBar field 1 populated — a second dialog abandoned mid-ticks, Undo restored the count, clean exit 0)

## Comments

**2026-08-27 (implementation)** — Landed as `scoped.rs` (the wrapper), `ui/door.rs` (the door: thread-local
depth `Cell`, `Opened` drop guard, a `Modal` trait naming the three wxdragon dialog types, `door::show` /
`door::modal_open`), and `pathmaster-core/tests/modal_door.rs` (the scan — deliberately a token scan,
comments included, with two self-staleness guards: the door file must still contain `show_modal`, and the
walk must find sources at all). One design consequence beyond the ticket's letter: ADR-0011's closure rule
("no page rebuild inside a `with` closure") meant `ScopePage::render` could no longer take `&Session` — it
now takes owned `Row`s (`scope_page::Row`, path + composed status), which `Row::compose` builds inside the
scoped access and callers render after the closures have died. `render_status` reads the same `Row`s, so
the Status column has one composition path; `ScopePage` consequently no longer holds the Catalogue.
Verified: `just ci` green, and a staged-copy live probe (Add dialog through the door held across ticks →
commit → pass collected → second dialog abandoned mid-tick → Undo → clean exit 0).
