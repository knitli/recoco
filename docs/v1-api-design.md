<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

# Upstream v1 Architecture Analysis & Recoco Planning Notes

> **Status**: Research document — describes what upstream CocoIndex v1 is actually building,
> corrects prior assumptions, and outlines implications for recoco's independent development path.
>
> **Last Updated**: 2026-03-28
> **Upstream SHA**: `8089620b72bc2ac67b448a5b40dffdd121087314` (v1 branch)
> **Relevant feature branches**: `g/v1-api-upgrade`, `g/v1-syntax-sugar`

---

## TL;DR

The upstream CocoIndex v1 is a **complete architectural redesign** from v0. All three of the
assumptions in the original planning question were incorrect in significant ways:

1. **Sources/targets/functions are NOT waiting to be added to the Rust layer** — they have been
   deliberately moved to Python and are already present and actively developed there.
2. **Upstream will NOT pull existing Rust ops back in** — the intent is a Python-first architecture
   where Rust provides the engine only.
3. **Existing Rust ops are NOT compatible** with v1's engine design — the engine uses Python
   callbacks; there is no Rust-level op registry in v1.

**For recoco**: This is actually good news. Since upstream is going Python, recoco's mission of
being a pure Rust library is now *more* differentiated and *more* independently scoped. We own
the Rust ops layer entirely. The engine sync from upstream remains valid and important.

---

## 1. What the Assumptions Were

The problem statement raised three specific concerns:

> **Assumption 1**: Upstream v1 has no targets/functions/sources, and those remain to be
> developed — for our library and theirs.

> **Assumption 2**: Upstream was building out the core and would likely reuse the existing
> sources/targets/functions, pulling them into the branch when the engine was ready.

> **Assumption 3**: The question is whether they would be compatible with the v1 architecture.

Also asked: whether there are signs of sources/targets shifting to the Python API in the v1
branch, and whether project plans or other intent documents exist.

---

## 2. What Upstream v1 Actually Is

Upstream v1 is a **from-scratch redesign** of CocoIndex. The CLAUDE.md in the v1 branch
states explicitly:

> *"The current codebase is for CocoIndex v1, which is a fundamental redesign from CocoIndex v0.
> Currently the `v1` branch is the main branch for CocoIndex v1 code."*

### 2.1 Confirmed: Sources/Targets Have Moved to Python

The v1 branch already has a full set of connectors (sources + targets) — all in Python:

**`python/cocoindex/connectors/`** (as of 2026-03-28):

| Connector | Direction | Notes |
|-----------|-----------|-------|
| `localfs` | source + target | File system; walk_dir, DirTarget |
| `postgres` | source + target | TableTarget, vector support |
| `qdrant` | target | CollectionTarget |
| `lancedb` | target | TableTarget |
| `amazon_s3` | source + target | async boto3 |
| `sqlite` | target | sqlite-vec extension |
| `surrealdb` | target | Graph + document DB |
| `doris` | target | Apache Doris analytics DB |
| `google_drive` | source | Google Drive file listing |

This is **more connectors** than the old recoco Rust implementation had. These are actively
maintained. The connector count is growing (e.g., surrealdb and doris were added during v1 development).

**`python/cocoindex/ops/`** (functions):

| Op | Description |
|----|-------------|
| `text.py` | RecursiveSplitter (Python wrapper around `rust/ops_text`) |
| `sentence_transformers.py` | Sentence embedding via SentenceTransformers |
| `litellm.py` | LLM function calls via litellm |

Functions are implemented as Python `@coco.fn` decorators, which is the standard user-facing
mechanism. The `rust/ops_text` crate provides performance-critical implementations callable
from Python.

### 2.2 Confirmed: No Rust-Native Op Registry in v1

The upstream v1 `rust/core/` contains **no sources, targets, or functions**:

