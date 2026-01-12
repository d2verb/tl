# Releasing

This document describes how to create a new release of `tl`.

## Prerequisites

- CI must be green on `main`
- `CARGO_REGISTRY_TOKEN` secret configured in GitHub repository settings

## Release Process

### 1. Update version, commit, tag, and push

```bash
vim Cargo.toml  # Change version = "X.Y.Z"
git add Cargo.toml Cargo.lock
git commit -m "release: vX.Y.Z"
git tag vX.Y.Z
git push origin main --tags
```

### 2. Wait for CI

The release workflow will:
- Build binaries for 5 platforms (Linux x86_64/ARM64, macOS Intel/Apple Silicon, Windows x86_64)
- Publish to crates.io
- Create a GitHub Release with downloadable binaries and installer scripts

Monitor progress at: https://github.com/d2verb/tl/actions

**Expected duration:** 10-15 minutes

### 3. Add release notes

After CI completes:

```bash
./scripts/update-release-notes.sh vX.Y.Z
```

This script:
1. Generates release notes using Claude Code
2. Opens them in your editor for review
3. Updates the GitHub Release

## Recovery from Failed Release

If the release fails:

```bash
# Delete the tag locally and remotely
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z

# Fix the issue, then re-release
vim Cargo.toml  # or fix whatever failed
git add .
git commit -m "release: vX.Y.Z (retry)"
git tag vX.Y.Z
git push origin main --tags
```

## Post-release Verification

After release completes:

- [ ] All 5 platform binaries present in GitHub Release
- [ ] Installer scripts downloadable
- [ ] `cargo install tl-cli` installs the new version
- [ ] crates.io shows the new version

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| CI not triggered | Tag doesn't match pattern | Use `v0.3.0` not `0.3.0` |
| crates.io publish fails | Token expired/invalid | Regenerate token in GitHub secrets |
| GitHub Release not created | Publish job failed | Fix issue, delete tag, re-tag |

## Design Documentation

For detailed design decisions, see [docs/design/0006-release-workflow.md](docs/design/0006-release-workflow.md).
