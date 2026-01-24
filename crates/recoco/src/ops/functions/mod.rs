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

#[cfg(feature = "function-detect-lang")]
pub mod detect_program_lang;
#[cfg(feature = "function-embed")]
pub mod embed_text;
#[cfg(feature = "function-extract-llm")]
pub mod extract_by_llm;
#[cfg(feature = "function-json")]
pub mod parse_json;
#[cfg(feature = "function-split")]
pub mod split_by_separators;
#[cfg(feature = "function-split")]
pub mod split_recursively;

#[cfg(test)]
mod test_utils;
