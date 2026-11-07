// Lints commit messages with commitlint (Conventional Commits), from actions/github-script.
// Usage in a workflow step:
//   const { default: lintCommits } = await import('${{ github.workspace }}/.github/scripts/lint-commits.mjs')
//   await lintCommits({ github, context, core })
// Needs `npm install` first (commitlint and the config come from node_modules).
import load from '@commitlint/load';
import lint from '@commitlint/lint';

const ZERO_SHA = /^0+$/;

/** The commits to lint for the event that triggered the workflow. */
async function commitsToLint({ github, context, core }) {
  const { owner, repo } = context.repo;

  if (context.eventName === 'pull_request') {
    return github.paginate(github.rest.pulls.listCommits, {
      owner,
      repo,
      pull_number: context.payload.pull_request.number,
      per_page: 100,
    });
  }

  if (context.eventName === 'push') {
    const { before, after, forced } = context.payload;
    if (!forced && !ZERO_SHA.test(before)) {
      const { data } = await github.rest.repos.compareCommitsWithBasehead({
        owner,
        repo,
        basehead: `${before}...${after}`,
        per_page: 100,
      });
      if (data.total_commits > data.commits.length) {
        core.warning(`Only the first ${data.commits.length} of ${data.total_commits} pushed commits are linted.`);
      }
      return data.commits;
    }
    core.info(`Force push or new branch (previous head ${before}): linting every commit.`);
  }

  // Force push, new branch, manual run: the whole history of the commit.
  return github.paginate(github.rest.repos.listCommits, {
    owner,
    repo,
    sha: context.sha,
    per_page: 100,
  });
}

export default async function lintCommits({ github, context, core }) {
  const config = await load();
  const options = {
    parserOpts: config.parserPreset?.parserOpts ?? {},
    plugins: config.plugins ?? {},
    ignores: config.ignores ?? [],
    defaultIgnores: config.defaultIgnores ?? true,
    helpUrl: config.helpUrl,
  };

  const commits = await commitsToLint({ github, context, core });
  let invalid = 0;

  for (const { sha, commit } of commits) {
    const header = commit.message.split('\n', 1)[0];
    const result = await lint(commit.message, config.rules, options);
    const short = sha.slice(0, 7);
    for (const warning of result.warnings) {
      core.warning(`${short} ${header}: ${warning.message} [${warning.name}]`);
    }
    if (result.valid) {
      core.info(`✔ ${short} ${header}`);
      continue;
    }
    invalid += 1;
    for (const error of result.errors) {
      core.error(`${short} ${header}: ${error.message} [${error.name}]`);
    }
  }

  if (invalid > 0) {
    core.setFailed(
      `${invalid} of ${commits.length} commit(s) do not follow Conventional Commits. See docs/Workflow/Development.md.`,
    );
  } else {
    core.info(`${commits.length} commit(s) linted, all valid.`);
  }
}
