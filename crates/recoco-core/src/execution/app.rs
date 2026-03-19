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

//! Progress watching API for monitoring flow execution.
//!
//! This module provides the ability to subscribe to progress updates during
//! indexing runs, allowing applications to track the state of long-running
//! flows and provide observability to users.

use super::stats::{IndexUpdateInfo, SourceUpdateInfo};
use tokio::sync::watch;

/// A snapshot of the current progress state.
#[derive(Debug, Clone)]
pub struct ProgressSnapshot {
    /// Statistics for each source in the flow.
    pub sources: Vec<SourceUpdateInfo>,
    /// Whether the flow execution is complete.
    pub is_complete: bool,
}

impl ProgressSnapshot {
    /// Get the total number of rows processed across all sources.
    pub fn total_rows_processed(&self) -> i64 {
        self.sources
            .iter()
            .map(|s| {
                s.stats.num_insertions.get()
                    + s.stats.num_updates.get()
                    + s.stats.num_deletions.get()
                    + s.stats.num_no_change.get()
                    + s.stats.num_errors.get()
            })
            .sum()
    }

    /// Get the total number of rows currently being processed.
    pub fn total_rows_in_process(&self) -> i64 {
        self.sources
            .iter()
            .map(|s| s.stats.processing.get_in_process())
            .sum()
    }

    /// Check if any source has errors.
    pub fn has_errors(&self) -> bool {
        self.sources.iter().any(|s| s.stats.num_errors.get() > 0)
    }
}

impl std::fmt::Display for ProgressSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Progress Snapshot:")?;
        writeln!(f, "  Total processed: {}", self.total_rows_processed())?;
        writeln!(f, "  In process: {}", self.total_rows_in_process())?;
        writeln!(f, "  Complete: {}", self.is_complete)?;
        writeln!(f, "  Sources:")?;
        for source in &self.sources {
            writeln!(f, "    - {}", source)?;
        }
        Ok(())
    }
}

/// A handle for watching progress updates.
///
/// This allows callers to subscribe to progress updates during flow execution
/// and receive periodic snapshots of the current state.
pub struct ProgressWatcher {
    receiver: watch::Receiver<ProgressSnapshot>,
}

impl ProgressWatcher {
    /// Create a new progress watcher from a watch receiver.
    pub fn new(receiver: watch::Receiver<ProgressSnapshot>) -> Self {
        Self { receiver }
    }

    /// Wait for the next progress update.
    ///
    /// Returns `None` if the sender has been dropped (execution complete).
    pub async fn changed(&mut self) -> Option<ProgressSnapshot> {
        self.receiver.changed().await.ok()?;
        Some(self.receiver.borrow().clone())
    }

    /// Get the current progress snapshot without waiting.
    pub fn get_current(&self) -> ProgressSnapshot {
        self.receiver.borrow().clone()
    }

    /// Check if the execution is complete.
    pub fn is_complete(&self) -> bool {
        self.receiver.borrow().is_complete
    }
}

/// Progress publisher for updating watchers.
///
/// This is typically held by the execution engine and used to publish
/// progress updates to all subscribed watchers.
pub struct ProgressPublisher {
    sender: watch::Sender<ProgressSnapshot>,
}

impl ProgressPublisher {
    /// Create a new progress publisher with an initial snapshot.
    pub fn new(initial: ProgressSnapshot) -> (Self, ProgressWatcher) {
        let (sender, receiver) = watch::channel(initial);
        let publisher = Self { sender };
        let watcher = ProgressWatcher::new(receiver);
        (publisher, watcher)
    }

    /// Create a progress publisher from an IndexUpdateInfo.
    pub fn from_index_update_info(info: &IndexUpdateInfo) -> (Self, ProgressWatcher) {
        let snapshot = ProgressSnapshot {
            sources: info.sources.clone(),
            is_complete: false,
        };
        Self::new(snapshot)
    }

    /// Update the progress snapshot.
    ///
    /// This will notify all watchers of the new state.
    pub fn update(&self, snapshot: ProgressSnapshot) {
        // Ignore errors - they just mean no one is watching
        let _ = self.sender.send(snapshot);
    }

    /// Update progress from an IndexUpdateInfo.
    pub fn update_from_index_info(&self, info: &IndexUpdateInfo, is_complete: bool) {
        let snapshot = ProgressSnapshot {
            sources: info.sources.clone(),
            is_complete,
        };
        self.update(snapshot);
    }

    /// Mark the execution as complete.
    pub fn mark_complete(&self, final_info: &IndexUpdateInfo) {
        self.update_from_index_info(final_info, true);
    }

