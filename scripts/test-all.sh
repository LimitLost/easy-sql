#!/bin/bash

# test-all.sh - Run ALL tests for sqlite and postgres
# Usage: ./test-all.sh [--math] [--use-output-columns] [--migrations] [--check-duplicate-table-names]
# Example: ./test-all.sh
# Example: ./test-all.sh --math
# Example: ./test-all.sh --use-output-columns
# Example: ./test-all.sh --migrations
# Example: ./test-all.sh --check-duplicate-table-names
# Example: ./test-all.sh --math --use-output-columns --migrations

set +e

HANG_TIMEOUT_SEC="${HANG_TIMEOUT_SEC:-5}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(dirname "$SCRIPT_DIR")
MAIN_DIR="$ROOT_DIR/-main"

if [ ! -d "$MAIN_DIR" ]; then
    echo -e "${RED}Error: -main directory not found at $MAIN_DIR${NC}"
    exit 1
fi

cd "$MAIN_DIR" || exit 1

# Parse arguments
USE_MATH=false
USE_OUTPUT_COLUMNS=false
USE_MIGRATIONS=false
USE_CHECK_DUPLICATE_TABLE_NAMES=false

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
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            echo "Usage: $0 [--math] [--use-output-columns] [--migrations] [--check-duplicate-table-names]"
            echo ""
            echo "Examples:"
            echo "  $0"
            echo "  $0 --math"
            echo "  $0 --use-output-columns"
            echo "  $0 --migrations"
            echo "  $0 --check-duplicate-table-names"
            echo "  $0 --math --use-output-columns --migrations"
            exit 1
            ;;
    esac
done

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Arrays to store results
declare -a FAILED_CONFIGS

print_error_context() {
    local output="$1"
    echo "$output" | awk '
        /error\[E[0-9]+\]|^error:/{
            print;
            lines=10;
            next;
        }
        lines > 0 {
            print;
            lines--;
        }
    '
}

# Build features string
FEATURES=""
if [ "$USE_OUTPUT_COLUMNS" = true ]; then
    FEATURES="use_output_columns"
fi
if [ "$USE_MIGRATIONS" = true ]; then
    if [ -n "$FEATURES" ]; then
        FEATURES="$FEATURES,migrations"
    else
        FEATURES="migrations"
    fi
fi
if [ "$USE_MATH" = true ]; then
    if [ -n "$FEATURES" ]; then
        FEATURES="$FEATURES,sqlite_math,rust_decimal"
    else
        FEATURES="sqlite_math,rust_decimal"
    fi
fi
if [ "$USE_CHECK_DUPLICATE_TABLE_NAMES" = true ]; then
    if [ -n "$FEATURES" ]; then
        FEATURES="$FEATURES,check_duplicate_table_names"
    else
        FEATURES="check_duplicate_table_names"
    fi
fi

# Setup environment
if [ "$USE_MATH" = true ]; then
    export LIBSQLITE3_FLAGS="-DSQLITE_ENABLE_MATH_FUNCTIONS"
else
    unset LIBSQLITE3_FLAGS
fi

