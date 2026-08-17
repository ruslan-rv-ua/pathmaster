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

## Carried in from tickets 01 and 03

Two of this ticket's questions are already answered at source; a third became the ticket's most important job.

- **There are no prebuilt libraries.** The README is stale. wxdragon 0.9.18 downloads pinned wxWidgets **3.3.3
  source** and compiles it as a CMake subproject — static libs (`wxBUILD_SHARED OFF`), non-monolithic, MSVC via
  Ninja at `RelWithDebInfo`, release CRT forced even for debug Rust profiles. So the first build is a full
  wxWidgets compile, and the measured cost belongs in this ticket's answer.
- **The CRT is a build-flag decision, not a requirement rewrite.** `wxdragon-sys/build.rs:365-374` reads
  `CARGO_CFG_TARGET_FEATURE`: with `crt-static` present it builds wxWidgets as `MultiThreaded` (`/MT`),
  otherwise `MultiThreadedDLL` (`/MD`, needing the VC++ redistributable). Confirm by building **both** ways and
  checking the resulting exe's imports for `VCRUNTIME140.dll` — that is the direct test of 🔴 NFR-portable.
- **Grep the generated `setup.h` for `wxUSE_ACCESSIBILITY` once the build finishes.** This is ticket 01's open
  UNKNOWN and this ticket is the first to produce a build. It matters because wxdragon's C layer wraps every
  accessibility entry point in `#if wxUSE_ACCESSIBILITY` with **silent no-op `#else` branches**: a 0 there
  compiles, runs, and does nothing — the quietest way this product could fail.
- Pin **wxdragon ≥ 0.9.17** in whatever the build config recommends (earlier `AccRole` discriminants are
  mis-ordered).
