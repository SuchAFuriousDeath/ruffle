#!/usr/bin/env bash
# Temporary: runs the SWF test binary in one configuration and records timings.
# Usage: supervisor_variant.sh <nextest|supervisor> <jobs> <label>
set +e +o pipefail
VARIANT="$1"
JOBS="$2"
NAME="$3"
LOGS="$PWD/supervisor-logs"
mkdir -p "$LOGS"

exe=$(cargo nextest list --profile ci --cargo-profile ci -p tests --test tests --locked --features "${FEATURES}" \
  --list-type binaries-only --message-format json | jq -r '."rust-binaries"["tests::tests"]."binary-path"')

start=$(date +%s)
case "$VARIANT" in
  nextest)
    cargo nextest run --profile ci --cargo-profile ci -p tests --test tests --locked --features "${FEATURES}" \
      --no-fail-fast -j "$JOBS" > "$LOGS/$NAME.log" 2>&1
    ;;
  supervisor)
    (cd tests && RUFFLE_TEST_WORKERS="$JOBS" RUFFLE_TEST_NEXTEST_PROFILE=ci \
      RUFFLE_TEST_TIMINGS="$LOGS/$NAME.tsv" "$exe") > "$LOGS/$NAME.log" 2>&1
    ;;
esac
status=$?
end=$(date +%s)

echo "[RESULT] os=$RUNNER_OS/$RUNNER_ARCH variant=$NAME exit=$status wall=$((end - start))s" | tee -a "$LOGS/summary.txt"
grep -E "^test result|Summary \[|FAILED|CRASHED|TIMED OUT|FLAKY|flaky|SIGSEGV|SIGABRT" "$LOGS/$NAME.log" | tail -40
exit 0
