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

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};

use crate::base::{schema, value};
use recoco_utils::error::{SharedError, SharedResultExtRef};
use recoco_utils::fingerprint::{Fingerprint, Fingerprinter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCacheEntry {
    time_sec: i64,
    value: serde_json::Value,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoredMemoizationInfo {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub cache: HashMap<Fingerprint, StoredCacheEntry>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub uuids: HashMap<Fingerprint, Vec<uuid::Uuid>>,

    /// TO BE DEPRECATED. Use the new `processed_source_fp` column instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

pub type CacheEntryCell =
    Arc<tokio::sync::OnceCell<std::result::Result<value::Value, SharedError>>>;
enum CacheData {
    /// Existing entry in previous runs, but not in current run yet.
    Previous(serde_json::Value),
    /// Value appeared in current run.
    Current(CacheEntryCell),
}

struct CacheEntry {
    time: chrono::DateTime<chrono::Utc>,
    data: CacheData,
}

#[derive(Default)]
struct UuidEntry {
    uuids: Vec<uuid::Uuid>,
    num_current: usize,
}

impl UuidEntry {
    fn new(uuids: Vec<uuid::Uuid>) -> Self {
        Self {
            uuids,
            num_current: 0,
        }
    }

    fn into_stored(self) -> Option<Vec<uuid::Uuid>> {
        if self.num_current == 0 {
            return None;
        }
        let mut uuids = self.uuids;
        if self.num_current < uuids.len() {
            uuids.truncate(self.num_current);
        }
        Some(uuids)
    }
}

pub struct EvaluationMemoryOptions {
    pub enable_cache: bool,

    /// If true, it's for evaluation only.
    /// In this mode, we don't memoize anything.
    pub evaluation_only: bool,
}

pub struct EvaluationMemory {
    current_time: chrono::DateTime<chrono::Utc>,
    cache: Option<Mutex<HashMap<Fingerprint, CacheEntry>>>,
    uuids: Mutex<HashMap<Fingerprint, UuidEntry>>,
    evaluation_only: bool,
}

impl EvaluationMemory {
    pub fn new(
        current_time: chrono::DateTime<chrono::Utc>,
        stored_info: Option<StoredMemoizationInfo>,
        options: EvaluationMemoryOptions,
    ) -> Self {
        let (stored_cache, stored_uuids) = stored_info
            .map(|stored_info| (stored_info.cache, stored_info.uuids))
            .unzip();
        Self {
            current_time,
            cache: options.enable_cache.then(|| {
                Mutex::new(
                    stored_cache
                        .into_iter()
                        .flat_map(|iter| iter.into_iter())
                        .map(|(k, e)| {
                            (
                                k,
                                CacheEntry {
                                    time: chrono::DateTime::from_timestamp(e.time_sec, 0)
                                        .unwrap_or(chrono::DateTime::<chrono::Utc>::MIN_UTC),
                                    data: CacheData::Previous(e.value),
                                },
                            )
                        })
                        .collect(),
                )
            }),
            uuids: Mutex::new(
                (!options.evaluation_only)
                    .then_some(stored_uuids)
                    .flatten()
                    .into_iter()
                    .flat_map(|iter| iter.into_iter())
                    .map(|(k, v)| (k, UuidEntry::new(v)))
                    .collect(),
            ),
            evaluation_only: options.evaluation_only,
        }
    }

    pub fn into_stored(self) -> Result<StoredMemoizationInfo> {
        if self.evaluation_only {
            internal_bail!("For evaluation only, cannot convert to stored MemoizationInfo");
        }
        let cache = if let Some(cache) = self.cache {
            cache
                .into_inner()?
                .into_iter()
                .filter_map(|(k, e)| match e.data {
                    CacheData::Previous(_) => None,
                    CacheData::Current(entry) => match entry.get() {
                        Some(Ok(v)) => Some(serde_json::to_value(v).map(|value| {
                            (
                                k,
                                StoredCacheEntry {
                                    time_sec: e.time.timestamp(),
                                    value,
                                },
                            )
                        })),
                        _ => None,
                    },
                })
                .collect::<std::result::Result<_, _>>()?
        } else {
            internal_bail!("Cache is disabled, cannot convert to stored MemoizationInfo");
        };
        let uuids = self
            .uuids
            .into_inner()?
            .into_iter()
            .filter_map(|(k, v)| v.into_stored().map(|uuids| (k, uuids)))
            .collect();
        Ok(StoredMemoizationInfo {
            cache,
            uuids,
            content_hash: None,
        })
    }

    pub fn get_cache_entry(
        &self,
        key: impl FnOnce() -> Result<Fingerprint>,
        typ: &schema::ValueType,
        ttl: Option<chrono::Duration>,
    ) -> Result<Option<CacheEntryCell>> {
        let mut cache = if let Some(cache) = &self.cache {
            cache.lock().unwrap()
        } else {
            return Ok(None);
        };
        let result = match cache.entry(key()?) {
            std::collections::hash_map::Entry::Occupied(mut entry)
                if !ttl
                    .map(|ttl| entry.get().time + ttl < self.current_time)
                    .unwrap_or(false) =>
            {
                let entry_mut = &mut entry.get_mut();
                match &mut entry_mut.data {
                    CacheData::Previous(value) => {
                        let value = value::Value::from_json(std::mem::take(value), typ)?;
                        let cell = Arc::new(tokio::sync::OnceCell::from(Ok(value)));
                        let time = entry_mut.time;
                        entry.insert(CacheEntry {
                            time,
                            data: CacheData::Current(cell.clone()),
                        });
                        cell
                    }
                    CacheData::Current(cell) => cell.clone(),
                }
            }
            entry => {
                let cell = Arc::new(tokio::sync::OnceCell::new());
                entry.insert_entry(CacheEntry {
                    time: self.current_time,
                    data: CacheData::Current(cell.clone()),
                });
                cell
            }
        };
        Ok(Some(result))
    }

    pub fn next_uuid(&self, key: Fingerprint) -> Result<uuid::Uuid> {
        let mut uuids = self.uuids.lock().unwrap();

        let entry = uuids.entry(key).or_default();
        let uuid = if self.evaluation_only {
            let fp = Fingerprinter::default()
                .with(&key)?
                .with(&entry.num_current)?
                .into_fingerprint();
            uuid::Uuid::new_v8(fp.0)
        } else if entry.num_current < entry.uuids.len() {
            entry.uuids[entry.num_current]
        } else {
            let uuid = uuid::Uuid::new_v4();
            entry.uuids.push(uuid);
            uuid
        };
        entry.num_current += 1;
        Ok(uuid)
    }
}

pub async fn evaluate_with_cell<Fut>(
    cell: Option<&CacheEntryCell>,
    compute: impl FnOnce() -> Fut,
) -> Result<Cow<'_, value::Value>>
where
    Fut: Future<Output = Result<value::Value>>,
{
    let result = match cell {
        Some(cell) => Cow::Borrowed(
            cell.get_or_init(|| {
                let fut = compute();
                async move { fut.await.map_err(SharedError::from) }
            })
            .await
            .into_result()?,
        ),
        None => Cow::Owned(compute().await?),
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::schema;
    use serde_json::json;

    fn test_fingerprint() -> Fingerprint {
        Fingerprint([0u8; 16])
    }

    /// Fixed evaluation timestamp used across all tests to keep them deterministic.
    fn test_now() -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(1_700_000_100, 0).unwrap()
    }

    /// Build a `StoredMemoizationInfo` whose single cache entry timestamp aligns with
    /// `test_now()`, so TTL logic (if ever introduced) doesn't interfere.
    fn make_stored_info_with_str(fp: Fingerprint, str_value: &str) -> StoredMemoizationInfo {
        let mut cache = HashMap::new();
        cache.insert(
            fp,
            StoredCacheEntry {
                time_sec: test_now().timestamp(),
                value: json!(str_value),
            },
        );
        StoredMemoizationInfo {
            cache,
            uuids: HashMap::new(),
            content_hash: None,
        }
    }

    #[test]
    fn cache_hit_when_stored_info_present() {
        // Verifies that EvaluationMemory returns pre-populated cached values from StoredMemoizationInfo.
        let fp = test_fingerprint();
        let stored_info = make_stored_info_with_str(fp, "cached_value");
        let memory = EvaluationMemory::new(
            test_now(),
            Some(stored_info),
            EvaluationMemoryOptions {
                enable_cache: true,
                evaluation_only: false,
            },
        );

        let str_type = schema::ValueType::Basic(schema::BasicValueType::Str);
        let cell = memory
            .get_cache_entry(|| Ok(fp), &str_type, None)
            .unwrap()
            .expect("should return a cache entry cell on hit");

        // A cache hit from stored data means the cell is already initialized with the cached value.
        match cell.get() {
            Some(Ok(value::Value::Basic(value::BasicValue::Str(s)))) => {
                assert_eq!(s.as_ref(), "cached_value");
            }
            other => panic!("expected cached str value 'cached_value', got {other:?}"),
        }
    }

    #[test]
    fn clearing_stored_cache_before_construction_bypasses_stale_entries() {
        // Simulates the full_reprocess path in row_indexer::update_source_row:
        // when full_reprocess is set, stored_info.cache.clear() is called before
        // passing the info to EvaluationMemory::new, so no stale cached values
        // are returned during re-evaluation.
        let fp = test_fingerprint();
        let mut stored_info = make_stored_info_with_str(fp, "stale_cached_value");

        // This is the fix: clear the stored cache before constructing EvaluationMemory,
        // mirroring the upstream full_reprocess guard.
        stored_info.cache.clear();

        let memory = EvaluationMemory::new(
            test_now(),
            Some(stored_info),
            EvaluationMemoryOptions {
                enable_cache: true,
                evaluation_only: false,
            },
        );

        let str_type = schema::ValueType::Basic(schema::BasicValueType::Str);
        let cell = memory
            .get_cache_entry(|| Ok(fp), &str_type, None)
            .unwrap()
            .expect("should return a cache entry cell even on miss");

        // Cache miss: the cell is uninitialized — the stale stored value was not used.
        assert!(
            cell.get().is_none(),
            "cache entry must not be pre-initialized when stored cache was cleared (full_reprocess)"
        );
    }

    #[test]
    fn cache_disabled_returns_none() {
        // Verifies that get_cache_entry returns None when enable_cache is false.
        let memory = EvaluationMemory::new(
            test_now(),
            None,
            EvaluationMemoryOptions {
                enable_cache: false,
                evaluation_only: true,
            },
        );
        let str_type = schema::ValueType::Basic(schema::BasicValueType::Str);
        let result = memory
            .get_cache_entry(|| Ok(test_fingerprint()), &str_type, None)
            .unwrap();
        assert!(
            result.is_none(),
            "get_cache_entry should return None when cache is disabled"
        );
    }
}
