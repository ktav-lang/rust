#!/usr/bin/env bash
# Convenience wrapper for `cargo bench` with sensible defaults and
# criterion-compatible filtering.
#
# Usage:
#   ./bench.sh                  # quick run (warmup 1s, measurement 2s)
#   ./bench.sh full             # criterion defaults — slower, more accurate
#   ./bench.sh parse            # run only parse-related benches
#   ./bench.sh render           # run only render benches
#   ./bench.sh "parse|render"   # any criterion regex
#
# Criterion stores the last baseline in target/criterion/ and
# automatically diffs against it on the next run.

set -euo pipefail

cd "$(dirname "$0")/.."

MODE="${1:-quick}"
shift || true

# "full" → criterion defaults (warmup 3s, measurement 5s, 100 samples).
# "quick" → short warmup for fast iteration.
if [[ "$MODE" == "full" ]]; then
    FILTER="${1:-}"
    exec cargo bench -p ktav --bench parse -- "$FILTER"
fi

# Anything else is treated as a filter.
FILTER="$MODE"
exec cargo bench -p ktav --bench parse -- \
    --warm-up-time 1 \
    --measurement-time 2 \
    --sample-size 20 \
    "$FILTER"