```
rust/core/src/
├── engine/
│   ├── app.rs            # App and AppEnvironment management
│   ├── component.rs      # Processing component execution
│   ├── context.rs        # Context value storage and retrieval
│   ├── environment.rs    # Environment lifecycle
│   ├── execution.rs      # Core execution engine (63k bytes)
│   ├── function.rs       # Function handle abstractions
│   ├── id_sequencer.rs   # Stable ID generation
│   ├── logic_registry.rs # Logic (function) registration
│   ├── profile.rs        # Execution profiling
│   ├── runtime.rs        # Tokio runtime
│   ├── stats.rs          # Processing statistics
│   ├── target_state.rs   # Target state tracking/apply
│   └── txn_batcher.rs    # Transaction batching
├── inspect/              # Database inspection utilities
├── state/
│   ├── db_schema.rs      # LMDB schema
│   ├── stable_path.rs    # Stable component paths
│   ├── stable_path_set.rs
│   └── target_state_path.rs
├── lib.rs
└── prelude.rs
```

The `logic_registry.rs` provides an interface for *registering* functions — but these are
Python callables (via PyO3), not Rust-native implementations. There is no equivalent of
recoco's old `ops/registry.rs` with domain-specific Rust ops.

### 2.3 The Only Rust Ops: Performance-Critical Text Processing

The `rust/ops_text/` crate provides:
- `split/` — Text splitting (separator-based and recursive)
- `output_positions.rs` — Position tracking for splits
- `pattern_matcher.rs` — Glob pattern matching
- `prog_langs.rs` — Programming language detection (tree-sitter based)

These map directly to recoco's `crates/splitters/` crate. The upstream exposes these to Python
via `rust/py/`; recoco exposes them as a standalone Rust library.

---

## 3. The v1 Python API Design

Understanding the Python API design helps recoco decide what a Rust-native equivalent should look like.

### 3.1 Core Mental Model

v1 uses a **declarative reactive model** inspired by React:

> *"Think of it like: React declares UI as function of state → React re-renders what changed;
> CocoIndex declares target states as function of source → engine syncs what changed."*

The key shift from v0: users no longer write ETL pipelines. They **declare what target states
should exist** and the engine handles create/update/delete incrementally.

### 3.2 Key Concepts

**App** — Top-level runnable unit. Bundles a main function with its configuration.

**Processing Component** — Unit of execution owning a set of target states. Created by `mount()`
or `use_mount()`. Has a stable `component_subpath` for change detection across runs.

**Component Path** — A stable hierarchical identifier (e.g., `coco.component_subpath("file", filename)`)
that maps a component to its previous run's state for incremental processing.

**Target State** — What should exist in an external system (a database row, a file, a vector
collection entry). Declared via `target.declare_row(...)`, `target.declare_file(...)`, etc.

**Function** — Python function decorated with `@coco.fn`. Supports `memo=True` for memoization.

**Context** — React-style dependency injection: `ContextKey[T]`, `builder.provide()`, `coco.use_context(key)`.

### 3.3 Primary API Surface

```python
# App construction
@coco.fn
async def app_main(sourcedir: pathlib.Path) -> None:
    target_db = coco.use_context(PG_DB)
    table = await target_db.mount_table_target(...)
    files = localfs.walk_dir(sourcedir, ...)
    coco_aio.mount_each(process_file, files, table)

app = coco.App(coco.AppConfig(name="MyApp"), app_main, sourcedir=Path("./data"))
app.update_blocking()

# Core APIs (all async)
await coco_aio.mount(subpath, fn, *args)        # independent child component
await coco_aio.use_mount(subpath, fn, *args)    # dependent component, returns value
coco_aio.mount_each(fn, items, *args)           # one component per keyed item
await coco_aio.mount_target(target_state)       # mount a target connector
await coco_aio.map(fn, items, *args)            # concurrent map, no mounting
```

### 3.4 Connector Pattern

Connectors provide both a "declaration" method and a convenience "mount" method:

