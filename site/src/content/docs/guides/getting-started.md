---
title: Getting Started
description: A step-by-step guide to building your first data flow with Recoco.
---

Welcome to Recoco! This guide will walk you through installing Recoco, understanding core concepts, and building your first data processing flow.

## Installation

Add Recoco to your `Cargo.toml`:

```toml
[dependencies]
recoco = { version = "0.2", features = ["function-split"] }
tokio = { version = "1", features = ["full"] }
serde_json = "1"
anyhow = "1"
```

The `function-split` feature enables text splitting functionality. See the [Core Crate](/Recoco/reference/core-crate/) documentation for a complete list of available features.

## Core Concepts

Before building your first flow, let's understand the key concepts:

### Flows

A **Flow** is a data processing pipeline that transforms input data through a series of operations:

```
Sources → Functions → Targets
```

- **Sources**: Ingest data (files, databases, APIs)
- **Functions**: Transform data (split, embed, extract, map)
- **Targets**: Persist results (databases, vector stores)

### Execution Modes

Recoco supports two execution modes:

1. **Transient Flows**: In-memory processing without state persistence
   - Fast and simple
   - Perfect for testing and one-off transformations
   - No database required

2. **Persisted Flows**: Database-backed with incremental updates
   - Tracks data lineage
   - Re-executes only affected computations when data changes
   - Requires database setup

This guide focuses on **transient flows** to get you started quickly.

## Your First Flow

Let's build a simple flow that splits text into words.

### Step 1: Initialize Library Context

Every Recoco application must initialize the library context first:

```rust
use recoco::builder::FlowBuilder;
use recoco::execution::evaluator::evaluate_transient_flow;
use recoco::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the library context
    recoco::lib_context::init_lib_context(
        Some(recoco::settings::Settings::default())
    ).await?;
    
    // Your flow code goes here...
    
    Ok(())
}
```

This loads the operation registry and sets up runtime configuration.

### Step 2: Create a Flow Builder

The `FlowBuilder` constructs your data flow:

```rust
let mut builder = FlowBuilder::new("my_first_flow").await?;
```

### Step 3: Define Input

Specify what data your flow accepts:

```rust
let input = builder.add_direct_input(
    "text_input".to_string(),
    schema::make_output_type(schema::BasicValueType::Str),
)?;
```

This creates an input that accepts string values.

### Step 4: Add Transformations

Apply operations to transform your data:

```rust
let words = builder.transform(
    "SplitBySeparators".to_string(),
    json!({
        "separators_regex": [" "],
        "keep_separator": null,
        "include_empty": false,
        "trim": true
    }).as_object().unwrap().clone(),
    vec![(input, Some("text".to_string()))],
    None,
    "word_splitter".to_string(),
).await?;
```

This uses the `SplitBySeparators` function to split text by spaces.

### Step 5: Set Output

Define what your flow returns:

```rust
builder.set_direct_output(words)?;
```

### Step 6: Build and Execute

Build the flow and run it:

```rust
// Build transient flow
let flow = builder.build_transient_flow().await?;

// Prepare input
let input_text = "Hello World from Recoco";
let input_value = value::Value::Basic(
    value::BasicValue::Str(input_text.into())
);

// Execute
let result = evaluate_transient_flow(&flow.0, &vec![input_value]).await?;

println!("Result: {:?}", result);
```

### Complete Example

Here's the full code:

```rust
use recoco::builder::FlowBuilder;
use recoco::execution::evaluator::evaluate_transient_flow;
use recoco::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize library context
    recoco::lib_context::init_lib_context(
        Some(recoco::settings::Settings::default())
    ).await?;

    // 2. Create builder
    let mut builder = FlowBuilder::new("my_first_flow").await?;

    // 3. Define input
    let input = builder.add_direct_input(
        "text_input".to_string(),
        schema::make_output_type(schema::BasicValueType::Str),
    )?;

    // 4. Add transformation
    let words = builder.transform(
        "SplitBySeparators".to_string(),
        json!({
            "separators_regex": [" "],
            "keep_separator": null,
            "include_empty": false,
            "trim": true
        }).as_object().unwrap().clone(),
        vec![(input, Some("text".to_string()))],
        None,
        "word_splitter".to_string(),
    ).await?;

    // 5. Set output
    builder.set_direct_output(words)?;

    // 6. Build and execute
    let flow = builder.build_transient_flow().await?;
    
    let input_text = "Hello World from Recoco";
    let input_value = value::Value::Basic(
        value::BasicValue::Str(input_text.into())
    );
    
    let result = evaluate_transient_flow(&flow.0, &vec![input_value]).await?;
    
    println!("Result: {:?}", result);

    Ok(())
}
```

Run it with:

```bash
cargo run --features function-split
```

## Common Patterns

### Processing Multiple Items

To process multiple inputs, call the flow in a loop:

```rust
let inputs = vec!["First line", "Second line", "Third line"];

for text in inputs {
    let input_value = value::Value::Basic(
        value::BasicValue::Str(text.into())
    );
    let result = evaluate_transient_flow(&flow.0, &vec![input_value]).await?;
    println!("Processed: {:?}", result);
}
```

### Chaining Operations

You can chain multiple transformations:

```rust
// First transform: split by spaces
let words = builder.transform(
    "SplitBySeparators".to_string(),
    json!({"separators_regex": [" "]}).as_object().unwrap().clone(),
    vec![(input, Some("text".to_string()))],
    None,
    "word_splitter".to_string(),
).await?;

// Second transform: process each word further
let processed = builder.transform(
    "AnotherFunction".to_string(),
    json!({...}).as_object().unwrap().clone(),
    vec![(words, Some("input".to_string()))],
    None,
    "processor".to_string(),
).await?;

builder.set_direct_output(processed)?;
```

## Troubleshooting

### "Operation not found" Error

Make sure you've enabled the required feature for the operation:

```toml
[dependencies]
recoco = { version = "0.2", features = [
    "function-split",    # For SplitBySeparators
    "function-embed",    # For embedding operations
    "function-json",     # For JSON operations
] }
```

### Library Context Not Initialized

Always call `init_lib_context()` before creating any flows:

```rust
recoco::lib_context::init_lib_context(
    Some(recoco::settings::Settings::default())
).await?;
```

### Type Mismatch Errors

Ensure input/output types match between operations. Use the correct `BasicValueType`:

- `BasicValueType::Str` for strings
- `BasicValueType::Int` for integers
- `BasicValueType::Float` for floats
- `BasicValueType::Bool` for booleans

## Next Steps

Now that you've built your first flow, explore:

- **[Examples](/Recoco/examples/transient-flow/)** - More complex examples
- **[Architecture](/Recoco/guides/architecture/)** - Deep dive into Recoco's design
- **[Core Crate](/Recoco/reference/core-crate/)** - Available features and operations
- **[Contributing](/Recoco/guides/contributing/)** - Help improve Recoco

Happy coding! 🚀
