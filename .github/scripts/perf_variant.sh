#!/usr/bin/env bash
# Temporary: runs the perf-experiment test subset for one variant.
set +e +o pipefail
VARIANT="$1"
JOBS="${2:-1}"
RUNS="${3:-2}"

case "$VARIANT" in
  *novalid*) export WGPU_VALIDATION=0 WGPU_DEBUG=0 ;;
esac
case "$VARIANT" in
  *dx12only*) export WGPU_BACKEND=dx12 ;;
esac
case "$VARIANT" in
  *skiprender*) export PERF_SKIP_TICK_RENDER=1 ;;
esac
case "$VARIANT" in
  *dxc*) export PATH="/c/dxc:$PATH" WGPU_DX12_COMPILER=dxc ;;
esac

NEXTEST=(cargo nextest run --profile ci --cargo-profile ci -p tests --locked --features "${FEATURES}"
  --no-fail-fast --success-output immediate --failure-output immediate)

echo "=== VARIANT $VARIANT probe"
RUST_LOG="error,wgpu_hal=debug,wgpu_core=info" "${NEXTEST[@]}" -j 1 -E 'test(=visual/gradient_issue_9892)' 2>&1 \
  | grep -iE "fxc|dxc|debug layer|debug interface|TIMING-INFO|WARN|ERROR|PASS|FAIL" | head -60

for run in $(seq 1 "$RUNS"); do
  echo "=== VARIANT $VARIANT run $run"
  "${NEXTEST[@]}" -j "$JOBS" -E "$TEST_FILTER" 2>&1 | sed "s/^/[V=$VARIANT R=$run] /"
done
exit 0
