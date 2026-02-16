#!/bin/bash

# Script to build all subfolders that have both Cargo.toml and README.docify.md
# This script runs `cargo build --features _generate_readme --no-default-features`
# on each qualifying folder.
#
# Usage: ./build_readme_docs.sh [--help|-h]

set -e  # Exit on any error

# Check for help flag
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Usage: $0 [--help|-h]"
    echo ""
    echo "This script builds documentation for all subfolders that contain both:"
    echo "  - Cargo.toml"
    echo "  - README.docify.md"
    echo ""
    echo "It runs the following command in each qualifying folder:"
    echo "  cargo build --features _generate_readme --no-default-features"
    echo ""
    echo "Options:"
    echo "  --help, -h    Show this help message and exit"
    exit 0
fi

# Get the project root directory (parent of the scripts folder)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Get easy-sql package version from -main/Cargo.toml and normalize to major.minor
EASY_SQL_CARGO_TOML="$PROJECT_ROOT/-main/Cargo.toml"

if [ ! -f "$EASY_SQL_CARGO_TOML" ]; then
    echo "❌ Could not find easy-sql manifest at: $EASY_SQL_CARGO_TOML"
    exit 1
fi

EASY_SQL_FULL_VERSION="$(awk -F'"' '/^version[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"[[:space:]]*$/ { print $2; exit }' "$EASY_SQL_CARGO_TOML")"

if [ -z "$EASY_SQL_FULL_VERSION" ]; then
    echo "❌ Could not read easy-sql version from: $EASY_SQL_CARGO_TOML"
    exit 1
fi

EASY_SQL_README_VERSION="$(echo "$EASY_SQL_FULL_VERSION" | cut -d. -f1,2)"

echo "Project root: $PROJECT_ROOT"
echo "Using easy-sql README version: $EASY_SQL_README_VERSION"
echo "Building documentation for folders with both Cargo.toml and README.docify.md..."

# Counter for processed folders
processed=0

# Find all directories that contain both Cargo.toml and README.docify.md
# Skip target directories to avoid building packaged crates.
cargo_tomls=($(find "$PROJECT_ROOT" -path "*/target/*" -prune -o -name "Cargo.toml" -type f -print))

for cargo_toml in "${cargo_tomls[@]}"; do
    dir="$(dirname "$cargo_toml")"
    
    # Skip the root Cargo.toml
    if [ "$dir" = "$PROJECT_ROOT" ]; then
        continue
    fi
    
    # Check if README.docify.md exists in the same directory
    if [ -f "$dir/README.docify.md" ]; then
        # Update easy-sql dependency version in README.docify.md (major.minor format)
        sed -E -i "s/(easy-sql\s*=\s*\{\s*version\s*=\s*\")[0-9]+\.[0-9]+(\"\s*,)/\1$EASY_SQL_README_VERSION\2/g" "$dir/README.docify.md"

        echo ""
        echo "===================================================="
        echo "Processing: $(basename "$dir")"
        echo "Directory: $dir"
        echo "Updated README.docify.md easy-sql version to: $EASY_SQL_README_VERSION"
        echo "===================================================="
        
        # Change to the directory and run cargo build
        cd "$dir"
        
        echo "Running: cargo build --features _generate_readme --no-default-features"
        if cargo build --features _generate_readme --no-default-features; then
            echo "✅ Successfully built $(basename "$dir")"
            processed=$((processed + 1))
        else
            echo "❌ Failed to build $(basename "$dir")"
            exit 1
        fi
        
        # Return to project root
        cd "$PROJECT_ROOT"
    fi
done

echo ""
echo "===================================================="
echo "Build complete!"
echo "Processed $processed folders successfully."
echo "===================================================="