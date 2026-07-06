#!/bin/bash

# test-all.sh - Run ALL tests for sqlite and postgres
# Usage: ./test-all.sh [--math] [--use-output-columns] [--migrations] [--check-duplicate-table-names] [--extra-default-all|--eda] [--extra-default-time|--edt] [--extra-default-chrono|--edc]
# Example: ./test-all.sh
# Example: ./test-all.sh --math
# Example: ./test-all.sh --use-output-columns
# Example: ./test-all.sh --migrations
# Example: ./test-all.sh --check-duplicate-table-names
# Example: ./test-all.sh --extra-default-all
# Example: ./test-all.sh --extra-default-time
# Example: ./test-all.sh --extra-default-chrono
# Example: ./test-all.sh --math --use-output-columns --migrations

set +e

HANG_TIMEOUT_SEC="${HANG_TIMEOUT_SEC:-5}"

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/test-common.sh"

init_test_environment "${BASH_SOURCE[0]}" || exit 1

# Parse arguments
USE_MATH=false
USE_OUTPUT_COLUMNS=false
USE_MIGRATIONS=false
USE_CHECK_DUPLICATE_TABLE_NAMES=false
USE_EXTRA_DEFAULT_ALL=false
USE_EXTRA_DEFAULT_TIME=false
USE_EXTRA_DEFAULT_CHRONO=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --math)
            USE_MATH=true
            shift
            ;;
        --use-output-columns)
            USE_OUTPUT_COLUMNS=true
            shift
            ;;
        --migrations)
            USE_MIGRATIONS=true
            shift
            ;;
        --check-duplicate-table-names)
            USE_CHECK_DUPLICATE_TABLE_NAMES=true
            shift
            ;;
        --extra-default-all|--eda)
            USE_EXTRA_DEFAULT_ALL=true
            shift
            ;;
        --extra-default-time|--edt)
            USE_EXTRA_DEFAULT_TIME=true
            shift
            ;;
        --extra-default-chrono|--edc)
            USE_EXTRA_DEFAULT_CHRONO=true
            shift
            ;;
        --*)
            echo -e "${RED}Error: Unknown option $1${NC}"
            echo "Usage: $0 [--math] [--use-output-columns] [--migrations] [--check-duplicate-table-names] [--extra-default-all|--eda] [--extra-default-time|--edt] [--extra-default-chrono|--edc]"
            echo ""
            echo "Examples:"
            echo "  $0"
            echo "  $0 --math"
            echo "  $0 --use-output-columns"
            echo "  $0 --migrations"
            echo "  $0 --check-duplicate-table-names"
            echo "  $0 --extra-default-all"
            echo "  $0 --extra-default-time"
            echo "  $0 --extra-default-chrono"
            echo "  $0 --math --use-output-columns --migrations"
            exit 1
            ;;
        *)
            echo -e "${RED}Error: This script does not accept positional arguments: '$1'${NC}"
            echo "Usage: $0 [--math] [--use-output-columns] [--migrations] [--check-duplicate-table-names] [--extra-default-all|--eda] [--extra-default-time|--edt] [--extra-default-chrono|--edc]"
            exit 1
            ;;
    esac
done

validate_extra_default_flag_conflicts || exit 1

build_features_string
setup_math_environment

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Arrays to store results
declare -a FAILED_CONFIGS
declare -a SKIPPED_CONFIGS

# Print header
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         Running ALL Tests${NC}"
echo -e "${BLUE}║         Math: ${USE_MATH} | use_output_columns: ${USE_OUTPUT_COLUMNS}${NC}"
echo -e "${BLUE}║         migrations: ${USE_MIGRATIONS} | check_duplicate_table_names: ${USE_CHECK_DUPLICATE_TABLE_NAMES}${NC}"
echo -e "${BLUE}║         extra_default: all=${USE_EXTRA_DEFAULT_ALL}, time=${USE_EXTRA_DEFAULT_TIME}, chrono=${USE_EXTRA_DEFAULT_CHRONO}${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to run all tests for a database
run_all_tests() {
    local db=$1
    local db_name=$2

    echo -e "${YELLOW}━━━ Testing: $db_name ━━━${NC}"

    # Build features string for this database
    local test_features="$db"
    if [ -n "$FEATURES" ]; then
        test_features="$db,$FEATURES"
    fi

    # Build + pre-count + streamed run with live counter and split build/test timers.
    stream_cargo_test "$db_name" "$test_features" "" ""

    # Build failure: surface compiler errors and record a failed leg.
    if [ "$RUN_BUILD_FAILED" = true ]; then
        echo -e "${RED}✗ Build failed${NC}"
        print_error_context "$RUN_OUTPUT"
        echo "$RUN_OUTPUT" | tail -200
        FAILED_CONFIGS+=("$db_name")
        ((FAILED_TESTS++))
        ((TOTAL_TESTS++))
        record_leg "$db_name" fail 0 "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
        return 1
    fi

    local has_compile_error=false
    if echo "$RUN_OUTPUT" | grep -qE "error\[E[0-9]+\]|^error:|could not compile"; then
        has_compile_error=true
    fi

    # Check if tests ran for this backend
    if [ "$RUN_PASSED" -eq 0 ] && [ "$RUN_FAILED" -eq 0 ] && [ "$RUN_SAW_NONZERO" -eq 0 ]; then
        if [ $RUN_STATUS -ne 0 ]; then
            echo -e "${YELLOW}⚠ No tests ran - likely compilation failure${NC}"
            if [ "$has_compile_error" = true ]; then
                echo -e "${RED}Compilation errors detected:${NC}"
                print_error_context "$RUN_OUTPUT"
            else
                echo -e "${RED}Tests failed before execution${NC}"
            fi
            echo "$RUN_OUTPUT" | tail -200
            FAILED_CONFIGS+=("$db_name (no tests ran)")
            ((FAILED_TESTS++))
            ((TOTAL_TESTS++))
            record_leg "$db_name" fail 0 "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
            return 1
        fi

        echo -e "${YELLOW}⚠ Skipped: no tests discovered for backend${NC}"
        SKIPPED_CONFIGS+=("$db_name")
        ((SKIPPED_TESTS++))
        ((TOTAL_TESTS++))
        record_leg "$db_name" skip 0 "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
        return 0
    fi

    # Show results
    if [ $RUN_STATUS -eq 0 ] && [ "$RUN_FAILED" -eq 0 ]; then
        echo -e "${GREEN}✓ All tests passed: $RUN_PASSED tests${NC}"
        ((PASSED_TESTS++))
        record_leg "$db_name" pass "$RUN_PASSED" "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
    else
        echo -e "${RED}✗ Failed: $RUN_FAILED tests | Passed: $RUN_PASSED tests${NC}"
        FAILED_CONFIGS+=("$db_name")
        ((FAILED_TESTS++))

        # Show failure summary (not full details to keep output minimal)
        if [ "$has_compile_error" = true ]; then
            print_error_context "$RUN_OUTPUT"
            echo "$RUN_OUTPUT" | tail -200
        else
            echo "$RUN_OUTPUT" | grep -A 20 "^failures:" | head -25
        fi
        record_leg "$db_name" fail "$RUN_PASSED" "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
    fi

    ((TOTAL_TESTS++))
    echo ""
}

