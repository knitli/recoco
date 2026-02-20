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

use const_format::concatcp;

pub static COCOINDEX_PREFIX: &str = "CocoIndex/";

/// Present for bytes and str. It points to fields that represents the original file name for the data.
/// Type: AnalyzedValueMapping
pub static CONTENT_FILENAME: &str = concatcp!(COCOINDEX_PREFIX, "content_filename");

/// Present for bytes and str. It points to fields that represents mime types for the data.
/// Type: AnalyzedValueMapping
pub static CONTENT_MIME_TYPE: &str = concatcp!(COCOINDEX_PREFIX, "content_mime_type");

/// Present for chunks. It points to fields that the chunks are for.
/// Type: AnalyzedValueMapping
pub static CHUNK_BASE_TEXT: &str = concatcp!(COCOINDEX_PREFIX, "chunk_base_text");

/// Base text for an embedding vector.
pub static _EMBEDDING_ORIGIN_TEXT: &str = concatcp!(COCOINDEX_PREFIX, "embedding_origin_text");
