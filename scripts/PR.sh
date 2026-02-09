#!/usr/bin/env bash
set -euo pipefail

# PR.sh - Prepare current changes for a pull request
# Steps:
#   1) Switch to local dependencies
#   2) Run full test suite
#   3) Build README docs
#   4) Join READMEs

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(dirname "$SCRIPT_DIR")

run_step() {
  local name="$1"
  shift
  echo
  echo "========================================"
  echo "Step: $name"
  echo "========================================"
  if "$@"; then
    echo "✅ Completed: $name"
  else
    echo "❌ Failed: $name"
    exit 1
  fi
}

run_step "Switch to local dependencies" "$SCRIPT_DIR/switch_to_local_deps.sh"
run_step "Run full test suite" "$SCRIPT_DIR/test-full.sh"
run_step "Build README docs" "$SCRIPT_DIR/build_readme_docs.sh"
run_step "Join README files" "$SCRIPT_DIR/readme_join.sh"

echo
echo "✅ PR preparation completed successfully."
