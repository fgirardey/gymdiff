#!/usr/bin/env bash
# Renders the Pencil stories touched by the current branch, publishes the PNGs
# on the `pr-assets` branch and prints the Markdown table for the PR body.
#
#   just pr-screenshots              # previews changed since main
#   just pr-screenshots all          # every preview
#   just pr-screenshots screens/history shell/app-shell
#
# See docs/Workflow/Pull requests.md.
set -euo pipefail

base="${PR_BASE:-main}"
assets_branch="pr-assets"
ui="crates/gymdiff-app/ui"
root="$(git rev-parse --show-toplevel)"
cd "$root"

branch="$(git branch --show-current)"
slug="${branch//\//-}"
out="target/pr-screenshots/$slug"

# Which previews to render: the arguments, or the ones the branch changed.
previews=()
if [[ $# -gt 0 && "$1" != "all" ]]; then
    previews=("$@")
elif [[ $# -eq 0 ]]; then
    changed="$(git diff --name-only "$base"...HEAD -- "$ui")"
    if [[ -z "$changed" ]]; then
        echo "No UI change since $base: nothing to screenshot." >&2
        exit 0
    fi
    # Tokens, images or preview helpers change every screen: render everything.
    if ! grep -qE "^$ui/(theme\.slint|preview-kit/|images/)" <<<"$changed"; then
        while IFS= read -r file; do
            [[ "$file" == *.slint ]] || continue
            rel="${file#"$ui"/}"
            rel="${rel%.slint}"
            rel="${rel%.preview}"
            [[ -f "$ui/$rel.preview.slint" ]] && previews+=("$rel")
        done <<<"$changed"
        if [[ ${#previews[@]} -eq 0 ]]; then
            echo "No previewed UI file changed since $base: nothing to screenshot." >&2
            exit 0
        fi
        # Deduplicate (a component and its preview may both have changed).
        unique=()
        while IFS= read -r rel; do unique+=("$rel"); done < <(printf '%s\n' "${previews[@]}" | sort -u)
        previews=("${unique[@]}")
    fi
fi

rm -rf "$out"
cargo run --quiet -p gymdiff-app --example screenshots -- --out "$out" ${previews[@]+"${previews[@]}"} >/dev/null
if [[ ! -s "$out/index.tsv" ]]; then
    echo "The changed previews have no Pencil story: nothing to screenshot." >&2
    exit 0
fi

# Publish on the orphan `pr-assets` branch, in a throwaway worktree.
worktree="target/pr-assets-worktree"
git worktree remove --force "$worktree" 2>/dev/null || true
if git fetch --quiet origin "$assets_branch" 2>/dev/null; then
    git worktree add --quiet --force -B "$assets_branch" "$worktree" "origin/$assets_branch"
else
    git worktree add --quiet --force --detach "$worktree"
    git -C "$worktree" checkout --quiet --orphan "$assets_branch"
    git -C "$worktree" rm -rf --quiet .
fi
rm -rf "${worktree:?}/$slug"
mkdir -p "$worktree/$slug"
cp "$out"/*.png "$worktree/$slug/"
git -C "$worktree" add --all "$slug"
if ! git -C "$worktree" diff --cached --quiet; then
    git -C "$worktree" commit --quiet -m "chore: add screenshots for $branch"
fi
git -C "$worktree" push --quiet origin "$assets_branch"
sha="$(git -C "$worktree" rev-parse HEAD)"
git worktree remove --force "$worktree"

# Markdown for the "Screenshots" section of the PR template.
remote="$(git remote get-url origin)"
repo="$(sed -E 's#(git@github\.com:|https://github\.com/)##; s#\.git$##' <<<"$remote")"
echo "| Pencil frame | Preview | Screenshot |"
echo "|---|---|---|"
while IFS=$'\t' read -r file frame preview; do
    echo "| $frame | \`$preview\` | <img src=\"https://github.com/$repo/raw/$sha/$slug/$file\" width=\"300\"> |"
done <"$out/index.tsv"
