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

use recoco::builder::FlowBuilder;
use recoco::execution::evaluator::evaluate_transient_flow;
use recoco::prelude::*;
use serde_json::json;

/// This example demonstrates how to build and run a transient flow
/// which processes data in-memory without persistent state or side effects.
///
/// Run with: cargo run -p recoco --example transient --features function-split
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize library context (required for registry)
    recoco::lib_context::init_lib_context(Some(recoco::settings::Settings::default())).await?;

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
    // We use the "SplitBySeparators" function which splits a string into a KTable of chunks.
    let split_slice = builder
        .transform(
            "SplitBySeparators".to_string(),
            json!({
                "separators_regex": [" "],
                "keep_separator": null,
                "include_empty": false,
                "trim": true
            })
            .as_object()
            .unwrap()
            .clone(),
            vec![(input_slice, Some("text".to_string()))],
            None,
            "splitter".to_string(),
        )
        .await?;

    // 5. Set output
    // The output of the flow will be the result of the split operation
    builder.set_direct_output(split_slice)?;

    // 6. Build transient flow
    let flow = builder.build_transient_flow().await?;

    // 7. Execute
    let input_text = "Hello World ReCoco";
    let input_value = value::Value::Basic(value::BasicValue::Str(input_text.into()));

    println!("Executing flow with input: '{}'", input_text);
    let result = evaluate_transient_flow(&flow.0, &vec![input_value]).await?;

    println!("Result: {:?}", result);

    Ok(())
}
