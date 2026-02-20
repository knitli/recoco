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

#[cfg(feature = "persistence")]
pub(crate) mod db_tracking_setup;
#[cfg(feature = "persistence")]
pub mod dumper;
pub mod evaluator;
#[cfg(feature = "persistence")]
pub(crate) mod indexing_status;
pub(crate) mod memoization;
#[cfg(feature = "persistence")]
pub(crate) mod row_indexer;
#[cfg(feature = "persistence")]
pub(crate) mod source_indexer;
pub(crate) mod stats;

#[cfg(feature = "persistence")]
mod live_updater;
#[cfg(feature = "persistence")]
pub use live_updater::*;

#[cfg(feature = "persistence")]
mod db_tracking;
