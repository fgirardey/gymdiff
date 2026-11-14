# Pull requests

Every pull request follows `.github/pull_request_template.md`, which GitHub pre-fills. The goal: a reviewer understands what changes **for the user** without checking out the branch.

## The template
- **What and why**: the user-visible change and its reason, with a link to the feature note (`docs/Product/Features/`).
- **Screenshots**: one image per Pencil frame touched, generated (below). "No UI change." otherwise.
- **How to try it**: the `just` command that shows the change, and what the League client must be doing.
- **Notes for the reviewer**: where to start, trade-offs, follow-ups.
- **Checklist**: `just check`, docs, Pencil match, [[Riot policies]], Windows testing.

## Screenshots
```sh
just pr-screenshots                    # previews changed since main
just pr-screenshots all                # every preview
just pr-screenshots screens/history    # chosen previews (paths under ui/)
just screenshots [paths…]              # only render to target/screenshots, publish nothing
```
`just pr-screenshots`:
1. picks the `*.preview.slint` files whose component or preview changed since `main` (all of them when `theme.slint`, `images/` or `preview-kit/` changed);
2. renders each **Pencil story** of those files (see [[Previews]]) headless, with Slint's software renderer at 2×: `crates/cooldown-app/examples/screenshots.rs`;
3. commits the PNGs to `<branch-name>/` on the orphan **`pr-assets`** branch and pushes it, so images never land in `main`'s history;
4. prints a Markdown table (frame, preview, image) to paste in the **Screenshots** section. Images are linked by commit, so they stay valid after the branch is updated.

Screenshots use the fonts of the machine that renders them (macOS here, Segoe UI on Windows): they show layout and content, not the exact Windows rendering.

A screen without a Pencil story has no screenshot: add the story first ([[Previews]]). For a flow (navigation, live state from the League client), add a recording of `just dev` by hand.

## Builds
A pull request runs only the checks (`commitlint`, `fmt`, clippy and tests on Windows and macOS, `.github/workflows/ci.yml`); a new push cancels the previous run. Release builds take about 15 minutes, so they run on `main` after every merge, and on a pull request **on demand**: add the **`build`** label (for a risky Windows change, or to try a build before merging).

With the label, every push builds the pull request (`.github/workflows/pr-build.yml`) and CI comments the builds on the PR, in one comment updated on every build: a link to `cooldown.exe` (Windows) and `cooldown-macos.tar.gz` (macOS), from the run of the latest commit. GitHub serves them as zip files, kept 14 days, to signed-in users. The builds come from `.github/workflows/build.yml` (channel `preview`, About shows `· preview`), which also uploads their debug symbols and sources to Sentry ([[Releasing]]). The `pr-builds` job runs `.github/scripts/pr-builds-comment.mjs` after the builds, even when one of them failed (the comment then says which). Pull requests from forks get no comment: their token is read-only. Remove the label to stop building.

## Opening the PR
```sh
git push -u origin <branch>
just pr-screenshots > target/screenshots.md
gh pr create --base main --title "<conventional commit header>" --body-file <filled template>
```
The title follows the commit convention ([[Development]]): `type(scope): subject`.
