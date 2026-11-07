# GymDiff task runner — run `just` to list recipes.
# Install: `brew install just` (macOS) / `winget install Casey.Just` (Windows).

set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

sim_lockfile := justfile_directory() / "target" / "simulator" / "lockfile"
sim_live_url := "http://127.0.0.1:12999"

# Windows PC reachable over SSH (see docs/Workflow/Windows testing.md)
win_host := env("GYMDIFF_WIN_HOST", "")
win_repo := env("GYMDIFF_WIN_REPO", "C:\\dev\\gymdiff")

# List recipes
default:
    @just --list

# ---------------------------------------------------------------- setup

# One-time setup: commit tooling (needs Node, see docs/Workflow/Development.md)
setup:
    npm install
    git config core.hooksPath .githooks

# Check a commit message, or the last commit when none is given
commitlint message="":
    {{ if message == "" { "npx --no -- commitlint --last" } else { "printf '%s' " + quote(message) + " | npx --no -- commitlint" } }}

# ---------------------------------------------------------------- dev loop

# App + simulator together (the main dev loop on macOS)
[unix]
dev speed="10" scenario="fixtures/scenarios/synthetic-full-game/scenario.json":
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build -p gymdiff-simulator -p gymdiff-app
    ./target/debug/gymdiff-simulator --speed {{speed}} --loop \
        --scenario "{{scenario}}" --lockfile "{{sim_lockfile}}" &
    SIM_PID=$!
    trap 'kill -INT $SIM_PID 2>/dev/null || true' EXIT
    sleep 1
    GYMDIFF_LOCKFILE="{{sim_lockfile}}" GYMDIFF_LIVE_CLIENT_URL="{{sim_live_url}}" \
        ./target/debug/gymdiff

# Only the simulator (pass extra args, e.g. `just sim --speed 5`)
sim *args:
    cargo run -p gymdiff-simulator -- --lockfile "{{sim_lockfile}}" {{args}}

# The app against the simulator started with `just sim`
[unix]
app-sim:
    GYMDIFF_LOCKFILE="{{sim_lockfile}}" GYMDIFF_LIVE_CLIENT_URL="{{sim_live_url}}" \
        cargo run -p gymdiff-app

# The app against the real League client
run:
    cargo run -p gymdiff-app

# Live preview of ui/<path>.preview.slint (e.g. components/primary-button, app-window):
# its gallery, or one story (cargo install slint-viewer)
preview path story="":
    slint-viewer --auto-reload {{ if story == "" { "" } else { "--component " + story } }} crates/gymdiff-app/ui/{{path}}.preview.slint

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

# Type-check the Windows build from macOS (catches #[cfg(windows)] errors)
check-windows:
    cargo check --workspace --all-targets --target x86_64-pc-windows-msvc

# ---------------------------------------------------------------- Windows PC

# Push, then pull + build a release on the Windows PC over SSH
[unix]
win-build: _require-win-host
    git push
    ssh {{win_host}} "cd /d {{win_repo}} && git pull --ff-only && cargo build --release -p gymdiff-app"
    @echo "Built on {{win_host}}: {{win_repo}}\target\release\gymdiff.exe"

# Fetch the latest GymDiff log from the Windows PC into target/windows.log
[unix]
win-logs: _require-win-host
    ssh {{win_host}} 'powershell -NoProfile -Command "Get-Content -Tail 5000 (Get-ChildItem $env:LOCALAPPDATA\GymDiff\logs | Sort-Object LastWriteTime | Select-Object -Last 1).FullName"' > target/windows.log
    @echo "Saved to target/windows.log"

[unix]
_require-win-host:
    @test -n "{{win_host}}" || (echo "Set GYMDIFF_WIN_HOST (e.g. florian@gaming-pc.local)" && exit 1)
