## Summary

Adopt upstream CocoIndex PR #1704 to make LMDB `max_dbs` and `map_size` parameters configurable via the `Settings` struct.

## Upstream Reference

- **PR:** [cocoindex-io/cocoindex#1704](https://github.com/cocoindex-io/cocoindex/pull/1704)
- **Type:** Architectural change / Configurability improvement
- **Difficulty:** Easy
- **Recommendation:** Adopt

## Status

🚫 **BLOCKED** — LMDB is not yet integrated into Recoco. This issue tracks the configuration aspect only; LMDB integration itself needs a separate tracking issue.

## Context

### What are these parameters?

**`max_dbs`** — Maximum number of named databases in the LMDB environment
- Must be set when opening the environment
- Cannot be changed after creation
- Typical values: 10-100

**`map_size`** — Size of the memory-mapped file
- LMDB doesn't auto-grow for performance reasons
- Must be set upfront (though can be increased later)
- Should be multiple of OS page size
- Typical values: 1GB - 10GB

### Why make them configurable?

Hardcoded values cause issues:
- Small `map_size` → users hit "Environment mapsize limit reached" errors with large datasets
- Small `max_dbs` → users hit "Environment maxdbs limit reached" with complex flows
- Overprovisioning wastes resources in constrained environments

## Prerequisites

Before this issue can be implemented:

- [ ] LMDB integration into Recoco core (tracking issue: **TBD**)
- [ ] Determine LMDB environment initialization location (`lib_context.rs`, new `engine/` module, etc.)
- [ ] Choose Rust LMDB crate (`heed` recommended, or `lmdb-rs`)

## Implementation (Once Unblocked)

### 1. Add LMDB settings
```rust
// In lib_context.rs or dedicated settings module
pub struct LmdbSettings {
    pub max_dbs: u32,        // Default: 50
    pub map_size: usize,     // Default: 1GB (1024 * 1024 * 1024)
}

pub struct Settings {
    // ... existing fields
    pub lmdb: LmdbSettings,
}
```

### 2. Wire to environment setup
```rust
// In execution layer where LMDB env is created
let env = unsafe {
    EnvOpenOptions::new()
        .map_size(settings.lmdb.map_size)
        .max_dbs(settings.lmdb.max_dbs)
        .open(path)?
};
```

### 3. Documentation
- Document when users need to tune these parameters
- Explain limitations (map_size should be adequate from start, max_dbs immutable)
- Provide sizing guidance based on data volume and flow complexity

### 4. Tests
- Test with custom LMDB settings
- Verify sensible defaults work

**Estimated effort once unblocked:** 4-8 hours
**Risk level:** Low (pure configuration exposure)

## Related Issues

**Part of v1-parity roadmap:**
- Phase 1 tracking: [#29](https://github.com/knitli/recoco/issues/29)
- Bootstrap scan: [#27](https://github.com/knitli/recoco/issues/27)

**Other LMDB-dependent issues (also blocked):**
- [#35](https://github.com/knitli/recoco/issues/35) — Batch LMDB write transactions (depends on this for environment setup)
- [#36](https://github.com/knitli/recoco/issues/36) — Track destructive target changes (uses LMDB state layer)
- [#37](https://github.com/knitli/recoco/issues/37) — Memoization bypass fix (uses LMDB engine)
- [#57-60](https://github.com/knitli/recoco/issues/57) — Phase 4 core engine changes (all use LMDB)

**Architectural foundation:**
- [#53](https://github.com/knitli/recoco/issues/53) — Dedicated DB schema for tracking tables (may affect LMDB integration approach)

## Next Steps

1. Create LMDB integration tracking issue (or identify existing one)
2. Complete LMDB foundational work
3. Return to this issue for configuration exposure
4. Consider combining with [#35](https://github.com/knitli/recoco/issues/35) for implementation efficiency