run_all_macros_tests() {
    local manifest_path="$ROOT_DIR/macros/Cargo.toml"

    build_macros_full_suite_features_string
    warn_macros_ignored_flags

    echo -e "${YELLOW}━━━ Testing: Macros crate ━━━${NC}"
    echo -e "${YELLOW}Features: $(macros_feature_display "$MACROS_FEATURES")${NC}"

    # Same streamed runner as the DB legs, so the macros crate now shows a live counter too.
    stream_cargo_test "Macros" "$MACROS_FEATURES" "" "$manifest_path"

    if [ "$RUN_BUILD_FAILED" = true ]; then
        echo -e "${RED}✗ Build failed${NC}"
        print_error_context "$RUN_OUTPUT"
        echo "$RUN_OUTPUT" | tail -200
        FAILED_CONFIGS+=("Macros")
        ((FAILED_TESTS++))
        ((TOTAL_TESTS++))
        record_leg "Macros" fail 0 "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
        return 1
    fi

    local has_compile_error=false
    if echo "$RUN_OUTPUT" | grep -qE "error\[E[0-9]+\]|^error:|could not compile"; then
        has_compile_error=true
    fi

    if [ "$RUN_PASSED" -eq 0 ] && [ "$RUN_FAILED" -eq 0 ] && [ "$RUN_SAW_NONZERO" -eq 0 ]; then
        echo -e "${RED}✗ No macros tests were discovered${NC}"
        echo "$RUN_OUTPUT" | tail -200
        FAILED_CONFIGS+=("Macros (no tests ran)")
        ((FAILED_TESTS++))
        ((TOTAL_TESTS++))
        record_leg "Macros" fail 0 "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
        return 1
    fi

    if [ $RUN_STATUS -eq 0 ] && [ "$RUN_FAILED" -eq 0 ]; then
        echo -e "${GREEN}✓ All tests passed: $RUN_PASSED tests${NC}"
        ((PASSED_TESTS++))
        record_leg "Macros" pass "$RUN_PASSED" "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
    else
        echo -e "${RED}✗ Failed: $RUN_FAILED tests | Passed: $RUN_PASSED tests${NC}"
        FAILED_CONFIGS+=("Macros")
        ((FAILED_TESTS++))

        if [ "$has_compile_error" = true ]; then
            print_error_context "$RUN_OUTPUT"
            echo "$RUN_OUTPUT" | tail -200
        else
            echo "$RUN_OUTPUT" | grep -A 20 "^failures:" | head -25
        fi
        record_leg "Macros" fail "$RUN_PASSED" "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
    fi

    ((TOTAL_TESTS++))
    echo ""
}

# Run tests for both databases (plus the macros crate)
reset_leg_summary
run_all_tests "sqlite" "SQLite"
run_all_tests "postgres" "PostgreSQL"
run_all_macros_tests

# Cleanup environment
cleanup_math_environment

# Print summary
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                         SUMMARY${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"

print_leg_table

if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}✗ Failed: $FAILED_TESTS / $TOTAL_TESTS${NC}"
    echo -e "${RED}Failed configurations: ${FAILED_CONFIGS[*]}${NC}"
    if [ $SKIPPED_TESTS -gt 0 ]; then
        echo -e "${YELLOW}⚠ Skipped: $SKIPPED_TESTS / $TOTAL_TESTS (${SKIPPED_CONFIGS[*]})${NC}"
    fi
    exit 1
else
    echo -e "${GREEN}✓ Passed configurations: $PASSED_TESTS / $TOTAL_TESTS${NC}"
    if [ $SKIPPED_TESTS -gt 0 ]; then
        echo -e "${YELLOW}⚠ Skipped: $SKIPPED_TESTS / $TOTAL_TESTS (${SKIPPED_CONFIGS[*]})${NC}"
    fi
    exit 0
fi