    /// Get a new watcher for this publisher.
    pub fn subscribe(&self) -> ProgressWatcher {
        ProgressWatcher::new(self.sender.subscribe())
    }

    /// Check if there are any active watchers.
    pub fn has_watchers(&self) -> bool {
        self.sender.receiver_count() > 0
    }
}

/// Builder for creating progress watchers with custom configuration.
pub struct ProgressWatcherBuilder {
    buffer_size: Option<usize>,
}

impl Default for ProgressWatcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressWatcherBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self { buffer_size: None }
    }

    /// Set the buffer size for the watcher.
    ///
    /// Note: tokio::sync::watch doesn't support configurable buffer sizes,
    /// so this is provided for API compatibility and future extensibility.
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = Some(size);
        self
    }

    /// Build a progress publisher and watcher pair.
    pub fn build(self) -> (ProgressPublisher, ProgressWatcher) {
        let initial = ProgressSnapshot {
            sources: Vec::new(),
            is_complete: false,
        };
        ProgressPublisher::new(initial)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::stats::UpdateStats;

    fn create_test_stats() -> UpdateStats {
        let stats = UpdateStats::default();
        stats.num_insertions.inc(5);
        stats.num_updates.inc(3);
        stats.processing.start(2);
        stats
    }

    #[test]
    fn test_progress_snapshot_totals() {
        let sources = vec![
            SourceUpdateInfo {
                source_name: "source1".to_string(),
                stats: create_test_stats(),
            },
            SourceUpdateInfo {
                source_name: "source2".to_string(),
                stats: create_test_stats(),
            },
        ];

        let snapshot = ProgressSnapshot {
            sources,
            is_complete: false,
        };

        assert_eq!(snapshot.total_rows_processed(), 16); // (5 + 3) * 2
        assert_eq!(snapshot.total_rows_in_process(), 4); // 2 * 2
        assert!(!snapshot.has_errors());
    }

    #[test]
    fn test_progress_snapshot_with_errors() {
        let stats = UpdateStats::default();
        stats.num_errors.inc(1);

        let sources = vec![SourceUpdateInfo {
            source_name: "source1".to_string(),
            stats,
        }];

        let snapshot = ProgressSnapshot {
            sources,
            is_complete: false,
        };

        assert!(snapshot.has_errors());
        assert_eq!(snapshot.total_rows_processed(), 1);
    }

    #[tokio::test]
    async fn test_progress_watcher_updates() {
        let (publisher, mut watcher) = ProgressPublisher::new(ProgressSnapshot {
            sources: Vec::new(),
            is_complete: false,
        });

        // Spawn a task to publish updates
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            let stats = create_test_stats();
            let sources = vec![SourceUpdateInfo {
                source_name: "test".to_string(),
                stats,
            }];

            publisher.update(ProgressSnapshot {
                sources,
                is_complete: false,
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            let stats = UpdateStats::default();
            stats.num_insertions.inc(10);

            let sources = vec![SourceUpdateInfo {
                source_name: "test".to_string(),
                stats,
            }];

            publisher.update(ProgressSnapshot {
                sources,
                is_complete: true,
            });
        });

        // Wait for first update
        let snapshot = watcher.changed().await.expect("Should receive update");
        assert!(!snapshot.is_complete);
        assert_eq!(snapshot.sources.len(), 1);

        // Wait for second update
        let snapshot = watcher.changed().await.expect("Should receive update");
        assert!(snapshot.is_complete);
        assert_eq!(snapshot.sources[0].stats.num_insertions.get(), 10);
    }

    #[test]
    fn test_progress_publisher_subscribe() {
        let (publisher, _watcher1) = ProgressPublisher::new(ProgressSnapshot {
            sources: Vec::new(),
            is_complete: false,
        });

        assert!(publisher.has_watchers());

        let _watcher2 = publisher.subscribe();
        assert!(publisher.has_watchers());
    }

    #[test]
    fn test_progress_watcher_builder() {
        let builder = ProgressWatcherBuilder::new().buffer_size(100);
        let (publisher, watcher) = builder.build();

        assert!(!watcher.is_complete());
        assert!(publisher.has_watchers());
    }

    #[test]
    fn test_progress_from_index_update_info() {
        let stats = create_test_stats();
        let info = IndexUpdateInfo {
            sources: vec![SourceUpdateInfo {
                source_name: "test".to_string(),
                stats,
            }],
        };

        let (_publisher, watcher) = ProgressPublisher::from_index_update_info(&info);
        let snapshot = watcher.get_current();

        assert_eq!(snapshot.sources.len(), 1);
        assert_eq!(snapshot.sources[0].source_name, "test");
        assert!(!snapshot.is_complete);
    }
}
