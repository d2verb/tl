#!/usr/bin/env bash
#
# Generate and update release notes for a GitHub Release
#
# Usage: ./scripts/update-release-notes.sh <version>
# Example: ./scripts/update-release-notes.sh v0.3.0
#
# Prerequisites:
# - gh CLI installed and authenticated
# - claude CLI installed (Claude Code)
# - $EDITOR set (defaults to vi)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_FILE="$SCRIPT_DIR/release-notes-template.md"

# Check arguments
if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v0.3.0"
    exit 1
fi

VERSION="$1"

# Validate version format
if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+.*$ ]]; then
    echo "Error: Version must start with 'v' followed by semver (e.g., v0.3.0)"
    exit 1
fi

# Check prerequisites
command -v gh >/dev/null 2>&1 || { echo "Error: gh CLI not found"; exit 1; }
command -v claude >/dev/null 2>&1 || { echo "Error: claude CLI not found"; exit 1; }

# Check if release exists
if ! gh release view "$VERSION" >/dev/null 2>&1; then
    echo "Error: Release $VERSION not found. Wait for CI to complete."
    exit 1
fi

# Get previous version
PREV_VERSION=$(git describe --tags --abbrev=0 "${VERSION}^" 2>/dev/null || echo "")
if [[ -z "$PREV_VERSION" ]]; then
    echo "Note: No previous version found. Using all commits up to $VERSION."
    GIT_RANGE="$VERSION"
else
    echo "Generating release notes for $PREV_VERSION..$VERSION"
    GIT_RANGE="$PREV_VERSION..$VERSION"
fi

# Get git log
GIT_LOG=$(git log --pretty=format:"- %s (%h)" "$GIT_RANGE" 2>/dev/null || git log --pretty=format:"- %s (%h)" "$VERSION" -n 50)

# Read template
TEMPLATE=$(cat "$TEMPLATE_FILE")

# Create temp file for notes
NOTES_FILE=$(mktemp)
trap 'rm -f "$NOTES_FILE"' EXIT

# Get existing release notes (cargo-dist generates install instructions)
echo "Fetching existing release notes..."
EXISTING_NOTES=$(gh release view "$VERSION" --json body --jq '.body')

# Generate release notes using Claude Code
echo "Generating release notes with Claude Code..."
PROMPT="$TEMPLATE

Git log:
$GIT_LOG"

GENERATED_NOTES=$(claude --print "$PROMPT")

# Combine: generated notes first, then existing notes
{
    echo "$GENERATED_NOTES"
    echo ""
    echo "$EXISTING_NOTES"
} > "$NOTES_FILE"

# Open in editor for review
EDITOR="${EDITOR:-vi}"
echo "Opening release notes in $EDITOR for review..."
"$EDITOR" "$NOTES_FILE"

# Confirm
echo ""
read -rp "Update GitHub Release with these notes? [y/N] " CONFIRM
if [[ ! "$CONFIRM" =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Update release
echo "Updating release $VERSION..."
gh release edit "$VERSION" --notes-file "$NOTES_FILE"

echo "Done! Release notes updated for $VERSION"
