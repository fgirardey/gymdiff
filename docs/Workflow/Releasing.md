# Releasing

How a GymDiff version is numbered, released and made readable in Sentry. Decision: [[ADR-0008 Versioning and releases]].

## Which build am I running?
- In the app: About shows `0.2.0 (build 57 · a1b2c3d)`; `· alpha` means a build of `main` (it updates itself, see [[Auto-update]]), `· preview` a pull request build, `· dev` a local build.
- In a terminal: `gymdiff --version` → `gymdiff 0.2.0+57.a1b2c3d`. Windows release builds have no console, so pipe the output in PowerShell: `.\gymdiff.exe --version | Write-Output`. From the checkout: `just version`.
- On Windows: Explorer → Properties → Details of `gymdiff.exe`.
- In Sentry: the release `gymdiff@0.2.0+57`, the `commit`, `build` and `channel` tags, the environment (`production`, `alpha`, `preview`, `development`).

## Cutting a release
1. Merge pull requests into `main` as usual. Their Conventional Commit types decide the next version: `fix` → patch, `feat` → minor; before 1.0 a breaking change (`!`) is also a minor bump.
2. release-please keeps a pull request **`chore: release x.y.z`** open: it bumps `Cargo.toml` and `Cargo.lock`, and writes `CHANGELOG.md`. Edit its changelog if needed; CI runs on it like on any pull request.
3. Merge it. `.github/workflows/release.yml` then:
   - tags `vx.y.z` and creates the GitHub Release (release-please),
   - builds Windows and macOS with channel `release` (`.github/workflows/build.yml`), uploading symbols and sources to Sentry,
   - attaches `gymdiff-x.y.z-windows-x64.exe`, `gymdiff-recorder-x.y.z-windows-x64.exe`, `gymdiff-x.y.z-macos-arm64.tar.gz` and `SHA256SUMS` to the release, finalizes the Sentry release and records a `production` deploy (`.github/scripts/publish-release.mjs`).

To force a version (e.g. `1.0.0`), add `Release-As: 1.0.0` to the body of a commit on `main`.

## The alpha channel
Every merge to `main` is built with channel `alpha` and published by the `publish-alpha` job of `.github/workflows/ci.yml` (`.github/scripts/publish-alpha.mjs`) on the rolling **`alpha` pre-release**: the `alpha` tag moves to the built commit, and the assets `gymdiff-alpha-windows-x64.exe` and `latest.json` are replaced. Alpha builds read `latest.json` and update themselves ([[Auto-update]], [[ADR-0010 Alpha channel and self-update]]). A build older than the published one (two quick merges) is not published.

To test the updater locally, serve a manifest and point `GYMDIFF_UPDATE_URL` at it (any build then updates itself, so use a copy of the executable):
```sh
python3 -m http.server 8765 --directory <folder with latest.json and the new executable>
GYMDIFF_UPDATE_URL=http://127.0.0.1:8765/latest.json ./copy-of-gymdiff
```

## Symbols in Sentry
Every CI release build (`main`, releases, and pull requests labelled `build`) runs `.github/scripts/sentry-upload.mjs`: it reads the version from `gymdiff --version`, uploads the executable, its debug file (`gymdiff.pdb` / `gymdiff.dSYM`) and a source bundle with `sentry-cli debug-files upload --include-sources --wait`, and on Windows creates the Sentry release and links its commits. Without `SENTRY_AUTH_TOKEN`, `SENTRY_ORG` or `SENTRY_PROJECT` (pull requests from forks, for example) it skips with a warning; a Sentry failure is a warning too, so the build keeps its artifacts.

Local builds are not uploaded: their issues (environment `development`) show addresses only. To symbolicate one anyway:
```sh
cargo build --release -p gymdiff-app
npx @sentry/cli@3.8.2 debug-files upload --include-sources target/release/gymdiff target/release/gymdiff.dSYM
```
(needs `SENTRY_AUTH_TOKEN`, `SENTRY_ORG=florian-girardey-eu`, `SENTRY_PROJECT=gymdiff`).

## Checking that an issue is symbolicated
With a throwaway data folder that has both switches on (`{"crash_reports": true, "logs": true}` in `diagnostics.json`) and `GYMDIFF_SENTRY_DSN` set, start a build with `GYMDIFF_TEST_CRASH`:

| Value | What happens | Expected in Sentry |
|---|---|---|
| `error` | logs a test `ERROR` | issue with a thread stack trace |
| `panic` | panics in a thread | `panic` issue with its stack trace |
| `stack-overflow` | overflows the stack: a native crash | fatal `EXC_BAD_ACCESS` (macOS) or `EXCEPTION_STACK_OVERFLOW` (Windows) issue, mechanism `minidump`, with every thread. The crashed thread may have no frames: its stack pointer is in the guard page. |

The frames must show GymDiff function names, file names and lines, and the source around them for CI builds. Paths are file names only: no user name anywhere.

## One-time setup
| Where | Name | Value |
|---|---|---|
| Secret | `SENTRY_AUTH_TOKEN` | Sentry organization token (*Settings → Developer Settings → Organization Tokens*) |
| Variable | `SENTRY_ORG` | `florian-girardey-eu` |
| Variable | `SENTRY_PROJECT` | `gymdiff` |
| Secret | `RELEASE_PLEASE_TOKEN` | fine-grained personal token on this repository: *Contents*, *Pull requests* and *Issues* (labels) read/write; renew it before it expires. Without it release-please uses `GITHUB_TOKEN`, which needs *Allow GitHub Actions to create and approve pull requests*, and CI does not run on its pull request. |
| Sentry project | *Security & Privacy* | *Store Native Crash Reports* off, *Data Scrubber* on, *Prevent Storing of IP Addresses* on ([[Privacy]]) |
