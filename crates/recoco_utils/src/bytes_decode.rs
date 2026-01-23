// ReCoco is a Rust-only fork of CocoIndex, by [CocoIndex.io](https://cocoindex.io)
// Original code from CocoIndex is copyrighted by CocoIndex.io
// SPDX-FileCopyrightText: 2025-2026 CocoIndex.io (upstream)
// SPDX-FileContributor: CocoIndex Contributors
//
// All modifications from the upstream for ReCoco are copyrighted by Knitli Inc.
// SPDX-FileCopyrightText: 2026 Knitli Inc. (ReCoco)
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// Both the upstream CocoIndex code and the ReCoco modifications are licensed under the Apache-2.0 License.
// SPDX-License-Identifier: Apache-2.0

use encoding_rs::Encoding;

pub fn bytes_to_string<'a>(bytes: &'a [u8]) -> (std::borrow::Cow<'a, str>, bool) {
    // 1) BOM sniff first (definitive for UTF-8/16; UTF-32 is not supported here).
    if let Some((enc, bom_len)) = Encoding::for_bom(bytes) {
        let (cow, had_errors) = enc.decode_without_bom_handling(&bytes[bom_len..]);
        return (cow, had_errors);
    }
    // 2) Otherwise, try UTF-8 (accepts input with or without a UTF-8 BOM).
    let (cow, had_errors) = encoding_rs::UTF_8.decode_with_bom_removal(bytes);
    (cow, had_errors)
}
