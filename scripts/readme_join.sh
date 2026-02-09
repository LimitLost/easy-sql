#!/usr/bin/env bash
set -euo pipefail

# Script: readme_join.sh
# Purpose: Join "Readme_Start.md" and "README.md" into "README-CRATES-IO.md"
# in each subfolder of the workspace, skipping folders named "main-crate" and common
# build or VCS dirs. 

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DRY_RUN=false
HEADER_DEFAULT="$ROOT_DIR/scripts/readme_join_helpers/Readme_Start.md"
HEADER_PATH=""

# If a Rust trimmer exists, compile it now. We'll run it after we determine the
# resolved HEADER path so we can pass the explicit header file to the trimmer.
TRIM_RS="$ROOT_DIR/scripts/readme_join_helpers/trim_readme_start.rs"
TRIM_BIN="$ROOT_DIR/scripts/readme_join_helpers/trim_readme_start"
if [[ -f "$TRIM_RS" ]]; then
  echo "Found README_Start trimmer, compiling"
  if [[ ! -x "$TRIM_BIN" || "$TRIM_RS" -nt "$TRIM_BIN" ]]; then
    rustc "$TRIM_RS" -O -o "$TRIM_BIN" || echo "rustc failed to compile $TRIM_RS"
  fi
fi

usage() {
  cat <<EOF
Usage: $(basename "$0") [--apply] [--root <path>]

By default this writes a to README-CRATES-IO.md files. Use --dry to see what would be done.

Options:
  --apply        Write the output files instead of dry-run. (enabled by default)
  --dry          Perform a dry-run and print what would be done.
  --root <path>  Start search from <path> instead of the repository root (default: repo root).
  -h, --help     Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry) DRY_RUN=true; shift ;;
    --apply) DRY_RUN=false; shift ;;
    --root) ROOT_DIR="$2"; shift 2 ;;
    --header) HEADER_PATH="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1"; usage; exit 2 ;;
  esac
done

# Print repository root as a relative (basename) to keep logs concise
REPO_BASE="$(basename "$ROOT_DIR")"
echo "Repository root: $ROOT_DIR"
if [[ "$DRY_RUN" == "true" ]]; then
  echo "Mode: dry-run"
else
  echo "Mode: apply"
fi

# Determine header file
if [[ -n "$HEADER_PATH" ]]; then
  HEADER="$HEADER_PATH"
else
  HEADER="$HEADER_DEFAULT"
fi

if [[ ! -f "$HEADER" ]]; then
  echo "Header file not found: $HEADER" >&2
  echo "Provide one with --header <path>" >&2
  exit 2
fi
HEADER_DISPLAY="${HEADER#$ROOT_DIR/}"
echo "Header: $HEADER_DISPLAY"

# If we compiled the trimmer binary, run it with the explicit header path to
# trim trailing whitespace/newlines. The trimmer now requires the path as its
# first argument.
if [[ -x "$TRIM_BIN" ]]; then
 echo "==="
  echo "Running trimmer on header"
  "$TRIM_BIN" "$HEADER_DISPLAY" || echo "Trimmer exited non-zero"
  echo "==="
fi

# Find candidate directories: those that contain Sub-Crate-Readme_Start.md and README.md
# Exclude top-level 'main' and '.git' directories.

SKIP_DIRS=("-main" ".git" )
is_skipped() {
  local d="$1"
  for s in "${SKIP_DIRS[@]}"; do
    if [[ "${d##*/}" == "$s" ]]; then
      return 0
    fi
  done
  return 1
}

echo "==="
count=0
# Iterate immediate subdirectories of ROOT_DIR
for d in "$ROOT_DIR"/*/; do
  [[ -d "$d" ]] || continue
  # strip trailing slash
  dir="${d%/}"
  if is_skipped "$dir"; then
    # Print skipped directory relative to repo root
    rel_dir="${dir#$ROOT_DIR/}"
    echo "Skipping $rel_dir"
    continue
  fi
  readme="$dir/README.md"
  out="$dir/README-CRATES-IO.md"
  if [[ -f "$readme" ]]; then
    count=$((count+1))
    rel_dir="${dir#$ROOT_DIR/}"
    rel_out="${out#$ROOT_DIR/}"
    echo "Found README in: $rel_dir -> $rel_out"
    if [[ "$DRY_RUN" == "true" ]]; then
      echo "--- Begin (dry) $rel_out ---"
      sed -n '1,200p' "$HEADER" || true
      sed -n '1,200p' "$readme" || true
      echo "--- End (dry) $rel_out ---"
      echo
    else
      tmpfile=$(mktemp)
      cat "$HEADER" > "$tmpfile"
      cat "$readme" >> "$tmpfile"
      mv "$tmpfile" "$out"
      echo "Wrote $rel_out"
    fi
  echo "==="
  fi
done

echo "==="

if [[ $count -eq 0 ]]; then
  echo "No README files found in immediate subdirectories. Nothing to do."
else
  echo "Processed $count directories."
fi

# After joining is done, copy the main README into the repository root and
# insert README.structure.md above the Installation section.
MAIN_README="$ROOT_DIR/-main/README.md"
ROOT_README="$ROOT_DIR/README.md"
STRUCTURE_README="$ROOT_DIR/README.structure.md"

echo "==="
if [[ ! -f "$MAIN_README" ]]; then
  echo "Main README not found: $MAIN_README" >&2
  exit 2
fi
if [[ ! -f "$STRUCTURE_README" ]]; then
  echo "Structure README not found: $STRUCTURE_README" >&2
  exit 2
fi

if [[ "$DRY_RUN" == "true" ]]; then
  echo "Dry-run: would copy $MAIN_README -> $ROOT_README"
  echo "Dry-run: would insert $STRUCTURE_README above Installation in $ROOT_README"
else
  cp "$MAIN_README" "$ROOT_README"
  tmpfile=$(mktemp)
  awk -v insert_file="$STRUCTURE_README" '
    BEGIN { inserted = 0 }
    {
      if (!inserted && $0 ~ /^##[[:space:]]+Installation/) {
        while ((getline line < insert_file) > 0) print line
        print ""
        close(insert_file)
        inserted = 1
      }
      print
    }
    END {
      if (!inserted) {
        while ((getline line < insert_file) > 0) print line
        print ""
        close(insert_file)
      }
    }
  ' "$ROOT_README" > "$tmpfile"
  mv "$tmpfile" "$ROOT_README"
  echo "Updated $ROOT_README"
fi

exit 0
