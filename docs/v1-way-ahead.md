# Recoco v1.0.0: Way Ahead

## 1. Upstream Reality Check

### What the upstream is doing

CocoIndex's v1 is a complete rewrite. The engine moved from a declarative FlowBuilder/DAG model to a **persistent-state-driven model** built around an `App` -> `Component` tree with LMDB-backed state. This engine architecture is shared: Recoco's `v1.0.0` branch already has it (in `crates/core/src/engine/`).

The critical divergence is at the **API and integration layer**. Upstream has committed fully to Python as the user-facing language. Their v1 API uses Python decorators (`@coco.function`, `@coco.lifespan`), Python async/await, and PyO3 to bridge into the Rust engine. All sources, targets, and functions are being rebuilt as Python-callable operations. There is no Rust SDK, no plan for one, and no `crates.io` publication.

Previous documents referenced upstream issue #1372 (a founder saying they "plan to support Rust natively") and issue #1667 (a contributor's Rust API proposal that received feedback from a team member). Neither represents a commitment. The founder's statement was aspirational. The team member's feedback on #1667 was constructive engagement with a contributor's ideas -- not an endorsement or roadmap commitment. The upstream's v1 work has since made the direction unambiguous: Python API, Rust engine internals only.

### What this means for Recoco

Recoco and CocoIndex now have clearly different identities:

- **CocoIndex**: Python-first data processing framework with a Rust engine under the hood. Users write Python.
- **Recoco**: Pure Rust incremental data processing library. Users write Rust.

The shared engine layer (`EngineProfile`, `Component`, `App`, LMDB state, memoization, target reconciliation) remains a common foundation. But everything above it -- the programming model, API surface, operations, and integrations -- is where Recoco must chart its own course.

## 2. Current State

### Recoco `main` branch

The main branch has the **pre-v1 architecture**: FlowBuilder, DataSlice DAG, PostgreSQL state tracking, global `init_lib_context()` singleton. It includes a full set of feature-gated operations:

| Category | Operations |
|----------|-----------|
| Sources | `LocalFile`, `Postgres`, `S3`, `Azure`, `GDrive` |
| Functions | `SplitBySeparators`, `SplitRecursively`, `SentenceTransformerEmbed`, `ExtractByLlm`, `ParseJson`, `DetectProgrammingLanguage` |
| Targets | `Postgres`, `Qdrant`, `Neo4j`, `Kuzu` |

These operations are the main branch's primary asset. The FlowBuilder API that wraps them has known ergonomic issues (JSON-based config, string-based op names, verbose value wrapping, two-trait custom op pattern).

### Recoco `v1.0.0` branch

The v1.0.0 branch has the **new engine** (`crates/core/src/engine/`):

- `App<Prof>`, `Component<Prof>`, `ComponentProcessor` trait
- `Environment<Prof>`, `EnvironmentSettings`
- LMDB-backed memoization, execution, and state tracking
- `EngineProfile` trait (generic, currently needs a concrete Rust implementation)
- Target state reconciliation, transaction batching, progress reporting
- `inspect` module for LMDB database inspection

It has **no operations** -- no sources, no functions, no targets. No user-facing API layer. No concrete `EngineProfile` implementation.

### The gap

The engine is ready. The operations exist but are wired for the old architecture. There is no bridge between them and no user-facing API for the new engine.

## 3. Recommended Direction

### Programming Model

Recoco should adopt a **persistent-state-driven model** conceptually aligned with what upstream describes, but expressed through Rust idioms:

