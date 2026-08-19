# Elevation is a whole-app relaunch, never a write helper

PathMaster edits the System `PATH` only from an elevated process, and it gets that process by
relaunching **the whole application** via `ShellExecuteEx("runas")` — there is no elevated write
helper, even though single-exe packaging would make a helper "free" (the same binary with a flag).
The alternatives were real, so the choice is recorded.

**The helper was the seductive option.** `pathmaster.exe --elevated-write` would keep the UI
unelevated: Editing Sessions would survive, the UAC prompt would appear only at Apply, and the
NVDA-and-elevated-windows question (below) would vanish. It loses on three counts:

- **It is an elevation-of-privilege surface.** A binary that accepts arguments and writes
  `HKLM` under an elevated token must authenticate its caller and seal its input channel, or it is
  a confused deputy waiting for a hostile caller. Microsoft's own guidance for the COM elevation
  moniker — the sanctioned version of this pattern — warns that doing it without an exploitable
  hole is not simple. v0.1.0 has no security review budget to spend on this.
- **It prompts per write.** The Windows UX guide's rule is "once elevated, stay elevated until the
  task is done"; a helper shows the secure-desktop prompt on every System Apply, and the secure
  desktop is exactly the screen NVDA cannot read without its settings copied to the system config.
  One prompt per elevated session beats one per write.
- **Precedent is unanimous.** PowerToys Environment Variables and Rapid Environment Editor — the
  closest neighbours in this domain — both relaunch the whole application as administrator.

**The relaunch's cost is contained by decisions already made.** An Editing Session never survives
a process boundary (ticket 06), so the relaunch destroys unsaved changes — but it runs *through*
the close-confirm flow, never around it, and the dialog names what is lost in its title. Two live
instances are already a designed state (ticket 07, no single-instance lock), and the Data
Directory's inherited DACLs already guarantee an elevated instance's files stay writable to a
later unelevated run (measured, ticket 07).

## Consequences

- **One entry point.** A single menu command — "Restart as Administrator", disabled when already
  elevated — is the only way in. Read-only Data names its reason but does not grow a second
  elevation offer; the System tab's disabled editing controls do not either. The command does what
  it says: on a successful spawn the original instance exits.
- **Detection is `GetTokenInformation(TokenElevation)`**, never `TokenElevationType` — the latter
  returns `Default` for the built-in Administrator and UAC-off configurations, precisely the cases
  a writability decision must not misread.
- **A declined UAC prompt is never silent.** `ShellExecuteEx` reports `ERROR_CANCELLED`; the
  application answers with a dialog (message in the title, per the dialog discipline) and carries
  on unelevated. Silence after a security prompt is treated as a defect.
- **The elevated instance is a separate accessibility surface.** NVDA interacts with elevated
  windows only when installed (uiAccess) or itself elevated — a portable NVDA copy goes deaf on
  the elevated instance. The verification checklist (ticket 19) must run its steps against the
  elevated instance explicitly, and the README must name the installed-NVDA requirement.
- **Only the active tab crosses the boundary** (one command-line argument). Session state does
  not; the boundary ticket 06 declared impenetrable stays impenetrable.
