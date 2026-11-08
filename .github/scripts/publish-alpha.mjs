// Publishes the Windows build of main as its own GitHub pre-release, `v<version>-alpha.<build>`,
// from actions/github-script (see docs/Decisions/ADR-0010 Alpha channel and self-update.md).
// Usage in a workflow step, after actions/download-artifact of the Windows build:
//   const { default: publishAlpha } = await import('${{ github.workspace }}/.github/scripts/publish-alpha.mjs')
//   await publishAlpha({ github, context, core }, { exe: 'dist/gymdiff.exe', version: '0.2.0', build: 57 })
// The release is created as a draft, gets the executable and its manifest.json, then is published,
// so clients never see it half uploaded. Only the newest alpha releases are kept. Needs
// `contents: write`.
import { readFile } from 'node:fs/promises';

import { manifestFor, uploadAsset } from './release-manifest.mjs';

/** Alpha releases kept after publishing; older ones and their tags are deleted. */
const KEEP = 10;
/** `v0.2.0-alpha.57` → build 57. Stable tags (`v0.2.0`) never match. */
const ALPHA_TAG = /^v\d+\.\d+\.\d+-alpha\.(\d+)$/;

export default async function publishAlpha({ github, context, core }, { exe, version, build }) {
  if (!version || !Number.isInteger(build) || build <= 0) {
    core.setFailed(`Invalid build: version "${version}", build "${build}"`);
    return;
  }
  const { owner, repo } = context.repo;
  const commit = context.sha;
  const tag = `v${version}-alpha.${build}`;
  const exeName = `gymdiff-${version}-alpha.${build}-windows-x64.exe`;

  const existing = await releaseByTag(github, { owner, repo, tag });
  if (existing && !existing.draft) {
    core.notice(`${tag} is already published.`);
    return;
  }
  const { data: release } = existing
    ? { data: existing }
    : await github.rest.repos.createRelease({
        owner,
        repo,
        tag_name: tag,
        target_commitish: commit,
        name: tag,
        body: [
          `Build ${build} of \`main\` (${commit.slice(0, 7)}), for the alpha channel.`,
          '',
          `Alpha builds update themselves to the newest alpha release. To install one, download \`${exeName}\` into a folder you own and unblock it (Properties → Unblock). See [Auto-update](https://github.com/${owner}/${repo}/blob/main/docs/Product/Features/Auto-update.md).`,
        ].join('\n'),
        draft: true,
        prerelease: true,
        make_latest: 'false',
      });

  const data = await readFile(exe);
  await uploadAsset(github, { owner, repo, release, name: exeName, data, type: 'application/octet-stream' });
  const manifest = manifestFor({ owner, repo, tag, exeName, data, version, build, commit });
  await uploadAsset(github, {
    owner,
    repo,
    release,
    name: 'manifest.json',
    data: Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`),
    type: 'application/json',
  });
  await github.rest.repos.updateRelease({ owner, repo, release_id: release.id, draft: false, make_latest: 'false' });
  core.info(`Published ${tag} (${manifest.sha256})`);

  await prune(github, { owner, repo, core });
}

/** The release with `tag`, drafts included, or `undefined`. */
async function releaseByTag(github, { owner, repo, tag }) {
  const releases = await github.paginate(github.rest.repos.listReleases, { owner, repo, per_page: 100 });
  return releases.find((release) => release.tag_name === tag);
}

/** Deletes alpha releases and tags beyond the newest KEEP, and the old rolling `alpha` release. */
async function prune(github, { owner, repo, core }) {
  const releases = await github.paginate(github.rest.repos.listReleases, { owner, repo, per_page: 100 });
  const alphas = releases
    .map((release) => ({ release, match: ALPHA_TAG.exec(release.tag_name) }))
    .filter(({ release, match }) => match && !release.draft)
    .sort((a, b) => Number(b.match[1]) - Number(a.match[1]));
  const stale = alphas.slice(KEEP).map(({ release }) => release);
  // Rolling pre-release of the first alpha builds, replaced by one release per build.
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
}