- **Transformations as plain Rust functions** -- no DSL, no graph builder, no string-based dispatch. Users write `async fn` functions that take typed inputs and return typed outputs.
- **Memoization and incrementality via proc macros** -- `#[recoco::function(memo)]` generates `ComponentProcessor` impls, fingerprint computation, and cache key management. The macro is optional; manual `ComponentProcessor` impl is always available.
- **Explicit context, no global state** -- `Environment` holds LMDB state and shared resources. `App` is the execution unit. Resources are accessed via typed `ContextKey<T>` (not type-erased lookup), supporting multiple instances of the same type.
- **Sources as iterators, functions as methods, targets as declarative mounts** -- operations are regular Rust types. `local_file::walk_dir()` returns an iterator. `RecursiveSplitter::split()` is a direct method call. `PgTarget::mount_table()` + `declare_row()` for targets.

This model preserves the upstream's core value proposition (incremental processing, lineage, memoization, fault tolerance) while being natural Rust rather than a Rust translation of Python patterns.

### Core API Shape

```rust
// Environment + App
let env = Environment::open(EnvironmentSettings {
    db_path: ".recoco".into(),
    ..Default::default()
})?;
env.provide(&EMBEDDER, MyEmbedder::new("model")?);

let app = App::new("pipeline", &env)?;
let handle = app.update(
    pipeline_main.with_args(source_dir),
    AppUpdateOptions::default(),
)?;
let stats = handle.await?;

// Processing functions
#[recoco::function(memo)]
async fn process_file(ctx: &Ctx, file: FileEntry, table: &TableTarget) -> Result<()> {
    let text = file.read_text().await?;
    let chunks = RecursiveSplitter::default().split(&text, SplitOptions::default());
    let embedder = ctx.use_resource(&EMBEDDER);
    for chunk in &chunks {
        let embedding = embedder.embed(&chunk.text).await?;
        table.declare_row(/* ... */)?;
    }
    Ok(())
}

// Parallel processing with incremental tracking
ctx.mount_each(process_file, files, &table).await?;
```

### Key Design Decisions

**`ContextKey<T>` over `ctx.get::<T>()`**: Named, typed keys allow multiple instances of the same type (e.g., two database connections) and make resource dependencies explicit and discoverable.

**`Environment` + `App` separation**: Matches the engine's actual architecture. Multiple apps can share one LMDB environment. Resources are environment-scoped.

**Declarative target pattern (`declare_row`/`declare_file`)**: The engine handles insert/update/delete reconciliation against LMDB state. This is richer than simple file writes and supports vector DBs, graph DBs, and relational targets.

**Feature-gated everything**: Maintain the modular compilation model. Every source, function, and target is independently optional.

**Transient mode**: One-shot pipeline execution without LMDB persistence. Functions are also directly callable outside the engine context.

## 4. Implementation Phases

### Phase 1: Foundation (concrete profile + core API)

1. Implement `RecocoProfile` -- the concrete `EngineProfile` with Rust-native associated types
2. Build `Ctx` type with `use_resource()` and `ContextKey<T>`
3. Build `Environment` and `App` convenience wrappers
4. Implement `mount()`, `mount_each()`, `map()` helpers on `Ctx`

This phase is the hardest and most consequential. Every decision here constrains everything downstream.

### Phase 2: Proc macros (`recoco-macros` crate)

1. `#[recoco::function]` -- generates `ComponentProcessor` impl, emits code hash constant
2. `#[recoco::function(memo)]` -- adds fingerprint computation and cache wrapping
3. `#[recoco::function(batching)]` and `#[recoco::function(memo, batching)]`
4. `version = N` for manual cache invalidation

### Phase 3: Port operations

Rewrite main-branch operations as standalone Rust types (not registry-based):

- **Sources**: `local_file::walk_dir()` returning `impl Iterator<Item = FileEntry>`, `postgres::query()`, etc.
- **Functions**: `RecursiveSplitter::split()`, `SentenceTransformerEmbed::embed()`, `ExtractByLlm::extract()`, etc.
- **Targets**: `PgTarget::mount_table()` returning `TableTarget`, `QdrantTarget::mount_collection()`, etc.

### Phase 4: Transient/direct mode

1. One-shot pipeline execution without LMDB
2. Standalone function calls without engine context
3. Backward-compatible usage of functions as plain Rust types

