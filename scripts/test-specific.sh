#!/bin/bash

# test-specific.sh - Run a specific main or macros test target/pattern.
# Usage: ./test-specific.sh [--crate main|macros] [--math] [--use-output-columns] [--migrations] [--check-duplicate-table-names] [--watcher] [--extra-default-all|--eda] [--extra-default-time|--edt] [--extra-default-chrono|--edc] <test_name_pattern>
# Example: ./test-specific.sh test_insert
# Example: ./test-specific.sh --math test_function_sqrt
# Example: ./test-specific.sh --use-output-columns test_custom_select
# Example: ./test-specific.sh --migrations test_insert
# Example: ./test-specific.sh --crate macros ui_query_capabilities_compile_fail
# Example: ./test-specific.sh --crate macros ui_migration_attribute_conflicts_compile_fail
# Example: ./test-specific.sh --crate macros ui_sql_unknown_attr_keys_compile_fail
# Example: ./test-specific.sh --check-duplicate-table-names test_insert
# Example: ./test-specific.sh --extra-default-all test_insert
# Example: ./test-specific.sh --extra-default-time test_insert
# Example: ./test-specific.sh --extra-default-chrono test_insert
# Example: TRYBUILD=overwrite ./test-specific.sh --crate macros ui_query_capabilities_compile_fail
# Example: TRYBUILD=overwrite ./test-specific.sh --crate macros ui_migration_attribute_conflicts_compile_fail
# Example: TRYBUILD=overwrite ./test-specific.sh --crate macros ui_sql_unknown_attr_keys_compile_fail

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
USE_WATCHER=false
TEST_TARGET_CRATE="main"
TEST_PATTERN=""

print_usage() {
    echo "Usage: $0 [--crate main|macros] [--math] [--use-output-columns] [--migrations] [--check-duplicate-table-names] [--watcher] [--extra-default-all|--eda] [--extra-default-time|--edt] [--extra-default-chrono|--edc] <test_name_pattern>"
    echo ""
    echo "Examples:"
    echo "  $0 test_insert"
    echo "  $0 --crate macros ui_query_capabilities_compile_fail"
    echo "  $0 --crate macros ui_migration_attribute_conflicts_compile_fail"
    echo "  $0 --crate macros ui_sql_unknown_attr_keys_compile_fail"
    echo "  $0 --math test_function_sqrt"
    echo "  $0 --use-output-columns test_custom_select"
    echo "  $0 --migrations test_insert"
    echo "  $0 --check-duplicate-table-names test_insert"
    echo "  $0 --extra-default-all test_insert"
    echo "  $0 --extra-default-time test_insert"
    echo "  $0 --extra-default-chrono test_insert"
    echo "  TRYBUILD=overwrite $0 --crate macros ui_query_capabilities_compile_fail"
    echo "  TRYBUILD=overwrite $0 --crate macros ui_migration_attribute_conflicts_compile_fail"
    echo "  TRYBUILD=overwrite $0 --crate macros ui_sql_unknown_attr_keys_compile_fail"
    echo "  $0 --math --use-output-columns --migrations test_query"
}

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
        --watcher)
            USE_WATCHER=true
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
        --crate)
            shift
            if [ -z "$1" ]; then
                echo -e "${RED}Error: --crate requires a value (main|macros)${NC}"
                exit 1
            fi
            case "$1" in
                main|macros)
                    TEST_TARGET_CRATE="$1"
                    ;;
                *)
                    echo -e "${RED}Error: Unsupported crate '$1'. Use main or macros${NC}"
                    exit 1
                    ;;
            esac
            shift
            ;;
        --*)
            echo -e "${RED}Error: Unknown option $1${NC}"
            print_usage
            exit 1
            ;;
        *)
            if [[ "$1" == --* ]]; then
                echo -e "${RED}Error: Unknown option $1${NC}"
                exit 1
            fi
            if [ -n "$TEST_PATTERN" ]; then
                echo -e "${RED}Error: Multiple test patterns provided: '$TEST_PATTERN' and '$1'${NC}"
                exit 1
            fi
            TEST_PATTERN="$1"
            shift
            ;;
    esac
done

# Check if test pattern is provided
if [ -z "$TEST_PATTERN" ]; then
    echo -e "${RED}Error: No test pattern provided${NC}"
    print_usage
    exit 1
fi

validate_extra_default_flag_conflicts || exit 1

build_features_string
setup_math_environment

