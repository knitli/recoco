<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

# Dependency Update Analysis for Issue #48

**Date:** 2026-03-11
**Issue:** [#48 - Upstream Dependency Updates](https://github.com/knitli/recoco/issues/48)
**Upstream PRs:** CocoIndex #1710, #1711 (not publicly accessible)

## Executive Summary

This document analyzes available dependency updates for Recoco in relation to upstream CocoIndex dependency updates. Since the upstream PRs are not publicly accessible, this analysis is based on the current state of Recoco's dependencies and available updates from crates.io.

### Key Findings

- **151 packages** have updates available, all compatible with Rust 1.89 MSRV
- **19 additional packages** could be updated with MSRV bump to Rust 1.91.1
- All updates are **patch or minor versions** (no major breaking changes in direct deps)
- All 20 recent **Dependabot PRs (#5-34) were closed** without merging
- Updates are **safe to apply incrementally**

## Available Updates (Rust 1.89 Compatible)

### High-Priority Direct Dependencies

These are dependencies explicitly declared in `Cargo.toml` that have updates available:

| Dependency | Current | Available | Type | Priority |
|------------|---------|-----------|------|----------|
| `anyhow` | 1.0.100 | 1.0.102 | Patch | High |
| `chrono` | 0.4.43 | 0.4.44 | Patch | High |
| `futures` | 0.3.31 | 0.3.32 | Patch | High |
| `tokio` | 1.49.0 | 1.50.0 | Minor | High |
| `uuid` | 1.19.0 | 1.22.0 | Minor | High |
| `serde_with` | 3.16.1 | 3.17.0 | Minor | Medium |
| `redis` | 1.0.2 | 1.0.5 | Patch | Medium |
| `qdrant-client` | 1.16.0 | 1.17.0 | Minor | Medium |
| `regex` | 1.12.2 | 1.12.3 | Patch | Medium |
| `rustls` | 0.23.36 | 0.23.37 | Patch | Medium |
| `json5` | 1.3.0 | 1.3.1 | Patch | Low |

### AWS SDK Updates

| Dependency | Current | Available (1.89) | Available (1.91.1) |
|------------|---------|------------------|-------------------|
| `aws-config` | 1.8.12 | 1.8.13 | 1.8.15 |
| `aws-sdk-s3` | 1.120.0 | 1.122.0 | 1.125.0 |
| `aws-sdk-sqs` | 1.92.0 | 1.93.0 | 1.96.0 |

### Google Cloud SDK Updates

| Dependency | Current | Available |
|------------|---------|-----------|
| `google-cloud-aiplatform-v1` | 1.5.0 | 1.7.0 |
| `google-cloud-gax` | 1.5.0 | 1.7.0 |

### Tree-sitter Updates

| Dependency | Current | Available |
|------------|---------|-----------|
| `tree-sitter-language` | 0.1.6 | 0.1.7 |
| `tree-sitter-md` | 0.5.2 | 0.5.3 |

## Closed Dependabot PRs Analysis

Recent Dependabot PRs that match available updates:

- **PR #32**: `chrono 0.4.43 → 0.4.44` ✅ Safe patch update
- **PR #33**: `serde_with 3.16.1 → 3.17.0` ✅ Safe minor update
- **PR #31**: `tree-sitter-language 0.1.6 → 0.1.7` ✅ Safe patch update
- **PR #34**: `azure_core 0.31.0 → 0.33.0` ⚠️ Minor version jump, needs testing
- **PR #30**: `rand 0.9 → 0.10` ❌ Major version, breaking changes, defer

## Recommendations

### Immediate Actions (Low Risk)

1. **Apply all patch-level updates** for direct dependencies:
   ```bash
   # Update Cargo.toml with newer patch versions
   # Then run cargo update
   ```

2. **Test with all feature combinations** after updates:
   ```bash
   cargo test --features full
   cargo test --features all-sources
   cargo test --features all-targets
   cargo test --features all-functions
   cargo test  # default features
   ```

3. **Monitor for regressions** in:
   - AWS S3 operations (source-s3, target interactions)
   - Google Cloud AI operations (function-extract-llm, function-embed)
   - PostgreSQL operations (source-postgres, target-postgres)
   - Vector database operations (target-qdrant)

### Deferred Actions

- **Major version bumps** (e.g., `rand 0.9 → 0.10`): Defer until separate analysis
- **MSRV bump to 1.91.1**: Defer until broader discussion
- **Azure updates**: Test carefully due to version jump

## Testing Checklist

After applying updates:

- [ ] `cargo build --features full` succeeds
- [ ] `cargo test --features full` passes
- [ ] `cargo test --features all-sources` passes
- [ ] `cargo test --features all-targets` passes
- [ ] `cargo test --features all-functions` passes
- [ ] `cargo test` (default features) passes
- [ ] `cargo clippy --all-features` has no warnings
- [ ] `cargo fmt --all --check` passes
- [ ] Examples run successfully:
  - [ ] `cargo run -p recoco --example transient --features function-split`
  - [ ] `cargo run -p recoco --example custom_op`
  - [ ] `cargo run -p recoco --example detect_lang --features function-detect-lang`

## Implementation Strategy

### Phase 1: Core Dependencies (Lowest Risk)
Update these in `Cargo.toml`:
```toml
anyhow = "1.0.102"
chrono = "0.4.44"
futures = "0.3.32"
tokio = "1.50.0"
uuid = "1.22.0"
regex = "1.12.3"
rustls = "0.23.37"
```

### Phase 2: SDK Dependencies (Medium Risk)
Update these after Phase 1 passes tests:
```toml
aws-config = "1.8.13"
aws-sdk-s3 = "1.122.0"
aws-sdk-sqs = "1.93.0"
google-cloud-aiplatform-v1 = "1.7.0"
google-cloud-gax = "1.7.0"
qdrant-client = "1.17.0"
redis = "1.0.5"
```

### Phase 3: Utility Dependencies (Low Risk)
Update these last:
```toml
serde_with = "3.17.0"
json5 = "1.3.1"
```

## Risk Assessment

| Risk Level | Count | Examples |
|------------|-------|----------|
| **Low** | 133 | Patch updates, security fixes |
| **Medium** | 18 | Minor version bumps in SDKs |
| **High** | 0 | No high-risk updates in this batch |
| **Deferred** | 19 | Require MSRV 1.91.1 |

## Time Estimates

- **Phase 1 Implementation**: 30 minutes
- **Phase 1 Testing**: 15 minutes (CI time)
- **Phase 2 Implementation**: 30 minutes
- **Phase 2 Testing**: 15 minutes (CI time)
- **Phase 3 Implementation**: 15 minutes
- **Phase 3 Testing**: 15 minutes (CI time)
- **Total**: ~2 hours (including testing)

## Notes on Upstream Comparison

Since upstream CocoIndex PRs #1710 and #1711 are not publicly accessible, this analysis is based on:
1. Available updates from crates.io
2. Recoco's current MSRV policy (Rust 1.89)
3. Semantic versioning best practices
4. Recent Dependabot PR history

If you have access to the upstream repository, you can compare lockfiles:
```bash
git remote add upstream https://github.com/cocoindex-io/cocoindex.git
git fetch upstream
git diff upstream/main:rust/Cargo.lock ./Cargo.lock | grep "^[+-]version"
```

## Conclusion

Recoco has 151 safe dependency updates available. All are compatible with the current MSRV (Rust 1.89) and represent routine maintenance. The recommended approach is to apply updates in phases, testing thoroughly between each phase, starting with the lowest-risk core dependencies.
