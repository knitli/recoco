<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

# recoco-utils

**Common utilities for the [Recoco](https://github.com/knitli/recoco) ecosystem.**

This crate provides shared building blocks used across Recoco's core and operation modules. While primarily intended for internal use within Recoco, these utilities can be useful for developing custom Recoco operations or for standalone use in Rust projects.

## Installation

```toml
[dependencies]
recoco-utils = { version = "0.2", features = ["batching", "fingerprint"] }
```

## 📦 Available Features

`recoco-utils` is highly modular with **no default features** to keep dependencies minimal. Enable only what you need.

### Core Utilities

| Feature | Description | Key Dependencies | Use When |
|---------|-------------|------------------|----------|
| `batching` | Async batch processing with concurrency control | `tokio-util`, `serde` | Building efficient data pipelines with batch operations |
| `bytes_decode` | Smart encoding detection and UTF-8 decoding | `encoding_rs` | Processing files with unknown or mixed encodings |
| `concur_control` | Concurrency limiting and rate control primitives | `tokio` | Managing concurrent operations and backpressure |
| `deserialize` | JSON deserialization helpers with better error messages | `serde`, `serde_json`, `serde_path_to_error` | Parsing JSON with detailed error reporting |
| `fingerprint` | Content hashing (BLAKE3) and fingerprinting | `blake3`, `base64`, `hex` | Change detection, deduplication, caching |
| `immutable` | Immutable data structures (Arc-based collections) | None | Safe concurrent access to shared data |
| `retryable` | Exponential backoff retry logic | `tokio`, `rand`, `time` | Network calls, external APIs, unreliable operations |
| `str_sanitize` | String cleaning and SQL-safe sanitization | `serde`, `sqlx` | Input validation, SQL injection prevention |
| `yaml` | YAML parsing and serialization | `yaml-rust2`, `base64` | Configuration files, structured data |


> [!NOTE] This list isn't exhaustive. It doesn't include features that are intended for recoco-core. The above features cover all functionality of the crate, providing granular by-module access.

## 🛠️ Key Modules & Usage

### Batching

Efficient batch processing with concurrency control:

```rust
use recoco_utils::batching::{Batcher, BatchConfig};

let config = BatchConfig {
    max_batch_size: 100,
    max_wait_ms: 1000,
    max_inflight: 10,
};

let batcher = Batcher::new(config, |batch| async move {
    // Process batch
    Ok(())
}).await?;

batcher.send(item).await?;
```

### Fingerprinting

Content-addressable hashing with BLAKE3:

```rust
use recoco_utils::fingerprint::{fingerprint, Fingerprint};

let hash = fingerprint(b"hello world");
let hex_string = hash.to_hex();
let base64_string = hash.to_base64();
```

### Retry Logic

Exponential backoff for unreliable operations:

```rust
use recoco_utils::retryable::{retry_with_backoff, RetryConfig};

let result = retry_with_backoff(
    || async { 
        // Your operation that might fail
        api_call().await
    },
    RetryConfig {
        max_attempts: 5,
        initial_delay_ms: 100,
        max_delay_ms: 10000,
        backoff_multiplier: 2.0,
    }
).await?;
```

### Concurrency Control

Limit concurrent operations:

```rust
use recoco_utils::concur_control::Semaphore;

let sem = Semaphore::new(10); // Max 10 concurrent operations

let _permit = sem.acquire().await?;
// Do work while holding permit
// Permit is released when dropped
```

### Immutable Collections

Arc-based collections for safe sharing:

```rust
use recoco_utils::immutable::{ImmArcVec, ImmArcMap};

let vec = ImmArcVec::from(vec![1, 2, 3]);
let cloned = vec.clone(); // Cheap Arc clone

let map = ImmArcMap::from([("key", "value")]);
```

### Bytes Decoding

Smart encoding detection:

```rust
use recoco_utils::bytes_decode::decode_bytes_to_string;

let text = decode_bytes_to_string(&bytes)?;
// Automatically detects UTF-8, UTF-16, latin1, etc.
```

## 📊 Feature Dependencies

Some features depend on others. Most are fully independent, except:

- `batching` requires `concur_control`, `fingerprint`, and `retryable`
- `fingerprint` requires `deserialize`

When you enable a feature, its dependencies are automatically enabled.

## 🎯 Common Feature Combinations

### For Data Processing Pipelines
```toml
recoco-utils = { version = "0.2", features = ["batching", "fingerprint", "retryable"] }
```

### For HTTP APIs
```toml
recoco-utils = { version = "0.2", features = ["server", "deserialize", "uuid"] }
```

### For Cloud Storage
```toml
recoco-utils = { version = "0.2", features = ["s3", "azure", "retryable"] }
```

### For Database Operations
```toml
recoco-utils = { version = "0.2", features = ["sqlx", "uuid", "fingerprint"] }
```

### For LLM Applications
```toml
recoco-utils = { version = "0.2", features = ["openai", "batching", "retryable"] }
```

## 🔧 Development

This crate is part of the Recoco workspace. See the [main repository](https://github.com/knitli/recoco) for development guidelines.

## 📄 License

Apache-2.0. See [main repository](https://github.com/knitli/recoco) for details.
