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

use async_stream::try_stream;
use std::borrow::Cow;
use std::fs::Metadata;
use std::path::Path;
use std::{path::PathBuf, sync::Arc};
use tracing::warn;

use crate::base::field_attrs;
use crate::{fields_value, ops::sdk::*};
use recoco_splitters::pattern_matcher::PatternMatcher;

#[derive(Debug, Serialize, Deserialize)]
pub struct Spec {
    path: String,
    binary: bool,
    included_patterns: Option<Vec<String>>,
    excluded_patterns: Option<Vec<String>>,
    max_file_size: Option<i64>,
    watch_changes: Option<bool>,
}

struct Executor {
    root_path: PathBuf,
    canonical_root_path: Option<PathBuf>,
    binary: bool,
    pattern_matcher: PatternMatcher,
    max_file_size: Option<i64>,
    watch_changes: bool,
}

async fn ensure_metadata<'a>(
    path: &Path,
    metadata: &'a mut Option<Metadata>,
) -> std::io::Result<&'a Metadata> {
    if metadata.is_none() {
        // Follow symlinks.
        *metadata = Some(tokio::fs::metadata(path).await?);
    }
    Ok(metadata.as_ref().unwrap())
}

#[async_trait]
impl SourceExecutor for Executor {
    async fn list(
        &self,
        options: &SourceExecutorReadOptions,
    ) -> Result<BoxStream<'async_trait, Result<Vec<PartialSourceRow>>>> {
        let root_component_size = self.root_path.components().count();
        let mut dirs = Vec::new();
        dirs.push(Cow::Borrowed(&self.root_path));
        let mut new_dirs = Vec::new();
        let stream = try_stream! {
            while let Some(dir) = dirs.pop() {
                let mut entries = match tokio::fs::read_dir(dir.as_ref()).await {
                    Ok(entries) => entries,
                    Err(e) => {
                        warn!("Failed to read directory {}: {}", dir.display(), e);
                        continue;
                    }
                };
                loop {
                    let entry = match entries.next_entry().await {
                        Ok(Some(entry)) => entry,
                        Ok(None) => break,
                        Err(e) => {
                            warn!("Failed to read directory entry in {}: {}", dir.display(), e);
                            continue;
                        }
                    };
                    let path = entry.path();
                    let mut path_components = path.components();
                    for _ in 0..root_component_size {
                        path_components.next();
                    }
                    let Some(relative_path) = path_components.as_path().to_str() else {
                        warn!("Skipped ill-formed file path: {}", path.display());
                        continue;
                    };
                    // We stat per entry at most once when needed.
                    let mut metadata: Option<Metadata> = None;

                    // For symlinks, if the target doesn't exist, log and skip.
                    let file_type = match entry.file_type().await {
                        Ok(ft) => ft,
                        Err(e) => {
                            warn!("Failed to get file type for {}: {}", path.display(), e);
                            continue;
                        }
                    };
                    if file_type.is_symlink()
                        && let Err(e) = ensure_metadata(&path, &mut metadata).await {
                            warn!("Skipped symlink {}: {}", path.display(), e);
                            continue;
                        }
                    let is_dir = if file_type.is_dir() {
                        true
                    } else if file_type.is_symlink() {
                        // Follow symlinks to classify the target.
                        match ensure_metadata(&path, &mut metadata).await {
                            Ok(md) => md.is_dir(),
                            Err(e) => {
                                warn!("Failed to get metadata for symlink {}: {}", path.display(), e);
                                continue;
                            }
                        }
                    } else {
                        false
                    };
                    if is_dir {
                        if !self.pattern_matcher.is_excluded(relative_path) {
                            new_dirs.push(Cow::Owned(path));
                        }
                    } else if self.pattern_matcher.is_file_included(relative_path) {
                        // Check file size limit
                        if let Some(max_size) = self.max_file_size
                            && let Ok(metadata) = ensure_metadata(&path, &mut metadata).await
                            && metadata.len() > max_size as u64
                        {
                            continue;
                        }
                        let ordinal: Option<Ordinal> = if options.include_ordinal {
                            match ensure_metadata(&path, &mut metadata).await {
                                Ok(md) => match md.modified() {
                                    Ok(modified) => match modified.try_into() {
                                        Ok(ord) => Some(ord),
                                        Err(e) => {
                                            warn!("Failed to convert modification time for {}: {}", path.display(), e);
                                            None
                                        }
                                    },
                                    Err(e) => {
                                        warn!("Failed to get modification time for {}: {}", path.display(), e);
                                        None
                                    }
                                },
                                Err(e) => {
                                    warn!("Failed to get metadata for {}: {}", path.display(), e);
                                    None
                                }
                            }
                        } else {
                            None
                        };
                        yield vec![PartialSourceRow {
                            key: KeyValue::from_single_part(relative_path.to_string()),
                            key_aux_info: serde_json::Value::Null,
                            data: PartialSourceRowData {
                                ordinal,
                                content_version_fp: None,
                                value: None,
                            },
                        }];
                    }
                }
                dirs.extend(new_dirs.drain(..).rev());
            }
        };
        Ok(stream.boxed())
    }

    async fn get_value(
        &self,
        key: &KeyValue,
        _key_aux_info: &serde_json::Value,
        options: &SourceExecutorReadOptions,
    ) -> Result<PartialSourceRowData> {
        let path = key.single_part()?.str_value()?.as_ref();
        let path_obj = Path::new(path);

        // Prevent path traversal vulnerabilities by verifying the path
        // doesn't contain parent directory or absolute components.
        if path_obj.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        }) || !self.pattern_matcher.is_file_included(path)
        {
            return Ok(PartialSourceRowData {
                value: Some(SourceValue::NonExistence),
                ordinal: Some(Ordinal::unavailable()),
                content_version_fp: None,
            });
        }

        let path = self.root_path.join(path);

        // Mitigate symlink-based path traversal by canonicalizing and checking boundaries
        if let Some(root_canon) = &self.canonical_root_path {
            let path_canon = match tokio::fs::canonicalize(&path).await {
                Ok(c) => c,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // Target file doesn't exist.
                    return Ok(PartialSourceRowData {
                        value: Some(SourceValue::NonExistence),
                        ordinal: Some(Ordinal::unavailable()),
                        content_version_fp: None,
                    });
                }
                Err(e) => Err(e)?,
            };

            if !path_canon.starts_with(root_canon) {
                // Symlink points outside the allowed root directory.
                return Ok(PartialSourceRowData {
                    value: Some(SourceValue::NonExistence),
                    ordinal: Some(Ordinal::unavailable()),
                    content_version_fp: None,
                });
            }
        } else {
            // Root doesn't exist (failed to canonicalize during setup), so the file cannot exist.
            return Ok(PartialSourceRowData {
                value: Some(SourceValue::NonExistence),
                ordinal: Some(Ordinal::unavailable()),
                content_version_fp: None,
            });
        }

        let mut metadata: Option<Metadata> = None;
        // Check file size limit
        if let Some(max_size) = self.max_file_size
            && let Ok(metadata) = ensure_metadata(&path, &mut metadata).await
            && metadata.len() > max_size as u64
        {
            return Ok(PartialSourceRowData {
                value: Some(SourceValue::NonExistence),
                ordinal: Some(Ordinal::unavailable()),
                content_version_fp: None,
            });
        }
        let ordinal = if options.include_ordinal {
            let metadata = ensure_metadata(&path, &mut metadata).await?;
            Some(metadata.modified()?.try_into()?)
        } else {
            None
        };
        let value = if options.include_value {
            match std::fs::read(path) {
                Ok(content) => {
                    let content = if self.binary {
                        fields_value!(content)
                    } else {
                        let (s, _) = utils::bytes_decode::bytes_to_string(&content);
                        fields_value!(s)
                    };
                    Some(SourceValue::Existence(content))
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    Some(SourceValue::NonExistence)
                }
                Err(e) => Err(e)?,
            }
        } else {
            None
        };
        Ok(PartialSourceRowData {
            value,
            ordinal,
            content_version_fp: None,
        })
    }

    fn provides_ordinal(&self) -> bool {
        true
    }

    async fn change_stream(
        &self,
    ) -> Result<Option<BoxStream<'async_trait, Result<SourceChangeMessage>>>> {
        if !self.watch_changes {
            return Ok(None);
        }

        use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
        use tokio::sync::mpsc;

        let root_path = self.root_path.clone();
        let root_component_size = root_path.components().count();
        let pattern_matcher = self.pattern_matcher.clone();

        let (tx, mut rx) = mpsc::channel::<PathBuf>(100);

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| match res {
                Ok(event) => {
                    for path in event.paths {
                        if let Err(err) = tx.try_send(path) {
                            use tokio::sync::mpsc::error::TrySendError;
                            match err {
                                TrySendError::Full(_) => {
                                    warn!(
                                        "File watcher channel is full; dropping file change event"
                                    );
                                }
                                TrySendError::Closed(_) => {
                                    warn!(
                                        "File watcher channel is closed; dropping file change event"
                                    );
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("File watcher error: {}", e);
                }
            },
            Config::default(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to create file watcher: {}", e))?;

        watcher
            .watch(&root_path, RecursiveMode::Recursive)
            .map_err(|e| anyhow::anyhow!("Failed to watch path: {}", e))?;

        let stream = async_stream::stream! {
            // Keep the watcher alive for the duration of the stream
            let _watcher = watcher;

            while let Some(path) = rx.recv().await {
                // Skip directory paths - notify can emit events for directories,
                // and reading a directory as a file would produce an EISDIR error.
                let is_dir = match std::fs::metadata(&path) {
                    Ok(metadata) => metadata.is_dir(),
                    Err(err) => {
                        // If the file no longer exists, this may be a deletion event; do not skip it.
                        if err.kind() != std::io::ErrorKind::NotFound {
                            warn!("Failed to read metadata for path {:?}: {}", path, err);
                        }
                        false
                    }
                };
                if is_dir {
                    continue;
                }

                let mut path_components = path.components();
                for _ in 0..root_component_size {
                    path_components.next();
                }
                let Some(relative_path) = path_components.as_path().to_str() else {
                    continue;
                };

                // Skip events that correspond to the root directory itself or yield no relative path.
                if relative_path.is_empty() {
                    continue;
                }

                // Filter through pattern matcher
                if pattern_matcher.is_file_included(relative_path) {
                    yield Ok(SourceChangeMessage {
                        changes: vec![SourceChange {
                            key: KeyValue::from_single_part(relative_path.to_string()),
                            key_aux_info: serde_json::Value::Null,
                            data: PartialSourceRowData {
                                ordinal: None,
                                content_version_fp: None,
                                value: None,
                            },
                        }],
                        ack_fn: None,
                    });
                }
            }
        };

        Ok(Some(stream.boxed()))
    }
}

pub struct Factory;

#[async_trait]
impl SourceFactoryBase for Factory {
    type Spec = Spec;

    fn name(&self) -> &str {
        "LocalFile"
    }

    async fn get_output_schema(
        &self,
        spec: &Spec,
        _context: &FlowInstanceContext,
    ) -> Result<EnrichedValueType> {
        let mut struct_schema = StructSchema::default();
        let mut schema_builder = StructSchemaBuilder::new(&mut struct_schema);
        let filename_field = schema_builder.add_field(FieldSchema::new(
            "filename",
            make_output_type(BasicValueType::Str),
        ));
        schema_builder.add_field(FieldSchema::new(
            "content",
            make_output_type(if spec.binary {
                BasicValueType::Bytes
            } else {
                BasicValueType::Str
            })
            .with_attr(
                field_attrs::CONTENT_FILENAME,
                serde_json::to_value(filename_field.to_field_ref())?,
            ),
        ));

        Ok(make_output_type(TableSchema::new(
            TableKind::KTable(KTableInfo { num_key_parts: 1 }),
            struct_schema,
        )))
    }

    async fn build_executor(
        self: Arc<Self>,
        _source_name: &str,
        spec: Spec,
        _context: Arc<FlowInstanceContext>,
    ) -> Result<Box<dyn SourceExecutor>> {
        let root_path = PathBuf::from(spec.path);
        let canonical_root_path = tokio::fs::canonicalize(&root_path).await.ok();

        Ok(Box::new(Executor {
            root_path,
            canonical_root_path,
            binary: spec.binary,
            pattern_matcher: PatternMatcher::new(spec.included_patterns, spec.excluded_patterns)?,
            max_file_size: spec.max_file_size,
            watch_changes: spec.watch_changes.unwrap_or(false),
        }))
    }
}
