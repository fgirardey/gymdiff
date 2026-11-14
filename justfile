# cooldown.lol task runner — run `just` to list recipes.
# Install: `brew install just` (macOS) / `winget install Casey.Just` (Windows).

# Load `.env` (git-ignored, see `.env.example`) for every recipe.
set dotenv-load

set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

# Windows PC reachable over SSH (see docs/Workflow/Windows testing.md)
win_host := env("COOLDOWN_WIN_HOST", "")
win_repo := env("COOLDOWN_WIN_REPO", "C:\\dev\\cooldown.lol")

# List recipes
default:
    @just --list

# ---------------------------------------------------------------- setup

# One-time setup: commit tooling (needs Node) and the Windows cross-check (see docs/Workflow/Development.md)
setup:
    npm install
    git config core.hooksPath .githooks
    cargo install cargo-xwin --version 0.23.1 --locked

# Check a commit message, or the last commit when none is given
commitlint message="":
    {{ if message == "" { "npx --no -- commitlint --last" } else { "printf '%s' " + quote(message) + " | npx --no -- commitlint" } }}

# ---------------------------------------------------------------- dev loop

# The app against the League client running on this machine (the main dev loop)
dev:
    cargo run -p cooldown-app

# Full version of a build of the current checkout, e.g. 0.2.0+57.a1b2c3d (see ADR-0008)
version:
    @cargo run -q -p cooldown-app -- --version

# Live preview of ui/<path>.preview.slint (e.g. components/primary-button, app-window):
# its gallery, or one story (cargo install slint-viewer)
preview path story="":
    slint-viewer --auto-reload {{ if story == "" { "" } else { "--component " + story } }} crates/cooldown-app/ui/{{path}}.preview.slint

# Render the Pencil stories of ui/<path>.preview.slint files to target/screenshots (all when none given)
screenshots *paths:
    cargo run -q -p cooldown-app --example screenshots -- {{paths}}

# Screenshots of the UI changed since main, published on the pr-assets branch, as a PR table
[unix]
pr-screenshots *paths:
    .github/scripts/pr-screenshots.sh {{paths}}

# ---------------------------------------------------------------- quality

# Everything CI checks
check: fmt-check clippy test check-windows

# Format all code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Lint (pedantic, warnings denied)
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Run all tests
test:
    cargo test --workspace

# Type-check the Windows build from macOS (catches #[cfg(windows)] errors). cargo-xwin brings
# the Windows C headers that C dependencies need (ring, through Velopack); llvm-ar from
# rustup's llvm-tools stands in for llvm-lib (an LLVM tool picks its mode from its name).
[unix]
check-windows:
    #!/usr/bin/env bash
    set -euo pipefail
    tools="{{justfile_directory()}}/target/xwin-tools"
    mkdir -p "$tools"
    ln -sf "$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/host: //p')/bin/llvm-ar" "$tools/llvm-lib"
    PATH="$tools:$PATH" cargo xwin check --workspace --all-targets --target x86_64-pc-windows-msvc

# On Windows the native check is enough
[windows]
check-windows:
    cargo check --workspace --all-targets

# ---------------------------------------------------------------- Windows PC

# Push, then pull + build a release on the Windows PC over SSH
[unix]
win-build: _require-win-host
    git push
    ssh {{win_host}} "cd /d {{win_repo}} && git pull --ff-only && cargo build --release -p cooldown-app"
    @echo "Built on {{win_host}}: {{win_repo}}\target\release\cooldown.exe"

# Fetch the latest cooldown.lol log from the Windows PC into target/windows.log
[unix]
win-logs: _require-win-host
    ssh {{win_host}} 'powershell -NoProfile -Command "Get-Content -Tail 5000 (Get-ChildItem $env:LOCALAPPDATA\cooldown.lol\logs | Sort-Object LastWriteTime | Select-Object -Last 1).FullName"' > target/windows.log
    @echo "Saved to target/windows.log"

[unix]
_require-win-host:
    @test -n "{{win_host}}" || (echo "Set COOLDOWN_WIN_HOST (e.g. florian@gaming-pc.local)" && exit 1)
