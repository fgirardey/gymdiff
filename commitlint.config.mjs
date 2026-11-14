// Commit messages follow Conventional Commits (https://www.conventionalcommits.org).
// See docs/Workflow/Development.md#Branches and commits.
export default {
  extends: ['@commitlint/config-conventional'],
  // Release commits are written by release-please (docs/Workflow/Releasing.md):
  // their body is the changelog, whose lines can exceed the length limits.
  ignores: [(message) => /^chore: release \d+\.\d+\.\d+/.test(message)],
  rules: {
    // Optional scope; when present, one of the repository areas.
    'scope-enum': [
      2,
      'always',
      ['core', 'lol', 'app', 'ui', 'design', 'docs', 'deps', 'ci', 'tooling'],
    ],
  },
};
