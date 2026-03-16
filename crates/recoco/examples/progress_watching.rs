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

//! # Progress Watching API Example
//!
//! This example demonstrates how to use Recoco's progress watching API to monitor
//! long-running indexing operations. The progress API provides real-time visibility
//! into data processing pipelines through:
//!
//! - **UpdateStats**: Per-source statistics tracking insertions, updates, deletions, and errors
//! - **OperationInProcessStats**: Granular per-operation tracking of in-flight rows
//! - **FlowLiveUpdater**: Subscribe to progress updates during flow execution
//!
//! ## Features Required
//!
//! This example requires the `persistence` feature to enable progress tracking:
//! ```bash
//! cargo run -p recoco --example progress_watching --features persistence
//! ```
//!
//! ## Progress Watching API Overview
//!
//! ### FlowLiveUpdater Methods
//!
//! - `next_status_updates()`: Async method that blocks until new progress updates are available
//! - `index_update_info()`: Get a snapshot of current statistics for all sources
//! - `operation_in_process_stats`: Access per-operation in-process counts
//! - `wait()`: Wait for all indexing tasks to complete
//!
//! ### UpdateStats Fields
//!
//! - `num_insertions`: Counter for newly inserted rows
//! - `num_updates`: Counter for updated rows
//! - `num_deletions`: Counter for deleted rows
//! - `num_reprocesses`: Counter for reprocessed rows (logic changes)
//! - `num_no_change`: Counter for unchanged rows
//! - `num_errors`: Counter for errors encountered
//! - `processing`: ProcessingCounters tracking in-flight rows
//!
//! ### OperationInProcessStats Methods
//!
//! - `get_operation_in_process_count(name)`: Get in-process count for a specific operation
//! - `get_all_operations_in_process()`: Get snapshot of all operations
//! - `get_total_in_process_count()`: Get total in-process rows across all operations

#[cfg(feature = "persistence")]
use recoco::prelude::*;

#[cfg(feature = "persistence")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Note: In a real application, you would set up tracing with tracing_subscriber
    // tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    println!("=== Recoco Progress Watching API Demo ===\n");
    println!("This example demonstrates the progress watching API.");
    println!("NOTE: This is a demonstration of the API structure.\n");
    println!("In a real application, you would:");
    println!("  1. Set up a database connection pool");
    println!("  2. Create a persisted flow with sources");
    println!("  3. Start the FlowLiveUpdater");
    println!("  4. Subscribe to progress updates\n");

    // Example 1: Working with UpdateStats directly
    println!("--- Example 1: UpdateStats Structure ---");
    let update_stats = stats::UpdateStats::default();

    // Simulate some processing
    update_stats.processing.start(100);
    update_stats.num_insertions.inc(45);
    update_stats.num_updates.inc(30);
    update_stats.num_errors.inc(2);
    update_stats.processing.end(77); // 77 completed, 23 still in process

    println!("UpdateStats: {}", update_stats);
    println!(
        "In-process rows: {}",
        update_stats.processing.get_in_process()
    );
    println!("Has changes: {}\n", update_stats.has_any_change());

    // Example 2: Per-operation tracking
    println!("--- Example 2: OperationInProcessStats ---");
    let op_stats = stats::OperationInProcessStats::default();

    // Track multiple operations
    op_stats.start_processing("import_users", 50);
    op_stats.start_processing("transform_data", 30);
    op_stats.start_processing("export_to_db", 20);

    println!(
        "Total in-process: {}",
        op_stats.get_total_in_process_count()
    );
    println!("\nPer-operation breakdown:");
    for (op_name, count) in op_stats.get_all_operations_in_process() {
        println!("  - {}: {} rows", op_name, count);
    }

    // Simulate completion
    op_stats.finish_processing("import_users", 25);
    println!("\nAfter finishing 25 rows from 'import_users':");
    println!(
        "  import_users: {} rows",
        op_stats.get_operation_in_process_count("import_users")
    );
    println!("  Total: {} rows\n", op_stats.get_total_in_process_count());

    // Example 3: FlowLiveUpdater API structure
    println!("--- Example 3: FlowLiveUpdater API Pattern ---");
    println!(
        "
// In a real application with a persisted flow:

// 1. Start the live updater
let updater = execution::FlowLiveUpdater::start(
    flow_ctx,
    &pool,
    execution::FlowLiveUpdaterOptions {{
        live_mode: false,       // Run once
        print_stats: true,      // Print progress to console
        reexport_targets: false,
        full_reprocess: false,
    }}
).await?;

// 2. Subscribe to progress updates in a separate task
tokio::spawn(async move {{
    loop {{
        let updates = updater.next_status_updates().await?;

        // Check which sources have updates
        for source in updates.updated_sources {{
            println!(\"Source {{}} has new data\", source);
        }}

        // Check active sources
        if updates.active_sources.is_empty() {{
            break; // All done
        }}

        // Get per-operation stats
        let op_stats = &updater.operation_in_process_stats;
        let total_in_process = op_stats.get_total_in_process_count();
        println!(\"Total in-process: {{}}\", total_in_process);
    }}
    Ok::<(), Error>(())
}});

// 3. Wait for completion
updater.wait().await?;

// 4. Get final statistics
let final_stats = updater.index_update_info();
for source_info in final_stats.sources {{
    println!(\"Source: {{}}\", source_info.source_name);
    println!(\"  Stats: {{}}\", source_info.stats);
}}
"
    );

    println!("\n--- API Documentation ---");
    println!("For more information, see:");
    println!("  - crates/recoco-core/src/execution/stats.rs");
    println!("  - crates/recoco-core/src/execution/live_updater.rs");
    println!("\nThe progress watching API is now part of recoco's public API");
    println!("and can be accessed via `use recoco::prelude::*;`");

    Ok(())
}

#[cfg(not(feature = "persistence"))]
fn main() {
    eprintln!("This example requires the 'persistence' feature.");
    eprintln!("Run with: cargo run -p recoco --example progress_watching --features persistence");
    std::process::exit(1);
}
