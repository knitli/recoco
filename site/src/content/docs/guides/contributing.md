---
title: Contributing
---
<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. (ReCoco)
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

We love contributions! Here is how to get started.

## Quick Links

- 📝 **[Open an Issue](https://github.com/knitli/recoco/issues/new)** - Report bugs or request features
- 💬 **[Discussions](https://github.com/knitli/recoco/discussions)** - Ask questions and share ideas
- 🔀 **[Pull Requests](https://github.com/knitli/recoco/pulls)** - View open PRs
- 📋 **[Issue Tracker](https://github.com/knitli/recoco/issues)** - Browse existing issues
- 📖 **[README](https://github.com/knitli/recoco/blob/main/README.md)** - Project overview

## Should I Submit My Issue Here or Upstream at CocoIndex?

That depends.

### Submit your issue to CocoIndex when it...

- Directly touches any file in the [`ops` module](https://github.com/knitli/recoco/blob/main/crates/recoco/src/ops/mod.rs). We regularly merge changes to this module into ReCoco, so please help everyone by submitting upstream.
- If you want to add a new function, source, or target.  Minimally submit a feature request upstream. If they reject it, you can resubmit it here for consideration, but we'll only consider new functions/targets/sources once they've been rejected by CocoIndex. (We're likely to accept new additions here because we feature gate them -- there's no extra weight to adding it).

**Submitting to CocoIndex**: First, please read and follow their contribution guidelines

## Development

This is a Rust project managed by Cargo.

### Build

Build the project:

```bash
cargo build
```

### Test

Run the test suite:

```bash
cargo test
```

Ensure code is linted and formatted:

```bash
cargo clippy
cargo fmt
```

## Pull Requests

- **Use Conventional Commits**: We follow [Conventional Commits](https://www.conventionalcommits.org/). Start your commit messages with `feat:`, `fix:`, `docs:`, etc. This is required for our changelog generation (via `git cliff`).
- **Keep it small**: Focus on one logical change per pull request.
- **Test your changes**: Run the tests above before submitting.

Thank you for helping improve ReCoco!