```python
# Connector pattern (postgres example):
target_db.table_target(...)          # Returns TargetState (use with mount_target())
await target_db.mount_table_target(...) # Convenience: mounts + returns TableTarget

# Source pattern (localfs example):
files = localfs.walk_dir(sourcedir, ...)  # Returns keyed async iterator
coco_aio.mount_each(process_file, files, target)  # One component per file
```

Sources are **keyed async iterators** (not push-based streams). The key provides the component
path segment; the value is the data.

Targets declare **container states** (a table, a directory, a collection) and return a
**child provider** for declaring individual items (rows, files, points).

### 3.5 Feature Branches: API Evolution

**`g/v1-api-upgrade`** (has `API_UPGRADE_SPEC.md`):
This branch documents the evolution from `mount_run()` to `use_mount()` and introduces
the full set of convenience APIs (`mount_each`, `mount_target`, `map`). This branch is
the specification for what gets merged into v1. Key changes:
- `mount_run(subpath, fn, *args).result()` → `await use_mount(subpath, fn, *args)` (simpler)
- Loop of `mount()` calls → `mount_each(fn, items, *args)` (ergonomic)
- `with component_subpath("setup"): await mount_run(...).result()` → `await mount_target(target)` (automatic path derivation)

**`g/v1-syntax-sugar`**: Further convenience improvements on top of the v1 API.

---

## 4. Answering the Original Questions

### 4.1 "Assumes upstream v1 has no targets/functions/sources"

**Incorrect assumption** — but in the opposite direction from expected. Upstream v1 does NOT
have Rust-native targets/functions/sources, AND they are not planned for the Rust layer.
Instead, a comprehensive set of connectors and ops already exists in Python, and this set
is growing (surrealdb, doris were recent additions).

The v1 design is intentional: **Python for connectors and ops, Rust for the engine**.

### 4.2 "My assumption was that they were building out the core and would likely reuse existing sources/targets/functions"

**Incorrect** — Upstream v1 did not "pull in" the old Rust ops. The architecture changed so
fundamentally that the old op model doesn't apply. The v0 had a Rust registry of typed ops;
v1 has Python callables registered as `@coco.fn` functions. These are architecturally different.

### 4.3 "Whether they would be compatible with the v1 architecture"

**Not directly compatible**. The v1 Rust core uses Python function handles (via PyO3) and
Python callbacks for all domain logic. There is no mechanism to register a Rust-only op
without going through PyO3. The old `SimpleFunctionExecutor`, `SourceOperator`, `TargetOperator`
traits do not exist in v1.

### 4.4 "Signs of shifting sources/targets to the Python API"

**Confirmed — already happened**. This is not a future plan; it's the current state. All
connectors are Python. This is a deliberate, stable design decision.

### 4.5 "Look for any projects or other plans"

The feature branches (`g/v1-api-upgrade`, `g/v1-syntax-sugar`) show **Python API refinement**
as the current development focus. The upstream's AI agent prompt (`.github/_upstream_agent_prompt.md`)
confirms this: upstream sync agents are told to skip "Python-only (no Rust code changes)" and
to focus on `rust/core`, `rust/utils`, `rust/ops_text`. These exact three Rust crates map to
recoco's `crates/core`, `crates/utils`, `crates/splitters`.

The upstream is actively adding Python connectors (surrealdb, doris) and improving the
Python API ergonomics. There are NO signals of moving any of this back to Rust.

---

## 5. Implications for Recoco

### 5.1 The Engine Layer: Continue Syncing from Upstream

The upstream `rust/core/` is the most important thing to track and merge. Every engine
improvement (execution correctness, incremental processing, memoization, stats, target state
ownership) applies directly to recoco's `crates/core/`.

