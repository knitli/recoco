# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/claude-code) when working with code in this repository.

## Build and Test Commands

This project is a pure Rust workspace.

### Building

```bash
cargo build              # Build all crates
cargo build --release    # Build in release mode
```

### Testing

```bash
cargo test               # Run unit tests
cargo test --all-features # Run tests with all features enabled
cargo clippy             # Run linter
cargo fmt                # Run formatter
```

## Code Structure

```
recoco/
├── crates/
│   ├── recoco/             # Main crate - core indexing engine
│   │   └── src/
│   │       ├── base/       # Core types: schema, value, spec, json_schema
│   │       ├── builder/    # Flow/pipeline builder logic
│   │       ├── execution/  # Runtime execution: evaluator, indexer
│   │       ├── llm/        # LLM integration
│   │       ├── ops/        # Operations: sources, targets, functions
│   │       ├── service/    # Service layer
│   │       └── setup/      # Setup and configuration
│   ├── recoco_utils/       # General utilities
│   └── recoco_extra_text/  # Text processing utilities
├── examples/               # Rust examples
├── Cargo.toml              # Workspace manifest
└── README.md
```

## Key Concepts

- **ReCoco** is a pure Rust fork of CocoIndex
- **Flows** define data transformation pipelines from sources to targets
- **Operations** (ops) include sources, functions, and targets
- **Features** are used to gate heavy dependencies (s3, postgres, etc.)
