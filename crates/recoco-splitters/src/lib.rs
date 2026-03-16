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

//! Extra text processing utilities for CocoIndex.
//!
//! This crate provides text processing functionality including:
//! - Programming language detection and tree-sitter support
//! - Text splitting by separators
//! - Recursive text chunking with syntax awareness
//! - Pattern matching for file filtering

#[cfg(feature = "pattern-matching")]
pub mod pattern_matcher;
pub mod prog_langs;
pub mod split;
pub mod by_separators;
pub(crate) mod output_positions;
pub mod recursive;
