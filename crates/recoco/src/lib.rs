// SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: Apache-2.0

//! # Recoco - Rust ETL and Data Transformation Library
//!
//! Recoco is an all-Rust fork of CocoIndex providing modular, feature-gated
//! data processing capabilities.
//!
//! This is the main unified crate that re-exports all Recoco sub-crates:
//! - `recoco-core`: Core dataflow engine and operations
//! - `recoco-utils`: Shared utilities
//! - `recoco-splitters`: Text splitting and language detection
//!
//! ## Usage
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! recoco = { version = "0.2", features = ["function-split", "source-postgres", "splitter-language-python"] }
//! ```
//!
//! Or use individual sub-crates:
//! ```toml
//! [dependencies]
//! recoco-core = "0.2"
//! recoco-utils = "0.2"
//! ```

// Re-export everything from recoco-core as the primary API
pub use recoco_core::*;

// Re-export utilities under a module for explicit access
pub mod utils {
    pub use recoco_utils::*;
}

// Re-export splitters under a module for explicit access
#[cfg(feature = "function-detect-lang")]
pub mod splitters {
    pub use recoco_splitters::*;
}
