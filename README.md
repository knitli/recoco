<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. (ReCoco)
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

<div align="center">
  <img src="assets/recoco.webp" alt="ReCoco Logo" width="200"/>

  # ReCoco

  **Incremental ETL and Data Processing Framework for Rust**

  [![Crates.io](https://img.shields.io/crates/v/recoco.svg)](https://crates.io/crates/recoco)
  [![Documentation](https://docs.rs/recoco/badge.svg)](https://docs.rs/recoco)
  [![CI](https://github.com/knitli/recoco/actions/workflows/ci.yml/badge.svg)](https://github.com/knitli/recoco/actions/workflows/ci.yml)
  [![MSRV](https://img.shields.io/badge/MSRV-1.89-blue.svg)](https://blog.rust-lang.org/2025/02/20/Rust-1.89.0.html)
  [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
  [![REUSE Compliance](https://img.shields.io/badge/reuse-compliant-brightgreen)](https://reuse.software/)

</div>

---

**[ReCoco](https://github.com/knitli/recoco)** is a pure Rust fork of the excellent [CocoIndex](https://github.com/cocoindex-io/cocoindex), a high-performance, incremental ETL and data processing framework.

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

For [Knitli](https://knitli.com), I particularly needed dependency control. I wanted to use CocoIndex as an ETL engine for [Thread](https://github.com/knitli/thread/), but Thread needs to be edge-deployable. The dependencies were way too heavy and would never compile to WASM. Thread, of course, is also a Rust project, so pulling in a lot of Python dependencies didn't make sense for me.

> [!NOTE] Knitli and ReCoco have no official relationship with CocoIndex and this project is not endorsed by them. **We will contribute as much as we can upstream**, our [contribution guidelines](CONTRIBUTING.md) encourage you to submit PRs and issues affecting shared code upstream to help both projects.

## How ReCoco is Different from CocoIndex

1. **ReCoco fully exposes a Rust API.** You can use ReCoco to support your rust ETL projects directly. **Build on it.**

2. **Every target, source, and function is independently feature-gated. Use only what you want.** 

> The minimum install now uses **600 fewer crates** (820 -> 620)

We will regularly merge in upstream fixes and changes, particularly sources, targets, and functions.

## ✨ Key Features

### Unique to ReCoco

- 🦀 **Pure Rust**: No Python dependencies, interpreters, or build tools required
- 🚀 **Additional optimizations**: We add additional compile-time optimizations and use some faster alternatives (i.e. `blake2` -> `blake3`) to make ReCoco as fast as possible
- 📦 **Workspace Structure**: Clean separation into `recoco`, `recoco-utils`, and `recoco-splitters` crates
- 🎯 **Modular Architecture**: Feature-gated sources, targets, and functions - use only what you need
- 🔌 **Rich Connector Ecosystem**:
  - **Sources**: Local Files, PostgreSQL, S3, Azure Blob, Google Drive
  - **Targets**: PostgreSQL, Qdrant, Neo4j, Kùzu
  - **Functions**: Text splitting, LLM embedding and calling, Embedding generation 
  - **Individual splitter languages, or none at all**: Choose which grammars to install for the tree-sitter based text splitter. Or, keep them all disabled if you don't need text splitting.
  - **Run without Server or persistence-related dependencies**: If you just want to run local tasks, or integrate ReCoco into an existing server, you can keep it turned off. 
    - You can also run ReCoco as a memory-only pipeline for use with lightweight tasks.[^1] 
    - If you don't need these features, you get a very lightweight library without dependencies like axum, tower, or postgres.

[^1]: In-memory operations aren't quite as feature rich; you lose incremental indexing, for example. At least, for now.

### CocoIndex and ReCoco

- ⚡ **Incremental Processing**: Built on a dataflow engine that processes only changed data
(SentenceTransformers), JSON parsing, language detection
- 🚀 **Async API**: Fully async/await compatible API based on Tokio
- 🔄 **Data Lineage Tracking**: Automatic tracking of data dependencies for smart incremental updates (requires `persistence` feature flag)

## 🎯 Use Cases

ReCoco, like CocoIndex, enables scalable data pipelines with intelligent incremental processing for many use cases, including:

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
| `function-embed` | Text embedding (OpenAI, Vertex AI, Voyage) | 📦 LLM APIs |
| `function-extract-llm` | Use LLM to extract data | 📦 LLM APIs |
| `function-detect-lang` | Programming language detection | ✅ Lightweight |
| `function-json` | JSON/JSON5 parsing and manipulation | ✅ Lightweight |

#### 🤖 LLM Providers

Required for `function-embed` and `function-extract-llm`.

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `provider-anthropic` | Anthropic (Claude) | 📦 reqwest |
| `provider-azure` | Azure OpenAI | 📦 async-openai |
| `provider-bedrock` | AWS Bedrock | 📦 reqwest |
| `provider-gemini` | Google Gemini | 📦 google-cloud-aiplatform |
| `provider-litellm` | LiteLLM: Many agents/models | 📦 async-openai |
| `provider-ollama` | Ollama (Local LLMs) | 📦 reqwest |
| `provider-openai` | OpenAI (GPT-5, etc.) | 📦 async-openai |
| `provider-openrouter` | OpenRouter: Many agents/models | 📦 async-openai |
| `provider-voyage` | Voyage AI | 📦 reqwest |
| `provider-vllm` | vLLM: Many agents | 📦 async-openai |

#### 🔤 Splitter Languages

When using `function-split`, you can enable specific Tree-sitter grammars to reduce binary size.

| Feature | Description |
|---------|-------------|
| `splitter-language-rust` | Rust grammar |
| `splitter-language-python` | Python grammar |
| `splitter-language-javascript` | JavaScript grammar |
| `splitter-language-markdown` | Markdown grammar |
| ... and many more | See [`Cargo.toml`](crates/recoco-core/Cargo.toml) for full list (c, cpp, go, java, etc.) |

#### 🏗️ Capability-based Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `persistence` | SQLx-based state tracking & DB metadata | 📦 sqlx |
| `server` | Axum-based HTTP server components | 📦 axum, tower |
| `json-schema` | JSON Schema generation support | 📦 schemars |

#### 📦 Feature Bundles

| Feature | Description |
|---------|-------------|
| `all-sources` | Enable all source connectors |
| `all-targets` | Enable all target connectors |
| `all-functions` | Enable all transform functions |
| `all-llm-providers` | Enable all LLM providers |
| `all-splitter-languages` | Enable all Tree-sitter grammars |
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
    recoco::lib_context::init_lib_context(Some(recoco::settings::Settings::default())).await?;

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

## ⚙️ Configuration

ReCoco is configured via the `Settings` struct passed to `init_lib_context`.

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Controls logging verbosity (e.g., `info`, `debug`, `recoco=trace`) | `info` |

### Library Settings

The `recoco::settings::Settings` struct controls global behavior:

```rust
use recoco::settings::{Settings, DatabaseConnectionSpec, GlobalExecutionOptions};

let settings = Settings {
    // Database configuration for persisted flows
    database: Some(DatabaseConnectionSpec {
        url: "postgres://user:pass@localhost:5432/recoco_db".to_string(),
        user: Some("user".to_string()),
        password: Some("pass".to_string()),
        max_connections: 10,
        min_connections: 1,
    }),
    
    // Concurrency controls
    global_execution_options: GlobalExecutionOptions {
        source_max_inflight_rows: Some(1000),
        source_max_inflight_bytes: Some(10 * 1024 * 1024), // 10MB
    },
    
    // Other options
    app_namespace: "my_app".to_string(),
    ignore_target_drop_failures: false,
};

recoco::lib_context::init_lib_context(Some(settings)).await?;
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

## 🗺️ Roadmap

- [ ] **WASM Support**: Compile core logic to WASM for edge deployment
- [ ] **More Connectors**: Add support for Redis, ClickHouse, and more
- [ ] **Python Bindings**: Re-introduce optional Python bindings for hybrid workflows
- [ ] **UI Dashboard**: Simple web UI for monitoring flows
- [ ] **Upstream Sync**: Regular merges from upstream CocoIndex

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

Code headers maintain dual copyright (CocoIndex upstream, Knitli Inc. for ReCoco modifications) under Apache-2.0.

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