**Sync mapping** (from `dev/major_fork_guide.md`):
- `rust/core/` → `crates/core/`
- `rust/utils/` → `crates/utils/`
- `rust/ops_text/` → `crates/splitters/`
- `rust/py/` → NOT synced (Python bindings, irrelevant to recoco)
- `python/` → NOT synced (Python-only)

This mapping is **correct and should be maintained**.

### 5.2 The Connector/Op Layer: Recoco Owns This Independently

Upstream Python connectors are not a source of truth for recoco's Rust ops. Recoco needs
to develop and maintain Rust-native implementations independently.

However, the **semantics** of upstream connectors can inform recoco's design:
- What interfaces do connectors expose? (walk_dir, declare_row, etc.)
- What is the expected source/target contract? (keyed iterables, container+child pattern)
- What connectors exist? (can serve as a feature roadmap for recoco)

### 5.3 Recoco's Architecture Remains Valid

Recoco's current approach (Rust-native sources, targets, functions with feature gating) is
the **correct and necessary approach** for a pure Rust library. Since upstream went Python,
recoco is now the *primary* Rust implementation of this ecosystem and is uniquely positioned
as the Rust-native alternative.

The existing recoco ops layer is independent and correct for its purpose.

### 5.4 API Design Question: Rust Equivalent of v1 Python API

This is the most important open architectural question for recoco's own v1 design:

The upstream Python API centers on `App`, `@coco.fn`, `mount()`, `use_mount()`, `mount_each()`,
component paths, context injection, and target state declarations. Recoco needs a Rust-native
equivalent of this conceptual model.

Options to evaluate:
1. **Builder pattern** (current recoco approach) — explicit flow construction via `FlowBuilder`
2. **Trait-based declarative API** — Rust traits analogous to `@coco.fn`
3. **Macro-based API** — proc macros that generate the registration/wiring code
4. **Closure-based API** — closures as the function unit, with an `App::new()` builder

The upstream v1 conceptual model (component paths, target state declarations, memoization,
context injection) is sound and should inform recoco's API design even if the implementation
language differs.

---

## 6. What Does NOT Apply from Upstream v1

| Upstream v1 Component | Recoco Relevance |
|-----------------------|-----------------|
| `rust/core/` (engine) | ✅ Directly relevant — sync and adapt |
| `rust/utils/` | ✅ Directly relevant — sync and adapt |
| `rust/ops_text/` | ✅ Directly relevant — maps to `crates/splitters` |
| `rust/py/` (PyO3 bindings) | ❌ Not relevant — no Python in recoco |
| `rust/py_utils/` | ❌ Not relevant — Python bridging utilities |
| `python/cocoindex/connectors/` | ⚠️ Semantics useful; implementation not portable |
| `python/cocoindex/ops/` | ⚠️ Concepts useful; implementation not portable |
| `python/cocoindex/_internal/` | ❌ Python-only internals |
| v1 Python API design | ⚠️ Conceptual inspiration for Rust API design |

---

## 7. Recommended Next Steps

### Immediate (for this planning cycle)

1. **Confirm the engine sync process** from `dev/major_fork_guide.md` is still accurate given
   the v1 structural changes (it appears correct; verify Cargo.toml workspace structure matches).

2. **Audit recoco's current ops layer** against what upstream v1 Python connectors offer:
   - Are there connectors in upstream v1 Python that recoco should add in Rust?
   - Are there recoco Rust ops that have no upstream equivalent (and thus need independent maintenance)?

3. **Decide on recoco's own API design** for the next major version:
   - Does recoco want to model the `App`/`mount()`/component-path concept in Rust?
   - What does a Rust-native "declarative target state" API look like?
   - This is a significant design question that shapes the entire public API surface.

### Medium Term

4. **Track upstream engine improvements** via the existing upstream-sync workflow (already
   set up via `.github/_upstream_agent_prompt.md`). Focus on:
   - `rust/core/` commits (engine, execution, stats, target state)
   - `rust/utils/` commits (utilities)
   - `rust/ops_text/` commits (splitter improvements)

