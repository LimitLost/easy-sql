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
    MACROS_DIR="$ROOT_DIR/macros"

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

append_feature_to_csv_var() {
    local var_name="$1"
    local feature="$2"
    local current_value="${!var_name}"

    if [ -z "$feature" ]; then
        return
    fi

    if [ -z "$current_value" ]; then
        printf -v "$var_name" '%s' "$feature"
        return
    fi

    case ",$current_value," in
        *",$feature,"*) ;;
        *) printf -v "$var_name" '%s' "$current_value,$feature" ;;
    esac
}

append_feature_dedup() {
    local feature="$1"

    append_feature_to_csv_var "FEATURES" "$feature"
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

    if [ "$USE_WATCHER" = true ]; then
        append_feature_dedup "watcher"
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

build_macros_features_string() {
    MACROS_FEATURES=""

    if [ "$USE_OUTPUT_COLUMNS" = true ]; then
        append_feature_to_csv_var "MACROS_FEATURES" "use_output_columns"
    fi

    if [ "$USE_MIGRATIONS" = true ]; then
        append_feature_to_csv_var "MACROS_FEATURES" "migrations"
    fi

    if [ "$USE_CHECK_DUPLICATE_TABLE_NAMES" = true ]; then
        append_feature_to_csv_var "MACROS_FEATURES" "check_duplicate_table_names"
    fi
}

build_macros_full_suite_features_string() {
    build_macros_features_string
    append_feature_to_csv_var "MACROS_FEATURES" "_ui_tests"
}

macros_feature_display() {
    local features="$1"

    if [ -n "$features" ]; then
        printf '%s' "$features"
    else
        printf '(none)'
    fi
}

is_macros_ui_exact_target() {
    case "$1" in
        ui_query_capabilities_compile_fail|ui_migration_attribute_conflicts_compile_fail|ui_sql_unknown_attr_keys_compile_fail)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

macros_requested_unsupported_flags() {
    local ignored_flags=()

    if [ "$USE_MATH" = true ]; then
        ignored_flags+=("--math")
    fi
    if [ "$USE_EXTRA_DEFAULT_ALL" = true ]; then
        ignored_flags+=("--extra-default-all")
    fi
    if [ "$USE_EXTRA_DEFAULT_TIME" = true ]; then
        ignored_flags+=("--extra-default-time")
    fi
    if [ "$USE_EXTRA_DEFAULT_CHRONO" = true ]; then
        ignored_flags+=("--extra-default-chrono")
    fi

    printf '%s' "${ignored_flags[*]}"
}

warn_macros_ignored_flags() {
    local ignored_flags
    ignored_flags=$(macros_requested_unsupported_flags)

    if [ -n "$ignored_flags" ]; then
        echo -e "${YELLOW}⚠ Ignoring macros-unsupported flags: $ignored_flags${NC}"
    fi
}

resolve_macros_test_features() {
    local test_target="$1"
    local ignored_flags

    MACROS_TARGET_FEATURES=""
    MACROS_TARGET_KIND="pattern"

    case "$test_target" in
        ui_query_capabilities_compile_fail)
            ignored_flags=$(macros_requested_unsupported_flags)
            if [ -n "$ignored_flags" ]; then
                echo -e "${RED}Error: $test_target does not accept $ignored_flags${NC}"
                return 1
            fi
            MACROS_TARGET_FEATURES="_ui_tests"
            MACROS_TARGET_KIND="integration-test"
            return 0
            ;;
        ui_migration_attribute_conflicts_compile_fail)
            ignored_flags=$(macros_requested_unsupported_flags)
            if [ -n "$ignored_flags" ]; then
                echo -e "${RED}Error: $test_target does not accept $ignored_flags${NC}"
                return 1
            fi
            MACROS_TARGET_FEATURES="_ui_tests,migrations"
            MACROS_TARGET_KIND="integration-test"
            return 0
            ;;
        ui_sql_unknown_attr_keys_compile_fail)
            ignored_flags=$(macros_requested_unsupported_flags)
            if [ -n "$ignored_flags" ]; then
                echo -e "${RED}Error: $test_target does not accept $ignored_flags${NC}"
                return 1
            fi
            MACROS_TARGET_KIND="integration-test"
            return 0
            ;;
        *)
            build_macros_features_string
            MACROS_TARGET_FEATURES="$MACROS_FEATURES"
            return 0
            ;;
    esac
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

