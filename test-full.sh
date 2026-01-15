#!/bin/bash

# test.sh - Comprehensive test script for easy-sql
# Tests all combinations of features: postgres, sqlite, use_output_columns
# Also tests with and without LIBSQLITE3_FLAGS environment variable

# Don't exit on error - we want to test all combinations
set +e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Arrays to store configurations
declare -a FAILED_CONFIGS
declare -a PASSED_CONFIGS
declare -a ALL_CONFIGS

# Function to print section header
print_header() {
    echo -e "\n${BLUE}================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================================${NC}\n"
}

# Function to print test result
print_result() {
    local status=$1
    local config=$2
    local env_info=$3
    local full_config="$config $env_info"
    
    ALL_CONFIGS+=("$full_config|$status")
    
    if [ $status -eq 0 ]; then
        echo -e "${GREEN}✓ PASSED${NC}: $config $env_info"
        ((PASSED_TESTS++))
        PASSED_CONFIGS+=("$full_config")
    else
        echo -e "${RED}✗ FAILED${NC}: $config $env_info"
        ((FAILED_TESTS++))
        FAILED_CONFIGS+=("$full_config")
    fi
    ((TOTAL_TESTS++))
}

# Function to print summary table
print_summary_table() {
    echo -e "\n${CYAN}╔════════════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                         DETAILED TEST RESULTS                                  ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════════════════════╝${NC}\n"
    
    # Print passed configurations
    if [ ${#PASSED_CONFIGS[@]} -gt 0 ]; then
        echo -e "${GREEN}✓ PASSED CONFIGURATIONS (${#PASSED_CONFIGS[@]}):${NC}"
        for config in "${PASSED_CONFIGS[@]}"; do
            echo -e "  ${GREEN}✓${NC} $config"
        done
        echo ""
    fi
    
    # Print failed configurations
    if [ ${#FAILED_CONFIGS[@]} -gt 0 ]; then
        echo -e "${RED}✗ FAILED CONFIGURATIONS (${#FAILED_CONFIGS[@]}):${NC}"
        for config in "${FAILED_CONFIGS[@]}"; do
            echo -e "  ${RED}✗${NC} $config"
        done
        echo ""
    fi
}

# Function to run a single test configuration
run_test() {
    local features=$1
    local use_env=$2
    local config_name=$3
    
    local env_info=""
    if [ "$use_env" = "true" ]; then
        env_info="[with LIBSQLITE3_FLAGS]"
        export LIBSQLITE3_FLAGS="-DSQLITE_ENABLE_MATH_FUNCTIONS"
    else
        env_info="[without LIBSQLITE3_FLAGS]"
        unset LIBSQLITE3_FLAGS
    fi
    
    echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Testing:${NC} $config_name $env_info"
    echo -e "${YELLOW}Features:${NC} $features"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # Build first
    echo -e "${BLUE}[BUILD]${NC} cargo build --no-default-features --features \"$features\""
    local build_output=$(cargo build --no-default-features --features "$features" 2>&1)
    local build_status=$?
    echo "$build_output" | grep -E "(Compiling|Finished|error)" | tail -10
    
    # Also check if build output contains "error" to catch compile errors
    if echo "$build_output" | grep -q "^error"; then
        build_status=1
    fi
    
    if [ $build_status -eq 0 ]; then
        # Run tests
        echo -e "${BLUE}[TEST]${NC} cargo test --no-default-features --features \"$features\""
        local test_output=$(cargo test --no-default-features --features "$features" 2>&1)
        local test_status=$?
        echo "$test_output" | tail -20
        
        # Also check if test output contains "error" to catch test errors
        if echo "$test_output" | grep -q "^error"; then
            test_status=1
        fi
        
        if [ $test_status -eq 0 ]; then
            print_result 0 "$config_name" "$env_info"
            return 0
        else
            print_result 1 "$config_name" "$env_info"
            return 1
        fi
    else
        echo -e "${RED}Build failed${NC}"
        echo "$build_output" | tail -20
        print_result 1 "$config_name" "$env_info"
        return 1
    fi
    
    if [ "$use_env" = "true" ]; then
        unset LIBSQLITE3_FLAGS
    fi
}

# Main test execution
print_header "Starting Comprehensive Feature Tests"

echo "This script will test all combinations of:"
echo "  - postgres (feature)"
echo "  - sqlite (feature)"
echo "  - use_output_columns (feature)"
echo "  - LIBSQLITE3_FLAGS environment variable"
echo ""
echo "Each combination will be built and tested with --no-default-features"
echo ""

# Generate all feature combinations
# Features: postgres, sqlite, use_output_columns
# We'll test each combination with and without LIBSQLITE3_FLAGS

declare -a FEATURE_COMBINATIONS=(
    # Single features
    "postgres|Postgres only"
    "sqlite|SQLite only"
    "use_output_columns|use_output_columns only"
    
    # Two features
    "postgres,use_output_columns|Postgres + use_output_columns"
    "sqlite,use_output_columns|SQLite + use_output_columns"
    
    # Note: postgres + sqlite combination doesn't make sense as they're mutually exclusive database backends
    # But we can test them together to verify the build handles it
    "postgres,sqlite|Postgres + SQLite (both backends)"
    "postgres,sqlite,use_output_columns|All features"
)

# Test each feature combination with and without LIBSQLITE3_FLAGS
for combo in "${FEATURE_COMBINATIONS[@]}"; do
    IFS='|' read -r features name <<< "$combo"
    
    # Test without LIBSQLITE3_FLAGS
    run_test "$features" "false" "$name"
    
    # Test with LIBSQLITE3_FLAGS
    run_test "$features" "true" "$name"
done

# Also test with no features at all
print_header "Testing with no features"
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Testing:${NC} No features [without LIBSQLITE3_FLAGS]"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}[BUILD]${NC} cargo build --no-default-features"
build_output=$(cargo build --no-default-features 2>&1)
build_status=$?
echo "$build_output" | grep -E "(Compiling|Finished|error)" | tail -10

# Also check if build output contains "error" to catch compile errors
if echo "$build_output" | grep -q "^error"; then
    build_status=1
fi

if [ $build_status -eq 0 ]; then
    echo -e "${BLUE}[TEST]${NC} cargo test --no-default-features"
    test_output=$(cargo test --no-default-features 2>&1)
    test_status=$?
    echo "$test_output" | tail -20
    
    # Also check if test output contains "error" to catch test errors
    if echo "$test_output" | grep -q "^error"; then
        test_status=1
    fi
    
    if [ $test_status -eq 0 ]; then
        print_result 0 "No features" "[without LIBSQLITE3_FLAGS]"
    else
        print_result 1 "No features" "[without LIBSQLITE3_FLAGS]"
    fi
else
    echo -e "${RED}Build failed${NC}"
    echo "$build_output" | tail -20
    print_result 1 "No features" "[without LIBSQLITE3_FLAGS]"
fi

echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Testing:${NC} No features [with LIBSQLITE3_FLAGS]"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
export LIBSQLITE3_FLAGS="-DSQLITE_ENABLE_MATH_FUNCTIONS"
echo -e "${BLUE}[BUILD]${NC} cargo build --no-default-features"
build_output=$(cargo build --no-default-features 2>&1)
build_status=$?
echo "$build_output" | grep -E "(Compiling|Finished|error)" | tail -10

# Also check if build output contains "error" to catch compile errors
if echo "$build_output" | grep -q "^error"; then
    build_status=1
fi

if [ $build_status -eq 0 ]; then
    echo -e "${BLUE}[TEST]${NC} cargo test --no-default-features"
    test_output=$(cargo test --no-default-features 2>&1)
    test_status=$?
    echo "$test_output" | tail -20
    
    # Also check if test output contains "error" to catch test errors
    if echo "$test_output" | grep -q "^error"; then
        test_status=1
    fi
    
    if [ $test_status -eq 0 ]; then
        print_result 0 "No features" "[with LIBSQLITE3_FLAGS]"
    else
        print_result 1 "No features" "[with LIBSQLITE3_FLAGS]"
    fi
else
    echo -e "${RED}Build failed${NC}"
    echo "$build_output" | tail -20
    print_result 1 "No features" "[with LIBSQLITE3_FLAGS]"
fi
unset LIBSQLITE3_FLAGS

# Print detailed summary table
print_summary_table

# Print summary
print_header "Test Summary"
echo -e "Total configurations tested: ${BLUE}$TOTAL_TESTS${NC}"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"

if [ $FAILED_TESTS -gt 0 ]; then
    SUCCESS_RATE=$(awk "BEGIN {printf \"%.1f\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")
    echo -e "Success rate: ${YELLOW}${SUCCESS_RATE}%${NC}"
    echo -e "\n${RED}⚠ Some tests failed!${NC}"
    exit 1
else
    echo -e "\n${GREEN}✓ All tests passed! 🎉${NC}"
    exit 0
fi
