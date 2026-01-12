# Mini Design Doc: Automated Release Workflow with cargo-dist

*   **Author:** d2verb
*   **Status:** Approved
*   **Date:** 2026-01-12

## 1. Abstract

Implement an automated release workflow using cargo-dist that builds platform-specific binaries, creates GitHub Releases with attached artifacts, and publishes to crates.io. This eliminates manual release steps and ensures consistent, reproducible releases across all supported platforms.

## 2. Goals & Non-Goals

*   **Goals:**
    *   Automate binary builds for 5 target platforms (Linux x86_64/ARM64, macOS Intel/Apple Silicon, Windows x86_64)
    *   Automatically create GitHub Releases with downloadable binaries on git tag push
    *   Automatically publish to crates.io as part of the release process
    *   Provide shell/PowerShell installer scripts for easy installation

*   **Non-Goals:**
    *   Homebrew tap integration (can be added later)
    *   npm/other package manager publishing
    *   Fully automatic changelog generation (we use AI-assisted with human review instead)
    *   Automatic version bumping
    *   Code signing / notarization (macOS, Windows) - not required for this project
    *   musl target for maximum Linux compatibility (can be added later if needed)

## 3. Context & Problem Statement

Currently, releasing a new version requires:
1. Manual binary builds for each platform (if any)
2. Manual GitHub Release creation
3. Manual crates.io publishing

This process is error-prone, time-consuming, and doesn't scale. Users cannot easily install pre-built binaries for their platform.

## 4. Proposed Design

### 4.1 System Overview

```
Developer                     GitHub Actions                    Outputs
─────────────────────────────────────────────────────────────────────────

 1. Update version           2. Tag (v*) triggers             4. Results
    in Cargo.toml               release.yml
         │                          │                            │
         ▼                          ▼                            ▼
    ┌─────────┐              ┌─────────────┐              ┌──────────────┐
    │ git tag │──────────────│   Plan      │              │GitHub Release│
    │ v0.3.0  │              │   Job       │              │ + Binaries   │
    └─────────┘              └──────┬──────┘              └──────────────┘
                                    │
                          ┌─────────┼─────────┐           ┌──────────────┐
                          ▼         ▼         ▼           │  crates.io   │
                     ┌────────┐┌────────┐┌────────┐       │  tl-cli      │
                     │Linux   ││macOS   ││Windows │       └──────────────┘
                     │x86/arm ││x86/arm ││x86     │
                     └────┬───┘└────┬───┘└────┬───┘       ┌──────────────┐
                          │         │         │           │ Installer    │
                          └─────────┴─────────┘           │ Scripts      │
                     3. Parallel builds                   └──────────────┘
                                    │
                                    ▼
                             ┌─────────────┐
                             │  Host       │◄─── Creates Release
                             │  Job        │     (waits for publish)
                             └──────┬──────┘
                                    │
                                    ▼
                             ┌─────────────┐
                             │  Publish    │◄─── crates.io publish
                             │  Job        │     (runs before host)
                             └─────────────┘
```

### 4.2 Project Configuration (Validated)

The following have been validated for this project:

| Item | Current Value | Notes |
|------|---------------|-------|
| Crate name | `tl-cli` | Package name in Cargo.toml |
| Binary name | `tl` | Defined in `[[bin]]` section |
| Workspace | Single crate | No workspace dependencies |
| Tag pattern | `v*` (e.g., `v0.3.0`) | Must match this pattern to trigger CI |

### 4.3 Detailed Configuration

**Cargo.toml additions:**

```toml
# Optimized build profile for distribution
[profile.dist]
inherits = "release"
lto = "thin"

# cargo-dist configuration (single crate, not workspace)
[package.metadata.dist]
cargo-dist-version = "0.27.0"
ci = "github"
targets = [
  "x86_64-unknown-linux-gnu",
  "aarch64-unknown-linux-gnu",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin",
  "x86_64-pc-windows-msvc",
]
installers = ["shell", "powershell"]
pr-run-mode = "skip"  # Don't run dist on PRs
```

**Notes:**
- Uses `[package.metadata.dist]` because this is a single crate (not a workspace)
- The `cargo-dist-version` field pins the version used in CI, ensuring reproducible builds
- Actual installer filenames will be confirmed via `dist plan` during implementation

**GitHub Secrets required:**

| Secret | Purpose |
|--------|---------|
| `CARGO_REGISTRY_TOKEN` | crates.io API token for publishing |

Note: `GITHUB_TOKEN` is automatically available.

### 4.4 Release Workflow File

`dist init` generates `.github/workflows/release.yml`. We will customize it to ensure proper ordering:

