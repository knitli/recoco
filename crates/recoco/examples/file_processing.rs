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
use serde_json::json;
use tokio::io::AsyncBufReadExt;
use tokio::fs::File;
use tokio::io::BufReader;

/// This example demonstrates how to process a file line-by-line using a transient flow.
/// The application controls the I/O, using ReCoco as a transformation engine.
///
/// Run with: cargo run -p recoco --example file_processing --features function-split
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize library context
    recoco::lib_context::init_lib_context(Some(recoco::settings::Settings::default())).await?;

    println!("Building file processing flow...");

    // 2. Create FlowBuilder
    let mut builder = FlowBuilder::new("file_processor").await?;

    // 3. Add input: "line" (String)
    let line_input = builder.add_direct_input(
        "line".to_string(),
        schema::make_output_type(schema::BasicValueType::Str),
    )?;

    // 4. Transform: Split line by spaces
    // Using "SplitBySeparators" to tokenise the line
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
    let file_path = "crates/recoco/Cargo.toml";
    println!("Processing file: {}", file_path);

    let file = File::open(file_path).await?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut line_num = 0;

    while reader.read_line(&mut line).await? > 0 {
        line_num += 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            line.clear();
            continue;
        }

        let input_value = value::Value::Basic(value::BasicValue::Str(line.clone().into()));
        
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
