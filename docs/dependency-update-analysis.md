<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

# Dependency Update Analysis

**Date:** 2026-03-10
**Issue:** #48 - Track dependency updates from upstream (PRs #1710, #1711)
**Status:** Analysis Complete

## Executive Summary

Recoco has **152 dependency updates** available as of March 10, 2026. Of these:
- **133 updates** are compatible with current MSRV (Rust 1.89)
- **19 updates** require MSRV bump to Rust 1.91.1
- All updates are **minor or patch versions** (no major breaking changes in direct dependencies)

## Current Dependency Management

### Automated System
- **Dependabot** configured for weekly checks (`.github/dependabot.yml`)
- Monitors both Cargo dependencies and GitHub Actions
- Auto-labels PRs with `dependencies` + specific ecosystem tags
- Uses conventional commits format (`ci(deps): ...`)

### Recent Activity
Dependabot has created 24 dependency update PRs since January 24, 2026. Recent closed (not merged) PRs:
- PR #34: `azure_core 0.31.0 → 0.33.0`
- PR #33: `serde_with 3.16.1 → 3.17.0`
- PR #32: `chrono 0.4.43 → 0.4.44`
- PR #31: `tree-sitter-language 0.1.6 → 0.1.7`
- PR #30: `rand 0.9.2 → 0.10.0` (⚠️ major version bump with breaking changes)

## Available Updates (Highlights)

### Critical Libraries
```
anyhow:     1.0.100 → 1.0.102
chrono:     0.4.43  → 0.4.44
futures:    0.3.31  → 0.3.32
tokio:      (locked at 1.49.0, likely up to date)
serde:      (locked at 1.0.228, likely up to date)
reqwest:    (locked at 0.12.24)
```

### AWS SDK Updates
Multiple AWS crates have updates available (both MSRV 1.89 and 1.91.1+):
```
aws-config:      1.8.12 → 1.8.13 (1.8.15 available, needs 1.91.1)
aws-sdk-s3:      1.120.0 → 1.122.0 (1.125.0 needs 1.91.1)
aws-sdk-sqs:     1.92.0 → 1.93.0 (1.96.0 needs 1.91.1)
aws-runtime:     1.5.18 → 1.6.0 (1.7.2 needs 1.91.1)
```

### Azure SDK Updates
```
azure_core:      0.31.0 (0.33.0 available per PR #34)
azure_identity:  0.21.0
azure_storage:   0.21.0
```

### Google Cloud Updates
```
google-cloud-aiplatform-v1: 1.5.0 → 1.7.0
google-cloud-api:           1.2.0 → 1.3.0
```

## Recommendations

### Immediate Actions (Low Risk)

1. **Merge safe minor/patch updates**
   - Reopen and merge PRs #32, #33 (chrono, serde_with) after testing
   - Review PR #31 (tree-sitter-language)
   - Consider PR #34 (azure_core) if using Azure features

2. **Batch update remaining patches**
   ```bash
   cargo update --package anyhow
   cargo update --package futures
   # Test thoroughly
   git add Cargo.lock
   git commit -m "chore(deps): update patch versions for anyhow, futures, and others"
   ```

3. **Test all feature combinations**
   ```bash
   cargo test --features full
   cargo test --features all-sources
   cargo test --features all-targets
   cargo test --features all-functions
   ```

### Medium-Term Actions

4. **Evaluate MSRV bump to 1.91.1**
   - Would unlock 19 additional updates
   - Consider after Rust 1.91.1 reaches stable
   - Update `rust-version` in `Cargo.toml` workspace config

5. **Defer major version bumps**
   - Keep PR #30 (`rand 0.9 → 0.10`) closed for now
   - `rand 0.10` has breaking API changes and requires Edition 2024
   - Plan migration when ready to adopt Edition 2024

### Upstream Alignment

6. **Compare with upstream CocoIndex PRs #1710, #1711**
   - Requires access to upstream repository
   - Manual comparison of `Cargo.lock` files:
     ```bash
     # If you have upstream access
     git remote add upstream https://github.com/cocoindex-io/cocoindex.git
     git fetch upstream
     git diff upstream/main:rust/Cargo.lock ./Cargo.lock | grep "^[+-]version"
     ```

## Testing Checklist

Before merging any dependency updates:

- [ ] Run `cargo build --features full` successfully
- [ ] Run `cargo test --features full` with all tests passing
- [ ] Run `cargo clippy --all-features -- -D warnings` with no issues
- [ ] Run `cargo fmt --all -- --check` to ensure formatting
- [ ] Test specific feature combinations:
  - [ ] `--features default` (source-local-file only)
  - [ ] `--features all-sources`
  - [ ] `--features all-targets`
  - [ ] `--features all-functions`
- [ ] Run examples:
  - [ ] `cargo run --example transient --features function-split`
  - [ ] `cargo run --example detect_lang --features function-detect-lang`
- [ ] Check for any new deprecation warnings
- [ ] Verify documentation builds: `cargo doc --no-deps --features full`

## Risk Assessment

**Overall Risk:** 🟢 **Low**

- Most updates are minor/patch versions
- Feature-gated architecture isolates dependency changes
- Dependabot provides detailed changelogs
- Can be done incrementally

**Time Estimate:**
- Review and testing: 2-3 hours
- Implementation: 1 hour (update + commit)
- CI validation: ~15 minutes

## References

- **Issue #48:** This tracking issue
- **Issue #27:** [upstream-sync] Bootstrap (comprehensive upstream tracking)
- **Dependabot config:** `.github/dependabot.yml`
- **Upstream PRs:** #1710, #1711 (not directly accessible)

## Generated Output

Full `cargo update --dry-run` output has been saved for reference. To regenerate:

```bash
cargo update --dry-run > dependency-updates-$(date +%Y-%m-%d).txt
```

## Next Steps

1. Maintainer review this analysis
2. Decide on MSRV policy (stay at 1.89 or bump to 1.91.1)
3. Create a PR with safe updates
4. Test thoroughly with all feature combinations
5. Merge incrementally to isolate any issues