run_macros_test() {
    local manifest_path="$ROOT_DIR/macros/Cargo.toml"

    resolve_macros_test_features "$TEST_PATTERN" || return 1

    if ! is_macros_ui_exact_target "$TEST_PATTERN"; then
        warn_macros_ignored_flags
    fi

    echo -e "${YELLOW}━━━ Testing: macros crate ━━━${NC}"
    echo -e "${YELLOW}Target: $TEST_PATTERN | Features: $(macros_feature_display "$MACROS_TARGET_FEATURES")${NC}"

    # UI exact targets are trybuild integration binaries (run via --test); any other pattern is a
    # plain test-name filter.
    local mode="filter"
    [ "$MACROS_TARGET_KIND" = "integration-test" ] && mode="test-target"

    stream_cargo_test "macros" "$MACROS_TARGET_FEATURES" "$TEST_PATTERN" "$manifest_path" "$mode"

    # A filter that matched nothing may actually name an integration-test target; retry once.
    if [ "$mode" = "filter" ] && [ "$RUN_BUILD_FAILED" = false ] && \
       [ "$RUN_PASSED" -eq 0 ] && [ "$RUN_FAILED" -eq 0 ] && [ "$RUN_SAW_NONZERO" -eq 0 ] && [ $RUN_STATUS -eq 0 ]; then
        stream_cargo_test "macros" "$MACROS_TARGET_FEATURES" "$TEST_PATTERN" "$manifest_path" "test-target"
    fi

    echo -e "${BLUE}  Timing: build ${RUN_BUILD_SECS}s | test ${RUN_TEST_SECS}s${NC}"

    if [ "$RUN_BUILD_FAILED" = true ]; then
        echo -e "${RED}✗ Build failed${NC}"
        print_error_context "$RUN_OUTPUT"
        echo "$RUN_OUTPUT" | tail -200
        return 1
    fi

    if [ "$RUN_PASSED" -eq 0 ] && [ "$RUN_FAILED" -eq 0 ] && [ "$RUN_SAW_NONZERO" -eq 0 ]; then
        if [ $RUN_STATUS -ne 0 ]; then
            echo -e "${RED}✗ Tests failed before execution${NC}"
            print_error_context "$RUN_OUTPUT"
            echo "$RUN_OUTPUT" | tail -200
            return 1
        fi

        echo -e "${RED}✗ No tests matched pattern '$TEST_PATTERN' in macros crate${NC}"
        echo "$RUN_OUTPUT" | tail -120
        return 1
    fi

    if [ $RUN_STATUS -eq 0 ] && [ "$RUN_FAILED" -eq 0 ]; then
        echo -e "${GREEN}✓ Passed: $RUN_PASSED tests${NC}"
        return 0
    fi

    echo -e "${RED}✗ Failed: $RUN_FAILED tests | Passed: $RUN_PASSED tests${NC}"
    print_error_context "$RUN_OUTPUT"
    echo "$RUN_OUTPUT" | tail -200
    return 1
}

if [ "$TEST_TARGET_CRATE" = "macros" ]; then
    run_macros_test
    status=$?
    cleanup_math_environment
    exit $status
fi

# Remember whether the requested pattern is present in the checked-in main test
# sources before running the driver matrix.
# Reason: an all-skipped feature-gated run should exit successfully, while a
# genuinely unknown pattern must still fail fast in the final summary.
MAIN_PATTERN_EXISTS_IN_SOURCE=false
if main_test_pattern_exists_in_sources "$TEST_PATTERN"; then
    MAIN_PATTERN_EXISTS_IN_SOURCE=true
fi


# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0
# Count configurations that produced at least one concrete test result.
# Reason: driver-specific cfg gates can legitimately hide a test on the
# non-target driver, but the script must still fail when nothing matched
# anywhere.
MATCHED_CONFIGS=0

# Arrays to store results
declare -a FAILED_CONFIGS
declare -a SKIPPED_CONFIGS

# Print header
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         Testing: ${TEST_PATTERN}${NC}"
echo -e "${BLUE}║         Math: ${USE_MATH} | use_output_columns: ${USE_OUTPUT_COLUMNS}${NC}"
echo -e "${BLUE}║         migrations: ${USE_MIGRATIONS} | check_duplicate_table_names: ${USE_CHECK_DUPLICATE_TABLE_NAMES}${NC}"
echo -e "${BLUE}║         extra_default: all=${USE_EXTRA_DEFAULT_ALL}, time=${USE_EXTRA_DEFAULT_TIME}, chrono=${USE_EXTRA_DEFAULT_CHRONO}${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Record a per-driver no-match as a skip so cfg-gated tests do not fail on the
# non-target driver.
# Reason: the final summary still turns an all-skipped run into a hard failure,
# which keeps genuine typo/no-match requests visible.
record_no_match_skip() {
    local db_name="$1"
    local no_match_output="$2"

    echo -e "${YELLOW}⚠ Skipped: No tests matched pattern '$TEST_PATTERN' for $db_name${NC}"
    if [ -n "$no_match_output" ]; then
        echo "$no_match_output" | tail -200
    fi

    SKIPPED_CONFIGS+=("$db_name (no matches)")
    ((SKIPPED_TESTS++))
    ((TOTAL_TESTS++))
    return 0
}

