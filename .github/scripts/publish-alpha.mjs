// Publishes the Windows build of main on the rolling `alpha` pre-release, from actions/github-script
// (see docs/Decisions/ADR-0010 Alpha channel and self-update.md).
// Usage in a workflow step, after actions/download-artifact of the Windows build:
//   const { default: publishAlpha } = await import('${{ github.workspace }}/.github/scripts/publish-alpha.mjs')
//   await publishAlpha({ github, context, core }, { exe: 'dist/gymdiff.exe', version: '0.2.0', build: 57 })
// Creates the pre-release on first use, moves the `alpha` tag to the built commit and replaces the
// executable and `latest.json`, the manifest alpha builds read. Skips a build older than the one
// already published (two merges in a row). Needs `contents: write`.
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';

const TAG = 'alpha';
const EXE_ASSET = 'gymdiff-alpha-windows-x64.exe';
const MANIFEST_ASSET = 'latest.json';
const TARGET = 'x86_64-pc-windows-msvc';

export default async function publishAlpha({ github, context, core }, { exe, version, build }) {
  if (!version || !Number.isInteger(build) || build <= 0) {
    core.setFailed(`Invalid build: version "${version}", build "${build}"`);
    return;
  }
  const { owner, repo } = context.repo;
  const commit = context.sha;
  const release = await alphaRelease(github, { owner, repo, commit });

  const published = await publishedBuild(github, { owner, repo, release });
  if (published >= build) {
    core.notice(`Alpha already at build ${published}: build ${build} not published.`);
    return;
  }

  await moveTag(github, { owner, repo, commit });

  const data = await readFile(exe);
  const url = `https://github.com/${owner}/${repo}/releases/download/${TAG}/${EXE_ASSET}`;
  const manifest = {
    version,
    build,
    commit,
    target: TARGET,
    url,
    size: data.length,
    sha256: createHash('sha256').update(data).digest('hex'),
    published_at: new Date().toISOString(),
  };
  // Executable first: a client never reads a manifest whose executable is not there yet.
  await replaceAsset(github, { owner, repo, release, name: EXE_ASSET, data, type: 'application/octet-stream' });
  const json = Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`);
  await replaceAsset(github, { owner, repo, release, name: MANIFEST_ASSET, data: json, type: 'application/json' });

  await github.rest.repos.updateRelease({
    owner,
    repo,
    release_id: release.id,
    name: `Alpha ${version}+${build}`,
    body: [
      `Latest build of \`main\`: **${version}+${build}** (${commit.slice(0, 7)}).`,
      '',
      `Download \`${EXE_ASSET}\` once into a folder you own, unblock it (Properties → Unblock), and it updates itself to every new build of \`main\`. See [Auto-update](https://github.com/${owner}/${repo}/blob/main/docs/Product/Features/Auto-update.md).`,
    ].join('\n'),
    prerelease: true,
    make_latest: 'false',
  });
  core.info(`Published alpha ${version}+${build} (${manifest.sha256})`);
}

/** The `alpha` pre-release, created on first use. */
async function alphaRelease(github, { owner, repo, commit }) {
  try {
    const { data } = await github.rest.repos.getReleaseByTag({ owner, repo, tag: TAG });
    return data;
  } catch (error) {
    if (error.status !== 404) throw error;
  }
  const { data } = await github.rest.repos.createRelease({
    owner,
    repo,
    tag_name: TAG,
    target_commitish: commit,
    name: 'Alpha',
    prerelease: true,
    make_latest: 'false',
  });
  return data;
}

/** Build number in the published manifest, or 0 when there is none. */
async function publishedBuild(github, { owner, repo, release }) {
  const asset = release.assets.find((candidate) => candidate.name === MANIFEST_ASSET);
  if (!asset) return 0;
  const { data } = await github.rest.repos.getReleaseAsset({
    owner,
    repo,
    asset_id: asset.id,
    headers: { accept: 'application/octet-stream' },
  });
  try {
    return JSON.parse(Buffer.from(data).toString('utf8')).build ?? 0;
  } catch {
    return 0;
  }
}

/** Points the `alpha` tag at the built commit. */
async function moveTag(github, { owner, repo, commit }) {
  try {
    await github.rest.git.updateRef({ owner, repo, ref: `tags/${TAG}`, sha: commit, force: true });
  } catch (error) {
    if (error.status !== 422 && error.status !== 404) throw error;
    await github.rest.git.createRef({ owner, repo, ref: `refs/tags/${TAG}`, sha: commit });
  }
}

/** Uploads an asset, replacing the one of the same name. */
async function replaceAsset(github, { owner, repo, release, name, data, type }) {
  const existing = release.assets.find((asset) => asset.name === name);
  if (existing) {
    await github.rest.repos.deleteReleaseAsset({ owner, repo, asset_id: existing.id });
  }
  await github.rest.repos.uploadReleaseAsset({
    owner,
    repo,
    release_id: release.id,
    name,
    data,
    headers: { 'content-type': type, 'content-length': data.length },
  });
}
