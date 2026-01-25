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

pub mod error;

#[cfg(feature = "batching")]
pub mod batching;
#[cfg(feature = "concur_control")]
pub mod concur_control;
#[cfg(any(feature = "sqlx", feature = "str_sanitize"))]
pub mod db;
#[cfg(feature = "deserialize")]
pub mod deser;
#[cfg(feature = "fingerprint")]
pub mod fingerprint;
#[cfg(feature = "immutable")]
pub mod immutable;
#[cfg(feature = "retryable")]
pub mod retryable;

pub mod prelude;

#[cfg(feature = "bytes_decode")]
pub mod bytes_decode;
#[cfg(any(feature = "reqwest", feature = "http"))]
pub mod http;
#[cfg(any(feature = "sqlx", feature = "str_sanitize"))]
pub mod str_sanitize;
#[cfg(feature = "yaml")]
pub mod yaml_ser;
