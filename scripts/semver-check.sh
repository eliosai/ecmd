#!/usr/bin/env bash
# Compare the public API against a revision through a real checkout, since a bare clone drops the README symlinks
# Usage: semver-check.sh [rev], defaulting to the last v* tag and to the crates.io release when there is none
set -euo pipefail

cd "$(dirname "$0")/.."

rev="${1:-$(git describe --tags --abbrev=0 --match 'v*' 2>/dev/null || true)}"
if [[ -z "$rev" ]]; then
    exec cargo semver-checks --workspace --all-features
fi

baseline=$(mktemp -d)
trap 'git worktree remove --force "$baseline" >/dev/null 2>&1 || rm -rf "$baseline"' EXIT
git worktree add --detach "$baseline" "$rev" >/dev/null
cargo semver-checks --workspace --all-features --baseline-root "$baseline"
