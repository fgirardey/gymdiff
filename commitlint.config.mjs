// Commit messages follow Conventional Commits (https://www.conventionalcommits.org).
// See docs/Workflow/Development.md#Branches and commits.
export default {
  extends: ['@commitlint/config-conventional'],
  rules: {
    // Optional scope; when present, one of the repository areas.
    'scope-enum': [
      2,
      'always',
      ['core', 'lol', 'simulator', 'app', 'ui', 'design', 'docs', 'fixtures', 'deps', 'ci', 'tooling'],
    ],
  },
};