# Function to run test for a database
run_test() {
    local db=$1
    local db_name=$2

    echo -e "${YELLOW}━━━ Testing: $db_name ━━━${NC}"

    # Build features string for this database
    local test_features="$db"
    if [ -n "$FEATURES" ]; then
        test_features="$db,$FEATURES"
    fi

    # Build + pre-count + streamed run with live counter and split build/test timers.
    stream_cargo_test "$db_name" "$test_features" "$TEST_PATTERN" ""

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

    # Fallback: treat the pattern as an integration-test target name (e.g.
    # ui_query_capabilities_compile_fail) when the plain filter matched nothing.
    if [ "$RUN_PASSED" -eq 0 ] && [ "$RUN_FAILED" -eq 0 ] && [ "$RUN_SAW_NONZERO" -eq 0 ] && [ $RUN_STATUS -eq 0 ]; then
        stream_cargo_test "$db_name" "$test_features" "$TEST_PATTERN" "" "test-target"
        if [ "$RUN_BUILD_FAILED" = false ]; then
            has_compile_error=false
            if echo "$RUN_OUTPUT" | grep -qE "error\[E[0-9]+\]|^error:|could not compile"; then
                has_compile_error=true
            fi
        fi
    fi

    # Check if any tests ran
    if [ "$RUN_PASSED" -eq 0 ] && [ "$RUN_FAILED" -eq 0 ] && [ "$RUN_SAW_NONZERO" -eq 0 ]; then
        if [ $RUN_STATUS -ne 0 ]; then
            if echo "$RUN_OUTPUT" | grep -qE "no test target named"; then
                record_no_match_skip "$db_name" "$RUN_OUTPUT"
                record_leg "$db_name" skip 0 "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
                return 0
            fi

            if [ "$has_compile_error" = true ]; then
                echo -e "${RED}✗ Compilation failed${NC}"
                print_error_context "$RUN_OUTPUT"
                echo "$RUN_OUTPUT" | tail -200
            else
                echo -e "${RED}✗ Tests failed before execution${NC}"
                echo "$RUN_OUTPUT" | tail -200
            fi
            FAILED_CONFIGS+=("$db_name (test failure)")
            ((FAILED_TESTS++))
            ((TOTAL_TESTS++))
            record_leg "$db_name" fail 0 "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
            return 1
        fi

        record_no_match_skip "$db_name" "$RUN_OUTPUT"
        record_leg "$db_name" skip 0 "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
        return 0
    fi

    ((MATCHED_CONFIGS++))

    # Show results
    if [ $RUN_STATUS -eq 0 ] && [ "$RUN_FAILED" -eq 0 ]; then
        echo -e "${GREEN}✓ Passed: $RUN_PASSED tests${NC}"
        ((PASSED_TESTS++))
        record_leg "$db_name" pass "$RUN_PASSED" "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
    else
        echo -e "${RED}✗ Failed: $RUN_FAILED tests | Passed: $RUN_PASSED tests${NC}"
        FAILED_CONFIGS+=("$db_name")
        ((FAILED_TESTS++))

        # Show failure details
        if [ "$has_compile_error" = true ]; then
            print_error_context "$RUN_OUTPUT"
            echo "$RUN_OUTPUT" | tail -200
        else
            echo "$RUN_OUTPUT" | grep -A 50 "^failures:" | grep -B 50 "^test result:" || echo "$RUN_OUTPUT" | tail -200
        fi
        record_leg "$db_name" fail "$RUN_PASSED" "$RUN_TOTAL" "$RUN_BUILD_SECS" "$RUN_TEST_SECS"
    fi

    ((TOTAL_TESTS++))
    echo ""
}

# Run tests for both databases
reset_leg_summary
run_test "sqlite" "SQLite"
run_test "postgres" "PostgreSQL"

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
elif [ $MATCHED_CONFIGS -eq 0 ]; then
    if [ "$MAIN_PATTERN_EXISTS_IN_SOURCE" = true ] && [ $SKIPPED_TESTS -gt 0 ]; then
        echo -e "${YELLOW}⚠ Known test pattern '$TEST_PATTERN' was skipped by the current feature/driver selection${NC}"
        echo -e "${YELLOW}⚠ Skipped: $SKIPPED_TESTS / $TOTAL_TESTS (${SKIPPED_CONFIGS[*]})${NC}"
        exit 0
    fi

    echo -e "${RED}✗ No tests matched pattern '$TEST_PATTERN' in any enabled configuration${NC}"
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
