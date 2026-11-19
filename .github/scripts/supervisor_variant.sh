#!/usr/bin/env bash
# Temporary: runs the SWF test binary in one configuration and records timings.
set +e +o pipefail
VARIANT="$1"
JOBS="${2:-4}"
NAME="${VARIANT}_${JOBS}"
LOGS="$PWD/supervisor-logs"
mkdir -p "$LOGS"

case "$VARIANT" in
  *dxc*) export PATH="/c/dxc:$PATH" WGPU_DX12_COMPILER=dxc ;;
esac

exe=$(cargo nextest list --cargo-profile ci -p tests --test tests --locked --features "${FEATURES}" \
  --list-type binaries-only --message-format json | jq -r '."rust-binaries"["tests::tests"]."binary-path"')

start=$(date +%s)
case "$VARIANT" in
  selftest)
    mkdir -p "$LOGS/selftest-scratch"
    (cd tests && RUFFLE_TEST_SUPERVISOR_SELF_TEST="$LOGS/selftest-scratch" RUFFLE_TEST_WORKERS=2 \
      RUFFLE_TEST_TIMEOUT=10 RUFFLE_TEST_RETRIES=1 "$exe" supervisor_self_test) > "$LOGS/$NAME.log" 2>&1
    ;;
  nextest)
    cargo nextest run --profile ci --cargo-profile ci -p tests --test tests --locked --features "${FEATURES}" \
      --no-fail-fast -j "$JOBS" --status-level all --final-status-level none > "$LOGS/$NAME.log" 2>&1
    ;;
  supervisor*)
    (cd tests && RUFFLE_TEST_WORKERS="$JOBS" RUFFLE_TEST_NEXTEST_PROFILE=ci \
      RUFFLE_TEST_TIMINGS="$LOGS/$NAME.tsv" "$exe") > "$LOGS/$NAME.log" 2>&1
    ;;
esac
status=$?
end=$(date +%s)

echo "[RESULT] variant=$NAME exit=$status wall=$((end - start))s" | tee -a "$LOGS/summary.txt"
grep -E "^test result|Summary|FAILED|CRASHED|TIMED OUT|FLAKY|flaky|TRY . FAIL|SIGSEGV|SIGABRT" "$LOGS/$NAME.log" | head -80
exit 0