### Phase 5: Query handlers and advanced integrations

1. Query handler registration on apps (search endpoints)
2. Graph DB mapping types for Neo4j/Kuzu
3. Reusable transform compositions

## 5. Areas Requiring Deep Planning

The following areas are not "just implement it" tasks. Each involves design decisions with significant downstream consequences and warrants dedicated design work before code.

### 5.1 `RecocoProfile` concrete types

The `EngineProfile` trait defines 8 associated types. Upstream fills these with PyO3-bridged Python types. Recoco needs Rust-native equivalents:

- `HostRuntimeCtx` -- resource registry design (thread-safe, typed, keyed)
- `ComponentProc` -- how user functions become components (trait object? enum? generic?)
- `FunctionData` -- serializable function return type (must work with LMDB persistence)
- `TargetHdl`, `TargetActionSink`, `TargetStateValue` -- target reconciliation abstractions

Wrong choices here ripple through the entire API. This needs a focused design spike examining how the engine actually uses each associated type in `app.rs`, `component.rs`, `execution.rs`, and `target_state.rs`.

### 5.2 Proc macro design

The `#[recoco::function]` macro must:

- Parse function signatures to extract `&Ctx` and other parameters
- Hash the function body token stream for cache invalidation
- Generate `ComponentProcessor` impls with correct fingerprint logic
- Handle the `memo` / `batching` / `memo + batching` matrix correctly
- Support both sync and async functions
- Produce useful error messages when misused

This is a separate crate (`recoco-macros`) that needs its own test harness, including compile-fail tests for error quality.

### 5.3 Operation porting strategy

Main-branch operations are deeply integrated with the old architecture (factory/executor traits, `OpArgsResolver`, `FlowInstanceContext`, string-based registry). Porting them to standalone types requires:

- Deciding which operations to port first (prioritize by user value)
- Defining standard patterns: how should a source, function, and target each look as a standalone Rust type?
- Handling configuration: the old ops used JSON specs; the new ones should use typed config structs with builder patterns
- Testing strategy: transient-mode tests that don't require LMDB setup

### 5.4 `Ctx` lifetime and ownership model

The `Ctx` type is the central abstraction users interact with. Its design must resolve:

- How does `Ctx` reference the LMDB environment without lifetime infection throughout user code?
- How do `scope()` and `mount_each()` create child contexts? (Arc? Rc? Scoped references?)
- How does `use_resource()` return references when the resource registry is behind shared ownership?
- Thread safety: `mount_each` runs items concurrently; `Ctx` and its children must be `Send + Sync`

Getting this wrong means either unsafe code, ergonomic nightmares with lifetimes, or performance penalties from excessive cloning/Arc.

### 5.5 Target state reconciliation for Rust types

The engine's target system (`TargetHandler`, `TargetActionSink`, `TargetStateProvider`) expects types that can be persisted to LMDB and diffed between runs. For Rust:

- What serialization format? (bincode? postcard? rkyv for zero-copy?)
- How do users define "identity" for target rows? (derive macro? trait?)
- How does `declare_row(T)` know how to diff `T` against the previous run's state?
- How do different target backends (Postgres, Qdrant, Neo4j) map from the generic `TargetAction` to backend-specific operations?

### 5.6 Incremental source change detection

The upstream model tracks source changes through stable paths and LMDB state. For Recoco:

- How does `local_file::walk_dir()` integrate with the engine's change detection? Is it a special source type, or does `mount_each` handle it generically via key-based diffing?
- CDC (change data capture) for database sources: how does this work without the upstream's Python-side polling infrastructure?
- File watching for live mode: `notify` crate integration, debouncing, and feeding changes back into the component tree

### 5.7 Crate structure

The current workspace mirrors upstream's layout, but with upstream abandoning Rust-facing functionality, there's no reason to track their crate boundaries. The guiding principles for Recoco's structure are:

