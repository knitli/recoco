# CLAUDE.md/AGENTS.md/GEMINI.md

This file provides guidance to Agents like Claude, Gemini, or ChatGPT.

## Project Overview

ReCoco is a pure Rust fork of CocoIndex, an incremental ETL and data processing framework. The project exposes a full Rust API with a modular, feature-gated architecture for sources, targets, and functions.

**Key Differentiators:**
- Pure Rust library (no Python dependencies)
- Fully feature-gated dependencies - every source, target, and function is independently optional
- Published to crates.io as `recoco`
- Workspace structure with three crates: `recoco`, `recoco_utils`, `recoco_splitters`

## Build and Development Commands

### Building
```bash
# Build with default features (source-local-file only)
cargo build

# Build with specific features
cargo build --features "function-split,source-postgres"

# Build with all features
cargo build --features full

# Build specific feature bundles
cargo build --features all-sources    # All source connectors
cargo build --features all-targets    # All target connectors
cargo build --features all-functions  # All transform functions
```

### Testing
```bash
# Run all tests with default features
cargo test

# Run tests with specific features
cargo test --features "function-split,source-postgres"

# Run tests with all features
cargo test --features full

# Run a specific test
cargo test test_name -- --nocapture
```

### Linting and Formatting
```bash
# Check code formatting
cargo fmt --all -- --check

# Format code
cargo fmt

# Run clippy with all features
cargo clippy --all-features -- -D warnings

# Run clippy for specific workspace member
cargo clippy -p recoco --all-features
```

### Running Examples
```bash
# Examples are in crates/recoco/examples/
# Must specify features needed by the example

# Transient flow example
cargo run -p recoco --example transient --features function-split

# File processing example
cargo run -p recoco --example file_processing --features function-split

# Custom operation example
cargo run -p recoco --example custom_op

# Language detection example
cargo run -p recoco --example detect_lang --features function-detect-lang
```

## Architecture

### Core Dataflow Model

ReCoco implements an **incremental dataflow engine** where data flows through **Flows**:

```
Sources → Transforms → Targets
```

**Sources** ingest data (files, database rows, cloud storage)
**Transforms** process data (split, embed, map, filter)
**Targets** persist results (vector DBs, graph DBs, databases)

The engine tracks **data lineage** - when source data changes, only affected downstream computations are re-executed.

### Two Flow Execution Modes

1. **Transient Flows** - In-memory execution without persistence
   - Use `FlowBuilder::build_transient_flow()`
   - Evaluate with `execution::evaluator::evaluate_transient_flow()`
   - No database tracking, ideal for testing and single-run operations

2. **Persisted Flows** - Tracked execution with incremental updates
   - Use `FlowBuilder::build_flow()` to create persisted flow spec
   - Requires database setup for state tracking
   - Enables incremental processing when data changes

### Module Organization

- **`base/`** - Core data types (schema, value, spec, json_schema)
- **`builder/`** - Flow construction API (`FlowBuilder`, analysis, planning)
- **`execution/`** - Runtime engine (evaluator, memoization, indexing, tracking)
- **`ops/`** - Operation implementations
  - `sources/` - Data ingestion (local-file, postgres, s3, azure, gdrive)
  - `functions/` - Transforms (split, embed, json, detect-lang, extract-llm)
  - `targets/` - Data persistence (postgres, qdrant, neo4j, kuzu)
  - `interface.rs` - Trait definitions for all operation types
  - `registry.rs` - Operation registration and lookup
  - `sdk.rs` - Public API for custom operations
- **`lib_context.rs`** - Global library initialization and context management
- **`prelude.rs`** - Common imports (`use recoco::prelude::*`)

### Flow Construction Pattern

All flows follow this pattern:

```rust
// 1. Initialize library context (loads operation registry)
recoco::lib_context::init_lib_context(Some(Settings::default())).await?;

// 2. Create builder
let mut builder = FlowBuilder::new("flow_name").await?;

// 3. Define inputs
let input = builder.add_direct_input(
    "input_name".to_string(),
    schema::make_output_type(schema::BasicValueType::Str),
)?;

// 4. Add transforms (chain operations)
let output = builder.transform(
    "OperationName".to_string(),
    json!({ "param": "value" }).as_object().unwrap().clone(),
    vec![(input, Some("arg_name".to_string()))],
    None,
    "step_name".to_string(),
).await?;

// 5. Set output
builder.set_direct_output(output)?;

// 6. Build and execute
let flow = builder.build_transient_flow().await?;
let result = evaluate_transient_flow(&flow.0, &vec![Value::Basic(...)]).await?;
```

### Custom Operation Pattern

Creating custom operations requires implementing:

1. **Executor** - Runtime logic (`SimpleFunctionExecutor` trait)
2. **Factory** - Analysis and construction (`SimpleFunctionFactoryBase` trait)
3. **Registration** - Add to registry before use

See `examples/custom_op.rs` for full implementation pattern.

### Feature System

Operations are feature-gated at the dependency level:

- **Sources**: `source-local-file`, `source-postgres`, `source-s3`, `source-azure`, `source-gdrive`
- **Targets**: `target-postgres`, `target-qdrant`, `target-neo4j`, `target-kuzu`
- **Functions**: `function-split`, `function-embed`, `function-extract-llm`, `function-detect-lang`, `function-json`

When adding new code:
- Check `Cargo.toml` features to understand which dependencies are available
- Conditional compilation uses `#[cfg(feature = "...")]` attributes
- The `full` feature enables everything (use sparingly, it's heavy)

### Library Context

The global library context (`lib_context::init_lib_context()`) must be initialized before creating flows. It:
- Loads the operation registry with all compiled-in operations
- Sets up authentication registries
- Initializes runtime configuration

Only call this once per application (usually in `main()`).

## Testing Philosophy

- Unit tests live alongside implementation files
- Integration tests use transient flows for fast execution
- Feature-specific tests are gated with `#[cfg(feature = "...")]`
- The CI matrix tests multiple feature combinations: default, full, all-sources, all-targets, all-functions

## Commit Message Convention

Use [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation
- `chore:` for maintenance tasks
- `refactor:` for code restructuring

This is required for changelog generation via `git cliff`.

## Relationship to Upstream

ReCoco is a **fork** of CocoIndex (https://github.com/cocoindex/cocoindex):
- **Upstream**: Python-focused with private Rust core, not on crates.io
- **ReCoco**: Rust-focused public API, published to crates.io, feature-gated architecture

Code headers maintain dual copyright (CocoIndex.io upstream, Knitli Inc. for ReCoco modifications) under Apache-2.0.
