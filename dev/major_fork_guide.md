<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

# Forking a major version change

## Clone and Clean
- Clone the upstream repository
- move ./rust/core/ to ./crates/recoco
- move ./rust/extra_text/ to ./crates/recoco0splitters
- move ./rust/utils/ to ./crates/recoco-utils
- in ReCoco repo:
  - delete ./crates, but preserve any sources/functions/targets not in the CocoIndex pull
  - Retain the `builder` module and files, moving them to a temporary location
  - copy over the new ./crates
  - Add back in any sources/functions/targets not in the new code

### Reintegrate Builder

- ReCoco's flow builder implementation has some significant differences to best align it with a rust-only API. This is the main "heavy lift" -- you need to identify what has changed about this module, compare the changes with the current ReCoco implementation, and either update the new crate to incorporate the Recoco functionality/considerations or vice versa. 

## Update to Align with ReCoco

- Updates:
  - Copy over any sources/functions/targets from ReCoco that aren't in the current repository
  - Add feature gates to all crates aligned with the Cargo.tomls you just copied over
  - Replace use of `blake2` with `blake3`, replace use of `derivative` with `derive_where`, migrate `schemars` < 1.0 to `schemars` 1.2+
     - Blake3 has a simpler API and is **much** faster than Blake2; you can usually just replace the construction, the update logic is the same but make sure to use references if not already that way; currently the only use is in `recoco_utils::fingerprint`
     - If Fingerprint's functions aren't marked with `#[inline(always)]`, add the attribute to improve performance -- it's a hotpath
     - derive_where is actively maintained, unlike derivative, and is essentially a drop-in replacement; replace `Derivative` with `derive_where` or remove the import and use `derive_where::derive_where`
     - Schemars adjustments are more involved; the API changed significantly. Research each change. (See below for a recap of what we changed on the original fork)
  - Remove use of `indicatif` and `owo_colors` from crates (CLI dependencies)
  - Update references in the crates to reflect the new structure and dependencies
  - Identify any new dependencies not already in the current repository and add them to the appropriate Cargo.toml files; be sure to evaluate their necessity and impact on the project and feature-gate if appropriate
  - Update any changed/different paths in CI/CD or references in docs/examples to reflect the new structure and dependencies (if the crates' internal structure changed)
  - Run `reuse annotate --year 2026 -r --fallback-dot-license -c 'CocoIndex (upstream)' -c 'Knitli Inc. (ReCoco)' -l Apache-2.0`
  - Specifically make sure `dev/sync_upstream_ops.py` is updated to reflect path changes.

## Final Checks and QA

  - Ensure all examples in ./examples and in docs, readme, CLAUDE.md, etc are updated to use any new patterns and reflect the new crate structure and dependencies.
  - Test the build and run all tests to ensure everything works correctly after the migration and updates
  - Document any changes made to the API or behavior in the CHANGELOG.md to ensure users are aware of the updates and any potential breaking changes.
  - Commit changes, ensuring that the commit message reflects the migration and breaking changes.
  - Ensure git cliff increments a major version.
  - Verify that the changelog has been updated correctly and reflects the major version change.
  - merge and release.


  ---
  
  ### Schemars Changes from 0.8 to 1.2+

`json_schema.rs`

- Changed imports from `schemars::schema::{...}` to `schemars::Schema`
- Rewrote `JsonSchemaBuilder` methods to construct JSON directly using `serde_json`
- Updated `BuildJsonSchemaOutput.schema` from `SchemaObject` to `Schema`
- Updated test helper `schema_to_json` signature
- Updated `test_description_concatenation` to use JSON access methods

`llm/mod.rs`

- Changed import to `schemars::Schema`
- Updated `OutputFormat::JsonSchema.schema` from `Cow<'a, SchemaObject>` to `Cow<'a, Schema>`

`llm/ollama.rs`

- Changed import to `schemars::Schema`
- Updated `OllamaFormat::JsonSchema` from `&'a SchemaObject` to `&'a Schema`

`ops/functions/extract_by_llm.rs`

- Changed import to `schemars::Schema`
- Updated `Executor.output_json_schema` from `SchemaObject` to `Schema`
