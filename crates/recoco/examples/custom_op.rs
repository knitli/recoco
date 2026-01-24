// ReCoco is a Rust-only fork of CocoIndex, by [CocoIndex](https://CocoIndex)
// Original code from CocoIndex is copyrighted by CocoIndex
// SPDX-FileCopyrightText: 2025-2026 CocoIndex (upstream)
// SPDX-FileContributor: CocoIndex Contributors
//
// All modifications from the upstream for ReCoco are copyrighted by Knitli Inc.
// SPDX-FileCopyrightText: 2026 Knitli Inc. (ReCoco)
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// Both the upstream CocoIndex code and the ReCoco modifications are licensed under the Apache-2.0 License.
// SPDX-License-Identifier: Apache-2.0

use recoco::prelude::*;
use recoco::builder::FlowBuilder;
use recoco::execution::evaluator::evaluate_transient_flow;
use recoco::ops::sdk::*;
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
    type ResolvedArgs = (); // We don't need to store resolved args for this simple op

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

/// This example demonstrates how to register and use a custom Rust operation in ReCoco.
/// Run: cargo run -p recoco --example custom_op
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize library
    recoco::lib_context::init_lib_context(Some(recoco::settings::Settings::default())).await?;

    // 3. Register the custom factory
    println!("Registering custom operation 'ReverseString'...");
    recoco::ops::register_factory(
        "ReverseString".to_string(),
        ExecutorFactory::SimpleFunction(Arc::new(ReverseStringFactory)),
    )?;

    // 4. Build flow using the custom op
    let mut builder = FlowBuilder::new("custom_op_flow").await?;
    
    let text_input = builder.add_direct_input(
        "text".to_string(), 
        schema::make_output_type(schema::BasicValueType::Str)
    )?;
    
    let reversed = builder.transform(
        "ReverseString".to_string(),
        json!({}).as_object().unwrap().clone(),
        vec![(text_input, Some("text".to_string()))],
        None,
        "reverser".to_string()
    ).await?;

    builder.set_direct_output(reversed)?;
    let flow = builder.build_transient_flow().await?;

    // 5. Execute
    let input_text = "ReCoco is Awesome";
    println!("Input:  {}", input_text);
    
    let input_val = value::Value::Basic(value::BasicValue::Str(input_text.into()));
    let result = evaluate_transient_flow(&flow.0, &vec![input_val]).await?;

    println!("Output: {:?}", result);

    // Verify
    if let value::Value::Basic(value::BasicValue::Str(s)) = result {
        assert_eq!(s.as_ref(), "emosewA si ocoCeR");
        println!("Verification successful!");
    }

    Ok(())
}
