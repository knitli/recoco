---
title: API Documentation
description: Comprehensive API documentation for Recoco crates.
---

Recoco provides detailed API documentation generated from source code comments. The documentation is hosted on docs.rs and covers all public APIs, traits, and types.

## Main Crate Documentation

### Recoco Core

The main `recoco` crate contains the core functionality:

**[docs.rs/recoco](https://docs.rs/recoco)** - Complete API reference for the core library

Includes:
- `FlowBuilder` - Construct data flows
- `execution` module - Runtime evaluation
- `ops` module - Operations (sources, functions, targets)
- `schema` module - Type system
- `value` module - Data values
- `prelude` - Common imports

## Utility Crates

### Recoco Utils

Helper utilities and shared functionality:

**[docs.rs/recoco-utils](https://docs.rs/recoco-utils)** - Utility functions and types

### Recoco Splitters

Text splitting and parsing with Tree-sitter:

**[docs.rs/recoco-splitters](https://docs.rs/recoco-splitters)** - Syntax-aware text splitting

## Quick Links

- **[Browse all Recoco crates](https://docs.rs/releases/search?query=recoco)** - All published versions
- **[GitHub Repository](https://github.com/knitli/recoco)** - Source code and examples
- **[crates.io](https://crates.io/crates/recoco)** - Package information

## Usage

To use Recoco in your project:

```toml
[dependencies]
recoco = "0.2"
```

See the [Getting Started](/recoco/guides/getting-started/) guide for a complete walkthrough.

## Features Documentation

Each feature is documented in the API docs with examples:

- **Sources**: `source-local-file`, `source-postgres`, `source-s3`, `source-azure`, `source-gdrive`
- **Targets**: `target-postgres`, `target-qdrant`, `target-neo4j`, `target-kuzu`
- **Functions**: `function-split`, `function-embed`, `function-extract-llm`, `function-detect-lang`, `function-json`

Check the [Core Crate](/recoco/reference/core-crate/) reference for a complete list of features.
