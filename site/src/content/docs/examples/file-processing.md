---
title: File Processing
description: Process files line-by-line using transient flows.
---

This example demonstrates how to process a file line-by-line using Recoco as a transformation engine, while your application controls the I/O.

## Overview

This pattern is useful for:
- Processing large files incrementally
- Applying transformations to each line independently
- Combining Recoco with custom I/O logic
- Tokenizing or analyzing text files

## Complete Example

```rust
use recoco::builder::FlowBuilder;
use recoco::execution::evaluator::evaluate_transient_flow;
use recoco::prelude::*;
use serde_json::json;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize library context
    recoco::lib_context::init_lib_context(
        Some(recoco::settings::Settings::default())
    ).await?;

    println!("Building file processing flow...");

    // 2. Create FlowBuilder
    let mut builder = FlowBuilder::new("file_processor").await?;

    // 3. Add input: "line" (String)
    let line_input = builder.add_direct_input(
        "line".to_string(),
        schema::make_output_type(schema::BasicValueType::Str),
    )?;

    // 4. Transform: Split line by spaces and punctuation
    // Using "SplitBySeparators" to tokenize the line
    let tokens = builder.transform(
        "SplitBySeparators".to_string(),
        json!({
            "separators_regex": [" ", "\t", "\\.", ","],
            "keep_separator": null,
            "include_empty": false,
            "trim": true
        }).as_object().unwrap().clone(),
        vec![(line_input, Some("text".to_string()))],
        None,
        "tokenizer".to_string(),
    ).await?;

    // 5. Output the tokens
    builder.set_direct_output(tokens)?;

    // 6. Build
    let flow = builder.build_transient_flow().await?;

    // 7. Process a file (using Cargo.toml as sample)
    let file_path = "Cargo.toml";
    println!("Processing file: {}", file_path);

    let file = File::open(file_path).await?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut line_num = 0;

    while reader.read_line(&mut line).await? > 0 {
        line_num += 1;
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            line.clear();
            continue;
        }

        let input_value = value::Value::Basic(
            value::BasicValue::Str(line.clone().into())
        );

        // Execute flow for this line
        let result = evaluate_transient_flow(&flow.0, &vec![input_value]).await?;

        // Inspect result (it should be a KTable of chunks)
        if let value::Value::KTable(chunks) = result {
            let count = chunks.len();
            if count > 0 {
                println!("Line {}: found {} tokens", line_num, count);
                
                // Optional: print first few tokens
                for (k, v) in chunks.iter().take(3) {
                    println!("  - {:?} -> {:?}", k, v);
                }
            }
        }

        line.clear();
    }

    Ok(())
}
```

## Running the Example

**Prerequisites:**

Add to your `Cargo.toml`:

```toml
[dependencies]
recoco = { version = "0.2", features = ["function-split"] }
tokio = { version = "1", features = ["full"] }
serde_json = "1"
anyhow = "1"
```

**Run:**

```bash
cargo run --features function-split
```

## Code Walkthrough

### Flow Setup

The flow is set up once before processing any lines:

```rust
let mut builder = FlowBuilder::new("file_processor").await?;
let line_input = builder.add_direct_input(
    "line".to_string(),
    schema::make_output_type(schema::BasicValueType::Str),
)?;
```

Each line will be passed through this same flow.

### Tokenization Configuration

```rust
let tokens = builder.transform(
    "SplitBySeparators".to_string(),
    json!({
        "separators_regex": [" ", "\t", "\\.", ","],
        "keep_separator": null,
        "include_empty": false,
        "trim": true
    }).as_object().unwrap().clone(),
    vec![(line_input, Some("text".to_string()))],
    None,
    "tokenizer".to_string(),
).await?;
```

This configuration:
- Splits on spaces, tabs, periods, and commas
- Discards separators (`keep_separator: null`)
- Ignores empty tokens
- Trims whitespace from tokens

### File Processing Loop

```rust
let file = File::open(file_path).await?;
let mut reader = BufReader::new(file);
let mut line = String::new();

while reader.read_line(&mut line).await? > 0 {
    // Skip empty lines and comments
    if trimmed.is_empty() || trimmed.starts_with('#') {
        line.clear();
        continue;
    }

    // Process line through flow
    let input_value = value::Value::Basic(
        value::BasicValue::Str(line.clone().into())
    );
    let result = evaluate_transient_flow(&flow.0, &vec![input_value]).await?;
    
    // Handle result...
    
    line.clear();
}
```

Key points:
- Read lines asynchronously with `tokio::io`
- Reuse the same flow for each line (efficient)
- Clear the line buffer after each iteration

## Expected Output

When processing a `Cargo.toml` file:

```
Building file processing flow...
Processing file: Cargo.toml
Line 2: found 3 tokens
  - Key([0]) -> Chunk { text: "name", ... }
  - Key([1]) -> Chunk { text: "recoco", ... }
Line 3: found 3 tokens
  - Key([0]) -> Chunk { text: "version", ... }
  - Key([1]) -> Chunk { text: "0", ... }
  - Key([2]) -> Chunk { text: "2", ... }
Line 4: found 3 tokens
  - Key([0]) -> Chunk { text: "edition", ... }
...
```

## Use Cases

### Log File Analysis

Process log files to extract patterns:

```rust
let tokens = builder.transform(
    "SplitBySeparators".to_string(),
    json!({
        "separators_regex": ["\\[", "\\]", " "],
        "keep_separator": null,
        "include_empty": false,
        "trim": true
    }).as_object().unwrap().clone(),
    vec![(line_input, Some("text".to_string()))],
    None,
    "log_parser".to_string(),
).await?;
```

### CSV Processing

Parse CSV files (simple case):

```rust
let fields = builder.transform(
    "SplitBySeparators".to_string(),
    json!({
        "separators_regex": [","],
        "keep_separator": null,
        "include_empty": true,  // Keep empty fields
        "trim": true
    }).as_object().unwrap().clone(),
    vec![(line_input, Some("text".to_string()))],
    None,
    "csv_splitter".to_string(),
).await?;
```

## Key Takeaways

- ✅ **Reuse flow for efficiency** - Build once, execute many times
- ✅ **You control I/O** - Recoco handles transformation logic
- ✅ **Async I/O friendly** - Works seamlessly with Tokio
- ✅ **Memory efficient** - Process one line at a time

## Next Steps

- Explore **[Custom Operations](/recoco/examples/custom-operation/)** to add domain-specific logic
- Learn about **[Language Detection](/recoco/examples/language-detection/)** for code files
- Try **[Transient Flow](/recoco/examples/transient-flow/)** for simpler examples

## Source Code

The complete source code for this example is available in the Recoco repository:
[`crates/recoco-core/examples/file_processing.rs`](https://github.com/knitli/recoco/blob/main/crates/recoco-core/examples/file_processing.rs)
