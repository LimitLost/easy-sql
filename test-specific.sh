#!/bin/bash

# test-specific.sh - Run specific test(s) for sqlite and postgres
# Usage: ./test-specific.sh [--math] [--use-output-columns] [--migrations] [--check-duplicate-table-names] <test_name_pattern>
# Example: ./test-specific.sh test_insert
# Example: ./test-specific.sh --math test_function_sqrt
# Example: ./test-specific.sh --use-output-columns test_custom_select
# Example: ./test-specific.sh --migrations test_insert
# Example: ./test-specific.sh --check-duplicate-table-names test_insert

set +e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
USE_MATH=false
USE_OUTPUT_COLUMNS=false
USE_MIGRATIONS=false
USE_CHECK_DUPLICATE_TABLE_NAMES=false
TEST_PATTERN=""

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
            TEST_PATTERN="$1"
            shift
            ;;
    esac
done

# Check if test pattern is provided
if [ -z "$TEST_PATTERN" ]; then
    echo -e "${RED}Error: No test pattern provided${NC}"
    echo "Usage: $0 [--math] [--use-output-columns] [--migrations] [--check-duplicate-table-names] <test_name_pattern>"
    echo ""
    echo "Examples:"
    echo "  $0 test_insert"
    echo "  $0 --math test_function_sqrt"
    echo "  $0 --use-output-columns test_custom_select"
    echo "  $0 --migrations test_insert"
    echo "  $0 --check-duplicate-table-names test_insert"
    echo "  $0 --math --use-output-columns --migrations test_query"
    exit 1
fi


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
echo -e "${BLUE}║         Testing: ${TEST_PATTERN}${NC}"
echo -e "${BLUE}║         Math: ${USE_MATH} | use_output_columns: ${USE_OUTPUT_COLUMNS}${NC}"
echo -e "${BLUE}║         migrations: ${USE_MIGRATIONS} | check_duplicate_table_names: ${USE_CHECK_DUPLICATE_TABLE_NAMES}${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

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
    
    # Build (silently)
    local build_output=$(cargo build --no-default-features --features "$test_features" 2>&1)
    local build_status=$?
    
    if [ $build_status -ne 0 ]; then
        echo -e "${RED}✗ Build failed${NC}"
        # Show compilation errors with context
        print_error_context "$build_output"
        FAILED_CONFIGS+=("$db_name")
        ((FAILED_TESTS++))
        ((TOTAL_TESTS++))
        return 1
    fi
    
    # Run tests (capture output)
    local test_output=$(cargo test --no-default-features --features "$test_features" "$TEST_PATTERN" 2>&1)
    local test_status=$?
    
    # Parse results
    local passed=$(echo "$test_output" | grep -oP '\d+(?= passed)' | head -1)
    local failed=$(echo "$test_output" | grep -oP '\d+(?= failed)' | head -1)
    
    if [ -z "$passed" ]; then
        passed=0
    fi
    if [ -z "$failed" ]; then
        failed=0
    fi
    
    # Check if any tests ran
    if [ "$passed" -eq 0 ] && [ "$failed" -eq 0 ]; then
        # Check if it's a compilation error or just no matching tests
        if echo "$test_output" | grep -q "error\[E"; then
            echo -e "${RED}✗ Compilation failed with use_output_columns${NC}"
            print_error_context "$test_output"
            FAILED_CONFIGS+=("$db_name (compilation failed)")
            ((FAILED_TESTS++))
            ((TOTAL_TESTS++))
            return 1
        else
            echo -e "${YELLOW}⚠ No tests matched pattern${NC}"
            ((TOTAL_TESTS++))
            return 0
        fi
    fi
    
    # Show results
    if [ $test_status -eq 0 ] && [ "$failed" -eq 0 ]; then
        echo -e "${GREEN}✓ Passed: $passed tests${NC}"
        ((PASSED_TESTS++))
    else
        echo -e "${RED}✗ Failed: $failed tests | Passed: $passed tests${NC}"
        FAILED_CONFIGS+=("$db_name")
        ((FAILED_TESTS++))
        
        # Show failure details
        echo "$test_output" | grep -A 50 "^failures:" | grep -B 50 "^test result:" || echo "$test_output" | tail -20
    fi
    
    ((TOTAL_TESTS++))
    echo ""
}

# Run tests for both databases
run_test "sqlite" "SQLite"
run_test "postgres" "PostgreSQL"

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
