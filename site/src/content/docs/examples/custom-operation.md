---
title: Custom Operation
description: Extend Recoco by creating custom operations.
---

This example demonstrates how to create a custom operation that you can use in your flows. You'll build a simple `ReverseString` function that reverses text.

## Overview

Custom operations allow you to:
- Implement domain-specific logic
- Integrate with external services
- Extend Recoco's functionality
- Reuse complex transformations across flows

## Complete Example

```rust
use recoco::builder::FlowBuilder;
use recoco::execution::evaluator::evaluate_transient_flow;
use recoco::ops::sdk::*;
use recoco::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

// 1. Define the Executor
pub struct ReverseStringExecutor;

#[async_trait]
impl SimpleFunctionExecutor for ReverseStringExecutor {
    async fn evaluate(&self, input: Vec<value::Value>) -> Result<value::Value> {
        // We expect one argument: string
        let val = &input[0];
        let s = val.as_str()?;
        let reversed: String = s.chars().rev().collect();
        Ok(value::Value::Basic(value::BasicValue::Str(reversed.into())))
    }
}

// 2. Define the Factory
pub struct ReverseStringFactory;

#[derive(Deserialize, Serialize)]
pub struct EmptySpec {}

#[async_trait]
impl SimpleFunctionFactoryBase for ReverseStringFactory {
    type Spec = EmptySpec;
    type ResolvedArgs = (); // No need to store args for this simple op

    fn name(&self) -> &str {
        "ReverseString"
    }

    async fn analyze<'a>(
        &'a self,
        _spec: &'a Self::Spec,
        args_resolver: &mut OpArgsResolver<'a>,
        _context: &FlowInstanceContext,
    ) -> Result<SimpleFunctionAnalysisOutput<Self::ResolvedArgs>> {
        // Define arguments: one required string argument "text"
        args_resolver
            .next_arg("text")?
            .expect_type(&ValueType::Basic(BasicValueType::Str))?
            .required()?;

        let output_schema = schema::make_output_type(schema::BasicValueType::Str);

        Ok(SimpleFunctionAnalysisOutput {
            resolved_args: (),
            output_schema,
            behavior_version: None,
        })
    }

    async fn build_executor(
        self: Arc<Self>,
        _spec: Self::Spec,
        _resolved_args: Self::ResolvedArgs,
        _context: Arc<FlowInstanceContext>,
    ) -> Result<impl SimpleFunctionExecutor> {
        Ok(ReverseStringExecutor)
    }
}

// 3. Use the custom operation in a flow
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize library context
    recoco::lib_context::init_lib_context(
        Some(recoco::settings::Settings::default())
    ).await?;

    // Register the custom operation
    let mut registry = recoco::ops::registry::OPERATION_REGISTRY.write().await;
    registry.register_simple_function(Arc::new(ReverseStringFactory));
    drop(registry); // Release lock

    // Build flow using the custom operation
    let mut builder = FlowBuilder::new("custom_op_example").await?;

    let input = builder.add_direct_input(
        "text".to_string(),
        schema::make_output_type(schema::BasicValueType::Str),
    )?;

    // Use our custom "ReverseString" operation
    let reversed = builder.transform(
        "ReverseString".to_string(),
        json!({}).as_object().unwrap().clone(),
        vec![(input, Some("text".to_string()))],
        None,
        "reverse".to_string(),
    ).await?;

    builder.set_direct_output(reversed)?;

    // Execute
    let flow = builder.build_transient_flow().await?;
    let input_text = "Hello Recoco!";
    let input_value = value::Value::Basic(
        value::BasicValue::Str(input_text.into())
    );

    println!("Input: {}", input_text);
    let result = evaluate_transient_flow(&flow.0, &vec![input_value]).await?;
    println!("Output: {:?}", result);

    Ok(())
}
```

## Running the Example

**Prerequisites:**

Add to your `Cargo.toml`:

```toml
[dependencies]
recoco = { version = "0.2" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
async-trait = "0.1"
```

**Run:**

```bash
cargo run
```

## Code Walkthrough

### 1. The Executor

The executor implements the actual runtime logic:

```rust
pub struct ReverseStringExecutor;

#[async_trait]
impl SimpleFunctionExecutor for ReverseStringExecutor {
    async fn evaluate(&self, input: Vec<value::Value>) -> Result<value::Value> {
        let val = &input[0];
        let s = val.as_str()?;
        let reversed: String = s.chars().rev().collect();
        Ok(value::Value::Basic(value::BasicValue::Str(reversed.into())))
    }
}
```

Key points:
- Receives a `Vec<value::Value>` with input arguments
- Returns a `Result<value::Value>`
- Can be `async` for I/O operations
- Errors propagate automatically

### 2. The Factory

The factory analyzes and constructs the operation:

