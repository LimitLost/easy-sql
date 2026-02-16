#!/usr/bin/env bash
set -euo pipefail

# publish.sh - Select and publish a crate from this repository
# Default behavior runs: cargo publish --dry-run
# Docs: https://doc.rust-lang.org/cargo/commands/cargo-publish.html

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(dirname "$SCRIPT_DIR")

PUBLISH_MODE="dry-run"
PASS_ARGS=()

usage() {
  cat <<EOF
Usage: $(basename "$0") [--dry-run|--publish] [-- <cargo publish args>]

Runs cargo publish for a selected crate. By default it runs a dry-run and
offers to perform the actual publish after a successful dry-run.

Options:
  --dry-run     Run with --dry-run (default)
  --publish     Actually publish (no --dry-run)
  -h, --help    Show this help

Examples:
  ./scripts/publish.sh
  ./scripts/publish.sh --dry-run -- --allow-dirty
  ./scripts/publish.sh --publish -- --registry my-registry
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) PUBLISH_MODE="dry-run"; shift ;;
    --publish) PUBLISH_MODE="publish"; shift ;;
    -h|--help) usage; exit 0 ;;
    --) shift; PASS_ARGS+=("$@"); break ;;
    *) PASS_ARGS+=("$1"); shift ;;
  esac
done

mapfile -t MANIFESTS < <(
  find "$ROOT_DIR" -name "Cargo.toml" \
    -not -path "*/target/*" \
    -not -path "*/.cargo-toml-backups/*" \
    -print
)

CRATE_NAMES=()
CRATE_MANIFESTS=()

get_crate_name() {
  local manifest="$1"
  awk '
    $0 ~ /^\[package\]/ {inpkg=1; next}
    $0 ~ /^\[/ {inpkg=0}
    inpkg && $1 ~ /^name/ {
      gsub(/"/, "", $3);
      print $3;
      exit
    }
  ' "$manifest"
}

for manifest in "${MANIFESTS[@]}"; do
  name=$(get_crate_name "$manifest")
  if [[ -n "$name" ]]; then
    CRATE_NAMES+=("$name")
    CRATE_MANIFESTS+=("$manifest")
  fi
done

if [[ ${#CRATE_NAMES[@]} -eq 0 ]]; then
  echo "No publishable crates found." >&2
  exit 1
fi

echo "Available crates:"
for i in "${!CRATE_NAMES[@]}"; do
  printf "  %2d) %s\n" $((i + 1)) "${CRATE_NAMES[$i]}"
done

echo
read -r -p "Select crate number to publish: " choice

if [[ ! "$choice" =~ ^[0-9]+$ ]] || (( choice < 1 || choice > ${#CRATE_NAMES[@]} )); then
  echo "Invalid selection." >&2
  exit 1
fi

index=$((choice - 1))
crate_name="${CRATE_NAMES[$index]}"
manifest="${CRATE_MANIFESTS[$index]}"

command=(cargo publish --manifest-path "$manifest")
if [[ "$PUBLISH_MODE" == "dry-run" ]]; then
  command+=(--dry-run)
fi
command+=("${PASS_ARGS[@]}")

echo
echo "Crate: $crate_name"
echo "Manifest: $manifest"
if [[ "$PUBLISH_MODE" == "dry-run" ]]; then
  echo "Mode: dry-run"
else
  echo "Mode: publish"
fi

echo
echo "Running: ${command[*]}"
"${command[@]}"

if [[ "$PUBLISH_MODE" == "dry-run" ]]; then
  echo
  read -r -p "Dry-run complete. Publish for real now? [y/N]: " confirm
  if [[ "$confirm" =~ ^[Yy]$ ]]; then
    publish_command=(cargo publish --manifest-path "$manifest" "${PASS_ARGS[@]}")
    echo
    echo "Running: ${publish_command[*]}"
    "${publish_command[@]}"
  else
    echo "Publish skipped."
  fi
fi
