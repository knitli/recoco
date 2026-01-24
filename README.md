<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. (ReCoco)
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

<div align="center">
  <img src="docs/assets/recoco.webp" alt="ReCoco Logo" width="200"/>

  # ReCoco

  **Incremental ETL and Data Processing Framework for Rust**

  [![Crates.io](https://img.shields.io/crates/v/recoco.svg)](https://crates.io/crates/recoco)
  [![Documentation](https://docs.rs/recoco/badge.svg)](https://docs.rs/recoco)
  [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
  [![REUSE Compliance](https://img.shields.io/badge/reuse-compliant-brightgreen)](https://reuse.software/)

</div>

---

**ReCoco** is a pure Rust fork of the excellent [CocoIndex](https://github.com/cocoindex-io/cocoindex), a high-performance, incremental ETL and data processing framework.

## 📑 Table of Contents

- [Why Fork?](#why-fork)
- [How ReCoco is Different](#how-recoco-is-different)
- [Key Features](#-key-features)
- [Use Cases](#-use-cases)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Examples](#examples)
- [Architecture](#️-architecture)
- [Development](#️-development)
- [Contributing](#-contributing)
- [Relationship to CocoIndex](#-relationship-to-cocoindex)
- [License](#-license)

## Why Fork?

I decided to create a Rust-only fork of CocoIndex for a couple reasons:

1. **CocoIndex is not a Rust library.**  CocoIndex is written in Rust, but it does not expose a Rust API and its packaging, documentation, and examples are only focused on *Python*. It exposes a more limited API through its Rust extensions. It's not even released on crates.io.

2. **CocoIndex is heavy.** CocoIndex has several very heavy dependencies that you probably don't need all of, including Google/AWS/Azure components, Qdrant/Postgres/Neo4j, and more. 

## How ReCoco is Different

1. **ReCoco fully exposes a Rust API.** You can use ReCoco to support your rust ETL projects directly. **Build on it.**

2. **Every target, source, and function is independently feature-gated. Use only what you want.**

## ✨ Key Features

- 🦀 **Pure Rust**: No Python dependencies, interpreters, or build tools required
- ⚡ **Incremental Processing**: Built on a dataflow engine that processes only changed data
- 🎯 **Modular Architecture**: Feature-gated sources, targets, and functions - use only what you need
- 🔌 **Rich Connector Ecosystem**:
  - **Sources**: Local Files, PostgreSQL, S3, Azure Blob, Google Drive
  - **Targets**: PostgreSQL, Qdrant, Neo4j, Kùzu
  - **Functions**: Text splitting, LLM embedding (OpenAI/Google), JSON parsing, language detection
- 🚀 **Async API**: Fully async/await compatible API based on Tokio
- 📦 **Workspace Structure**: Clean separation into `recoco`, `recoco_utils`, and `recoco_splitters` crates
- 🔄 **Data Lineage Tracking**: Automatic tracking of data dependencies for smart incremental updates

## 🎯 Use Cases

ReCoco is designed for building scalable data pipelines with intelligent incremental processing:

- **RAG (Retrieval-Augmented Generation) Pipelines**: Ingest documents, split them intelligently, generate embeddings, and store in vector databases
- **ETL Workflows**: Extract data from various sources, transform it, and load into databases or data warehouses
- **Document Processing**: Parse, chunk, and extract information from large document collections
- **Data Synchronization**: Keep data synchronized across multiple systems with automatic change detection
- **Custom Data Transformations**: Build domain-specific data processing pipelines with custom operations

## Installation

Add `recoco` to your `Cargo.toml`. Since ReCoco uses a modular feature system, you should enable only the features you need.

```toml
[dependencies]
recoco = { version = "0.1.0", default-features = false, features = ["source-local-file", "function-split"] }
```

### Available Features

#### 📥 Sources (Data Ingestion)

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `source-local-file` | Read files from local filesystem | ✅ Default - lightweight |
| `source-postgres` | Read from PostgreSQL (Change Data Capture) | 📦 PostgreSQL driver |
| `source-s3` | Read from Amazon S3 | 📦 AWS SDK |
| `source-azure` | Read from Azure Blob Storage | 📦 Azure SDK |
| `source-gdrive` | Read from Google Drive | 📦 Google APIs |

#### 📤 Targets (Data Persistence)

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `target-postgres` | Write to PostgreSQL | 📦 PostgreSQL driver |
| `target-qdrant` | Write to Qdrant Vector DB | 📦 Qdrant client |
| `target-neo4j` | Write to Neo4j Graph DB | 📦 Neo4j driver |
| `target-kuzu` | Write to Kùzu embedded Graph DB | 📦 Kùzu bindings |

#### ⚙️ Functions (Data Transformations)

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `function-split` | Text splitting utilities (recursive, semantic) | ✅ Lightweight |
| `function-embed` | Text embedding (OpenAI, Vertex AI) | 📦 LLM APIs |
| `function-extract-llm` | Information extraction via LLM | 📦 LLM APIs |
| `function-detect-lang` | Programming language detection | ✅ Lightweight |
| `function-json` | JSON/JSON5 parsing and manipulation | ✅ Lightweight |

#### 📦 Feature Bundles

| Feature | Description |
|---------|-------------|
| `all-sources` | Enable all source connectors |
| `all-targets` | Enable all target connectors |
| `all-functions` | Enable all transform functions |
| `full` | Enable everything (⚠️ heavy dependencies) |

## 🚀 Quick Start

### Basic Text Processing

Here's a simple example that processes a string using a transient flow (in-memory, no persistence):

```rust
use recoco::prelude::*;
use recoco::builder::FlowBuilder;
use recoco::execution::evaluator::evaluate_transient_flow;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize library context (loads operation registry)
    recoco::lib_context::init_lib_context(None).await?;

    // 2. Create a flow builder
    let mut builder = FlowBuilder::new("hello_world").await?;

    // 3. Define input schema
    let input = builder.add_direct_input(
        "text".to_string(),
        schema::make_output_type(schema::BasicValueType::Str),
    )?;

    // 4. Add a text splitting transformation
    let output = builder.transform(
        "SplitBySeparators".to_string(),
        json!({ "separators_regex": [" "] }).as_object().unwrap().clone(),
        vec![(input, Some("text".to_string()))],
        None,
        "splitter".to_string(),
    ).await?;

    // 5. Set the output of the flow
    builder.set_direct_output(output)?;

    // 6. Build and execute the flow
    let flow = builder.build_transient_flow().await?;
    let result = evaluate_transient_flow(
        &flow.0,
        &vec![value::Value::Basic("Hello ReCoco".into())]
    ).await?;

    println!("Result: {:?}", result);
    Ok(())
}
```

### Custom Operations

You can define custom operations by implementing the `SimpleFunctionExecutor` and `SimpleFunctionFactoryBase` traits:

```rust
use recoco::ops::sdk::*;

struct UpperCaseExecutor;

#[async_trait::async_trait]
impl SimpleFunctionExecutor for UpperCaseExecutor {
    async fn execute(&self, inputs: Vec<Value>) -> Result<Vec<Value>, ExecutionError> {
        let input_str = inputs[0].as_basic_str()
            .ok_or_else(|| ExecutionError::InvalidInput("Expected string".into()))?;

        Ok(vec![Value::Basic(input_str.to_uppercase().into())])
    }
}

// Register your custom operation
// See examples/custom_op.rs for complete implementation
```

### Flow Construction Pattern

All flows follow this consistent pattern:

```rust
// 1. Initialize library context
recoco::lib_context::init_lib_context(None).await?;

// 2. Create builder
let mut builder = FlowBuilder::new("my_flow").await?;

// 3. Define inputs
let input = builder.add_direct_input(/*...*/)?;

// 4. Chain transformations
let step1 = builder.transform(/*...*/)?;
let step2 = builder.transform(/*...*/)?;

// 5. Set outputs
builder.set_direct_output(step2)?;

// 6. Build and execute
let flow = builder.build_transient_flow().await?;
let result = evaluate_transient_flow(&flow.0, &inputs).await?;
```

## Examples

Check out the `examples/` directory for more usage patterns:

- `transient.rs`: Basic Hello World
- `file_processing.rs`: Line-by-line file processing
- `custom_op.rs`: Defining and registering custom Rust operations
- `detect_lang.rs`: Using built-in functions

Run examples with the required features:
```bash
# Basic transient flow
cargo run -p recoco --example transient --features function-split

# File processing
cargo run -p recoco --example file_processing --features function-split

# Custom operations
cargo run -p recoco --example custom_op

# Language detection
cargo run -p recoco --example detect_lang --features function-detect-lang
```

## 🛠️ Development

### Building

```bash
# Build with default features (source-local-file only)
cargo build

# Build with specific features
cargo build --features "function-split,source-postgres"

# Build with all features (includes all heavy dependencies)
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

# Run a specific test with output
cargo test test_name -- --nocapture
```

### Code Quality

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

## 🏗️ Architecture

### Core Dataflow Model

ReCoco implements an **incremental dataflow engine** where data flows through **Flows**:

```
Sources → Transforms → Targets
```

- **Sources**: Ingest data (files, database rows, cloud storage objects)
- **Transforms**: Process data (split, embed, map, filter, extract)
- **Targets**: Persist results (vector databases, graph databases, relational databases)

The engine tracks **data lineage** - when source data changes, only affected downstream computations are re-executed. This makes ReCoco highly efficient for processing large datasets that change incrementally.

### Two Flow Execution Modes

1. **Transient Flows** - In-memory execution without persistence
   - Use `FlowBuilder::build_transient_flow()`
   - Evaluate with `execution::evaluator::evaluate_transient_flow()`
   - No database tracking, ideal for testing and single-run operations
   - Fast and lightweight for one-off transformations

2. **Persisted Flows** - Tracked execution with incremental updates
   - Use `FlowBuilder::build_flow()` to create persisted flow spec
   - Requires database setup for state tracking
   - Enables incremental processing when data changes
   - Perfect for production ETL pipelines

### Module Organization

```
recoco/
├── base/          - Core data types (schema, value, spec, json_schema)
├── builder/       - Flow construction API (FlowBuilder, analysis, planning)
├── execution/     - Runtime engine (evaluator, memoization, indexing, tracking)
├── ops/           - Operation implementations
│   ├── sources/   - Data ingestion (local-file, postgres, s3, azure, gdrive)
│   ├── functions/ - Transforms (split, embed, json, detect-lang, extract-llm)
│   ├── targets/   - Data persistence (postgres, qdrant, neo4j, kuzu)
│   ├── interface.rs  - Trait definitions for all operation types
│   ├── registry.rs   - Operation registration and lookup
│   └── sdk.rs        - Public API for custom operations
├── lib_context.rs - Global library initialization and context management
└── prelude.rs     - Common imports (use recoco::prelude::*)
```

## 🤝 Contributing

Contributions are welcome! Here's how to get started:

1. **Fork the repository** and clone your fork
2. **Create a feature branch**: `git checkout -b feature/my-new-feature`
3. **Make your changes** following our code style and conventions
4. **Run tests**: `cargo test --features full`
5. **Run formatting**: `cargo fmt --all`
6. **Run clippy**: `cargo clippy --all-features -- -D warnings`
7. **Commit your changes** using [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation
   - `chore:` for maintenance tasks
   - `refactor:` for code restructuring
8. **Push to your fork** and **submit a pull request**

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

### Development Tips

- **Feature Gates**: When adding new dependencies, make them optional with feature flags
- **Testing**: Include unit tests alongside implementation files
- **Documentation**: Add doc comments for public APIs
- **Examples**: For significant features, consider adding an example in `crates/recoco/examples/`

## 🔗 Relationship to CocoIndex

ReCoco is a fork of [CocoIndex](https://github.com/cocoindex/cocoindex):

| Aspect | CocoIndex (Upstream) | ReCoco (Fork) |
|--------|---------------------|---------------|
| **Primary Language** | Python with Rust core | Pure Rust |
| **API Surface** | Python-only | Full Rust API |
| **Distribution** | Not on crates.io | Published to crates.io |
| **Dependencies** | All bundled together | Feature-gated and modular |
| **Target Audience** | Python developers | Rust developers |
| **License** | Apache-2.0 | Apache-2.0 |

We aim to maintain compatibility with CocoIndex's core dataflow engine to allow porting upstream improvements, while diverging significantly in the API surface and dependency management to better serve Rust users.

Code headers maintain dual copyright (CocoIndex.io upstream, Knitli Inc. for ReCoco modifications) under Apache-2.0.

## 📄 License

[Apache License 2.0](LICENSE); see [NOTICE](NOTICE) for full license text.

This project is [REUSE 3.3 compliant](https://reuse.software/).

## 🙏 Acknowledgments

ReCoco is built on the excellent foundation provided by [CocoIndex](https://github.com/cocoindex/cocoindex). We're grateful to the CocoIndex team for creating such a powerful and well-designed dataflow engine.

---

<div align="center">

**Built with 🦀 by [Knitli Inc.](https://knit.li)**

[Documentation](https://docs.rs/recoco) • [Crates.io](https://crates.io/crates/recoco) • [GitHub](https://github.com/knitli/recoco) • [Issues](https://github.com/knitli/recoco/issues)

</div>
