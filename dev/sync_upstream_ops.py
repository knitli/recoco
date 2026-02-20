#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)
# SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
#
# SPDX-License-Identifier: Apache-2.0
"""
Sync upstream CocoIndex ops to local Recoco ops, applying necessary replacements.
"""


import os
import re
import shutil
import re
import shutil
import subprocess
import sys
from pathlib import Path

# Configuration
UPSTREAM_URL = "https://github.com/cocoindex-io/cocoindex.git"
CACHE_DIR = Path(".upstream_cache")
LOCAL_ROOT = Path("crates/recoco/src/ops")
UPSTREAM_SUBPATH = Path("rust/cocoindex/src/ops")

# Replacements (Upstream -> Local)
# Basic crates replacements based on observed structure
REPLACEMENTS = [
    (r"cocoindex_utils", "recoco_utils"),
    (r"cocoindex_extra_text", "recoco_splitters"),
    # Add more specific replacements here if needed
]



def run_cmd(cmd: str, cwd: Path | None = None, check: bool = True) -> None:
    """Run a shell command in the specified directory."""
    try:
        subprocess.run(
            cmd,
            cwd=cwd,
            check=check,
            shell=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {cmd}")
        print(f"Stderr: {e.stderr.decode()}")
        raise



def setup_upstream() -> None:
    """Set up the upstream repository in the cache directory."""
    if not CACHE_DIR.exists():
        print(f"Cloning upstream from {UPSTREAM_URL}...")
        try:
            run_cmd(f"git clone {UPSTREAM_URL} {CACHE_DIR}")
        except subprocess.CalledProcessError:
            print("Git clone failed, trying GitHub CLI...")
            run_cmd(f"gh repo clone cocoindex-io/cocoindex {CACHE_DIR}")
    else:
        print("Fetching latest upstream...")
        run_cmd("git fetch origin", cwd=CACHE_DIR)
        run_cmd("git reset --hard origin/main", cwd=CACHE_DIR)



def apply_replacements(content: str) -> str:
    """Apply upstream-to-local replacements to the given content."""
    for pattern, replacement in REPLACEMENTS:
        content = re.sub(pattern, replacement, content)
    return content



def get_file_status(upstream_file: Path, local_file: Path) -> str:
    """Determine the status of a local file compared to the upstream file."""
    if not local_file.exists():
        return "NEW"

    # Read and transform upstream content for comparison
    upstream_content = upstream_file.read_text()
    transformed_upstream = apply_replacements(upstream_content)

    local_content = local_file.read_text()

    return "IDENTICAL" if transformed_upstream == local_content else "MODIFIED"



def scan_ops() -> list[dict]:
    """Scan the upstream ops directory and compare with local ops directory."""
    upstream_root = CACHE_DIR / UPSTREAM_SUBPATH
    changes = []

    if not upstream_root.exists():
        print(f"Error: Upstream ops directory not found at {upstream_root}")
        return []

    for root, _, files in os.walk(upstream_root):
        for file in files:
            if not file.endswith(".rs"):
                continue

            upstream_path = Path(root) / file
            rel_path = upstream_path.relative_to(upstream_root)
            local_path = LOCAL_ROOT / rel_path

            status = get_file_status(upstream_path, local_path)

            if status != "IDENTICAL":
                changes.append(
                    {
                        "rel_path": str(rel_path),
                        "status": status,
                        "upstream_path": upstream_path,
                        "local_path": local_path,
                    }
                )
    return changes



def apply_change(change: dict) -> None:
    """Apply a single change from the upstream to the local ops directory."""
    upstream_path = change["upstream_path"]
    local_path = change["local_path"]

    print(f"Applying {change['rel_path']}...")

    content = upstream_path.read_text()
    transformed_content = apply_replacements(content)

    local_path.parent.mkdir(parents=True, exist_ok=True)
    local_path.write_text(transformed_content)



def main() -> None:
    """Main entry point for syncing upstream ops with local ops."""
    if len(sys.argv) > 1 and sys.argv[1] == "clean":
        if CACHE_DIR.exists():
            shutil.rmtree(CACHE_DIR)
            print("Cache cleaned.")
        return

    setup_upstream()
    changes = scan_ops()

    if not changes:
        print("No changes detected from upstream.")
        return

    print(f"\nFound {len(changes)} changes from upstream:")
    for i, change in enumerate(changes):
        print(f"[{i}] {change['status'].ljust(8)} {change['rel_path']}")

    if len(sys.argv) > 1 and sys.argv[1] == "apply":
        # Apply all or specific
        if len(sys.argv) > 2:
            indices = [int(x) for x in sys.argv[2:]]
            to_apply = [changes[i] for i in indices]
        else:
            to_apply = changes

        for change in to_apply:
            apply_change(change)
        print("Done.")
    else:
        print("\nTo apply changes, run:")
        print(f"  python3 {sys.argv[0]} apply [indices...]")



if __name__ == "__main__":
    main()
