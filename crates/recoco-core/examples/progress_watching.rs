// Recoco is a Rust-only fork of CocoIndex, by [CocoIndex](https://CocoIndex)
// SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: Apache-2.0

//! Example demonstrating the progress watching API
//!
//! This example shows how to use the new `ProcessingStats` API to monitor
//! per-component processing progress with version tracking and change notifications.
//!
//! Run with:
//! ```bash
//! cargo run -p recoco-core --example progress_watching
//! ```

use recoco_core::execution::stats::{
    ProcessingStats, ProcessingStatsGroup, TERMINATED_VERSION,
};

#[tokio::main]
async fn main() {
    println!("=== Progress Watching API Example ===\n");

    // Create a ProcessingStats instance
    let stats = ProcessingStats::new();

    // Subscribe to version changes
    let mut version_rx = stats.subscribe();

    // Simulate processing in a background task
    let stats_clone = stats.clone();
    let handle = tokio::spawn(async move {
        // Simulate importing data
        stats_clone.update("import_users", |group| {
            group.num_execution_starts = 100;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Simulate some items being processed
        stats_clone.update("import_users", |group| {
            group.num_adds = 70;
            group.num_unchanged = 20;
            group.num_errors = 5;
        });

        // Start transform operations
        stats_clone.update("transform_data", |group| {
            group.num_execution_starts = 50;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Complete transform operations
        stats_clone.update("transform_data", |group| {
            group.num_reprocesses = 40;
            group.num_errors = 2;
        });

        // Signal completion
        stats_clone.notify_terminated();
    });

    // Monitor progress
    println!("Monitoring progress updates...\n");
    loop {
        // Wait for a version change
        if version_rx.changed().await.is_err() {
            break;
        }

        let version = *version_rx.borrow();
        if version == TERMINATED_VERSION {
            println!("\n✅ Processing completed!");
            break;
        }

        // Get a snapshot of current stats
        let snapshot = stats.snapshot();
        println!("📊 Version {}: {} component(s) active", version, snapshot.stats.len());

        for (component_name, group) in snapshot.stats.iter() {
            println!("   {} - {}", component_name, group);
        }
        println!();
    }

    // Wait for background task to finish
    handle.await.unwrap();

    // Print final stats
    println!("\n=== Final Statistics ===");
    let final_snapshot = stats.snapshot();
    for (component_name, group) in final_snapshot.stats.iter() {
        println!("{}: {}", component_name, group);
    }
}
