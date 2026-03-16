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

pub enum WriteAction {
    Insert,
    Update,
}

pub fn sanitize_identifier(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() || c == '_' {
            result.push(c);
        } else {
            result.push_str("__");
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_identifier_empty() {
        assert_eq!(sanitize_identifier(""), "");
    }

    #[test]
    fn test_sanitize_identifier_alphanumeric() {
        assert_eq!(sanitize_identifier("HelloWorld123"), "HelloWorld123");
    }

    #[test]
    fn test_sanitize_identifier_underscores() {
        assert_eq!(sanitize_identifier("hello_world_123"), "hello_world_123");
    }

    #[test]
    fn test_sanitize_identifier_non_alphanumeric() {
        assert_eq!(sanitize_identifier("hello-world.123"), "hello__world__123");
    }

    #[test]
    fn test_sanitize_identifier_only_non_alphanumeric() {
        assert_eq!(sanitize_identifier("!@#"), "______");
    }

    #[test]
    fn test_sanitize_identifier_unicode() {
        // '🚀' is not alphanumeric, should be replaced by "__"
        assert_eq!(sanitize_identifier("hello🚀"), "hello__");
        // 'ö' is alphanumeric according to rust's char::is_alphanumeric
        assert_eq!(sanitize_identifier("helloö"), "helloö");
    }
}
