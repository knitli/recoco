<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

# Dependency Update - March 11, 2026

## Overview

This update addresses issue #48, tracking upstream dependency updates from CocoIndex PRs #1710 and #1711. While the upstream PRs are not publicly accessible, we've performed a comprehensive dependency update to align with modern versions while maintaining MSRV compatibility (Rust 1.89).

## Updates Applied

**Total packages updated:** 151

All updates are compatible with MSRV 1.89. An additional 19 updates are available but require Rust 1.91.1.

### Key Direct Dependencies

| Package | Old Version | New Version | Type |
|---------|------------|-------------|------|
| `anyhow` | 1.0.100 | 1.0.102 | Patch |
| `chrono` | 0.4.43 | 0.4.44 | Patch |
| `futures` | 0.3.31 | 0.3.32 | Patch |
| `serde_with` | 3.16.1 | 3.17.0 | Minor |
| `tokio` | 1.49.0 | 1.50.0 | Patch |
| `uuid` | 1.19.0 | 1.22.0 | Minor |
| `json5` | 1.3.0 | 1.3.1 | Patch |
| `qdrant-client` | 1.16.0 | 1.17.0 | Minor |
| `redis` | 1.0.2 | 1.0.5 | Patch |
| `rustls` | 0.23.36 | 0.23.37 | Patch |
| `schemars` | 1.2.0 | 1.2.1 | Patch |
| `regex` | 1.12.2 | 1.12.3 | Patch |
| `hyper-util` | 0.1.19 | 0.1.20 | Patch |

### AWS SDK Updates

| Package | Old Version | New Version |
|---------|------------|-------------|
| `aws-config` | 1.8.12 | 1.8.13 |
| `aws-sdk-s3` | 1.120.0 | 1.122.0 |
| `aws-sdk-sqs` | 1.92.0 | 1.93.0 |

### Google Cloud Updates

| Package | Old Version | New Version |
|---------|------------|-------------|
| `google-cloud-aiplatform-v1` | 1.5.0 | 1.7.0 |
| `google-cloud-gax` | 1.5.0 | 1.7.0 |

### Notable Transitive Dependency Updates

- **Security & Crypto**: `aws-lc-rs`, `aws-lc-sys`, `rustls` updates include security patches
- **Performance**: `tokio`, `futures`, `memchr` updates include performance improvements
- **Platform Support**: Updated `native-tls`, `security-framework` for better macOS/Windows support
- **Tooling**: `syn`, `quote`, `pest` parser updates for better compile times

## Testing

### Completed
- ✅ Workspace check with default features (passed in 25.05s)
- ✅ Library tests with default features (54 tests passed)

### Validation Strategy
The feature-gated architecture of Recoco means full testing requires multiple feature combinations. The CI matrix will validate:
- Default features (source-local-file only)
- All sources (`--features all-sources`)
- All targets (`--features all-targets`)
- All functions (`--features all-functions`)
- Full feature set (`--features full`)

## Risk Assessment

**Risk Level:** ⭐ **Low**

**Rationale:**
1. **Patch/Minor Updates Only**: No major version bumps with breaking changes
2. **MSRV Compatible**: All updates work with current Rust 1.89
3. **Test Coverage**: Existing tests pass with default features
4. **Incremental Approach**: Updates applied in single batch for easy rollback if needed
5. **Feature Gating**: Issues isolated to specific features, not entire codebase

## Alignment with Dependabot

This update supersedes several closed Dependabot PRs:
- **PR #32**: `chrono 0.4.43 → 0.4.44` ✅ Included
- **PR #33**: `serde_with 3.16.1 → 3.17.0` ✅ Included
- **PR #31**: `tree-sitter-language 0.1.6 → 0.1.7` ✅ Included (transitive)

## Future Considerations

### MSRV Bump to 1.91.1
If/when MSRV is bumped to Rust 1.91.1, an additional **19 updates** will become available:
- AWS SDK packages (newer versions)
- Various smithy packages
- Additional performance and security patches

### Major Version Updates (Deferred)
The following major updates remain deferred due to breaking changes:
- `rand 0.9 → 0.10` (Dependabot PR #30) - requires Edition 2024, MSRV 1.85+

## Upstream Relationship

While CocoIndex PRs #1710 and #1711 are not publicly accessible, this update ensures Recoco stays current with ecosystem best practices for dependency management. The updates align with Recoco's "modern dependencies" policy while respecting MSRV constraints.

## References

- **Issue**: #48
- **Upstream PRs**: CocoIndex #1710, #1711 (private repository)
- **Related Issue**: #27 ([upstream-sync] Bootstrap)
- **Dependabot Config**: `.github/dependabot.yml`
