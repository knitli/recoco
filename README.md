<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

<div align="center">
  <img src="assets/recocov2.webp" alt="Recoco Logo" width="200"/>

  # Recoco

  **Incremental ETL and Data Processing Framework for Rust**

  [![Docs Site](https://img.shields.io/badge/docs-docs.knitli.com%2Frecoco-blue)](https://docs.knitli.com/recoco)
  [![Crates.io](https://img.shields.io/crates/v/recoco.svg)](https://crates.io/crates/recoco)
  [![API Docs](https://docs.rs/recoco/badge.svg)](https://docs.rs/recoco)
  [![CI](https://github.com/knitli/recoco/actions/workflows/ci.yml/badge.svg)](https://github.com/knitli/recoco/actions/workflows/ci.yml)
  [![MSRV](https://img.shields.io/badge/MSRV-1.89-blue.svg)](https://blog.rust-lang.org/2025/02/20/Rust-1.89.0.html)
  [![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
  [![REUSE Compliance](https://img.shields.io/badge/reuse-compliant-brightgreen)](https://reuse.software/)

</div>

---

**[Recoco](https://github.com/knitli/recoco)** is a **pure Rust fork** of the excellent **[CocoIndex](https://github.com/cocoindex-io/cocoindex)**, a high-performance, incremental ETL and data processing framework.

> [!TIP]
> **Full documentation — guides, examples, API reference, and more — is at [docs.knitli.com/recoco/](https://docs.knitli.com/recoco/).**

## Why Fork?

I decided to create a Rust-only fork of CocoIndex for a couple reasons:

1. **CocoIndex is not a Rust library.**  CocoIndex is written in Rust, but it does not expose a Rust API and its packaging, documentation, and examples are only focused on *Python*. It exposes a more limited API through its Rust extensions. It's not even released on crates.io.

2. **CocoIndex is heavy.** CocoIndex has several very heavy dependencies and unless you are actually Google, you probably don't need all of them. These include large packages like Google/AWS/Azure components, Qdrant/Postgres/Neo4j, and more.

For [Knitli](https://knitli.com), **I needed dependency control**. I wanted to use CocoIndex as an ETL engine for [Thread](https://github.com/knitli/thread/), but Thread needs to be edge-deployable. The dependencies were way too heavy and would never compile to WASM. Thread, of course, is also a Rust project, so pulling in a lot of Python dependencies didn't make sense for me.

> [!NOTE]
> Knitli and Recoco have no official relationship with CocoIndex and they don't endorse this project. **We will contribute as much as we can upstream**, our [contribution guidelines](CONTRIBUTING.md) encourage you to submit PRs and issues affecting shared code upstream to help both projects.

## How Recoco is Different from CocoIndex

1. **Recoco fully exposes a Rust API.** You can use Recoco to support your Rust ETL projects directly. **Build on it.**

2. **Every target, source, and function (i.e. transform) is independently feature-gated. Use only what you want.**

> The minimum install now uses **600 fewer crates** (820 → 220) — a <ins>~73% reduction from CocoIndex</ins>.

We will regularly merge in upstream fixes and changes, particularly sources, targets, and functions.

## Performance

Recoco and CocoIndex share the same Rust splitting/chunking engine. The difference is how you reach it: CocoIndex routes every call through Python → PyO3 → Rust → PyO3 → Python. Recoco calls the engine directly.

Benchmarks run on the same machine, same data, same operations ([details](benchmarks/)):

| Operation | Input | Recoco | CocoIndex | Speedup |
|-----------|-------|--------|-----------|---------|
| Paragraph split | 1 KB | 924 ns | 176 us | **191x** |
| Line split | 1 KB | 1.9 us | 198 us | **102x** |
| Recursive chunk (prose) | 1 KB | 3.0 us | 155 us | **51x** |
| Paragraph split | 100 KB | 82 us | 376 us | **4.6x** |
| Recursive chunk (prose) | 100 KB | 475 us | 971 us | **2.0x** |
| Recursive chunk (Rust code, tree-sitter) | 100 KB | 15.0 ms | 18.6 ms | **1.2x** |
| Line split | 10 MB | 18.9 ms | 574 ms | **30x** |
| Recursive chunk (Rust code, tree-sitter) | 10 MB | 1.53 s | 1.66 s | **1.1x** |

Small, frequent operations (the kind that happen thousands of times in a real pipeline) show the largest gains because the ~170us PyO3 round-trip overhead dominates. Heavy computation (tree-sitter parsing on large files) converges toward parity since both sides run the same Rust code.

## ✨ Key Features

- 🦀 **Pure Rust**: No Python dependencies, interpreters, or build tools required
- 🎯 **Modular Architecture**: Feature-gated sources, targets, and functions — use only what you need
- ⚡ **Incremental Processing**: Dataflow engine that processes only changed data; tracks lineage automatically
- 🚀 **Additional optimizations**: Faster alternatives where possible (e.g., `blake2` → `blake3`)
- 📦 **Workspace Structure**: Clean separation into `recoco`, `recoco-utils`, and `recoco-splitters` crates
- 🔌 **Rich Connector Ecosystem**: Local Files, PostgreSQL, S3, Azure, Google Drive, Qdrant, Neo4j, Kùzu, and more
- 🌐 **Async API**: Fully async/await compatible, built on Tokio

See the [Core Crate Reference](https://docs.knitli.com/recoco/reference/core-crate/) for a complete list of available features.

## 🎯 Use Cases

- **RAG Pipelines**: Ingest documents, split intelligently, generate embeddings, store in vector databases
- **ETL Workflows**: Extract, transform, and load across multiple data stores
- **Document Processing**: Parse, chunk, and extract information from large document collections
- **Data Synchronization**: Keep data in sync across systems with automatic change detection
- **Custom Pipelines**: Build domain-specific data flows with custom Rust operations

## Installation

Add `recoco` to your `Cargo.toml`, enabling only the features you need:

```toml
[dependencies]
recoco = { version = "0.2", default-features = false, features = ["source-local-file", "function-split"] }
```

For the full list of available features — sources, targets, functions, LLM providers, splitter languages, and capability bundles — see the **[Core Crate Reference](https://docs.knitli.com/recoco/reference/core-crate/)**.

## 🚀 Quick Start

```rust
use recoco::prelude::*;
use recoco::builder::FlowBuilder;
use recoco::execution::evaluator::evaluate_transient_flow;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    recoco::lib_context::init_lib_context(Some(recoco::settings::Settings::default())).await?;

    let mut builder = FlowBuilder::new("hello_world").await?;

    let input = builder.add_direct_input(
        "text".to_string(),
        schema::make_output_type(schema::BasicValueType::Str),
    )?;

    let output = builder.transform(
        "SplitBySeparators".to_string(),
        json!({ "separators_regex": [" "] }).as_object().unwrap().clone(),
        vec![(input, Some("text".to_string()))],
        None,
        "splitter".to_string(),
    ).await?;

    builder.set_direct_output(output)?;

    let flow = builder.build_transient_flow().await?;
    let result = evaluate_transient_flow(
        &flow.0,
        &vec![value::Value::Basic("Hello Recoco".into())]
    ).await?;

    println!("Result: {:?}", result);
    Ok(())
}
```

For step-by-step guidance, custom operations, file processing, and more, visit the **[Getting Started guide](https://docs.knitli.com/recoco/guides/getting-started/)** and **[Examples](https://docs.knitli.com/recoco/examples/transient-flow/)**.

## 🗺️ Roadmap

- [ ] **WASM Support**: Compile core logic to WASM for edge deployment
- [ ] **More Connectors**: Add support for Redis, ClickHouse, and more
- [ ] **UI Dashboard**: Simple web UI for monitoring flows
- [ ] **Upstream Sync**: Regular merges from upstream CocoIndex

## 🔗 Relationship to CocoIndex

Recoco is a fork of [CocoIndex](https://github.com/cocoindex-io/cocoindex):

| Aspect | CocoIndex (Upstream) | Recoco (Fork) |
|--------|---------------------|---------------|
| **Primary Language** | Python with Rust core | Pure Rust |
| **API Surface** | Python-only | Full Rust API |
| **Distribution** | Not on crates.io | Published to crates.io |
| **Dependencies** | All bundled together | Feature-gated and modular |
| **Target Audience** | Python developers | Rust developers |
| **License** | Apache-2.0 | Apache-2.0 |

We aim to maintain compatibility with CocoIndex's core dataflow engine to allow porting upstream improvements, while diverging significantly in the API surface and dependency management to better serve Rust users.

Code headers maintain dual copyright (CocoIndex upstream, Knitli Inc. for Recoco modifications) under Apache-2.0.

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) and the **[Contributing guide](https://docs.knitli.com/recoco/guides/contributing/)** for details.

## 📄 License

[Apache License 2.0](LICENSE); see [NOTICE](NOTICE) for full license text.

This project is [REUSE 3.3 compliant](https://reuse.software/).

## 🙏 Acknowledgments

Recoco is built on the excellent foundation provided by [CocoIndex](https://github.com/cocoindex-io/cocoindex). We're grateful to the CocoIndex team for creating such a powerful and well-designed dataflow engine.

---

<div align="center">

**Built with 🦀 by [Knitli Inc.](https://knit.li)**

[Docs](https://docs.knitli.com/recoco/) • [API Reference](https://docs.rs/recoco) • [Crates.io](https://crates.io/crates/recoco) • [GitHub](https://github.com/knitli/recoco) • [Issues](https://github.com/knitli/recoco/issues)

</div>

