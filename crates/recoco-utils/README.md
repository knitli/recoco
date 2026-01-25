<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. (ReCoco)
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

# ReCoco Utils

**Common utilities for the [ReCoco](https://github.com/knitli/recoco) ecosystem.**

This crate provides shared building blocks used across ReCoco's core and operation modules. While primarily intended for internal use within ReCoco, these utilities can be useful for developing custom ReCoco operations.

## 📦 Features

`recoco-utils` is highly modular to keep dependencies light.

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `batching` | Async batch processing utilities | `tokio-util` |
| `bytes_decode` | Smart encoding detection and decoding | `encoding_rs` |
| `concur_control` | Concurrency control and rate limiting | - |
| `db` | Database helpers (SQLx) | `sqlx` |
| `http` | HTTP client/server utilities | `reqwest`, `axum` |
| `azure` | Azure Storage utilities | `azure_storage` |
| `s3` | S3 compatibility utilities | `globset` |
| `google-drive` | Google Drive utilities | `google-drive3` |
| `openai` | OpenAI API helpers | `async-openai` |
| `qdrant` | Qdrant client helpers | `qdrant-client` |
| `neo4rs` | Neo4j driver helpers | `neo4rs` |
| `redis` | Redis client helpers | `redis` |
| `yaml` | YAML processing | `yaml-rust2` |

## 🛠️ Key Modules

- **`concur_control`**: Primitives for managing concurrency in ETL pipelines.
- **`retryable`**: Robust retry logic for network operations.
- **`fingerprint`**: Hashing and identity utilities for change tracking.
- **`str_sanitize`**: String cleaning and sanitization.
- **`immutable`**: Immutable data structures for safe sharing.

## 📄 License

Apache-2.0. See [main repository](https://github.com/knitli/recoco) for details.
