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

//! Extra text processing utilities for CocoIndex.
//!
//! This crate provides text processing functionality including:
//! - Programming language detection and tree-sitter support
//! - Text splitting by separators
//! - Recursive text chunking with syntax awareness
//! - File pattern matching for inclusion and exclusion

#[cfg(any(
    feature = "splitting",
    feature = "all",
    feature = "c",
    feature = "c-sharp",
    feature = "cpp",
    feature = "css",
    feature = "fortran",
    feature = "go",
    feature = "html",
    feature = "java",
    feature = "javascript",
    feature = "json",
    feature = "kotlin",
    feature = "markdown",
    feature = "pascal",
    feature = "php",
    feature = "python",
    feature = "r",
    feature = "ruby",
    feature = "rust",
    feature = "scala",
    feature = "solidity",
    feature = "sql",
    feature = "swift",
    feature = "toml",
    feature = "typescript",
    feature = "xml",
    feature = "yaml"
))]
pub mod prog_langs;

#[cfg(any(
    feature = "splitting",
    feature = "all",
    feature = "c",
    feature = "c-sharp",
    feature = "cpp",
    feature = "css",
    feature = "fortran",
    feature = "go",
    feature = "html",
    feature = "java",
    feature = "javascript",
    feature = "json",
    feature = "kotlin",
    feature = "markdown",
    feature = "pascal",
    feature = "php",
    feature = "python",
    feature = "r",
    feature = "ruby",
    feature = "rust",
    feature = "scala",
    feature = "solidity",
    feature = "sql",
    feature = "swift",
    feature = "toml",
    feature = "typescript",
    feature = "xml",
    feature = "yaml"
))]
pub mod split;

#[cfg(any(feature = "matching", feature = "all"))]
pub mod pattern_matcher;
