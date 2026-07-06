#!/bin/bash

# test-full.sh - Run the normal backend sweep plus the canonical macros UI suites.

set +e

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/test-common.sh"

init_test_environment "${BASH_SOURCE[0]}" || exit 1
cd "$ROOT_DIR" || exit 1

TOTAL_RUNS=0
FAILED_RUNS=0
declare -a FAILED_STEPS
# Per-step wall time + status for the end-of-run summary.
declare -a STEP_LABELS
declare -a STEP_SECS
declare -a STEP_OK

FULL_START_TS=$(date +%s)

run_step() {
    local label="$1"
    shift

    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE}$label${NC}"
    echo -e "${BLUE}================================================${NC}"

    local s0
    s0=$(date +%s)
    "$@"
    local status=$?
    local secs=$(( $(date +%s) - s0 ))

    ((TOTAL_RUNS++))
    STEP_LABELS+=("$label")
    STEP_SECS+=("$secs")

    if [ $status -ne 0 ]; then
        ((FAILED_RUNS++))
        FAILED_STEPS+=("$label")
        STEP_OK+=("fail")
    else
        STEP_OK+=("ok")
    fi

    echo ""
    return $status
}

run_step "Backend sweep + one macros pass" "$SCRIPT_DIR/test-all.sh" "$@"
run_step "Macros UI: query capabilities" "$SCRIPT_DIR/test-specific.sh" --crate macros ui_query_capabilities_compile_fail
run_step "Macros UI: migration attribute conflicts" "$SCRIPT_DIR/test-specific.sh" --crate macros ui_migration_attribute_conflicts_compile_fail
run_step "Macros UI: unknown sql attr keys" "$SCRIPT_DIR/test-specific.sh" --crate macros ui_sql_unknown_attr_keys_compile_fail

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}Full Test Summary${NC}"
echo -e "${BLUE}================================================${NC}"

# Per-step wall time, then the whole-run wall clock.
for i in "${!STEP_LABELS[@]}"; do
    if [ "${STEP_OK[$i]}" = "ok" ]; then
        printf "${GREEN}  ✓ %s${NC} (%ss)\n" "${STEP_LABELS[$i]}" "${STEP_SECS[$i]}"
    else
        printf "${RED}  ✗ %s${NC} (%ss)\n" "${STEP_LABELS[$i]}" "${STEP_SECS[$i]}"
    fi
done
echo -e "${BLUE}  ── total wall $(( $(date +%s) - FULL_START_TS ))s ──${NC}"

if [ $FAILED_RUNS -gt 0 ]; then
    echo -e "${RED}✗ Failed runs: $FAILED_RUNS / $TOTAL_RUNS${NC}"
    echo -e "${RED}Failed steps: ${FAILED_STEPS[*]}${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Passed runs: $TOTAL_RUNS / $TOTAL_RUNS${NC}"
exit 0
