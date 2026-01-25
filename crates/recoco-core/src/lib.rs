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

#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::similar_names)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::collapsible_if)]

pub mod base;
pub mod builder;
pub mod execution;
pub mod lib_context;
#[cfg(any(feature = "function-extract-llm", feature = "function-embed"))]
pub mod llm;
pub mod ops;
pub mod prelude;
#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "persistence")]
pub mod service;
pub mod settings;
pub mod setup;
