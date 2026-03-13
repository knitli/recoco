---
title: Configuration
description: Configure Recoco's runtime settings, database connections, and environment variables.
---

<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

Configure Recoco via the `Settings` struct passed to `init_lib_context`, and through environment variables.

## Library Initialization

Every Recoco application must initialize the library context before creating any flows:

```rust
use recoco::settings::Settings;

recoco::lib_context::init_lib_context(Some(Settings::default())).await?;
```

Call this **once** per application, typically at the start of `main()`. It loads the operation registry, sets up authentication registries, and applies your configuration.

## The `Settings` Struct

`recoco::settings::Settings` controls global behavior:

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

    // Optional: store internal tracking tables in a dedicated schema
    // instead of the connection's default schema (often `public`)
    db_schema_name: Some("recoco_state".to_string()),

    // Concurrency controls
    global_execution_options: GlobalExecutionOptions {
        source_max_inflight_rows: Some(1000),
        source_max_inflight_bytes: Some(10 * 1024 * 1024), // 10MB
    },

    // Application namespace (used to prefix database tables)
    app_namespace: "my_app".to_string(),

    // Whether to silently ignore errors when dropping targets during teardown
    ignore_target_drop_failures: false,
};

recoco::lib_context::init_lib_context(Some(settings)).await?;
```

### `DatabaseConnectionSpec`

Required when using **persisted flows** (requires the `persistence` feature).

| Field | Type | Description |
|-------|------|-------------|
| `url` | `String` | PostgreSQL connection URL |
| `user` | `Option<String>` | Override the username from the URL |
| `password` | `Option<String>` | Override the password from the URL |
| `max_connections` | `u32` | Maximum connection pool size |
| `min_connections` | `u32` | Minimum idle connections to keep open |

### `GlobalExecutionOptions`

Controls concurrency and backpressure during flow execution.

| Field | Type | Description |
|-------|------|-------------|
| `source_max_inflight_rows` | `Option<usize>` | Max rows in-flight from all sources at once |
| `source_max_inflight_bytes` | `Option<usize>` | Max bytes in-flight from all sources at once |

### Other Settings Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `db_schema_name` | `Option<String>` | `None` | PostgreSQL schema for internal Recoco tracking/metadata tables. When set, a schema is auto-created and all internal tables are placed there, keeping them separate from application tables. When unset, the connection's default schema (often `public`, but determined by `search_path`) is used. |
| `app_namespace` | `String` | `""` | Namespace prefix for database tables and keys (empty string means no prefix) |
| `ignore_target_drop_failures` | `bool` | `false` | Suppress errors when dropping target tables during teardown |

### `db_schema_name` Detail

The `db_schema_name` field places all Recoco-internal tables (e.g., `cocoindex_setup_metadata`, `<flow>__cocoindex_tracking`) into a dedicated PostgreSQL schema:

```rust
let settings = Settings {
    db_schema_name: Some("recoco_state".to_string()),
    ..Default::default()
};
```

- The schema is automatically created (`CREATE SCHEMA IF NOT EXISTS`) when Recoco sets up for the first time.
- Keeping internal tables in their own schema avoids name collisions with application tables and simplifies DB hygiene in multi-tenant or shared-database environments.
- When `db_schema_name` is `None` (the default), tables are created in the connection's default schema (the first entry in `search_path`, often `public`).

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Logging verbosity filter (e.g., `info`, `debug`, `recoco=trace`) | `info` |
| `DATABASE_URL` | PostgreSQL connection URL your application can read to populate `Settings::database` (Recoco does not read this automatically) | — |

### Logging Examples

```bash
# Enable info-level logging for all crates
RUST_LOG=info cargo run

# Verbose Recoco logging, quiet everything else
RUST_LOG=recoco=debug,warn cargo run

# Trace-level for a specific module
RUST_LOG=recoco::execution=trace cargo run
```

## Transient vs. Persisted Flows

### Transient (no database needed)

```rust
use recoco::builder::FlowBuilder;
use recoco::execution::evaluator::evaluate_transient_flow;
use recoco::prelude::*;
use recoco::settings::Settings;

// No database configuration required
recoco::lib_context::init_lib_context(Some(Settings::default())).await?;

let mut builder = FlowBuilder::new("my_flow").await?;
// ... add inputs and transforms ...
let flow = builder.build_transient_flow().await?;

let inputs = vec![value::Value::Basic("hello world".into())];
let result = evaluate_transient_flow(&flow.0, &inputs).await?;
```

### Persisted (requires `persistence` feature and a database)

```toml
[dependencies]
recoco = { version = "0.2", features = ["persistence", "source-local-file"] }
```

```rust
use recoco::builder::FlowBuilder;
use recoco::settings::{Settings, DatabaseConnectionSpec};

let settings = Settings {
    database: Some(DatabaseConnectionSpec {
        url: std::env::var("DATABASE_URL")?,
        user: None,
        password: None,
        max_connections: 10,
        min_connections: 1,
    }),
    ..Default::default()
};

recoco::lib_context::init_lib_context(Some(settings)).await?;

let mut builder = FlowBuilder::new("my_flow").await?;
// ... add inputs and transforms ...
let flow = builder.build_flow().await?;
```

## Next Steps

- [Getting Started](/recoco/guides/getting-started/) — Build your first flow
- [Architecture](/recoco/guides/architecture/) — How Recoco's dataflow engine works
- [Core Crate Reference](/recoco/reference/core-crate/) — Available features and operations
