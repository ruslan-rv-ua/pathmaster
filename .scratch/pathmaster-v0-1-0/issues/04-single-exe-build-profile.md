# Single-exe build profile

Type: research
Status: open
Blocked by: —

## Question

Can a wxdragon build satisfy "one exe, no runtime to install, ≤ 40 MB, cold start ≤ 2 s" — and with exactly
which build settings?

Prefer **measurement over documentation**: build the minimal app and report real numbers.

- **CRT linkage.** Do wxdragon's prebuilt libraries force `/MD` (needs the VC++ redistributable) or allow
  `/MT` (static)? Can it be forced from the consuming crate? Does the MinGW/UCRT route avoid the problem?
  NFR-portable — "runs with no dependency install" — is 🔴 must and hangs entirely on this answer.
- **Size.** Actual release `.exe` size for a minimal wxdragon window, and what `opt-level`, `lto`,
  `codegen-units`, `panic=abort` and `strip` each buy. Compare against the ≤ 40 MB budget set at charting.
- **Cold start.** Measured time from process start to a visible window on an SSD.
- **Embedded resources.** How the icon and the translation catalogs get inside the exe (`include_bytes!`,
  a Windows `.rc` resource, wx XRC), and how the application manifest is embedded — `requestedExecutionLevel:
  asInvoker`, per-monitor DPI awareness, and **comctl32 v6**, without which wx silently falls back to
  legacy-looking controls with different accessibility behaviour.
- **The config CI must pin**: toolchain, target (`x86_64-pc-windows-msvc`), and anything that must not drift.

Findings → `../research/04-build-profile.md`. If any 🔴 must is unreachable, say so plainly — that is a finding,
not a failure.
