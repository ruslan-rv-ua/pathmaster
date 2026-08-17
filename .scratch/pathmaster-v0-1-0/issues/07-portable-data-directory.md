# Portable data directory contract

Type: grilling
Status: open
Blocked by: —

## Question

Where does `data/` live, and what happens when the exe's own directory is not writable?

Settled at charting: portable-first, `data/` next to the exe, scoop declares `persist: data`, and
NFR-no-registry-writes is reworded from "nothing in AppData" to "nothing outside the app's own directory".
What remains is the behaviour at the edges.

- **Read-only install location.** Someone will drop the exe in `C:\Program Files\`. Refuse to start? Run in a
  read-only mode? Prompt for a writable location? Silently degrade is not an option — the app's whole promise is
  that it writes backups before it touches the registry.
- **Honesty of the "no traces" claim.** Under winget the exe lands inside `%LOCALAPPDATA%`, so `data/` does
  too. Decide how the README states this without the promise becoming a lie.
- **Path resolution.** `std::env::current_exe()` versus the symlink winget puts on `PATH`: resolving the link
  target and resolving the link itself place `data/` in different directories. Pick one and state why.
- **Two instances.** One elevated, one not, both writing `data/`. Single-instance lock, or last-write-wins?
  A second instance is *likely* here, because elevation is implemented by relaunching the app.
- **ACLs.** Files written by an elevated instance may be unwritable by the normal one afterwards — which breaks
  backup rotation silently. Mitigation, or accepted with a documented consequence?

Output: the rewritten NFR-no-registry-writes and TC-file-structure, plus the startup decision tree.
