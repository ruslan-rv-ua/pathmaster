# Command runner (https://just.systems) — `just` alone lists every recipe.
# Recipes mirror the workflow they stand in for, same flags and same order, so this
# file and CI are one list to keep in step, not two.

set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

# List the recipes
default:
    @just --list

# Formatting needs no compilation, so a drifted file fails in seconds instead of
# after a wxWidgets build — hence fmt first, the way CI orders it. (`just --list`
# reads the line right above a recipe as its description, so summaries sit last.)
# The push-CI gate (.github/workflows/ci.yml) run locally, in CI's order
ci: fmt-check test clippy

# Reformat the workspace
fmt:
    cargo fmt --all

# Formatting check exactly as CI runs it
fmt-check:
    cargo fmt --all -- --check

# Test the workspace exactly as CI runs it
test: _no-rustflags
    cargo test --workspace --locked

# Clippy exactly as CI runs it — warnings are errors
clippy: _no-rustflags
    cargo clippy --workspace --all-targets --locked -- -D warnings

# Run the app, dev profile
run: _no-rustflags
    cargo run -p pathmaster

# Slow by design: LTO and one codegen unit (spec §16). --target explicit is
# load-bearing — it keeps RUSTFLAGS off host build scripts and proc-macros.
# Lands at target\x86_64-pc-windows-msvc\release\PathMaster.exe.
# The release exe exactly as release CI builds it
release: _no-rustflags
    cargo build --release --locked --target x86_64-pc-windows-msvc

# Needs ImageMagick — see tools/README.md.
# Rasterise icon.svg into app.ico after editing the SVG; commit both
icon:
    .\tools\make-icon.ps1

# Wants the exe `just release` leaves behind — see tools/README.md.
# Regenerate the READMEs' screenshots (release-checklist step F1)
screenshots:
    .\tools\make-screenshots.ps1

# A RUSTFLAGS environment variable silently overrides .cargo/config.toml, dropping
# crt-static and with it the portable exe (research/04) — refuse to build under it.
_no-rustflags:
    @if ($env:RUSTFLAGS) { throw 'RUSTFLAGS is set — it silently overrides .cargo/config.toml (crt-static, research/04). Unset it first.' }