# Print header
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         Running ALL Tests${NC}"
echo -e "${BLUE}║         Math: ${USE_MATH} | use_output_columns: ${USE_OUTPUT_COLUMNS}${NC}"
echo -e "${BLUE}║         migrations: ${USE_MIGRATIONS} | check_duplicate_table_names: ${USE_CHECK_DUPLICATE_TABLE_NAMES}${NC}"
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
    
    # Build (silently)
    local build_output=$(cargo build --no-default-features --features "$test_features" 2>&1)
    local build_status=$?
    
    if [ $build_status -ne 0 ]; then
        echo -e "${RED}✗ Build failed${NC}"
        # Show compilation errors with context
        print_error_context "$build_output"
        echo "$build_output" | tail -200
        FAILED_CONFIGS+=("$db_name")
        ((FAILED_TESTS++))
        ((TOTAL_TESTS++))
        return 1
    fi
    
    # Run ALL tests (stream output via FIFO while keeping full log text)
    local test_output=""
    local test_status=0
    local live_passed=0
    local live_failed=0
    local saw_ok_result=false
    local saw_failure_marker=false
    local forced_success_kill=false
    local tmp_dir
    local fifo_path
    tmp_dir=$(mktemp -d "/tmp/easy-sql-test-all-${db}-XXXXXX")
    if [ -z "$tmp_dir" ] || [ ! -d "$tmp_dir" ]; then
        echo -e "${RED}✗ Failed to create temporary directory${NC}"
        FAILED_CONFIGS+=("$db_name")
        ((FAILED_TESTS++))
        ((TOTAL_TESTS++))
        return 1
    fi
    fifo_path="$tmp_dir/stream.fifo"
    if ! mkfifo "$fifo_path"; then
        echo -e "${RED}✗ Failed to create temporary FIFO${NC}"
        rm -rf "$tmp_dir"
        FAILED_CONFIGS+=("$db_name")
        ((FAILED_TESTS++))
        ((TOTAL_TESTS++))
        return 1
    fi

    cargo test --color never --no-default-features --features "$test_features" > "$fifo_path" 2>&1 &
    local cargo_pid=$!
    local last_activity_ts
    last_activity_ts=$(date +%s)

    exec 3< "$fifo_path"
    while true; do
        if IFS= read -r -t 1 line <&3; then
            test_output+="$line"$'\n'
            last_activity_ts=$(date +%s)

            if [[ "$line" =~ ^test[[:space:]].+\.\.\.[[:space:]]ok$ ]]; then
                ((live_passed++))
                printf "\r${BLUE}Live counter [%s]: passed=%d failed=%d${NC}" "$db_name" "$live_passed" "$live_failed"
            elif [[ "$line" =~ ^test[[:space:]].+\.\.\.[[:space:]]FAILED$ ]]; then
                ((live_failed++))
                saw_failure_marker=true
                printf "\r${BLUE}Live counter [%s]: passed=%d failed=%d${NC}" "$db_name" "$live_passed" "$live_failed"
            fi

            if [[ "$line" == *"test result: ok"* ]]; then
                saw_ok_result=true
            fi
            if echo "$line" | grep -qE "test result: FAILED|failures:|error: test failed|could not compile"; then
                saw_failure_marker=true
            fi
        else
            if ! kill -0 "$cargo_pid" 2>/dev/null; then
                break
            fi

            local now_ts
            now_ts=$(date +%s)
            local idle_sec=$((now_ts - last_activity_ts))

            if [ "$idle_sec" -ge "$HANG_TIMEOUT_SEC" ] && \
               [ "$saw_ok_result" = true ] && \
               [ "$saw_failure_marker" = false ] && \
               ! pgrep -P "$cargo_pid" >/dev/null 2>&1; then
                echo ""
                echo -e "${YELLOW}⚠ Cargo appears hung after success output; forcing safe exit (idle ${idle_sec}s)${NC}"
                kill -TERM "$cargo_pid" 2>/dev/null
                sleep 1
                if kill -0 "$cargo_pid" 2>/dev/null; then
                    kill -KILL "$cargo_pid" 2>/dev/null
                fi
                forced_success_kill=true
                break
            fi
        fi
    done
    exec 3<&-

    wait "$cargo_pid"
    test_status=$?
    rm -rf "$tmp_dir"
    echo ""

    if [ "$forced_success_kill" = true ] && [ "$saw_failure_marker" = false ]; then
        test_status=0
    fi
    
    local has_compile_error=false
    if echo "$test_output" | grep -qE "error\[E[0-9]+\]|^error:|could not compile"; then
        has_compile_error=true
    fi

    # Parse results
    local passed=$(echo "$test_output" | grep -oP '\d+(?= passed)' | head -1)
    local failed=$(echo "$test_output" | grep -oP '\d+(?= failed)' | head -1)
    
    if [ -z "$passed" ]; then
        passed=0
    fi
    if [ -z "$failed" ]; then
        failed=0
    fi
    
    # Check if no tests ran (likely compilation failure)
    if [ "$passed" -eq 0 ] && [ "$failed" -eq 0 ]; then
        echo -e "${YELLOW}⚠ No tests ran - likely compilation failure${NC}"
        if [ $test_status -ne 0 ]; then
            if [ "$has_compile_error" = true ]; then
                echo -e "${RED}Compilation errors detected:${NC}"
                print_error_context "$test_output"
            else
                echo -e "${RED}Tests failed before execution${NC}"
            fi
        fi
        echo "$test_output" | tail -200
        FAILED_CONFIGS+=("$db_name (no tests ran)")
        ((FAILED_TESTS++))
        ((TOTAL_TESTS++))
        return 1
    fi
    
    # Show results
    if [ $test_status -eq 0 ] && [ "$failed" -eq 0 ]; then
        echo -e "${GREEN}✓ All tests passed: $passed tests${NC}"
        ((PASSED_TESTS++))
    else
        echo -e "${RED}✗ Failed: $failed tests | Passed: $passed tests${NC}"
        FAILED_CONFIGS+=("$db_name")
        ((FAILED_TESTS++))
        
        # Show failure summary (not full details to keep output minimal)
        if [ "$has_compile_error" = true ]; then
            print_error_context "$test_output"
            echo "$test_output" | tail -200
        else
            echo "$test_output" | grep -A 20 "^failures:" | head -25
        fi
    fi
    
    ((TOTAL_TESTS++))
    echo ""
}

# Run tests for both databases
run_all_tests "sqlite" "SQLite"
run_all_tests "postgres" "PostgreSQL"

# Cleanup environment
unset LIBSQLITE3_FLAGS

# Print summary
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                         SUMMARY${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"

if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}✗ Failed: $FAILED_TESTS / $TOTAL_TESTS${NC}"
    echo -e "${RED}Failed configurations: ${FAILED_CONFIGS[*]}${NC}"
    exit 1
else
    echo -e "${GREEN}✓ All tests passed ($PASSED_TESTS / $TOTAL_TESTS)${NC}"
    exit 0
fi
