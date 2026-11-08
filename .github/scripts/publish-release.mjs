// Publishes a tagged release from actions/github-script (see docs/Workflow/Releasing.md):
// attaches the builds, their SHA256SUMS and the manifest.json of the Windows build (read by the
// updater's stable channel, ADR-0010) to the GitHub Release created by release-please, then
// finalizes the Sentry release and records a production deploy.
// Usage in a workflow step, after actions/download-artifact into `dist/`:
//   const { default: publishRelease } = await import('${{ github.workspace }}/.github/scripts/publish-release.mjs')
//   await publishRelease({ github, context, core, exec }, { tag, version, build, sentryRelease, dist: 'dist' })
// Needs `contents: write`. Sentry steps need SENTRY_AUTH_TOKEN, SENTRY_ORG and SENTRY_PROJECT.
import { createHash } from 'node:crypto';
import { readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';

import { manifestFor, uploadAsset } from './release-manifest.mjs';

/** sentry-cli, pinned like in sentry-upload.mjs. */
const SENTRY_CLI = ['--yes', '@sentry/cli@3.8.2'];

/** Artifact name prefix and file inside → asset name, `{v}` being the version. */
const ASSETS = [
  { artifact: 'gymdiff-windows-', file: 'gymdiff.exe', asset: 'gymdiff-{v}-windows-x64.exe' },
  { artifact: 'gymdiff-recorder-windows-', file: 'gymdiff-recorder.exe', asset: 'gymdiff-recorder-{v}-windows-x64.exe' },
  { artifact: 'gymdiff-macos-', file: 'gymdiff-macos.tar.gz', asset: 'gymdiff-{v}-macos-arm64.tar.gz' },
];

export default async function publishRelease({ github, context, core, exec }, { tag, version, build, sentryRelease, dist }) {
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

  const windows = files[0];
  const manifest = manifestFor({
    owner,
    repo,
    tag,
    exeName: windows.name,
    data: await readFile(windows.source),
    version,
    build,
    commit: context.sha,
  });
  const json = Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`);
  await uploadAsset(github, { owner, repo, release, name: 'manifest.json', data: json, type: 'application/json' });
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
