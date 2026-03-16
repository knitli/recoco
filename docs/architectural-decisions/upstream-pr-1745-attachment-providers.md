<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Claude (AI Assistant)

SPDX-License-Identifier: Apache-2.0
-->

# Architectural Decision: Upstream PR #1745 - Attachment Providers

**Date:** 2026-03-16
**Status:** Not Adopted - Architectural Divergence
**Issue:** #100
**Upstream PR:** https://github.com/cocoindex-io/cocoindex/pull/1745

## Context

Upstream CocoIndex PR #1745 introduced "attachment providers" for target states, adding an `attachment()` method to the `TargetHandler` trait in their engine layer (`rust/core/src/engine/target_state.rs`). This allows target handlers to expose auxiliary child target states (like database indexes, triggers, or SQL commands) alongside their primary data.

## Decision

**Recoco will NOT adopt this change.** Recoco already has a complete, working attachment provider system using a different architectural pattern that is more suitable for a pure-Rust library.

## Architectural Comparison

### Upstream (CocoIndex)

**Layer:** Engine/Runtime (`rust/core/src/engine/target_state.rs`)

**Pattern:** Handler-based (runtime resolution)

```rust
// Upstream's approach
pub trait TargetHandler<Prof: EngineProfile> {
    fn reconcile(...) -> Result<...>;

    // NEW in PR #1745
    fn attachment(&self, att_type: &str) -> Result<Option<Prof::TargetHdl>> {
        Ok(None)  // default: no attachments
    }
}

// Usage:
let handler: TargetHandler = ...;
let index_handler = handler.attachment("vector_index")?;
```

**Characteristics:**
- Runtime attachment handler resolution via method call
- Requires `EngineProfile` trait (for Python/Rust abstraction)
- Mixes handler logic with attachment provider resolution
- Designed to support both Python and Rust APIs

### Recoco

**Layer:** Factory/Operations (`crates/recoco-core/src/ops/`)

**Pattern:** Factory-based (registration-time resolution)

```rust
// Recoco's approach
pub trait TargetAttachmentFactory: Send + Sync {
    fn normalize_setup_key(&self, key: &serde_json::Value) -> Result<serde_json::Value>;
    fn get_state(...) -> Result<TargetAttachmentState>;
    async fn diff_setup_states(...) -> Result<Option<Box<dyn AttachmentSetupChange>>>;
}

// Implementation:
struct SqlCommandFactory;
impl TargetSpecificAttachmentFactoryBase for SqlCommandFactory {
    type TargetKey = TableId;
    type Spec = SqlCommandSpec;
    // ... type definitions and methods
}

// Registration:
SqlCommandFactory.register(registry)?;

// Usage:
let factory = get_attachment_factory("PostgresSqlCommand")?;
```

**Characteristics:**
- Compile-time factory registration in `ExecutorFactoryRegistry`
- Clean separation between factory setup and runtime execution
- No `EngineProfile` abstraction needed (pure Rust)
- Better type safety via associated types

## Rationale

### 1. Functional Equivalence

Both systems provide the same capabilities:
- Define attachment specifications (e.g., `SqlCommandSpec`)
- Track attachment state changes
- Execute setup/teardown operations
- Support multiple attachment types per target

**Example:** Recoco's `SqlCommandFactory` (lines 1000-1116 in `ops/targets/postgres.rs`) provides SQL command attachments identical to upstream's implementation.

### 2. Architectural Fit

Recoco's factory-based pattern aligns better with its design philosophy:

- **Pure Rust Library:** No Python bindings layer needed, so no `EngineProfile` abstraction
- **Feature-Gated Dependencies:** Factories register conditionally at compile time based on enabled features
- **Separation of Concerns:** Factory registration (setup-time) is separate from runtime execution

Upstream's handler-based pattern is designed for their dual Python/Rust API architecture, which recoco doesn't have.

### 3. Existing Implementation

Recoco already has a complete attachment system:

**Core Infrastructure:**
- `ops/interface.rs:350-371` - `TargetAttachmentFactory` trait
- `ops/factory_bases.rs:781-822` - `TargetSpecificAttachmentFactoryBase` trait
- `ops/registry.rs:24-25, 80-92` - Attachment factory registry
- `ops/registration.rs:119, 147` - Factory getter functions

**Working Implementation:**
- `ops/targets/postgres.rs:1000-1116` - `SqlCommandFactory`
- Registered in `postgres::register()` (line 1114)
- Fully tested and functional

### 4. Cost vs. Benefit

**Porting upstream's approach would require:**
1. Create `crates/recoco-core/src/engine/` directory
2. Implement `engine/target_state.rs` with `TargetHandler` trait
3. Implement `engine/profile.rs` with `EngineProfile` trait
4. Refactor all target connectors to implement `TargetHandler.attachment()`
5. Modify runtime evaluation to use engine-layer providers

**Estimated effort:** 2-3 weeks of full-time work
**Benefit:** None - we'd replace a working, simpler system with a more complex one

## Consequences

### Positive

- Maintain simpler, more Rust-idiomatic architecture
- Avoid unnecessary abstraction layers (no `EngineProfile` needed)
- Keep factory-based pattern consistent across all operation types (sources, functions, targets)
- No disruption to existing working code

### Negative

- Architectural divergence from upstream increases
- Future upstream changes to attachment system may be harder to evaluate
- Different patterns between upstream Python docs and recoco Rust docs

### Mitigation

- **Document the divergence:** This file serves as a record
- **Cross-reference existing impl:** Point users to `ops/targets/postgres.rs` example
- **Case-by-case evaluation:** Future attachment-related PRs will be assessed individually
- **Update fork documentation:** Note architectural differences in recoco's CLAUDE.md

## References

- **Upstream PR:** https://github.com/cocoindex-io/cocoindex/pull/1745
- **Upstream Release:** v1.0.0-alpha27
- **Recoco Issue:** #100
- **Related Recoco Files:**
  - `crates/recoco-core/src/ops/interface.rs` (TargetAttachmentFactory)
  - `crates/recoco-core/src/ops/factory_bases.rs` (TargetSpecificAttachmentFactoryBase)
  - `crates/recoco-core/src/ops/targets/postgres.rs` (SqlCommandFactory example)

## Future Considerations

If upstream makes **breaking changes** to attachment behavior that affect core functionality:
1. Re-evaluate whether to adopt upstream's pattern
2. Consider creating an adapter layer to map between patterns
3. Assess if the breaking change reveals limitations in recoco's factory approach

For now, the factory-based pattern is working well and should be maintained.
