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

use std::collections::hash_map;

use crate::prelude::*;

pub struct AuthRegistry {
    entries: RwLock<HashMap<String, serde_json::Value>>,
}

impl Default for AuthRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthRegistry {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    pub fn add(&self, key: String, value: serde_json::Value) -> Result<()> {
        let mut entries = self.entries.write().unwrap();
        match entries.entry(key) {
            hash_map::Entry::Occupied(entry) => {
                api_bail!("Auth entry already exists: {}", entry.key());
            }
            hash_map::Entry::Vacant(entry) => {
                entry.insert(value);
            }
        }
        Ok(())
    }

    pub fn add_transient(&self, value: serde_json::Value) -> Result<String> {
        let key = format!(
            "__transient_{}",
            utils::fingerprint::Fingerprinter::default()
                .with("cocoindex_auth")? // salt
                .with(&value)?
                .into_fingerprint()
                .to_base64()
        );
        self.entries
            .write()
            .unwrap()
            .entry(key.clone())
            .or_insert(value);
        Ok(key)
    }

    pub fn get<T: DeserializeOwned>(&self, entry_ref: &spec::AuthEntryReference<T>) -> Result<T> {
        let entries = self.entries.read().unwrap();
        match entries.get(&entry_ref.key) {
            Some(value) => Ok(utils::deser::from_json_value(value.clone())?),
            None => api_bail!(
                "Auth entry `{key}` not found.\n\
                Hint: If you're not referencing `{key}` in your flow, it will likely be caused by a previously persisted target using it. \
                You need to bring back the definition for the auth entry `{key}`, so that CocoIndex will be able to do a cleanup in the next `setup` run. \
                See https://CocoIndex/docs/core/flow_def#auth-registry for more details.",
                key = entry_ref.key
            ),
        }
    }
}
