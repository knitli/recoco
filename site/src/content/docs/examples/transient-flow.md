---
title: Transient Flow
description: Build a simple in-memory data flow without persistence.
---

This example demonstrates how to build and run a **transient flow** that processes data in-memory without persistent state or database requirements.

## Overview

A transient flow is perfect for:
- Quick data transformations
- Testing and prototyping
- Single-run operations
- Applications where you control the data lifecycle

## Complete Example

```rust
use recoco::builder::FlowBuilder;
use recoco::execution::evaluator::evaluate_transient_flow;
use recoco::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize library context (required for registry)
    recoco::lib_context::init_lib_context(
        Some(recoco::settings::Settings::default())
    ).await?;

    println!("Building transient flow...");

    // 2. Create FlowBuilder
    let mut builder = FlowBuilder::new("transient_example").await?;

    // 3. Add direct input
    // We define an input field "text_input" of type String
    let input_slice = builder.add_direct_input(
        "text_input".to_string(),
        schema::make_output_type(schema::BasicValueType::Str),
    )?;

    // 4. Transform: Split text by spaces
    // We use the "SplitBySeparators" function which splits a string 
    // into a KTable of chunks.
    let split_slice = builder.transform(
        "SplitBySeparators".to_string(),
        json!({
            "separators_regex": [" "],
            "keep_separator": null,
            "include_empty": false,
            "trim": true
        }).as_object().unwrap().clone(),
        vec![(input_slice, Some("text".to_string()))],
        None,
        "splitter".to_string(),
    ).await?;

    // 5. Set output
    // The output of the flow will be the result of the split operation
    builder.set_direct_output(split_slice)?;

    // 6. Build transient flow
    let flow = builder.build_transient_flow().await?;

    // 7. Execute
    let input_text = "Hello World ReCoco";
    let input_value = value::Value::Basic(
        value::BasicValue::Str(input_text.into())
    );

    println!("Executing flow with input: '{}'", input_text);
    let result = evaluate_transient_flow(&flow.0, &vec![input_value]).await?;

    println!("Result: {:?}", result);

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

### 1. Library Context Initialization

```rust
recoco::lib_context::init_lib_context(
    Some(recoco::settings::Settings::default())
).await?;
```

This loads the operation registry with all compiled-in operations. **Required before creating any flows.**

### 2. Create Flow Builder

```rust
let mut builder = FlowBuilder::new("transient_example").await?;
```

The flow builder is used to construct your data processing pipeline. The name is used for debugging and logging.

### 3. Define Input Schema

```rust
let input_slice = builder.add_direct_input(
    "text_input".to_string(),
    schema::make_output_type(schema::BasicValueType::Str),
)?;
```

This defines what data the flow accepts. In this case, a single string input.

### 4. Add Transformation

```rust
let split_slice = builder.transform(
    "SplitBySeparators".to_string(),
    json!({
        "separators_regex": [" "],
        "keep_separator": null,
        "include_empty": false,
        "trim": true
    }).as_object().unwrap().clone(),
    vec![(input_slice, Some("text".to_string()))],
    None,
    "splitter".to_string(),
).await?;
```

Key parameters:
- **Operation name**: `"SplitBySeparators"` - the function to use
- **Configuration**: JSON object with function-specific parameters
- **Inputs**: The input slice(s) this operation consumes
- **Step name**: `"splitter"` - identifier for this operation in the flow

### 5. Set Output

```rust
builder.set_direct_output(split_slice)?;
```

Defines what the flow returns - in this case, the split chunks.

### 6. Build & Execute

```rust
let flow = builder.build_transient_flow().await?;
let result = evaluate_transient_flow(&flow.0, &vec![input_value]).await?;
```

- `build_transient_flow()` - Creates an in-memory flow (no database)
- `evaluate_transient_flow()` - Executes the flow with provided inputs

## Expected Output

```
Building transient flow...
Executing flow with input: 'Hello World ReCoco'
Result: KTable([
    (Key([0]), Chunk { text: "Hello", ... }),
    (Key([1]), Chunk { text: "World", ... }),
    (Key([2]), Chunk { text: "ReCoco", ... })
])
```

The result is a `KTable` (keyed table) where each entry represents a word from the input.

## Key Takeaways

- ✅ **No database required** - Everything runs in-memory
- ✅ **Perfect for prototyping** - Quick iteration and testing
- ✅ **Stateless** - Each execution is independent
- ✅ **Fast** - No I/O overhead from persistence

## Next Steps

- Try **[File Processing](/ReCoco/examples/file-processing/)** to process files line-by-line
- Learn about **[Custom Operations](/ReCoco/examples/custom-operation/)** to extend ReCoco
- Explore **[Persisted Flows](/ReCoco/guides/architecture/#two-flow-execution-modes)** for incremental processing

## Source Code

The complete source code for this example is available in the ReCoco repository:
[`crates/recoco-core/examples/transient.rs`](https://github.com/knitli/recoco/blob/main/crates/recoco-core/examples/transient.rs)
