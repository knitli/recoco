---
title: Changelog
description: Version history and release notes for ReCoco.
---

All notable changes to ReCoco are documented here. We follow [Semantic Versioning](https://semver.org/) and use [Conventional Commits](https://www.conventionalcommits.org/) for our commit messages.

## [0.2.1] - 2026-01-25

### Features

- Add recoco crate as a wrapper around recoco-core, recoco-utils, and recoco-splitters.

### Miscellaneous Tasks

- **(release)** Bump version to 0.2.1 for new crate structure

## [0.2.0] - 2026-01-25

### Features

- **Fork CocoIndex** - Restructured as Rust-only library
  - Feature-gated all sources, targets, etc to make it lean and tailored
  - Exposed logical API paths publicly
  - Created new README.md
  - Created a rustified homage to the logo from scratch
  - Added starter examples in `examples/`
  - Removed all Python-related packaging, files, references, and all PyO3 links
  - Removed docs/ which were completely Python focused
  - Rewrote CI/CD for Rust-only and to publish to crates.io
- Add output position computation and recursive text chunking
- Add CLAUDE.md for project guidance and enhance README.md with key features and usage instructions
- Update README to better separate differences and similarities between CocoIndex and ReCoco
- **(infra)** Add Knitli CLA and CLI actions
- **Significantly improved feature-gating** - Reducing default library size by ~half from v0.1.0
- **Fully feature-gated `recoco-utils`** - We still need some core dependencies for broader ReCoco operations, but anyone wanting to use just one or two utilities for a project can now do just that
- Update internal dependencies to version 0.2.0 and make several more optional

### Fixed

- Rename recoco_extra_text to recoco_splitters in workflow and source files
- Trait overlap in some tests, corrected the organization name "CocoIndex.io" to just "CocoIndex"
- Update image path and replace CocoIndex link in README
- Updated schemars to 1.2 and updated all API calls
- ReCoco image artifacts
- Syntax errors in Cargo.toml
- Update SPDX license information across multiple files
- Autolink in prog_langs.rs
- **(release)** Dry-runs in release process caused release to fail
- **(ci)** Add permissions to CI workflow
- Update SPDX copyright and contributor information in workflow files
- **(ci)** Corrected syntax error in git cliff command
- Tests and linting issues caused by new feature gates

### Documentation

- Improved crate descriptions
- Update README to clarify capabilities

### Miscellaneous Tasks

- Removed CLI-focused dependencies and unused helper scripts
- Cleaned up all remaining loose ends; crate now compiles with no warnings or errors
- Update CONTRIBUTING.md, SECURITY.md, CLAUDE.md for the fork
- **Implemented REUSE specification compliance** - ReCoco is now REUSE 3.3 compliant
- Add migration guide in prep for v1, more corrections to CocoIndex org name
- Update mise.toml
- Symlink readme to main recoco crate
- **(release)** Update crate version to 0.2.0 to reflect major changes in feature gates

---

## Release Process

ReCoco uses [git-cliff](https://git-cliff.org/) to automatically generate changelog entries from conventional commits. The changelog is maintained in [`CHANGELOG.md`](https://github.com/knitli/recoco/blob/main/CHANGELOG.md) in the repository root.

## Contributing

See our [Contributing Guide](/ReCoco/guides/contributing/) for information on how to submit changes and follow our commit message conventions.
