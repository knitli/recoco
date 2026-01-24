<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. (ReCoco)
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

# ReCoco

**ReCoco** is a pure Rust fork of the excellent [CocoIndex](https://github.com/cocoindex-io/cocoindex), a high-performance, incremental ETL and data processing framework.

## Why Fork?

I decided to create a Rust-only fork of CocoIndex for a couple reasons:

1. **CocoIndex is not a Rust library.**  CocoIndex is written in Rust, but it does not expose a Rust API and its packaging, documentation, and examples are only focused on *Python*. It exposes a more limited API through its Rust extensions. It's not even released on crates.io.

2. **CocoIndex is heavy.** CocoIndex has several very heavy dependencies that you probably don't need all of, including Google/AWS/Azure components, Qdrant/Postgres/Neo4j, and more. 

## How ReCoco is Different

1. **ReCoco fully exposes a Rust API.** You can use ReCoco to support your rust ETL projects directly. **Build on it.**

2. **Every target, source, and function is independently feature-gated. Use only what you want.**

## Features

- **Pure Rust**: No Python dependencies, interpreters, or build tools required.
- **Incremental Processing**: Built on a dataflow engine that processes only changed data.
- **Modular Architecture**: Feature-gated sources, sinks, and functions.
- **Rich Connector Ecosystem**:
  - **Sources**: Local Files, PostgreSQL, S3, Azure Blob, Google Drive
  - **Targets**: PostgreSQL, Qdrant, Neo4j, Kùzu
  - **Functions**: Text splitting, LLM embedding (OpenAI/Google), JSON parsing, language detection
- **Async API**: Fully async/await compatible API based on Tokio.

## Installation

Add `recoco` to your `Cargo.toml`. Since ReCoco uses a modular feature system, you should enable only the features you need.

```toml
[dependencies]
recoco = { version = "0.1.0", default-features = false, features = ["source-local-file", "function-split"] }
```

### Available Features

| Feature | Description |
|---------|-------------|
| `source-local-file` | Read files from local filesystem (Default) |
| `source-postgres` | Read from PostgreSQL (Change Data Capture) |
| `source-s3` | Read from Amazon S3 |
| `source-azure` | Read from Azure Blob Storage |
| `source-gdrive` | Read from Google Drive |
| `target-postgres` | Write to PostgreSQL |
| `target-qdrant` | Write to Qdrant Vector DB |
| `target-neo4j` | Write to Neo4j Graph DB |
| `target-kuzu` | Write to Kùzu embedded Graph DB |
| `function-split` | Text splitting utilities |
| `function-embed` | Text embedding (OpenAI, Vertex AI) |
| `function-extract-llm` | Information extraction via LLM |
| `function-detect-lang` | Programming language detection |
| `function-json` | JSON parsing (JSON5 support) |

## Quick Start

Here is a simple example that processes a string using a transient flow (in-memory, no persistence):

```rust
use recoco::prelude::*;
use recoco::builder::FlowBuilder;
use recoco::execution::evaluator::evaluate_transient_flow;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize library context
    recoco::lib_context::init_lib_context(None).await?;

    // 2. Create Builder
    let mut builder = FlowBuilder::new("hello_world").await?;

    // 3. Define Input
    let input = builder.add_direct_input(
        "text".to_string(),
        schema::make_output_type(schema::BasicValueType::Str),
    )?;

    // 4. Transform (Split text)
    let output = builder.transform(
        "SplitBySeparators".to_string(),
        json!({ "separators_regex": [" "] }).as_object().unwrap().clone(),
        vec![(input, Some("text".to_string()))],
        None,
        "splitter".to_string(),
    ).await?;

    builder.set_direct_output(output)?;

    // 5. Build & Run
    let flow = builder.build_transient_flow().await?;
    let result = evaluate_transient_flow(
        &flow.0, 
        &vec![value::Value::Basic("Hello ReCoco".into())]
    ).await?;
    
    println!("Result: {:?}", result);
    Ok(())
}
```

## Examples

Check out the `examples/` directory for more usage patterns:

- `transient.rs`: Basic Hello World
- `file_processing.rs`: Line-by-line file processing
- `custom_op.rs`: Defining and registering custom Rust operations
- `detect_lang.rs`: Using built-in functions

Run an example:
```bash
cargo run -p recoco --example file_processing --features function-split
```

## Architecture

ReCoco processes data through **Flows**. A Flow consists of:
- **Sources**: Ingest data (e.g., files, DB rows)
- **Transforms**: Process data (e.g., split, embed, map)
- **Targets**: Store results (e.g., Vector DB, Graph DB)

Data flows through these nodes. ReCoco tracks data lineage, ensuring that when source data changes, only the affected downstream data is recomputed.

## Relationship to CocoIndex

ReCoco is a fork of [CocoIndex](https://github.com/cocoindex/cocoindex).
- **Upstream**: CocoIndex (Python-focused, private Rust core)
- **Downstream**: ReCoco (Rust-focused, public Rust API)

We aim to maintain compatibility with CocoIndex's core dataflow engine to allow porting upstream improvements, while diverging significantly in the API surface and dependency management to serve Rust users better.

## License

[Apache License 2.0](LICENSE); see [NOTICE](NOTICE)
