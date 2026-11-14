---
name: opening-a-pull-request
description: Use when opening or updating a cooldown.lol pull request — fills .github/pull_request_template.md, generates and publishes UI screenshots with just pr-screenshots, and creates the PR with gh
---

# Opening a pull request

Full workflow: `docs/Workflow/Pull requests.md`.

1. `just check` passes and the branch is pushed (`git push -u origin <branch>`).
2. Read `git log main..HEAD` and `git diff main...HEAD --stat`; read the feature notes the branch implements.
3. UI change? Run `just pr-screenshots` (it pushes the PNGs to the `pr-assets` branch and prints a Markdown table). Look at the PNGs in `target/pr-screenshots/<branch>/` before using them: a broken render is a bug to report, not to paste.
4. Fill every section of `.github/pull_request_template.md` in a file in the scratchpad, written **for the reviewer**: user-visible change first, implementation second. Paste the table under **Screenshots**; write "No UI change." when there is none. Tick only the checklist items you verified.
5. `gh pr create --base main --title "<type(scope): subject>" --body-file <file>`. To update an existing PR: `gh pr edit --body-file <file>`.
