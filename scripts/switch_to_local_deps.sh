#!/usr/bin/env bash

# Script to switch from version dependencies to local path dependencies
# Replaces version = "..." with path = "..." using internal crate versions

set -euo pipefail

# Get the project root directory (parent of scripts folder)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DRY_RUN=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    cat <<EOF
Usage: $(basename "$0") [--dry-run]

Options:
  --dry-run     Show changes without modifying files
  -h, --help    Show this help message
EOF
}

require_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo -e "${RED}✗ Required command not found: $1${NC}"
        exit 1
    fi
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            exit 1
            ;;
    esac
done

require_cmd realpath
require_cmd python3

# Internal crates for this repository
declare -A CRATE_PATHS=(
    ["easy-sql-build"]="build"
    ["easy-sql-macros"]="macros"
    ["easy-sql-compilation-data"]="compilation-data"
)

declare -A CRATE_VERSIONS=()

get_crate_version() {
    local manifest="$1"
    awk '
        $0 ~ /^\[package\]/ {inpkg=1; next}
        $0 ~ /^\[/ {inpkg=0}
        inpkg && $1 ~ /^version/ {
            gsub(/"/, "", $3);
            print $3;
            exit
        }
    ' "$manifest"
}

init_versions() {
    for crate in "${!CRATE_PATHS[@]}"; do
        local manifest="$PROJECT_ROOT/${CRATE_PATHS[$crate]}/Cargo.toml"
        if [ ! -f "$manifest" ]; then
            echo -e "${RED}✗ Missing Cargo.toml for $crate at $manifest${NC}"
            exit 1
        fi
        local version
        version="$(get_crate_version "$manifest")"
        if [ -z "$version" ]; then
            echo -e "${RED}✗ Failed to read version for $crate${NC}"
            exit 1
        fi
        CRATE_VERSIONS[$crate]="$version"
    done
}

collect_cargo_tomls() {
    find "$PROJECT_ROOT" -name "Cargo.toml" \
        -not -path "*/target/*" \
        -not -path "*/.cargo-toml-backups/*" \
        -print0
}


# Function to replace version with path for internal dependencies
replace_versions_with_paths() {
    echo -e "${YELLOW}Replacing version dependencies with path dependencies...${NC}"

    local files_scanned=0
    local files_changed=0

    while IFS= read -r -d '' toml_file; do
        ((files_scanned++)) || true
        toml_dir="$(dirname "$toml_file")"
        rel_to_root=$(realpath --relative-to="$toml_dir" "$PROJECT_ROOT")

        deps_args=()
        for crate in "${!CRATE_PATHS[@]}"; do
            local_path="$rel_to_root/${CRATE_PATHS[$crate]}"
            deps_args+=("--dep" "${crate}|${local_path}|${CRATE_VERSIONS[$crate]}")
        done

        result=$(python3 - "$toml_file" "local" "$DRY_RUN" "${deps_args[@]}" <<'PY'
import re
import sys

file_path = sys.argv[1]
mode = sys.argv[2]
dry_run = sys.argv[3] == "1"
deps_args = sys.argv[4:]

deps = []
for i in range(0, len(deps_args), 2):
    if deps_args[i] != "--dep":
        continue
    name, path, version = deps_args[i + 1].split("|", 2)
    deps.append((name, path, version))

def split_comment(line):
    in_str = False
    escape = False
    for idx, ch in enumerate(line):
        if ch == '"' and not escape:
            in_str = not in_str
        if ch == '#' and not in_str:
            return line[:idx].rstrip(), line[idx:]
        escape = (ch == '\\') and not escape
    return line.rstrip(), ""

def split_inline_table(text):
    parts = []
    current = ""
    in_str = False
    escape = False
    for ch in text:
        if ch == '"' and not escape:
            in_str = not in_str
        if ch == ',' and not in_str:
            if current.strip():
                parts.append(current.strip())
            current = ""
        else:
            current += ch
        escape = (ch == '\\') and not escape
    if current.strip():
        parts.append(current.strip())
    return parts

def parse_inline_table(text):
    items = []
    for part in split_inline_table(text):
        if '=' not in part:
            continue
        key, value = part.split('=', 1)
        items.append((key.strip(), value.strip()))
    return items

def rebuild_inline_table(items):
    return "{ " + ", ".join(f"{k} = {v}" for k, v in items) + " }"

changed = False
with open(file_path, "r", encoding="utf-8") as handle:
    lines = handle.readlines()

new_lines = []
for line in lines:
    line = line.rstrip("\r\n")
    base, comment = split_comment(line)
    updated = base
    for name, path, version in deps:
        pattern = re.compile(rf'^(\s*){re.escape(name)}\s*=\s*(.+?)\s*$')
        match = pattern.match(updated)
        if not match:
            continue
        prefix, value = match.group(1), match.group(2)
        value = value.strip()
        if value.startswith('{') and value.endswith('}'):
            inner = value[1:-1].strip()
            items = parse_inline_table(inner)
            keys = [k for k, _ in items]
            if mode == "local":
                if "path" in keys:
                    break
                if "version" in keys:
                    items = [
                        ("path", f'"{path}"') if k == "version" else (k, v)
                        for k, v in items
                    ]
                else:
                    items.insert(0, ("path", f'"{path}"'))
            else:
                if "path" in keys:
                    new_items = []
                    version_set = False
                    for k, v in items:
                        if k == "path":
                            continue
                        if k == "version":
                            new_items.append((k, f'"{version}"'))
                            version_set = True
                        else:
                            new_items.append((k, v))
                    if not version_set:
                        new_items.insert(0, ("version", f'"{version}"'))
                    items = new_items
                else:
                    break
            updated = f"{prefix}{name} = {rebuild_inline_table(items)}"
        elif value.startswith('"'):
            if mode == "local":
                updated = f"{prefix}{name} = {{ path = \"{path}\" }}"
            else:
                updated = f"{prefix}{name} = \"{version}\""
        else:
            break
        changed = True
        break
    new_lines.append(updated + comment + "\n")

if changed and not dry_run:
    with open(file_path, "w", encoding="utf-8") as handle:
        handle.writelines(new_lines)

print("changed" if changed else "unchanged")
PY
)

        if [ "$result" = "changed" ]; then
            ((files_changed++)) || true
        fi
    done < <(collect_cargo_tomls) || true

    echo -e "${GREEN}✓ Updated $files_changed of $files_scanned Cargo.toml file(s)${NC}"
    if [ "$DRY_RUN" -eq 1 ]; then
        echo -e "${YELLOW}Dry run enabled; no files were modified.${NC}"
    fi
}

# Main execution
echo
echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}Switching to Local Path Dependencies${NC}"
echo -e "${BLUE}=========================================${NC}"
echo

# Step 0: Initialize versions
init_versions

# Step 1: Replace dependencies
replace_versions_with_paths
echo

echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}Successfully switched to local dependencies!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo
echo -e "${YELLOW}To switch back to version dependencies, run:${NC}"
echo -e "${YELLOW}  ./scripts/switch_to_version_deps.sh${NC}"
echo
