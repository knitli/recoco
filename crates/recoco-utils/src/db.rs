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

/// Validates that a database identifier (table name, column name, schema name) is valid.
///
/// Valid identifiers:
/// - Must not be empty
/// - Must start with a letter (a-z, A-Z) or underscore (_)
/// - Can contain letters, digits (0-9), and underscores
/// - Should not exceed 63 characters (PostgreSQL limit, also reasonable for other databases)
///
/// This validation helps prevent SQL injection and provides clear error messages
/// when users provide invalid identifiers at configuration time.
pub fn validate_identifier(identifier: &str, identifier_type: &str) -> Result<(), String> {
    if identifier.is_empty() {
        return Err(format!("{} cannot be empty", identifier_type));
    }

    if identifier.len() > 63 {
        return Err(format!(
            "{} '{}' exceeds maximum length of 63 characters",
            identifier_type, identifier
        ));
    }

    let first_char = identifier.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return Err(format!(
            "{} '{}' must start with a letter or underscore, not '{}'",
            identifier_type, identifier, first_char
        ));
    }

    for (i, c) in identifier.chars().enumerate() {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return Err(format!(
                "{} '{}' contains invalid character '{}' at position {}. Only letters, digits, and underscores are allowed.",
                identifier_type, identifier, c, i
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_identifier_valid() {
        assert!(validate_identifier("users", "table name").is_ok());
        assert!(validate_identifier("user_data", "table name").is_ok());
        assert!(validate_identifier("_private", "table name").is_ok());
        assert!(validate_identifier("table123", "table name").is_ok());
        assert!(validate_identifier("TABLE_NAME", "table name").is_ok());
    }

    #[test]
    fn test_validate_identifier_invalid() {
        // Empty
        assert!(validate_identifier("", "table name").is_err());

        // Starts with number
        assert!(validate_identifier("123table", "table name").is_err());

        // Contains special characters
        assert!(validate_identifier("user-data", "table name").is_err());
        assert!(validate_identifier("user.data", "table name").is_err());
        assert!(validate_identifier("user data", "table name").is_err());
        assert!(validate_identifier("user$data", "table name").is_err());
        assert!(validate_identifier("user;DROP TABLE users", "table name").is_err());

        // Too long
        let long_name = "a".repeat(64);
        assert!(validate_identifier(&long_name, "table name").is_err());
    }
}
