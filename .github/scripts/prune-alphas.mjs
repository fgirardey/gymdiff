// Deletes alpha pre-releases beyond the newest 10, with their tags, from actions/github-script
// (see docs/Decisions/ADR-0011 Windows installer and updates with Velopack.md). Also removes the
// rolling `alpha` release of the very first alpha builds. Stable releases are never touched.
// Usage in a workflow step, after `vpk upload github`:
//   const { default: pruneAlphas } = await import('${{ github.workspace }}/.github/scripts/prune-alphas.mjs')
//   await pruneAlphas({ github, context, core })
// Needs `contents: write`.

/** Alpha releases kept; older ones and their tags are deleted. */
const KEEP = 10;
/** `v0.2.0-alpha.57` → build 57. Stable tags (`v0.2.0`) never match. */
const ALPHA_TAG = /^v\d+\.\d+\.\d+-alpha\.(\d+)$/;

export default async function pruneAlphas({ github, context, core }) {
  const { owner, repo } = context.repo;
  const releases = await github.paginate(github.rest.repos.listReleases, { owner, repo, per_page: 100 });
  const alphas = releases
    .map((release) => ({ release, match: ALPHA_TAG.exec(release.tag_name) }))
    .filter(({ release, match }) => match && !release.draft)
    .sort((a, b) => Number(b.match[1]) - Number(a.match[1]));
  const stale = alphas.slice(KEEP).map(({ release }) => release);
  stale.push(...releases.filter((release) => release.tag_name === 'alpha'));

  for (const release of stale) {
    await github.rest.repos.deleteRelease({ owner, repo, release_id: release.id });
    try {
      await github.rest.git.deleteRef({ owner, repo, ref: `tags/${release.tag_name}` });
    } catch (error) {
      if (error.status !== 422 && error.status !== 404) throw error;
    }
    core.info(`Deleted ${release.tag_name}`);
  }
  core.info(`Kept ${Math.min(alphas.length, KEEP)} alpha releases.`);
}
