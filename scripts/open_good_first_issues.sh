#!/usr/bin/env bash
# Opens the staged good-first-issues as real GitHub issues.
#
# Part D keeps ten curated, genuinely-good first issues in
# docs/backlog/good-first-issues/ as files: the repo is the source of
# truth, and the issues are opened from it. Requires the gh CLI, authed,
# with repo access. Idempotent-ish: titles include the number prefix, so
# re-running creates duplicates — check first.
set -euo pipefail

cd "$(dirname "$0")/.."
DIR="docs/backlog/good-first-issues"

for f in "$DIR"/*.md; do
  base="$(basename "$f" .md)"
  num="${base%%-*}"
  slug="${base#*-}"
  title="$(awk -F'- ' '/^- Title:/{print $2}' "$f" 2>/dev/null || true)"
  # title line format used in the files: "# NN — Title"
  if [ -z "$title" ]; then
    title="$(head -1 "$f" | sed 's/^# [0-9]* — //')"
  fi
  labels="$(awk -F'`: ' '/^- Labels:/{print $2}' "$f" | tr -d '`')"
  body="$(awk 'found || /^## Why this matters/{found=1; print}' "$f")"

  echo "== opening: $title (labels: $labels)"
  gh issue create \
    --title "good-first-issue: $title" \
    --body-file <(printf '%s\n\n---\n*Staged from `%s`; open that file for the canonical text.*\n' "$body" "$f") \
    --label "good-first-issue"
done
echo "done. Now tick each file's 'opened as issue #' below the title."
