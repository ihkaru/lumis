#!/usr/bin/env bash
# ==============================================================================
# Deterministic Line Count Verifier (< 350 lines per file)
# Protocol: Old-Coder & Enterprise Clean Architecture
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

MAX_LINES=350
VIOLATIONS=0
CHECKED=0

echo "==> Running Deterministic File Line Check (< $MAX_LINES lines per file)..."

while IFS= read -r file; do
  # Skip git, target, node_modules, dist, vendor, and lockfiles
  if [[ "$file" =~ \.git/ || "$file" =~ target/ || "$file" =~ node_modules/ || "$file" =~ dist/ || "$file" =~ \.lock$ || "$file" =~ \.png$ || "$file" =~ \.svg$ || "$file" =~ \.sqlite || "$file" =~ \.code-graph/ ]]; then
    continue
  fi

  if [[ -f "$file" ]]; then
    LINES=$(wc -l < "$file")
    CHECKED=$((CHECKED + 1))
    if [ "$LINES" -ge "$MAX_LINES" ]; then
      echo "  [✗] VIOLATION: $file has $LINES lines (exceeds $MAX_LINES limit)"
      VIOLATIONS=$((VIOLATIONS + 1))
    fi
  fi
done < <(find "$REPO_ROOT" -type f -not -path '*/.*' -not -path '*/target*' -not -path '*/node_modules*' -not -path '*/dist*' | sort)


if [ "$VIOLATIONS" -gt 0 ]; then
  echo "==> FAILED: Found $VIOLATIONS file(s) exceeding $MAX_LINES lines."
  exit 1
else
  echo "  [✓] SUCCESS: 100% of repository files ($CHECKED checked) are strictly within < $MAX_LINES lines."
  exit 0
fi
