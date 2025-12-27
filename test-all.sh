#!/bin/bash

# test-all.sh - Run ALL tests for sqlite and postgres
# Usage: ./test-all.sh [--math] [--use-output-columns]
# Example: ./test-all.sh
# Example: ./test-all.sh --math
# Example: ./test-all.sh --use-output-columns
# Example: ./test-all.sh --math --use-output-columns

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
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            echo "Usage: $0 [--math] [--use-output-columns]"
            echo ""
            echo "Examples:"
            echo "  $0"
            echo "  $0 --math"
            echo "  $0 --use-output-columns"
            echo "  $0 --math --use-output-columns"
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

# Build features string
FEATURES=""
if [ "$USE_OUTPUT_COLUMNS" = true ]; then
    FEATURES="use_output_columns"
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
        # Show compilation errors
        echo "$build_output" | grep -E "error\[E[0-9]+\]" | head -10
        FAILED_CONFIGS+=("$db_name")
        ((FAILED_TESTS++))
        ((TOTAL_TESTS++))
        return 1
    fi
    
    # Run ALL tests (capture output)
    local test_output=$(cargo test --no-default-features --features "$test_features" 2>&1)
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
    
    # Check if no tests ran (likely compilation failure)
    if [ "$passed" -eq 0 ] && [ "$failed" -eq 0 ]; then
        echo -e "${YELLOW}⚠ No tests ran - likely compilation failure with use_output_columns${NC}"
        # Try to detect compilation errors
        if echo "$test_output" | grep -q "error\[E"; then
            echo -e "${RED}Compilation errors detected:${NC}"
            echo "$test_output" | grep -E "error\[E[0-9]+\]" | head -5
        fi
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
        echo "$test_output" | grep -A 20 "^failures:" | head -25
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