**Required permissions:**
```yaml
permissions:
  contents: write  # Required for creating GitHub Releases
```

**Job execution order:**
1. **plan**: Determines what to build based on Cargo.toml
2. **build-local-artifacts**: Builds binaries for each target platform (parallel)
3. **build-global-artifacts**: Builds installers and checksums
4. **publish-crates**: Publishes to crates.io (runs BEFORE host)
5. **host**: Creates GitHub Release and uploads artifacts (depends on publish-crates success)

**Key customization:** The `host` job will depend on `publish-crates` success to prevent creating a GitHub Release when crates.io publish fails.

**Relationship with CI workflow:**
- Tests, clippy, and fmt checks run in the existing `.github/workflows/ci.yml`
- These checks should pass on `main` before tagging a release
- The release workflow (`release.yml`) focuses only on building and publishing

**Workflow maintenance policy:**
- `dist init` is run only during initial setup
- Subsequent changes are made by directly editing `.github/workflows/release.yml`
- When updating cargo-dist:
  1. Run `dist init` in a scratch branch
  2. Diff the generated workflow against the current one
  3. Manually merge relevant changes, preserving customizations
- Do NOT run `dist init` directly on main as it will overwrite customizations

### 4.5 Generated Artifacts

Per release, the following artifacts are created (exact filenames confirmed via `dist plan`):

| Artifact Type | Description |
|---------------|-------------|
| Linux x86_64 archive (`.tar.gz`) | Linux x86_64 binary |
| Linux ARM64 archive (`.tar.gz`) | Linux ARM64 binary |
| macOS Intel archive (`.tar.gz`) | macOS Intel binary |
| macOS Apple Silicon archive (`.tar.gz`) | macOS Apple Silicon binary |
| Windows x86_64 archive (`.zip`) | Windows x86_64 binary |
| Shell installer script | One-liner installation for Linux/macOS |
| PowerShell installer script | One-liner installation for Windows |

**Note:** Exact artifact filenames follow cargo-dist naming conventions and will be documented after running `dist plan`.

### 4.6 Signing Policy

**No code signing or notarization is implemented.** Rationale:
- This is a personal/open-source project
- Users can verify via checksums
- Can be added later if needed (requires Apple Developer account, Windows certificate)

### 4.7 Release Notes Generation

Release notes are generated using Claude Code and added to GitHub Releases after CI completion.

**File structure:**
```
scripts/
├── update-release-notes.sh    # Main script
└── release-notes-template.md  # Prompt template for Claude Code
```

**Script workflow:**

```
./scripts/update-release-notes.sh v0.3.0
                │
                ▼
    ┌───────────────────────┐
    │ Get previous version  │  ← git describe --tags --abbrev=0 HEAD^
    │ (e.g., v0.2.0)        │
    └───────────┬───────────┘
                │
                ▼
    ┌───────────────────────┐
    │ Generate release notes│
    │ - git log + template  │
    │ - Claude Code --print │
    └───────────┬───────────┘
                │
                ▼
    ┌───────────────────────┐
    │ Open in $EDITOR       │  ← User can review/edit
    └───────────┬───────────┘
                │
                ▼
    ┌───────────────────────┐
    │ Confirm: Continue?    │
    └───────────┬───────────┘
           y/   \n
          /     \
         ▼       ▼
   gh release   Abort &
   edit         cleanup
```

**Template content (`scripts/release-notes-template.md`):**
```markdown
Generate release notes for the given git log.
Format as GitHub Release notes with these sections:
- ## What's Changed
- ### New Features (if any)
- ### Improvements (if any)
- ### Bug Fixes (if any)

Keep it concise and user-focused. Omit internal refactoring details.
```

## 5. Implementation Plan

1.  **Phase 0: Pre-flight Validation**
    *   Validate binary name matches expectations (`tl`)
    *   Validate single-crate structure (no workspace publish ordering issues)
    *   Validate tag pattern (`v*`)
    *   Run `cargo publish --dry-run` to verify publishability
    *   Run `dist plan` to verify configuration

2.  **Phase 1: Setup**
    *   Install cargo-dist locally
    *   Run `dist init` to generate configuration
    *   Configure target platforms and installers
    *   Pin cargo-dist version in workflow

3.  **Phase 2: crates.io Integration**
    *   Add custom publish job for crates.io
    *   Configure job ordering (publish before host)
    *   Add `CARGO_REGISTRY_TOKEN` to GitHub Secrets

4.  **Phase 3: Test Release**
    *   Create a test release (e.g., v0.2.1) to validate workflow
    *   Verify all artifacts are correctly generated
    *   Document actual artifact filenames from the test release
    *   If failed, use recovery flow (delete tag, fix, re-tag)

