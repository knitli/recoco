#![allow(unused_imports)]

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

pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};
pub use futures::{FutureExt, StreamExt};
pub use futures::{
    future::{BoxFuture, Shared},
    prelude::*,
    stream::BoxStream,
};
pub use indexmap::{IndexMap, IndexSet};
pub use itertools::Itertools;
pub use serde::{Deserialize, Serialize, de::DeserializeOwned};
pub use std::any::Any;
pub use std::borrow::Cow;
pub use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
pub use std::fmt::Debug;
pub use std::hash::Hash;
pub use std::sync::{Arc, LazyLock, Mutex, OnceLock, RwLock, Weak};

pub use crate::base::{self, schema, spec, value};
pub use crate::builder::{self, exec_ctx, plan};
pub use crate::execution;
pub use crate::lib_context::{FlowContext, LibContext, get_lib_context, get_runtime};
pub use crate::ops::interface;
pub use crate::setup;
pub use crate::setup::AuthRegistry;
pub use recoco_utils as utils;
#[cfg(any(
    feature = "target-kuzu",
    feature = "function-embed",
    feature = "function-extract-llm"
))]
pub use recoco_utils::http;
pub use recoco_utils::{api_bail, api_error};
#[cfg(any(feature = "function-embed", feature = "source-azure", feature = "source-gdrive", feature = "source-s3", feature = "source-local-file", feature = "source-postgres"))]
pub use recoco_utils::batching;
pub use recoco_utils::{concur_control, retryable};

pub use async_stream::{stream, try_stream};
pub use tracing::{Span, debug, error, info, info_span, instrument, trace, warn};

pub use utils::prelude::*;
