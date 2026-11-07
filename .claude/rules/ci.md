---
paths:
  - ".github/**"
---

# CI (GitHub Actions)

- Logic beyond a one-line command is written in **JavaScript**, not bash: an ES module in `.github/scripts/<name>.mjs` exporting a default `async function ({ github, context, core })`, called from `actions/github-script` (see `lint-commits.mjs`, `pr-builds-comment.mjs`). A few lines can stay inline in the `script:` of the step.
- Use the GitHub API through the `github` client (`github.paginate(...)` for lists), report with `core.info` / `core.warning` / `core.setFailed`; never parse `gh` output in shell.
- Each module starts with a comment saying what it does, how a workflow calls it and the permissions it needs.
- Give each job only the `permissions` it needs, declared on the job.
- A change to CI updates `docs/Workflow/` in the same change (`Development.md`, `Pull requests.md`).
