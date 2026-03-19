// Recoco is a Rust-only fork of CocoIndex, by [CocoIndex](https://CocoIndex)
// Original code from CocoIndex is copyrighted by CocoIndex
// SPDX-FileCopyrightText: 2025-2026 CocoIndex (upstream)
// SPDX-FileContributor: CocoIndex Contributors
//
// All modifications from the upstream for Recoco are copyrighted by Knitli Inc.
// SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// Both the upstream CocoIndex code and the Recoco modifications are licensed under the Apache-2.0 License.
// SPDX-License-Identifier: Apache-2.0

//! Progress Watching Example
//!
//! This example demonstrates how to use the progress watching API to monitor
//! long-running flow executions. The progress API provides real-time visibility
//! into processing status across all components.
//!
//! Run with:
//! ```bash
//! cargo run --example progress_watching --features persistence
//! ```

use recoco::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for better visibility
    tracing_subscriber::fmt::init();

    println!("Progress Watching API Demo");
    println!("===========================\n");

    // Note: This is a conceptual example showing the progress API usage.
    // In a real application, you would:
    // 1. Set up a database for persistence
    // 2. Create a flow with sources and transforms
    // 3. Start a FlowLiveUpdater
    // 4. Monitor progress using the API

    println!("Progress watching provides:");
    println!("  - Per-component statistics (insertions, updates, deletions, errors)");
    println!("  - In-process row counts for each operation");
    println!("  - Total processed and in-process counts");
    println!("  - Real-time updates via async subscriptions\n");

    println!("Example API usage:");
    println!("```rust");
    println!("// Start a live updater for a flow");
    println!("let updater = FlowLiveUpdater::start(");
    println!("    flow_ctx,");
    println!("    &pool,");
    println!("    FlowLiveUpdaterOptions {{");
    println!("        live_mode: false,");
    println!("        print_stats: true,");
    println!("        ..Default::default()");
    println!("    }}");
    println!(").await?;");
    println!();
    println!("// Get a progress snapshot");
    println!("let progress = updater.get_progress_snapshot();");
    println!("println!(\"Total processed: {{}}\", progress.total_processed);");
    println!("println!(\"Total in-process: {{}}\", progress.total_in_process);");
    println!();
    println!("for component in &progress.components {{");
    println!("    println!(\"{{}} - {{}} rows processed, {{}} in-process\",");
    println!("        component.component_name,");
    println!("        component.stats.num_insertions.get(),");
    println!("        component.in_process_count");
    println!("    );");
    println!("}}");
    println!();
    println!("// Subscribe to status updates");
    println!("loop {{");
    println!("    let updates = updater.next_status_updates().await?;");
    println!("    if updates.updated_sources.is_empty() {{");
    println!("        break;");
    println!("    }}");
    println!("    println!(\"Updated sources: {{:?}}\", updates.updated_sources);");
    println!("}}");
    println!();
    println!("// Wait for completion");
    println!("updater.wait().await?;");
    println!("```");

    println!("\nKey types exposed in the public API:");
    println!("  - FlowLiveUpdater: Main entry point for flow execution monitoring");
    println!("  - FlowProgress: Snapshot of overall flow progress");
    println!("  - ComponentProgress: Per-component progress details");
    println!("  - UpdateStats: Detailed statistics (insertions, updates, deletions, etc.)");
    println!("  - OperationInProcessStats: Fine-grained operation tracking");

    Ok(())
}
