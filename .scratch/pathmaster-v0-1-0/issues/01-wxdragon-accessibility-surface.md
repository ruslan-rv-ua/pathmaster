# wxdragon accessibility surface

Type: research
Status: claimed
Blocked by: —

## Question

What accessibility surface does wxdragon actually expose to Rust, and what did its prebuilt wxWidgets
binaries enable? The stack is fixed, so this is a fact-finding job, not a comparison.

Answer each with evidence (file + line, or doc page), and say **"unknown — needs a spike"** where no source
settles it rather than inferring:

- Which wxWidgets version do wxdragon's prebuilt libraries bind, and in which configuration (MSVC vs MinGW,
  release, static vs shared)? Where does `build.rs` decide it?
- Is `wxUSE_ACCESSIBILITY=1` in the prebuilt libraries' `setup.h`? `wxAccessible` is compiled out entirely
  when it is not, and it is **not** the default in every build.
- Does wxdragon bind `wxAccessible` at all? Search the crate API and source for `accessible`, `MSAA`, `UIA`,
  `IAccessible`.
- Does it expose a native window handle (`wxWindow::GetHandle` → `HWND`) for any widget, and in what Rust type?
  Can that value be handed to the `windows` crate? This is the hinge for the announcement ticket.
- Does it bind `SetName`, `SetLabel`, `SetToolTip`, `SetHelpText` — the members that feed the MSAA name and
  description on wxMSW?
- Any issue or PR history in the wxdragon repo about accessibility, screen readers, or `HWND` access?
- Current published version, release cadence, and how a fork would be maintained if one were ever needed.

Findings → `../research/01-wxdragon-accessibility-surface.md`.
