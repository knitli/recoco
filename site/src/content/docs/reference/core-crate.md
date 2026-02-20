---
title: Core Crate
---
<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

# recoco

This is the core package for [Recoco](https://github.com/knitli/recoco), core provides direct access to Recoco's complete functionality. Recoco is a rust-only fork of [`CocoIndex`](https://cocoindex.io)

## When to use this crate

**Use `recoco`** when you want:
- ✅ To use multiple Recoco components together
- ✅ Feature parity with the full Recoco ecosystem
- ✅ Easy dependency management

**Use individual crates** (`recoco-core`, `recoco-utils`, `recoco-splitters`) when you want:
- ⚡ Fine-grained dependency control
- ⚡ Minimal compile times
- ⚡ Only specific functionality (e.g., just utils)

## Installation

```toml
[dependencies]
recoco = { version = "0.2", features = ["function-split", "source-postgres"] }
```

## 📦 Feature Flags

This crate mirrors all features from `recoco-core`. Enable only what you need to keep dependencies minimal.

### 🎯 Default Features

```toml
recoco = "0.2"  # Includes: persistence, server, source-local-file
```

### 📦 Feature Bundles (Convenience)

| Feature | Description | Use When |
|---------|-------------|----------|
| `full` | Everything (⚠️ very heavy) | You need all functionality |
| `all-sources` | All data source connectors | Working with multiple data sources |
| `all-targets` | All data target connectors | Writing to multiple databases |
| `all-functions` | All transformation functions | Need all data processing capabilities |
| `all-llm-providers` | All LLM provider integrations | Working with multiple AI APIs |
| `all-splitter-languages` | All Tree-sitter grammars | Processing many programming languages |

### 📥 Sources (Data Ingestion)

| Feature | Description |
|---------|-------------|
| `source-local-file` | Local filesystem (✅ default) |
| `source-postgres` | PostgreSQL with CDC |
| `source-s3` | Amazon S3 |
| `source-azure` | Azure Blob Storage |
| `source-gdrive` | Google Drive |

### 📤 Targets (Data Persistence)

| Feature | Description |
|---------|-------------|
| `target-postgres` | PostgreSQL database |
| `target-qdrant` | Qdrant vector database |
| `target-neo4j` | Neo4j graph database |
| `target-kuzu` | Kùzu embedded graph database |

### ⚙️ Functions (Data Transformations)

| Feature | Description |
|---------|-------------|
| `function-split` | Text splitting (recursive, semantic) |
| `function-embed` | Generate text embeddings |
| `function-extract-llm` | LLM-based data extraction |
| `function-detect-lang` | Programming language detection |
| `function-json` | JSON/JSON5 parsing |

### 🤖 LLM Providers

| Feature | Provider |
|---------|----------|
| `provider-openai` | OpenAI (GPT-4, etc.) |
| `provider-anthropic` | Anthropic (Claude) |
| `provider-azure` | Azure OpenAI |
| `provider-gemini` | Google Gemini |
| `provider-bedrock` | AWS Bedrock |
| `provider-ollama` | Ollama (local LLMs) |
| `provider-voyage` | Voyage AI (embeddings) |
| `provider-litellm` | LiteLLM (unified gateway) |
| `provider-openrouter` | OpenRouter (multi-provider) |
| `provider-vllm` | vLLM (inference server) |

### 🔤 Splitter Languages (Tree-sitter Grammars)

Enable specific programming language support for code splitting:

| Feature | Language |
|---------|----------|
| `splitter-language-c` | C |
| `splitter-language-c-sharp` | C# |
| `splitter-language-cpp` | C++ |
| `splitter-language-css` | CSS |
| `splitter-language-fortran` | Fortran |
| `splitter-language-go` | Go |
| `splitter-language-html` | HTML |
| `splitter-language-java` | Java |
| `splitter-language-javascript` | JavaScript |
| `splitter-language-json` | JSON |
| `splitter-language-kotlin` | Kotlin |
| `splitter-language-markdown` | Markdown |
| `splitter-language-php` | PHP |
| `splitter-language-python` | Python |
| `splitter-language-r` | R |
| `splitter-language-ruby` | Ruby |
| `splitter-language-rust` | Rust |
| `splitter-language-scala` | Scala |
| `splitter-language-sql` | SQL |
| `splitter-language-swift` | Swift |
| `splitter-language-toml` | TOML |
| `splitter-language-typescript` | TypeScript |
| `splitter-language-xml` | XML |
| `splitter-language-yaml` | YAML |


### 🏗️ Core Features

| Feature | Description |
|---------|-------------|
| `persistence` | Database-backed state tracking (✅ default) |
| `server` | HTTP server components (✅ default) |
| `json-schema` | JSON Schema support |

## 🎯 Common Use Cases

### Local File Processing
```toml
recoco = { version = "0.2", default-features = false, features = [
    "source-local-file",
    "function-split",
    "splitter-language-rust"
]}
```

### RAG Pipeline with OpenAI
```toml
recoco = { version = "0.2", features = [
    "source-s3",
    "function-split",
    "function-embed",
    "provider-openai",
    "target-qdrant",
    "all-splitter-languages"
]}
```

### Database ETL
```toml
recoco = { version = "0.2", features = [
    "source-postgres",
    "target-postgres",
    "function-json"
]}
```

### Multi-Cloud Data Sync
```toml
recoco = { version = "0.2", features = [
    "all-sources",
    "all-targets",
    "batching"
]}
```

### Lightweight Transient Processing (No Database)
```toml
recoco = { version = "0.2", default-features = false, features = [
    "function-split",
    "function-json"
]}
```

## 📚 Documentation

- **Main README**: [../../README.md](../../README.md)
- **API Docs**: [docs.rs/recoco](https://docs.rs/recoco)
- **Examples**: [examples/](../../examples/)
- **recoco-utils**: [../recoco-utils/README.md](../recoco-utils/README.md)

## 🔧 Development

This crate is part of the Recoco workspace:

```bash
# Build with specific features
cargo build -p recoco --features "function-split,source-postgres"

# Test with all features
cargo test -p recoco --features full

# Run examples
cargo run -p recoco --example transient --features function-split
```

## 📄 License

Apache-2.0. See [main repository](https://github.com/knitli/recoco) for details.