5.  **Phase 4: Release Notes Script**
    *   Create `scripts/update-release-notes.sh`
    *   Create `scripts/release-notes-template.md`
    *   Test script with a sample release

6.  **Phase 5: Documentation**
    *   Update README with installation instructions
    *   Create RELEASING.md with release process documentation

## 6. Risks & Mitigations

| Risk | Impact | Mitigation Strategy |
| :--- | :--- | :--- |
| CI build failure on specific platform | Medium | Each platform builds independently; failures are isolated |
| crates.io publish failure | High | Host job depends on publish success; release not created if publish fails |
| Token expiration | Low | Use long-lived token; document renewal process |
| Cross-compilation issues (ARM Linux) | Medium | cargo-dist uses `cross` with zig/QEMU; fallback to native runners if needed |
| Tag pattern mismatch | Medium | Document exact pattern (`v*`); CI only triggers on matching tags |
| glibc compatibility (old Linux) | Low | Using gnu target; musl can be added later if compatibility issues arise |
| macOS runners (Intel vs ARM) | Low | cargo-dist handles runner selection; no signing required |
| Workflow regeneration overwrites customizations | Medium | Never re-run `dist init`; manually edit workflow file |

## 7. Testing & Verification

*   **Prerequisites (before tagging):**
    *   CI must be green on `main` (tests, clippy, fmt via `ci.yml`)
    *   This is verified through normal development workflow

*   **Initial Setup / Config Changes Only:**
    *   `cargo publish --dry-run` - Verify crates.io publishability
    *   `dist plan` - Verify release plan is correct
    *   `dist build` - Local build test (optional)
    *   Not needed for routine releases

*   **Post-release Verification:**
    *   All 5 platform binaries present in GitHub Release
    *   Installer scripts downloadable and functional
    *   crates.io shows new version
    *   `cargo install tl-cli` works
    *   Installer script installation works:
        ```bash
        # Linux/macOS (review script before running, or verify checksum)
        curl --proto '=https' --tlsv1.2 -LsSf \
          https://github.com/d2verb/tl/releases/latest/download/tl-installer.sh | sh

        # Windows (PowerShell)
        irm https://github.com/d2verb/tl/releases/latest/download/tl-installer.ps1 | iex
        ```

*   **Success Metrics:**
    *   Release workflow completes in < 20 minutes
    *   All artifacts successfully uploaded
    *   Zero manual intervention required for standard releases

## 8. Alternatives Considered

| Alternative | Why Not Chosen |
|-------------|----------------|
| **Manual releases** | Error-prone, time-consuming, doesn't scale |
| **release-plz** | Focused on changelog/versioning; cargo-dist better for binary distribution |
| **Custom GitHub Actions** | More maintenance burden; cargo-dist is battle-tested |
| **cross for cross-compilation** | cargo-dist integrates this automatically |
| **musl target** | Added complexity; gnu target sufficient for now; can add later |

---

## Appendix A: Release Checklist

### Minimal Release Flow (Routine Releases)

```bash
# 1. Update version, commit, tag, and push
vim Cargo.toml  # Change version = "X.Y.Z"
git add Cargo.toml
git commit -m "release: vX.Y.Z"
git tag vX.Y.Z
git push origin main --tags

# 2. Wait for CI (10-15 min)
# https://github.com/d2verb/tl/actions

# 3. Add release notes (after CI completion)
./scripts/update-release-notes.sh vX.Y.Z
```

**Prerequisites:** CI must be green on `main` (verified through normal development workflow).

### Recovery from Failed Release

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

### Initial Setup / Config Changes Only

Run these only during initial setup or when changing cargo-dist configuration:

```bash
cargo publish --dry-run  # Verify publishability
dist plan                 # Verify release plan
```

## Appendix B: Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| CI not triggered | Tag doesn't match `v*` pattern | Use `v0.3.0` not `0.3.0` |
| crates.io publish fails | Token expired or invalid | Regenerate token and update secret |
| ARM Linux build fails | Cross-compilation toolchain issue | Check cargo-dist logs; may need zig |
| GitHub Release not created | Publish job failed | Fix issue, delete tag, re-tag |
| Workflow outdated after cargo-dist update | `dist init` not run | Diff in scratch branch, manually merge |
| Any release failure | Various | Delete tag, fix, re-tag (see Recovery flow) |

## Appendix C: Future Enhancements

- Homebrew tap integration
- musl target for maximum Linux compatibility
- Code signing for macOS/Windows (if distribution requires)
- Checksum verification in installer scripts
- CHANGELOG.md file generation (in addition to GitHub Release notes)
