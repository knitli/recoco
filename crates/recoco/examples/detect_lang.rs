// ReCoco is a Rust-only fork of CocoIndex, by [CocoIndex.io](https://cocoindex.io)
// Original code from CocoIndex is copyrighted by CocoIndex.io
// SPDX-FileCopyrightText: 2025-2026 CocoIndex.io (upstream)
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

/// Example: Detect programming language from filename using ReCoco
/// Run: cargo run -p recoco --example detect_lang --features function-detect-lang
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    recoco::lib_context::init_lib_context(Some(recoco::settings::Settings::default())).await?;

    println!("Building language detection flow...");
    let mut builder = FlowBuilder::new("lang_detector").await?;
    
    // Input: filename (String)
    let filename_input = builder.add_direct_input(
        "filename".to_string(), 
        schema::make_output_type(schema::BasicValueType::Str)
    )?;
    
    // Transform: Detect language
    let lang = builder.transform(
        "DetectProgrammingLanguage".to_string(),
        json!({}).as_object().unwrap().clone(),
        vec![(filename_input, Some("filename".to_string()))],
        None,
        "detector".to_string()
    ).await?;

    builder.set_direct_output(lang)?;
    
    let flow = builder.build_transient_flow().await?;

    // Test with various filenames
    let files = vec!["main.rs", "script.py", "index.ts", "style.css", "unknown.xyz", "pipeline.yaml"];
    
    println!("{:<15} | {:<15}", "Filename", "Language");
    println!("{:-<15}-|-{:-<15}", "", "");

    for f in files {
        let input = value::Value::Basic(value::BasicValue::Str(f.into()));
        let res = evaluate_transient_flow(&flow.0, &vec![input]).await?;
        
        let lang_str = match res {
            value::Value::Basic(value::BasicValue::Str(s)) => s.to_string(),
            value::Value::Null => "unknown".to_string(),
            _ => format!("{:?}", res),
        };
        
        println!("{:<15} | {:<15}", f, lang_str);
    }

    Ok(())
}
