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

pub mod batching;
pub mod concur_control;
pub mod db;
pub mod deser;
pub mod error;
pub mod fingerprint;
pub mod immutable;
pub mod retryable;

pub mod prelude;

#[cfg(feature = "bytes_decode")]
pub mod bytes_decode;
#[cfg(feature = "reqwest")]
pub mod http;
#[cfg(feature = "sqlx")]
pub mod str_sanitize;
#[cfg(feature = "yaml")]
pub mod yaml_ser;
