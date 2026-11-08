// Uploads the debug files and sources of a release build to Sentry, and creates its Sentry
// release, from actions/github-script (see docs/Workflow/Releasing.md).
// Usage in a workflow step, after `cargo build --release`:
//   const { default: sentryUpload } = await import('${{ github.workspace }}/.github/scripts/sentry-upload.mjs')
//   await sentryUpload({ core, exec }, { binary: 'target/release/gymdiff', debugFiles: [...], createRelease: true })
// Sets the step outputs `version` (SemVer) and `sentry-release` (e.g. gymdiff@0.2.0+57), read from
// `<binary> --version`. Needs SENTRY_AUTH_TOKEN, SENTRY_ORG and SENTRY_PROJECT in the environment;
// without them (pull requests from forks) it only sets the outputs. A Sentry failure is a warning:
// it never costs the build its artifacts. No GitHub permissions.

/** sentry-cli, pinned: CI must not change behaviour on its own. */
const SENTRY_CLI = ['--yes', '@sentry/cli@3.8.2'];

export default async function sentryUpload({ core, exec }, { binary, debugFiles, createRelease }) {
  const { stdout } = await exec.getExecOutput(binary, ['--version'], { silent: true });
  // `gymdiff 0.2.0+57.a1b2c3d`, from BuildInfo::full_version.
  const match = /^gymdiff (\S+?)\+(\d+)\./.exec(stdout.trim());
  if (!match) {
    core.setFailed(`Unexpected version output: ${stdout.trim()}`);
    return;
  }
  const [, version, build] = match;
  const release = `gymdiff@${version}+${build}`;
  core.setOutput('version', version);
  core.setOutput('sentry-release', release);
  core.info(`Built ${release}`);

  const missing = ['SENTRY_AUTH_TOKEN', 'SENTRY_ORG', 'SENTRY_PROJECT'].filter((name) => !process.env[name]);
  if (missing.length > 0) {
    core.warning(`Debug files not uploaded to Sentry: ${missing.join(', ')} not set.`);
    return;
  }
  const sentry = (...args) => exec.exec('npx', [...SENTRY_CLI, ...args]);

  // Symbols match events by debug id, whatever the release: every build that may run
  // somewhere (pull request artifacts included) gets its symbols uploaded.
  try {
    await sentry('debug-files', 'upload', '--include-sources', '--wait', ...debugFiles);
  } catch (error) {
    core.warning(`Debug files not uploaded to Sentry: ${error.message}`);
    return;
  }

  if (createRelease) {
    try {
      await sentry('releases', 'new', release);
      // Commits since the previous release, from the checkout (fetch-depth: 0).
      await sentry('releases', 'set-commits', release, '--local', '--ignore-missing');
    } catch (error) {
      core.warning(`Sentry release ${release} not created or not linked to its commits: ${error.message}`);
    }
  }
}
