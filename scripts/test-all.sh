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
    
    # Run ALL tests (capture output)
    local test_output=$(cargo test --no-default-features --features "$test_features" 2>&1)
    local test_status=$?
    
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