1. **Prefer more crates** -- idiomatic Rust, independent compilation, clear boundaries
2. **But only with reason** -- each crate should be at least potentially independently useful
3. **Heavy feature gating** -- remains a core design philosophy; every optional dependency is gated
4. **Engine stays separate** -- simplifies upstream sync and the engine has standalone value beyond Recoco's own use cases

#### Decided

- **`recoco-core`** (or `recoco-engine`): The LMDB-backed incremental engine. Separate crate. This is the piece we track upstream on, and it has genuine standalone value -- anyone building incremental/memoized processing in Rust could use it without the rest of Recoco's opinions about sources/targets/functions.
- **`recoco-macros`**: Proc macro crate. Required by Rust's compilation model. Ships `#[recoco::function]`.
- **`recoco-splitters`**: Text chunking algorithms. Already a separate crate and independently useful -- text splitting for RAG, search indexing, etc. doesn't require the engine or any other Recoco crate.
- **`recoco`**: The user-facing crate. Re-exports from engine and macros, contains the `Ctx`/`ContextKey`/`Environment`/`App` API layer, and hosts all feature-gated operations (sources, functions, targets) as modules within it.

#### Open questions

- **Should `recoco-utils` remain its own crate?** It currently holds fingerprinting, HTTP helpers, string sanitization. These are internal plumbing, not independently useful. Strong candidate for folding into `recoco-core` or `recoco` as private modules.
- **Individual target/source crates vs modules in `recoco`?** A crate like `recoco-target-qdrant` could version and release independently of the rest. But if feature flags within `recoco` already gate compilation cleanly, separate crates add release coordination overhead without clear user benefit. The decision depends on how heavy the per-target dependencies are and whether any target has an audience that doesn't want the rest of `recoco`.
- **Feature flags: crate-level vs module-level gating?** With operations inside `recoco`, features gate `#[cfg(feature = "target-qdrant")] mod qdrant;` within the crate. With separate crates, features gate `recoco-target-qdrant = { optional = true }` in `Cargo.toml`. Both work; the question is which gives cleaner `cargo add` ergonomics and which is easier to maintain as the operation count grows.
- **Public API paths**: With operations in `recoco`, users get `use recoco::targets::qdrant::QdrantTarget`. With separate crates re-exported, it's the same path but the re-export coupling means the umbrella crate's version bumps when any operation crate bumps. Need to decide what's acceptable.

The likely landing point is a 4-crate workspace (`recoco`, `recoco-core`, `recoco-macros`, `recoco-splitters`) with `recoco-utils` absorbed, and operations as feature-gated modules within `recoco`. This keeps the crate count honest -- every crate earns its place through independent utility -- while feature flags handle the combinatorial explosion of optional dependencies.

### 5.8 Error handling strategy

The current codebase uses `anyhow::Result` extensively. For a published library:

- Should Recoco define its own error types? (Probably yes for public API, `thiserror`-based)
- How do errors from user-provided functions propagate through the engine?
- How does the fault-tolerant retry logic interact with user-defined error types?
- Should operation-specific errors (e.g., database connection failures) be typed or opaque?

## 6. Relationship to Upstream Going Forward

Recoco should continue tracking upstream engine changes selectively. The engine layer (`crates/core/`) is where upstream invests heavily in correctness, performance, and reliability -- Recoco benefits from pulling those improvements.

Above the engine, the projects diverge. Upstream builds Python integrations; Recoco builds Rust APIs. There is no conflict here, just different audiences.

The upstream's "persistent-state-driven model" description is accurate for the shared engine semantics. Recoco's documentation should use this framing when describing the programming model, adapted for Rust idioms. The key principles carry over directly: transformations create new fields from input fields without hidden state; all data is observable; lineage is automatic.

What doesn't carry over is the Python-specific framing ("write simple transformations without learning new DSLs", "leverage the full Python ecosystem"). Recoco's equivalent: write plain Rust functions; the engine handles incrementality, memoization, and state management transparently.
