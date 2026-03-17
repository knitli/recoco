#!/usr/bin/env python3
"""Generate synthetic benchmark data at multiple scales.

Produces three tiers of test data:
  - small  (~1 KB)   : quick iteration, cache/startup benchmarks
  - medium (~100 KB)  : typical document size
  - large  (~10 MB)   : stress test, throughput measurement

Each tier has:
  - prose.txt       : English-like paragraph text
  - code_rust.rs    : Synthetic Rust source code
  - code_python.py  : Synthetic Python source code
  - mixed.txt       : Alternating prose and code blocks (markdown-ish)
"""

import os
import textwrap
from pathlib import Path

DATA_DIR = Path(__file__).parent / "data"

# ---------------------------------------------------------------------------
# Generators
# ---------------------------------------------------------------------------

LOREM = (
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. "
    "Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. "
    "Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris "
    "nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in "
    "reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla "
    "pariatur. Excepteur sint occaecat cupidatat non proident, sunt in "
    "culpa qui officia deserunt mollit anim id est laborum."
)


def generate_prose(target_bytes: int) -> str:
    """Generate paragraph-structured English text."""
    paragraphs = []
    size = 0
    i = 0
    while size < target_bytes:
        # Vary paragraph length
        sentences = ((i % 5) + 2)
        para = " ".join([LOREM] * sentences)
        paragraphs.append(para)
        size += len(para) + 2  # +2 for \n\n
        i += 1
    return "\n\n".join(paragraphs)[:target_bytes]


def generate_rust_code(target_bytes: int) -> str:
    """Generate synthetic Rust source code with realistic structure."""
    template = textwrap.dedent("""\
        /// Documentation for function_{n}.
        ///
        /// This function performs computation number {n} on the input data,
        /// transforming it through a series of operations.
        pub fn function_{n}(input: &str, count: usize) -> Result<Vec<String>, Error> {{
            let mut results = Vec::with_capacity(count);
            for i in 0..count {{
                let processed = input
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| format!("{{}}:{{}}", i, line.trim()))
                    .collect::<Vec<_>>();
                if processed.is_empty() {{
                    return Err(Error::new("empty input at iteration {{}}", i));
                }}
                results.extend(processed);
            }}
            Ok(results)
        }}

        #[cfg(test)]
        mod tests_{n} {{
            use super::*;

            #[test]
            fn test_function_{n}_basic() {{
                let result = function_{n}("hello\\nworld", 2).unwrap();
                assert_eq!(result.len(), 4);
            }}

            #[test]
            fn test_function_{n}_empty() {{
                let result = function_{n}("", 1);
                assert!(result.is_err());
            }}
        }}

    """)
    chunks = []
    chunks.append("use std::fmt;\n\n#[derive(Debug)]\npub struct Error(String);\n\n")
    chunks.append("impl Error {\n    pub fn new(msg: &str, ctx: usize) -> Self {\n")
    chunks.append("        Self(format!(\"{}: {}\", msg, ctx))\n    }\n}\n\n")
    size = sum(len(c) for c in chunks)
    n = 0
    while size < target_bytes:
        block = template.format(n=n)
        chunks.append(block)
        size += len(block)
        n += 1
    return "".join(chunks)[:target_bytes]


def generate_python_code(target_bytes: int) -> str:
    """Generate synthetic Python source code."""
    template = textwrap.dedent("""\
        class Processor{n}:
            \"\"\"Process data using strategy {n}.

            This class implements a multi-step processing pipeline
            that transforms input data through filtering, mapping,
            and aggregation stages.
            \"\"\"

            def __init__(self, config: dict | None = None):
                self.config = config or {{}}
                self._cache = {{}}

            def process(self, items: list[str]) -> list[str]:
                \"\"\"Process a list of items and return transformed results.\"\"\"
                results = []
                for item in items:
                    if not item.strip():
                        continue
                    transformed = self._transform(item)
                    if transformed not in self._cache:
                        self._cache[transformed] = len(self._cache)
                    results.append(f"{{self._cache[transformed]}}:{{transformed}}")
                return results

            def _transform(self, item: str) -> str:
                return item.strip().lower().replace(" ", "_")


        def run_processor_{n}(data: list[str]) -> list[str]:
            \"\"\"Convenience function for Processor{n}.\"\"\"
            proc = Processor{n}()
            return proc.process(data)


    """)
    chunks = ['"""Auto-generated benchmark data."""\n\n']
    size = len(chunks[0])
    n = 0
    while size < target_bytes:
        block = template.format(n=n)
        chunks.append(block)
        size += len(block)
        n += 1
    return "".join(chunks)[:target_bytes]


def generate_mixed(target_bytes: int) -> str:
    """Generate markdown-ish mixed prose and code."""
    sections = []
    size = 0
    n = 0
    while size < target_bytes:
        section = f"## Section {n}\n\n"
        section += LOREM + "\n\n"
        section += "```rust\n"
        section += f'pub fn example_{n}(x: i32) -> i32 {{\n    x * {n + 1}\n}}\n'
        section += "```\n\n"
        section += f"The function above multiplies the input by {n + 1}.\n\n"
        sections.append(section)
        size += len(section)
        n += 1
    return "".join(sections)[:target_bytes]


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

TIERS = {
    "small": 1_024,          # ~1 KB
    "medium": 100_000,       # ~100 KB
    "large": 10_000_000,     # ~10 MB
}

GENERATORS = {
    "prose.txt": generate_prose,
    "code_rust.rs": generate_rust_code,
    "code_python.py": generate_python_code,
    "mixed.txt": generate_mixed,
}


def main():
    for tier, target in TIERS.items():
        tier_dir = DATA_DIR / tier
        tier_dir.mkdir(parents=True, exist_ok=True)
        for filename, gen_fn in GENERATORS.items():
            path = tier_dir / filename
            content = gen_fn(target)
            path.write_text(content)
            actual = len(content.encode("utf-8"))
            print(f"  {tier}/{filename}: {actual:,} bytes")

    print("\nDone. Data written to:", DATA_DIR)


if __name__ == "__main__":
    main()