```rust
#[async_trait]
impl SimpleFunctionFactoryBase for ReverseStringFactory {
    type Spec = EmptySpec;  // Configuration (empty for this example)
    type ResolvedArgs = (); // Analyzed argument info

    fn name(&self) -> &str {
        "ReverseString"  // Operation name used in flows
    }

    async fn analyze<'a>(
        &'a self,
        _spec: &'a Self::Spec,
        args_resolver: &mut OpArgsResolver<'a>,
        _context: &FlowInstanceContext,
    ) -> Result<SimpleFunctionAnalysisOutput<Self::ResolvedArgs>> {
        // Define expected arguments
        args_resolver
            .next_arg("text")?
            .expect_type(&ValueType::Basic(BasicValueType::Str))?
            .required()?;

        // Define output type
        let output_schema = schema::make_output_type(schema::BasicValueType::Str);

        Ok(SimpleFunctionAnalysisOutput {
            resolved_args: (),
            output_schema,
            behavior_version: None,
        })
    }

    async fn build_executor(
        self: Arc<Self>,
        _spec: Self::Spec,
        _resolved_args: Self::ResolvedArgs,
        _context: Arc<FlowInstanceContext>,
    ) -> Result<impl SimpleFunctionExecutor> {
        Ok(ReverseStringExecutor)
    }
}
```

The `analyze` method:
- Validates input types at flow build time
- Defines output schema
- Enables type checking before execution

### 3. Registration

Register your operation before use:

```rust
let mut registry = recoco::ops::registry::OPERATION_REGISTRY.write().await;
registry.register_simple_function(Arc::new(ReverseStringFactory));
drop(registry); // Release lock
```

This makes your operation available by name in `builder.transform()`.

### 4. Using in a Flow

Once registered, use it like any built-in operation:

```rust
let reversed = builder.transform(
    "ReverseString".to_string(),
    json!({}).as_object().unwrap().clone(),  // Configuration (empty)
    vec![(input, Some("text".to_string()))],  // Input binding
    None,
    "reverse".to_string(),  // Step name
).await?;
```

## Expected Output

```
Input: Hello Recoco!
Output: Basic(Str("!ocoCeR olleH"))
```

## Advanced: Operation with Configuration

For operations that need configuration:

```rust
#[derive(Deserialize, Serialize)]
pub struct RepeatSpec {
    times: u32,
}

pub struct RepeatStringFactory;

#[async_trait]
impl SimpleFunctionFactoryBase for RepeatStringFactory {
    type Spec = RepeatSpec;
    type ResolvedArgs = ();

    fn name(&self) -> &str {
        "RepeatString"
    }

    async fn analyze<'a>(
        &'a self,
        spec: &'a Self::Spec,
        args_resolver: &mut OpArgsResolver<'a>,
        _context: &FlowInstanceContext,
    ) -> Result<SimpleFunctionAnalysisOutput<Self::ResolvedArgs>> {
        args_resolver
            .next_arg("text")?
            .expect_type(&ValueType::Basic(BasicValueType::Str))?
            .required()?;

        // Validate spec
        if spec.times == 0 {
            return Err(anyhow::anyhow!("times must be > 0"));
        }

        let output_schema = schema::make_output_type(schema::BasicValueType::Str);

        Ok(SimpleFunctionAnalysisOutput {
            resolved_args: (),
            output_schema,
            behavior_version: None,
        })
    }

    async fn build_executor(
        self: Arc<Self>,
        spec: Self::Spec,
        _resolved_args: Self::ResolvedArgs,
        _context: Arc<FlowInstanceContext>,
    ) -> Result<impl SimpleFunctionExecutor> {
        Ok(RepeatStringExecutor { times: spec.times })
    }
}

pub struct RepeatStringExecutor {
    times: u32,
}

#[async_trait]
impl SimpleFunctionExecutor for RepeatStringExecutor {
    async fn evaluate(&self, input: Vec<value::Value>) -> Result<value::Value> {
        let s = input[0].as_str()?;
        let repeated = s.repeat(self.times as usize);
        Ok(value::Value::Basic(value::BasicValue::Str(repeated.into())))
    }
}
```

Use with configuration:

```rust
let repeated = builder.transform(
    "RepeatString".to_string(),
    json!({ "times": 3 }).as_object().unwrap().clone(),
    vec![(input, Some("text".to_string()))],
    None,
    "repeater".to_string(),
).await?;
```

## Key Takeaways

- ✅ **Two components**: Executor (runtime) + Factory (analysis)
- ✅ **Type safety**: Validate inputs at build time
- ✅ **Reusable**: Register once, use in multiple flows
- ✅ **Composable**: Combine with built-in operations

## Next Steps

- Review **[Architecture](/Recoco/guides/architecture/#custom-operation-pattern)** for design patterns
- Explore **[SDK Documentation](/Recoco/reference/core-crate/)** for available traits
- Try **[File Processing](/Recoco/examples/file-processing/)** to combine custom ops with I/O

## Source Code

The complete source code for this example is available in the Recoco repository:
[`crates/recoco-core/examples/custom_op.rs`](https://github.com/knitli/recoco/blob/main/crates/recoco-core/examples/custom_op.rs)
