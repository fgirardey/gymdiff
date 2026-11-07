// Posts the release builds of the current CI run on its pull request, as one
// comment updated on every push, from actions/github-script.
// Usage in a workflow step:
//   const { default: commentBuilds } = await import('${{ github.workspace }}/.github/scripts/pr-builds-comment.mjs')
//   await commentBuilds({ github, context, core })
// Needs `actions: read` and `pull-requests: write`. See docs/Workflow/Pull requests.md.

/** Hidden marker that identifies the comment to update. */
const MARKER = '<!-- gymdiff:pr-builds -->';

/** Artifact name prefix → platform shown in the comment, in display order. */
const PLATFORMS = [
  ['gymdiff-windows-', 'Windows', '`gymdiff.exe`'],
  ['gymdiff-macos-', 'macOS', '`gymdiff-macos.tar.gz` (`tar -xzf` keeps the executable bit)'],
];

export default async function commentBuilds({ github, context, core }) {
  const pullRequest = context.payload.pull_request;
  if (!pullRequest) {
    core.info('Not a pull request: no comment.');
    return;
  }
  const { owner, repo } = context.repo;
  const runId = context.runId;
  const runUrl = `https://github.com/${owner}/${repo}/actions/runs/${runId}`;

  const artifacts = await github.paginate(github.rest.actions.listWorkflowRunArtifacts, {
    owner,
    repo,
    run_id: runId,
    per_page: 100,
  });

  const rows = PLATFORMS.map(([prefix, platform, contents]) => {
    const artifact = artifacts.find((candidate) => candidate.name.startsWith(prefix) && !candidate.expired);
    if (!artifact) {
      return `| ${platform} | build failed or missing, see the [run](${runUrl}) | |`;
    }
    const url = `${runUrl}/artifacts/${artifact.id}`;
    return `| ${platform} | [${artifact.name}.zip](${url}) (${megabytes(artifact.size_in_bytes)}) | ${contents} |`;
  });

  const headSha = pullRequest.head.sha.slice(0, 7);
  const body = [
    MARKER,
    `### Builds for ${headSha}`,
    '',
    '| Platform | Download | Inside the zip |',
    '|---|---|---|',
    ...rows,
    '',
    `Release builds from [this CI run](${runUrl}), kept 14 days. Downloading needs a GitHub login with access to the repository.`,
  ].join('\n');

  const comments = await github.paginate(github.rest.issues.listComments, {
    owner,
    repo,
    issue_number: pullRequest.number,
    per_page: 100,
  });
  const existing = comments.find((comment) => comment.user?.type === 'Bot' && comment.body?.includes(MARKER));

  if (existing) {
    await github.rest.issues.updateComment({ owner, repo, comment_id: existing.id, body });
    core.info(`Updated ${existing.html_url}`);
  } else {
    const { data } = await github.rest.issues.createComment({ owner, repo, issue_number: pullRequest.number, body });
    core.info(`Created ${data.html_url}`);
  }
}

function megabytes(bytes) {
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}
