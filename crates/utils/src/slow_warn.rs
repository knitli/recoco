// Recoco is a Rust-only fork of CocoIndex, by [CocoIndex](https://cocoindex.io)
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
use std::future::Future;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{Level, warn};

/// Runs an async operation and logs a warning if it takes longer than the threshold.
/// The operation continues running and its result is returned normally.
///
/// The message closure is only called if:
/// 1. The operation exceeds the threshold, AND
/// 2. Warn-level logging is enabled
pub async fn warn_if_slow<F, T, M>(msg_fn: &M, threshold: Duration, future: F) -> T
where
    F: Future<Output = T>,
    M: Fn() -> String,
{
    if !tracing::enabled!(Level::WARN) {
        return future.await;
    }

    tokio::pin!(future);

    tokio::select! {
        biased;
        result = &mut future => result,
        _ = sleep(threshold) => {
            let start = Instant::now();
            let msg = msg_fn();
            warn!("Taking longer than {}s: {msg}", threshold.as_secs_f32());
            let result = future.await;
            warn!("Finished after {}s: {msg}", start.elapsed().as_secs_f32() + threshold.as_secs_f32());
            result
        }
    }
}
