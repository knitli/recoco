// Recoco is a Rust-only fork of CocoIndex, by [CocoIndex](https://CocoIndex.io)
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
pub mod app;
pub mod component;
pub mod context;
pub mod environment;
pub mod execution;
pub mod function;
pub mod id_sequencer;
pub mod logic_registry;
pub mod profile;
pub mod runtime;
pub mod stats;
pub mod target_state;
pub mod txn_batcher;

pub use app::UpdateHandle;
pub use stats::{TERMINATED_VERSION, UpdateStats};
