// Publishes a tagged release from actions/github-script (see docs/Workflow/Releasing.md):
// attaches the builds and their SHA256SUMS to the GitHub Release created by release-please, then
// finalizes the Sentry release and records a production deploy. The Windows installer and update
// packages are added by the package-windows job (vpk, ADR-0011).
// Usage in a workflow step, after actions/download-artifact into `dist/`:
//   const { default: publishRelease } = await import('${{ github.workspace }}/.github/scripts/publish-release.mjs')
//   await publishRelease({ github, context, core, exec }, { tag, version, sentryRelease, dist: 'dist' })
// Needs `contents: write`. Sentry steps need SENTRY_AUTH_TOKEN, SENTRY_ORG and SENTRY_PROJECT.
import { createHash } from 'node:crypto';
import { readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';

/** sentry-cli, pinned like in sentry-upload.mjs. */
const SENTRY_CLI = ['--yes', '@sentry/cli@3.8.2'];

/** Artifact name prefix and file inside → asset name, `{v}` being the version. */
const ASSETS = [
  { artifact: 'gymdiff-windows-', file: 'gymdiff.exe', asset: 'gymdiff-{v}-windows-x64.exe' },
  { artifact: 'gymdiff-recorder-windows-', file: 'gymdiff-recorder.exe', asset: 'gymdiff-recorder-{v}-windows-x64.exe' },
  { artifact: 'gymdiff-macos-', file: 'gymdiff-macos.tar.gz', asset: 'gymdiff-{v}-macos-arm64.tar.gz' },
];

export default async function publishRelease({ github, context, core, exec }, { tag, version, sentryRelease, dist }) {
  const { owner, repo } = context.repo;
  const { data: release } = await github.rest.repos.getReleaseByTag({ owner, repo, tag });

  const folders = await readdir(dist);
  const files = [];
  for (const { artifact, file, asset } of ASSETS) {
    const folder = folders.find((name) => name.startsWith(artifact));
    if (!folder) {
      core.setFailed(`No artifact starting with ${artifact} in ${dist}`);
      return;
    }
    files.push({ source: path.join(dist, folder, file), name: asset.replace('{v}', version) });
  }

  const sums = [];
  for (const { source, name } of files) {
    const data = await readFile(source);
    sums.push(`${createHash('sha256').update(data).digest('hex')}  ${name}`);
    await uploadAsset(github, { owner, repo, release, name, data, type: 'application/octet-stream' });
  }
  const sumsFile = path.join(dist, 'SHA256SUMS');
  await writeFile(sumsFile, `${sums.join('\n')}\n`);
  await uploadAsset(github, { owner, repo, release, name: 'SHA256SUMS', data: await readFile(sumsFile), type: 'text/plain' });

  core.info(`Attached ${files.length} builds to ${release.html_url}`);

  const missing = ['SENTRY_AUTH_TOKEN', 'SENTRY_ORG', 'SENTRY_PROJECT'].filter((name) => !process.env[name]);
  if (missing.length > 0) {
    core.warning(`Sentry release not finalized: ${missing.join(', ')} not set.`);
    return;
  }
  const sentry = (...args) => exec.exec('npx', [...SENTRY_CLI, ...args]);
  await sentry('releases', 'finalize', sentryRelease);
  await sentry('deploys', 'new', '--release', sentryRelease, '--env', 'production', '--url', release.html_url);
}

/** Uploads one asset, replacing an asset of the same name (a re-run of the workflow). */
async function uploadAsset(github, { owner, repo, release, name, data, type }) {
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
