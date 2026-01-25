<!--
SPDX-FileCopyrightText: 2026 Knitli Inc. (ReCoco)
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: Apache-2.0
-->

# ReCoco Splitters

**Intelligent text splitting and parsing for [ReCoco](https://github.com/knitli/recoco).**

This crate implements sophisticated text splitting strategies, primarily leveraging **[Tree-sitter](https://tree-sitter.github.io/tree-sitter/)** to perform syntax-aware chunking of source code and structured documents.

## 🚀 Why Tree-sitter?

Standard text splitters often break code in the middle of functions or classes, destroying context. `recoco-splitters` understands the syntax of the language it is processing, ensuring that chunks respect logical boundaries (e.g., keeping a whole function together).

## 📦 Supported Languages

To minimize binary size, every language parser is feature-gated. Enable only what you need in your `Cargo.toml`.

```toml
[dependencies]
recoco-splitters = { version = "...", features = ["python", "rust"] }
```

| Feature | Language |
|---------|----------|
| `c` | C |
| `c-sharp` | C# |
| `cpp` | C++ |
| `css` | CSS |
| `go` | Go |
| `html` | HTML |
| `java` | Java |
| `javascript` | JavaScript |
| `json` | JSON |
| `kotlin` | Kotlin |
| `markdown` | Markdown |
| `php` | PHP |
| `python` | Python |
| `ruby` | Ruby |
| `rust` | Rust |
| `sql` | SQL |
| `typescript` | TypeScript |
| `yaml` | YAML |
| ... | (See `Cargo.toml` for full list) |

## 🧩 Splitter Strategies

- **Recursive Character Splitter**: Standard splitting by separators (paragraphs, newlines, etc.).
- **Recursive Syntax Splitter**: Tree-sitter based splitting that respects code blocks and syntax nodes.

## 📄 License

Apache-2.0. See [main repository](https://github.com/knitli/recoco) for details.
