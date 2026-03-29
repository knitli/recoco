<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)

SPDX-License-Identifier: Apache-2.0
-->
# Changelog


We document all important changes to Recoco here.

## [0.2.2] - 2026-03-18

### Features

- Add docs site and content
- *(site)* Updated theming to align with knitli styling
- Launched docs site
- Scheduled Claude workflow to monitor upstream cocoindex-io/cocoindex changes (#22)
- *(CI)* Add claude action for managing upstream change syncs.
- Workflow to create v1 upstream parity tracking issue + sub-issues (#28)
- Adopt dedicated PostgreSQL schema for Recoco internal tracking tables (#94)
- *(targets)* Add target-ladybug (Kuzu successor) (#95)
- Add filesystem watch support to local_file source (#97)
- Add cross-language benchmark suite (Recoco vs CocoIndex/Python)

### Fixed

- Added needed bindings to wrangler.jsonc
- Site deployment
- *(ci)* Switch to PAT for upstream sync agent
- *(ci)* Add mcp config to upstream agent params
- *(ci)* Add log outputs to upstream agent action
- Resolve `time` crate compile error under `source-azure` feature (#23)
- *(ci)* Added instructions for first run of upstream assistant after it repeatedly timed out, likely because of too broad a scope.
- *(upstream-agent)* Adding debugging to upstream agent
- *(upstream-agent)* Adding debugging to upstream agent
- *(upstream-sync)* Multi-line prompt env var and MCP config token expansion (#24)
- Resolve upstream-sync.yml workflow failures (#25)
- *(ci)* Resolve upstream-sync.yml Claude Code startup failure and log step fragility (#26)
- *(upstream-sync)* Streamline upstream data gathering and environment setup
- *(upstream-sync)* Improve output handling for session execution results
- *(upstream)* More debugging of upstream agent
- *(upstream)* Moving to github-scripts to handle weird auth issues with gh cli
- Fix upstream
- *(templates)* Move SPDX headers outside front matter in feature-request template
- *(docs)* Missing slash redirect failures
- Ensure execution plan initialized after target setup (#49) (#92)
- Tracking table setup must occur after all target setups (#93)
- Resilient LocalFile list() + notify feature gate + closure lifetime HRTB fix (#103)

### Refactor

- Complete documentation site overhaul: Fix critical issues and implement all recommended improvements (#18)
- Reorganize recoco-splitters module structure to match upstream (#107)

### Documentation

- Slim README, add configuration reference, eliminate passive voice (#55)
- *(site)* Update logo and wrangler settings
- *(readme)* Fix broken links
- Comprehensive v1.0.0 Rust API design plan
- Add query handlers and graph mappings to v1 API migration phases
- Incorporate approved upstream #1667 design into v1 API plan

### Performance

- Bounded concurrency for component setup I/O operations (#84)

### Testing

- Harden `RangeValue::extract_str` test with derived indices and UTF-8 coverage (#85)

### Miscellaneous Tasks

- Update dependencies
- Standardized name as Recoco vice ReCoco across codebase
- Return to reuse compliance
- Optimize github actions workflow to reduce redundant test runs (#45)
- *(cleaning)* Clean and update files
- *(docs)* Update image
- Sync tree-sitter dependency bumps from upstream (#61)
- *(deps)* Bump dorny/paths-filter from 3 to 4 (#86)
- Maintain assets and config for recoco
- Bump to v0.2.2

<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)

SPDX-License-Identifier: Apache-2.0
-->
# Changelog


We document all important changes to Recoco here.

## [0.2.1] - 2026-01-25

### Features

- Add recoco crate as a wrapper around recoco-core, recoco-utils, and recoco-splitters.

### Miscellaneous Tasks

- *(release)* Bump version to 0.2.1 for new crate structure

## [0.2.0] - 2026-01-25

### Features

- Feat: Fork CocoIndex and restructured as Rust-only library. \
  - Feature-gated all sources, targets, etc to make it lean and tailored \
  - Exposed logical API paths publicly \
  - Created new README.md \
  - Created a rustified homage to the logo from scratch \
  - Added some starter examples in `examples/` \
  - Removed all python-related packaging, files, references, and all pyO3 links  \
  - Removed docs/ which were completely python focused \
  - Rewrote CI/CD for rust-only and to publish to crates.io \
- Add output position computation and recursive text chunking
- Add CLAUDE.md for project guidance and enhance README.md with key features and usage instructions
- Update README to better separate differences and similarities between CocoIndex and Recoco
- *(infra)* Add Knitli CLA and CLI actions.
- Significantly improved feature-gating, reducing default library size by ~half from v0.1.0
- Fully feature-gated `recoco-utils`. We still need some core dependencies for broader Recoco operations, but anyone wanting to use just one or two utilities for a project can now do just that.
- Update internal dependencies to version 0.2.0 and make several more optional

### Fixed

- Rename recoco_extra_text to recoco_splitters in workflow and source files
- Trait overlap in some tests, corrected the organization name "CocoIndex.io" to just "CocoIndex"
- Update image path and replace CocoIndex link in README
- Updated schemars to 1.2 and updated all api calls
- Updated schemars to 1.2 and updated all api calls
- Recoco image artifacts
- Syntax error in Cargo.toml
- Syntax error in Cargo.toml
- Update SPDX license information across multiple files
- Autolink in prog_langs.rs
- *(release)* Dry-runs in release process caused release to fail
- *(ci)* Add permissions to CI workflow
- Update SPDX copyright and contributor information in workflow files
- *(ci)* Corrected syntax error in git cliff command
- Tests and linting issues caused by new feature gates

### Documentation

- Improved crate descriptions
- Update README to clarify capabilities

### Miscellaneous Tasks

- Removed CLI-focused dependencies and unused helper scripts
- Cleaned up all remaining loose ends; crate now compiles with no warnings or errors
- Update CONTRIBUTING.md, SECURITY.md, CLAUDE.md for the fork.
- Implemented reuse specification compliance. Recoco is now Reuse 3.3 compliant
- Add migration guide in prep for v1, more corrections to CocoIndex org name
- Update mise.toml
- Symlink readme to main recoco crate
- Symlink readme to main recoco crate
- *(release)* Update crate version to 0.2.0 to reflect major changes in feature gates.

<!-- generated by git-cliff -->