main_test_pattern_exists_in_sources() {
    local pattern="$1"
    local search_roots=()
    local root

    # Check the main crate test sources before relaxing an all-skipped run.
    # Reason: cargo reports feature-gated self-skips and plain typos as the same
    # "0 tests ran" outcome, but only the source-known case should succeed.
    if [ -d "$MAIN_DIR/src/tests" ]; then
        search_roots+=("$MAIN_DIR/src/tests")
    fi

    if [ -d "$MAIN_DIR/tests" ]; then
        search_roots+=("$MAIN_DIR/tests")
    fi

    if [ ${#search_roots[@]} -eq 0 ]; then
        return 1
    fi

    for root in "${search_roots[@]}"; do
        # Match function/module names and other checked-in test identifiers first.
        # Reason: a direct content hit is the strongest signal that the requested
        # pattern exists but is currently gated off.
        if command -v rg >/dev/null 2>&1; then
            if rg -l --fixed-strings --glob '*.rs' "$pattern" "$root" >/dev/null 2>&1; then
                return 0
            fi
        else
            if grep -R -l --include='*.rs' --fixed-strings "$pattern" "$root" >/dev/null 2>&1; then
                return 0
            fi
        fi

        # Fall back to file-name matches for patterns that target whole test files.
        # Reason: some test invocations use a file/target name rather than a single
        # Rust test function name.
        if find "$root" -type f -name "*${pattern}*.rs" -print -quit | grep -q .; then
            return 0
        fi
    done

    return 1
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

extract_test_result_counts() {
    local output="$1"
    local line
    local last_passed=0
    local last_failed=0
    local total_passed=0
    local total_failed=0
    local saw_nonzero=0

    while IFS= read -r line; do
        if [[ "$line" =~ test[[:space:]]result:[[:space:]](ok|FAILED)\.[[:space:]]([0-9]+)[[:space:]]passed\;[[:space:]]([0-9]+)[[:space:]]failed ]]; then
            local passed="${BASH_REMATCH[2]}"
            local failed="${BASH_REMATCH[3]}"
            last_passed="$passed"
            last_failed="$failed"

            if [ "$passed" -gt 0 ] || [ "$failed" -gt 0 ]; then
                total_passed=$((total_passed + passed))
                total_failed=$((total_failed + failed))
                saw_nonzero=1
            fi
        fi
    done <<< "$output"

    if [ "$saw_nonzero" -eq 1 ]; then
        echo "$total_passed $total_failed 1"
    else
        echo "$last_passed $last_failed 0"
    fi
}

# Braille spinner frames shared by the compile and test phases.
# Reason: long silent stretches (cold compile, DB-pool setup, doctest compile) looked hung; a
# spinner plus rising seconds prove the run is alive even while the numbers hold still.
SPINNER_FRAMES=(⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏)

# Render the compile-phase status line (TTY only): "<spin> [label] <cargo progress> … Ns".
# Shows the pinned cargo line (self-describing, e.g. "Compiling easy-sql v…") once one has
# appeared, otherwise the phase word ("Compiling" / "Preparing tests") as a fallback.
_render_compile_line() {
    local label="$1" phase="$2" last="$3" start="$4" spin="$5"
    local sp="${SPINNER_FRAMES[$(( spin % ${#SPINNER_FRAMES[@]} ))]}"
    local text="${last:-$phase}"
    printf "\r\033[K${BLUE}%s [%s] %s${NC} … %ds" \
        "$sp" "$label" "${text:0:60}" "$(( $(date +%s) - start ))"
}

# Drain a backgrounded cargo command's output ($fifo) until it exits, showing live progress.
# TTY: one collapsing spinner line (spinner + last line + seconds). Non-TTY: echo cargo's own lines
# plainly (no \r spam). All lines are appended to $out_file for later error context.
# Args: label phase out_file fifo pid ; returns the command's exit status.
drain_compile_stream() {
    local label="$1" phase="$2" out_file="$3" fifo="$4" pid="$5"
    local start spin=0 last_line="" line
    start=$(date +%s)

    exec 4< "$fifo"
    while true; do
        if IFS= read -r -t 1 line <&4; then
            printf '%s\n' "$line" >> "$out_file"
            # Pin the live text to cargo's crate-progress lines (Compiling/Finished/…), stripping
            # the leading indent — so the display names the current crate instead of flickering
            # through warning/error spew. Full output is still kept in out_file for error context.
            local is_progress=false
            if [[ "$line" =~ ^[[:space:]]*(Compiling|Building|Checking|Documenting|Finished|Downloading|Updating|Fresh|Installing)[[:space:]] ]]; then
                last_line="${line#"${line%%[![:space:]]*}"}"
                is_progress=true
            fi
            if [ -t 1 ]; then
                _render_compile_line "$label" "$phase" "$last_line" "$start" "$spin"
                ((spin++))
            elif [ "$is_progress" = true ]; then
                printf '%s\n' "$last_line"
            fi
        else
            kill -0 "$pid" 2>/dev/null || break
            if [ -t 1 ]; then
                _render_compile_line "$label" "$phase" "$last_line" "$start" "$spin"
                ((spin++))
            fi
        fi
    done
    exec 4<&-

    # Exit status comes from wait, not the loop end (the loop only sees FIFO EOF).
    wait "$pid"
    return $?
}

# Render the test-phase counter. TTY adds a spinner + elapsed seconds and clears the line; non-TTY
# reproduces the pre-existing plain counter so captured logs are unchanged ($2 = done count).
# Args: label done total passed failed left secs spin
_render_counter() {
    local label="$1" total="$3" passed="$4" failed="$5" left="$6" secs="$7" spin="$8"
    if [ -t 1 ]; then
        local sp="${SPINNER_FRAMES[$(( spin % ${#SPINNER_FRAMES[@]} ))]}"
        printf "\r\033[K${BLUE}%s [%s] %d/%d done | pass=%d fail=%d | left=%d | %ds${NC}" \
            "$sp" "$label" "$2" "$total" "$passed" "$failed" "$left" "$secs"
    else
        printf "\r${BLUE}[%s] %d/%d done | pass=%d fail=%d | left=%d${NC}" \
            "$label" "$2" "$total" "$passed" "$failed" "$left"
    fi
}

# Stream one `cargo test` invocation with a live done/total counter and split build/test timers.
# Reason: the build, pre-count, and live-streamed run are identical for every backend and the
# macros crate, so they live here once instead of being duplicated (and drifting) per script.
#
# Args: label  features_csv  [pattern]  [manifest_path]
# Results (globals, reset on each call):
#   RUN_BUILD_FAILED  true if `cargo build` failed (RUN_OUTPUT holds the build log)
#   RUN_OUTPUT        full captured text (build log on build failure, else the test log)
#   RUN_TOTAL         doctest-excluded test count for the "amount left" denominator
#   RUN_PASSED/FAILED authoritative counts from extract_test_result_counts
#   RUN_SAW_NONZERO   1 when at least one concrete "test result:" block was seen
#   RUN_STATUS        exit status of the test run (0 = ok)
#   RUN_BUILD_SECS    wall seconds for build + pre-count
#   RUN_TEST_SECS     wall seconds for the test run only
stream_cargo_test() {
    local label="$1"
    local features="$2"
    local pattern="$3"
    local manifest_path="$4"
    # mode: "filter" (pattern is a test-name filter, doctests excluded via --lib --tests) or
    # "test-target" (pattern is an integration-test target name run via --test, e.g. trybuild UI).
    local mode="${5:-filter}"
    local hang_timeout="${HANG_TIMEOUT_SEC:-5}"

    # Reset result globals.
    RUN_BUILD_FAILED=false
    RUN_OUTPUT=""
    RUN_TOTAL=0
    RUN_PASSED=0
    RUN_FAILED=0
    RUN_SAW_NONZERO=0
    RUN_STATUS=0
    RUN_BUILD_SECS=0
    RUN_TEST_SECS=0

    # Common cargo args shared by build / list / run.
    local base=(--color never --no-default-features --features "$features")
    if [ -n "$manifest_path" ]; then
        base=(--manifest-path "$manifest_path" "${base[@]}")
    fi

    # Shared tmp dir for the build log, the list output, and the test FIFO.
    local tmp_dir
    tmp_dir=$(mktemp -d "/tmp/easy-sql-stream-XXXXXX")
    if [ -z "$tmp_dir" ] || [ ! -d "$tmp_dir" ]; then
        echo -e "${RED}✗ Failed to create temporary directory${NC}"
        RUN_STATUS=1
        return 1
    fi

    # Step 1: build + pre-count, timed together as "build", each shown as a live compile phase.
    # Reason: `cargo build` compiles only the lib; the test harness is not built until
    # `cargo test`/`--list` runs. --list forces that harness compile (the big cold-build cost),
    # so it belongs in the build timer; the harness it produces is then reused by the run below,
    # leaving the test timer to measure near-pure execution. Both were silent captures before, which
    # looked like a hang on a cold build — now each streams a spinner + last line + timer.
    local b0
    b0=$(date +%s)

    # Compile phase: cargo build (all output is compile progress).
    local build_fifo="$tmp_dir/build.fifo"
    local build_log="$tmp_dir/build.log"
    : > "$build_log"
    if ! mkfifo "$build_fifo"; then
        echo -e "${RED}✗ Failed to create temporary FIFO${NC}"
        rm -rf "$tmp_dir"; RUN_STATUS=1; return 1
    fi
    cargo build "${base[@]}" > "$build_fifo" 2>&1 &
    drain_compile_stream "$label" "Compiling" "$build_log" "$build_fifo" "$!"
    local build_status=$?
    [ -t 1 ] && printf "\r\033[K"
    RUN_OUTPUT=$(cat "$build_log")
    if [ "$build_status" -ne 0 ]; then
        RUN_BUILD_FAILED=true
        RUN_STATUS=1
        RUN_BUILD_SECS=$(( $(date +%s) - b0 ))
        rm -rf "$tmp_dir"
        return 1
    fi

    # List phase: pre-count tests (the harness compile happens here). stdout = the test list
    # (counted), stderr = the "Compiling …" progress shown live.
    # Total excludes doctests via --lib --tests; a filtered pattern narrows it to the match set.
    # In test-target mode the count comes from the named integration binary instead.
    local list_cmd=(cargo test)
    if [ "$mode" = "test-target" ]; then
        list_cmd+=("${base[@]}" --test "$pattern" -- --list)
    else
        list_cmd+=(--lib --tests "${base[@]}")
        [ -n "$pattern" ] && list_cmd+=("$pattern")
        list_cmd+=(-- --list)
    fi
    local list_fifo="$tmp_dir/list.fifo"
    local list_out="$tmp_dir/list.out"
    local list_log="$tmp_dir/list.err"
    : > "$list_log"
    if ! mkfifo "$list_fifo"; then
        echo -e "${RED}✗ Failed to create temporary FIFO${NC}"
        rm -rf "$tmp_dir"; RUN_STATUS=1; return 1
    fi
    "${list_cmd[@]}" > "$list_out" 2> "$list_fifo" &
    drain_compile_stream "$label" "Preparing tests" "$list_log" "$list_fifo" "$!"
    [ -t 1 ] && printf "\r\033[K"
    RUN_TOTAL=$(grep -cE ': test$' "$list_out" 2>/dev/null)
    [ -z "$RUN_TOTAL" ] && RUN_TOTAL=0
    RUN_BUILD_SECS=$(( $(date +%s) - b0 ))

    # Keep the final compile time on screen before the test counter takes over.
    printf "${GREEN}✓ [%s] Compiled in %ds${NC}\n" "$label" "$RUN_BUILD_SECS"

    # Step 2: run tests, streaming output via FIFO while keeping the full log text.
    local test_output=""
    local live_passed=0
    local live_failed=0
    local live_ignored_nondoc=0
    local in_doctests=false
    local saw_ok_result=false
    local saw_failure_marker=false
    local forced_success_kill=false
    local spin=0
    local fifo_path="$tmp_dir/test.fifo"
    if ! mkfifo "$fifo_path"; then
        echo -e "${RED}✗ Failed to create temporary FIFO${NC}"
        rm -rf "$tmp_dir"
        RUN_STATUS=1
        return 1
    fi

    local run_cmd=(cargo test "${base[@]}")
    if [ "$mode" = "test-target" ]; then
        run_cmd+=(--test "$pattern")
    else
        [ -n "$pattern" ] && run_cmd+=("$pattern")
    fi

    local t0
    t0=$(date +%s)
    "${run_cmd[@]}" > "$fifo_path" 2>&1 &
    local cargo_pid=$!
    local last_activity_ts
    last_activity_ts=$(date +%s)

    # Draw the counter immediately (TTY) so the test phase is visible during DB-pool startup,
    # not only once the first test completes.
    if [ -t 1 ]; then
        _render_counter "$label" 0 "$RUN_TOTAL" 0 0 "$RUN_TOTAL" 0 "$spin"
        ((spin++))
    fi

    exec 3< "$fifo_path"
    while true; do
        if IFS= read -r -t 1 line <&3; then
            test_output+="$line"$'\n'
            last_activity_ts=$(date +%s)

            # Everything after the "Doc-tests" banner is the doctest binary; its tests run as
            # ignored and are excluded from RUN_TOTAL, so they must not advance the counter.
            if [[ "$line" =~ ^[[:space:]]*Doc-tests ]]; then
                in_doctests=true
            fi

            # Test-name portion must be a single token with no spaces or slashes: this counts real
            # libtest functions (foo::bar::baz) while skipping trybuild's per-case progress lines
            # (test tests/ui/x.rs ... ok) and doctest lines (test src/lib.rs - f (line N) ... ok),
            # which would otherwise inflate the counter past RUN_TOTAL.
            local changed=false
            if [[ "$line" =~ ^test[[:space:]][^[:space:]/]+[[:space:]]\.\.\.[[:space:]]ok$ ]]; then
                ((live_passed++)); changed=true
            elif [[ "$line" =~ ^test[[:space:]][^[:space:]/]+[[:space:]]\.\.\.[[:space:]]FAILED$ ]]; then
                ((live_failed++)); saw_failure_marker=true; changed=true
            elif [[ "$line" =~ ^test[[:space:]][^[:space:]/]+[[:space:]]\.\.\.[[:space:]]ignored ]]; then
                # Count non-doctest #[ignore] tests toward "done" so the bar still reaches total.
                if [ "$in_doctests" = false ]; then
                    ((live_ignored_nondoc++)); changed=true
                fi
            fi

            if [ "$changed" = true ]; then
                local done=$((live_passed + live_failed + live_ignored_nondoc))
                local left=$((RUN_TOTAL - done))
                [ "$left" -lt 0 ] && left=0
                _render_counter "$label" "$done" "$RUN_TOTAL" "$live_passed" "$live_failed" \
                    "$left" "$(( $(date +%s) - t0 ))" "$spin"
                ((spin++))
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

            # Tick the spinner + seconds (TTY) even with no new test line, so the phase never looks
            # frozen during the startup or doctest-compile gaps.
            if [ -t 1 ]; then
                local done=$((live_passed + live_failed + live_ignored_nondoc))
                local left=$((RUN_TOTAL - done))
                [ "$left" -lt 0 ] && left=0
                _render_counter "$label" "$done" "$RUN_TOTAL" "$live_passed" "$live_failed" \
                    "$left" "$(( $(date +%s) - t0 ))" "$spin"
                ((spin++))
            fi

            local now_ts
            now_ts=$(date +%s)
            local idle_sec=$((now_ts - last_activity_ts))

            if [ "$idle_sec" -ge "$hang_timeout" ] && \
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
    RUN_STATUS=$?
    RUN_TEST_SECS=$(( $(date +%s) - t0 ))
    rm -rf "$tmp_dir"
    echo ""

    if [ "$forced_success_kill" = true ] && [ "$saw_failure_marker" = false ]; then
        RUN_STATUS=0
    fi

    RUN_OUTPUT="$test_output"
    local passed failed saw_nonzero
    read -r passed failed saw_nonzero < <(extract_test_result_counts "$test_output")
    RUN_PASSED=${passed:-0}
    RUN_FAILED=${failed:-0}
    RUN_SAW_NONZERO=${saw_nonzero:-0}
    return 0
}

# Per-leg accumulators for the end-of-run summary table (shared by test-all / test-specific).
# Reason: both scripts want the same "backend : ✓ P/T | build Xs | test Ys" rollup, so the
# storage and rendering live here; each script keeps its own pass/fail exit decision.
reset_leg_summary() {
    LEG_LABELS=()
    LEG_STATUS=()
    LEG_PASSED=()
    LEG_TOTAL=()
    LEG_BUILD=()
    LEG_TEST=()
    SUITE_START_TS=$(date +%s)
}

# Record one leg. Args: label  status(pass|fail|skip)  passed  total  build_secs  test_secs
record_leg() {
    LEG_LABELS+=("$1")
    LEG_STATUS+=("$2")
    LEG_PASSED+=("$3")
    LEG_TOTAL+=("$4")
    LEG_BUILD+=("$5")
    LEG_TEST+=("$6")
}

# Render the per-leg rows plus total build/test/wall time. No exit decision here.
print_leg_table() {
    local i
    local total_build=0
    local total_test=0
    local wall=$(( $(date +%s) - ${SUITE_START_TS:-$(date +%s)} ))

    for i in "${!LEG_LABELS[@]}"; do
        local mark color
        case "${LEG_STATUS[$i]}" in
            pass) mark="✓"; color="$GREEN" ;;
            fail) mark="✗"; color="$RED" ;;
            *)    mark="⚠"; color="$YELLOW" ;;
        esac
        printf "${color}  %-11s %s %s/%s${NC} | build %ss | test %ss\n" \
            "${LEG_LABELS[$i]}" "$mark" "${LEG_PASSED[$i]}" "${LEG_TOTAL[$i]}" \
            "${LEG_BUILD[$i]}" "${LEG_TEST[$i]}"
        total_build=$(( total_build + ${LEG_BUILD[$i]} ))
        total_test=$(( total_test + ${LEG_TEST[$i]} ))
    done

    echo -e "${BLUE}  ── build ${total_build}s | test ${total_test}s | wall ${wall}s ──${NC}"
}
