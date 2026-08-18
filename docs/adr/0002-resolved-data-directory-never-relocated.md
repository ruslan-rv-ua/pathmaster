# The Data Directory is resolved from the executable, and never relocated

PathMaster's portability promise is that it writes nothing outside its own directory, so where that directory
is has to be answered exactly once and never negotiated. Two choices were open, and both were decided against
the obvious option.

**The location is resolved, not inherited.** `std::env::current_exe()` returns the path the process was
launched through, reparse points intact — measured: a binary started through a directory junction reports the
junction, not its target. Taking that path at face value would put `data\` wherever the launcher happened to
point, and under winget that is `%LOCALAPPDATA%\Microsoft\WinGet\Links\` — a directory shared with every other
portable package on the machine. So the executable's path is resolved deliberately, the `\\?\` prefix that
resolution introduces is stripped, and `data\` is placed beside the *real* binary. This also makes the
decision independent of a behaviour that could not be measured here (whether file symlinks resolve like
junctions), which is a large part of why it was chosen.

**When that directory cannot be written, the application degrades rather than relocating.** The tempting
option — offer the user another folder and remember the choice — is self-defeating: remembering a location
outside the application's own directory requires writing outside the application's own directory, which is
the whole promise. Refusing to start was also rejected, because for a screen-reader user an application that
exits explains less than one that opens and says why it is only reading. So the application starts in
**Read-only Data**: PATH is read, diagnostics run, existing Snapshots are listed, and every write path is
closed with the reason named.

## Consequences

- **There is no user-facing setting for the data location, and there cannot be one** without abandoning the
  portability promise. A future `--data-dir` switch stays viable only because it carries the location per
  launch instead of remembering it.
- **`data\` follows the binary, not the launcher.** An executable on a USB stick started from a desktop
  shortcut keeps its data on the stick. A scoop upgrade lands in the versioned directory where the
  `persist: data` junction already is.
- **Read-only Data is a whole-application mode, not a per-action failure.** It reuses the Editing Session's
  `writable` property, so every editing action is disabled rather than failing at Apply — a Working Copy that
  can never be applied is a trap.
- **The mode governs the UI only; Apply verifies by writing.** Apply begins with a Snapshot write and treats
  its failure as an Apply failure, so a stale startup verdict can never cause a silent one.
- **An executable in `C:\Program Files\` behaves differently elevated and unelevated** — writable in the first
  case, Read-only Data in the second. This is accepted as honest rather than special-cased, because the
  alternative is inventing a second data location.
- **The "no traces" claim becomes a claim about the process**, not about the machine, and package managers get
  documented separately: `winget uninstall` deletes the package directory and every Snapshot in it.
