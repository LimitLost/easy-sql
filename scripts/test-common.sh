#!/bin/bash

# Shared helpers for test scripts.

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

init_test_environment() {
    local caller_path="${1:-$0}"

    SCRIPT_DIR=$(cd "$(dirname "$caller_path")" && pwd)
    ROOT_DIR=$(dirname "$SCRIPT_DIR")
    MAIN_DIR="$ROOT_DIR/-main"

    if [ ! -d "$MAIN_DIR" ]; then
        echo -e "${RED}Error: -main directory not found at $MAIN_DIR${NC}"
        return 1
    fi

    cd "$MAIN_DIR" || return 1
}

append_feature() {
    local feature="$1"

    if [ -n "$FEATURES" ]; then
        FEATURES="$FEATURES,$feature"
    else
        FEATURES="$feature"
    fi
}

append_feature_dedup() {
    local feature="$1"

    if [ -z "$feature" ]; then
        return
    fi

    if [ -z "$FEATURES" ]; then
        FEATURES="$feature"
        return
    fi

    case ",$FEATURES," in
        *",$feature,"*) ;;
        *) FEATURES="$FEATURES,$feature" ;;
    esac
}

append_features_bundle() {
    local bundle="$1"
    local feature

    IFS=',' read -r -a features_array <<< "$bundle"
    for feature in "${features_array[@]}"; do
        append_feature_dedup "$feature"
    done
}

validate_extra_default_flag_conflicts() {
    if [ "$USE_EXTRA_DEFAULT_CHRONO" = true ] && \
       { [ "$USE_EXTRA_DEFAULT_ALL" = true ] || [ "$USE_EXTRA_DEFAULT_TIME" = true ]; }; then
        echo -e "${RED}Error: --extra-default-chrono cannot be combined with --extra-default-all or --extra-default-time${NC}"
        return 1
    fi
    if [ "$USE_EXTRA_DEFAULT_ALL" = true ] && \
       { [ "$USE_EXTRA_DEFAULT_CHRONO" = true ] || [ "$USE_EXTRA_DEFAULT_TIME" = true ]; }; then
        echo -e "${RED}Error: --extra-default-all cannot be combined with --extra-default-chrono or --extra-default-time${NC}"
        return 1
    fi

    return 0
}

build_features_string() {
    FEATURES=""

    if [ "$USE_OUTPUT_COLUMNS" = true ]; then
        append_feature_dedup "use_output_columns"
    fi

    if [ "$USE_MIGRATIONS" = true ]; then
        append_feature_dedup "migrations"
    fi

    if [ "$USE_MATH" = true ]; then
        append_feature_dedup "sqlite_math"
        append_feature_dedup "rust_decimal"
    fi

    if [ "$USE_CHECK_DUPLICATE_TABLE_NAMES" = true ]; then
        append_feature_dedup "check_duplicate_table_names"
    fi

    if [ "$USE_EXTRA_DEFAULT_ALL" = true ]; then
        append_features_bundle "check_duplicate_table_names,ipnet,ipnetwork,mac_address,bit_vec,uuid,json,chrono,time,bstr"
    fi

    if [ "$USE_EXTRA_DEFAULT_TIME" = true ]; then
        append_features_bundle "check_duplicate_table_names,ipnet,ipnetwork,mac_address,bit_vec,uuid,json,time,bstr"
    fi

    if [ "$USE_EXTRA_DEFAULT_CHRONO" = true ]; then
        append_features_bundle "check_duplicate_table_names,ipnet,ipnetwork,mac_address,bit_vec,uuid,json,chrono,bstr"
    fi
}

setup_math_environment() {
    if [ "$USE_MATH" = true ]; then
        export LIBSQLITE3_FLAGS="-DSQLITE_ENABLE_MATH_FUNCTIONS"
    else
        unset LIBSQLITE3_FLAGS
    fi
}

cleanup_math_environment() {
    unset LIBSQLITE3_FLAGS
}

print_error_context() {
    local output="$1"

    echo "$output" | awk '
        /error\[E[0-9]+\]|^error:/ {
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
