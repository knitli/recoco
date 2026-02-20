<!--
SPDX-FileCopyrightText: 2025-2026 CocoIndex (upstream)
SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
SPDX-FileContributor: CocoIndex Contributors

SPDX-License-Identifier: Apache-2.0
-->

# Recoco Examples

This directory contains examples demonstrating how to use Recoco as a Rust library.

## Prerequisite

Build the project:
```bash
cargo build -p recoco
```

## Transient Flow (Hello World)

A transient flow processes data in-memory without persistent state (no database required).

Run the example:
```bash
cargo run -p recoco --example transient --features function-split
```

This example:
1. Initializes the library context (in-memory).
2. Defines a flow that takes string input.
3. Splits the string by spaces.
4. Returns the result.
