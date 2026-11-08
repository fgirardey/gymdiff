// Shared by publish-alpha.mjs and publish-release.mjs: the manifest.json that GymDiff's updater
// reads on each release (see docs/Decisions/ADR-0010 Alpha channel and self-update.md), and the
// upload of a release asset. Not called from a workflow directly. The callers need `contents: write`.
import { createHash } from 'node:crypto';

/** Target triple of the published Windows executable. */
const TARGET = 'x86_64-pc-windows-msvc';

/** The manifest of the Windows executable `data`, attached to the release `tag` as `exeName`. */
export function manifestFor({ owner, repo, tag, exeName, data, version, build, commit }) {
  return {
    version,
    build,
    commit,
    target: TARGET,
    url: `https://github.com/${owner}/${repo}/releases/download/${tag}/${exeName}`,
    size: data.length,
    sha256: createHash('sha256').update(data).digest('hex'),
    published_at: new Date().toISOString(),
  };
}

/** Uploads one asset, replacing an asset of the same name (a re-run of the workflow). */
export async function uploadAsset(github, { owner, repo, release, name, data, type }) {
  const existing = (release.assets ?? []).find((asset) => asset.name === name);
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