5. **Evaluate new connectors from upstream Python** as candidates for Rust ports:
   - surrealdb, doris, sqlite-vec are new since the original fork
   - These should be evaluated for Rust-native implementations

### Do Not Do

- **Do not attempt to adapt upstream Python connectors directly** — they use Python-specific
  patterns (asyncpg, qdrant-client, etc.) that do not translate to Rust without a full rewrite.
- **Do not track upstream Python API changes** as part of the sync — they are not relevant.
- **Do not assume upstream will add Rust ops** — this is confirmed not happening.

---

## 8. Summary Table: Corrected Assumptions

| Assumption | Status | Reality |
|------------|--------|---------|
| Upstream v1 has no sources/targets/functions (to be developed) | **Incorrect** | v1 has a full connector set — all in Python, actively developed |
| Upstream would pull existing Rust ops into v1 | **Incorrect** | v1 uses a Python-only connector model; no Rust op integration planned |
| Existing Rust ops would be compatible with v1 architecture | **Incorrect** | v1's Rust engine uses Python callbacks; old op trait system doesn't exist in v1 |
| Sources/targets might shift to Python API | **Confirmed** | Already shifted — this happened during v1 design; it is the stable architecture |
| Upstream has no plans/intent documents | **Incorrect** | `API_UPGRADE_SPEC.md` in `g/v1-api-upgrade`, active feature branches, CLAUDE.md all document intent clearly |

---

## Appendix: Upstream v1 Connector Coverage vs. Old Recoco Rust Ops

For reference, here is how the old recoco Rust ops (from the v0-era codebase) compare to the
current upstream v1 Python connectors:

### Sources

| Source | Old Recoco Rust | Upstream v1 Python |
|--------|----------------|-------------------|
| Local filesystem | ✅ `source-local-file` | ✅ `localfs` |
| PostgreSQL | ✅ `source-postgres` | ✅ (via postgres connector) |
| Amazon S3 | ✅ `source-s3` | ✅ `amazon_s3` |
| Azure Blob Storage | ✅ `source-azure` | ❌ Not in v1 yet |
| Google Drive | ✅ `source-gdrive` | ✅ `google_drive` |

### Targets

| Target | Old Recoco Rust | Upstream v1 Python |
|--------|----------------|-------------------|
| PostgreSQL | ✅ `target-postgres` | ✅ `postgres` |
| Qdrant | ✅ `target-qdrant` | ✅ `qdrant` |
| Neo4j | ✅ `target-neo4j` | ❌ Not in v1 |
| Kuzu | ✅ `target-kuzu` | ❌ Not in v1 |
| LanceDB | ❌ | ✅ `lancedb` |
| SQLite | ❌ | ✅ `sqlite` (sqlite-vec) |
| SurrealDB | ❌ | ✅ `surrealdb` |
| Apache Doris | ❌ | ✅ `doris` |
| Local filesystem | ❌ | ✅ `localfs` (DirTarget) |

### Functions/Ops

| Function | Old Recoco Rust | Upstream v1 Python |
|----------|----------------|-------------------|
| Text splitting | ✅ `function-split` | ✅ (Python wrapping `rust/ops_text`) |
| Language detection | ✅ `function-detect-lang` | ✅ (Python wrapping `rust/ops_text`) |
| Sentence embedding | ✅ `function-embed` | ✅ `sentence_transformers.py` |
| LLM extraction | ✅ `function-extract-llm` | ✅ `litellm.py` |
| JSON operations | ✅ `function-json` | ❌ (subsumed by Python) |

**Observations**:
- Upstream v1 has added connectors (LanceDB, SQLite, SurrealDB, Doris) that recoco doesn't have in Rust
- Recoco has connectors (Neo4j, Kuzu, Azure) that upstream v1 doesn't have in Python
- The text-processing ops are the only ops that remain in Rust in upstream v1
