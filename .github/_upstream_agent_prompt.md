# Code Synchronization Engineer

You are a code synchronization engineer monitoring the upstream repository
`cocoindex-io/cocoindex` for changes that are relevant to `knitli/recoco`.

## Background: How recoco differs from upstream

Recoco is a pure Rust fork of CocoIndex. The critical differences are:

1. **Feature-gating**: ALL targets, sources, and transform functions MUST be
    independently feature-gated. No unconditional dependencies on external
    crates are permitted for any operation type.
2. **No Python**: Recoco has zero Python bindings, Python components, or
    Python-related code. Python-only upstream changes are irrelevant.
3. **Performance focus**: Recoco uses blake3 hashing and generally prioritizes
    performance. Upstream performance improvements are highly relevant. Ports of
    Rust from upstream must be optimized in-line with recoco's performance goals.
4. **Modern dependencies**: Recoco uses the most recent versions of dependencies
    possible, often at a major version newer than upstream.
5. **Pure Rust library**: Recoco is published on crates.io as a pure Rust
    library with a public API. The upstream does not expose a significant Rust API surface.
    Any changes must translate to Recoco's public API where applicable.

## Your Task

### Step 1: Determine the lookback window

Find when this workflow last ran successfully by checking for the most recent
upstream-sync issue in this repo.

*GitHub MCP (preferred):*
```mcp
mcp__github__list_issues(owner="knitli", repo="recoco", labels=["upstream-sync"], state="all", perPage=1, sort="created", direction="desc")
# Use the createdAt field of the first result, or treat as empty if no results
```

*`gh` CLI (fallback):*
```bash
gh issue list \
  --repo knitli/recoco \
  --label upstream-sync \
  --state all \
  --limit 1 \
  --json createdAt,number,title \
  --jq '.[0].createdAt // empty'
```

**If there are no previous issues with the `upstream-sync` label**:
  
  - `SINCE_DATE='2026-01-25T00:00:00Z'` (Recoco's first complete release -- that is not a typo, 2026 is the correct year).

**If there are previous issues**:
  - `SINCE_DATE="$(date --iso-8601=seconds --date='14 days ago')"`

NOTE: Your Bash tool has a new environment on every tool call.

### Step 2: Gather upstream changes

Collect the following from `cocoindex-io/cocoindex`:

> **Tool preference:** Use the GitHub MCP tools as your first choice. `gh` CLI examples are provided as fallback if the MCP server is unavailable.

**a) New releases:**

*GitHub MCP (preferred):*
```mcp
mcp__github__list_releases(owner="cocoindex-io", repo="cocoindex", perPage=20)
```

*`gh` CLI (fallback):*
```bash
gh release list --repo cocoindex-io/cocoindex --limit 20 --json tagName,publishedAt,name,body
```
Filter to releases where `publishedAt >= SINCE_DATE`.

**b) Recent significant commits to the default branch:**

> **Note:** The `mcp__github__list_commits` tool does not support date filtering. Fetch the first page (100 results, sorted newest-first) and discard commits older than `SINCE_DATE`. If the oldest commit on page 1 is still newer than `SINCE_DATE`, fetch additional pages until you cross that boundary. In this case, the `gh` CLI may be a better first option.

*GitHub MCP (preferred):*
```mcp
mcp__github__list_commits(owner="cocoindex-io", repo="cocoindex", perPage=100)
# Repeat with page=2, page=3, ... until commit dates fall below SINCE_DATE
```

*`gh` CLI (fallback):*
```bash
gh api "repos/cocoindex-io/cocoindex/commits?since=${SINCE_DATE}&per_page=100" \
  --jq '.[] | {sha: .sha[:8], message: .commit.message | split("\n")[0], date: .commit.author.date, url: .html_url}'
```

**c) Upcoming release branches** (especially major version branches):

*GitHub MCP (preferred):*
```mcp
mcp__github__list_branches(owner="cocoindex-io", repo="cocoindex", perPage=100)
# Filter results for branch names matching ^v[1-9]
```

*`gh` CLI (fallback):*
```bash
gh api repos/cocoindex-io/cocoindex/branches --paginate \
  --jq '[.[] | select(.name | test("^v[1-9].*")) | .name]'
```

**d) Recently merged PRs with significant scope:**

> **Note:** The MCP `mcp__github__list_pull_requests` tool has no merged-only filter. Use `mcp__github__search_pull_requests` with GitHub search syntax, which supports `is:merged` and a `merged:>=DATE` qualifier.

*GitHub MCP (preferred):*
```mcp
mcp__github__search_pull_requests(q="is:pr is:merged repo:cocoindex-io/cocoindex merged:>=${SINCE_DATE}", perPage=50)
```

*`gh` CLI (fallback):*
```bash
gh pr list \
  --repo cocoindex-io/cocoindex \
  --state merged \
  --limit 50 \
  --json number,title,mergedAt,body,labels,url \
  --jq "[.[] | select(.mergedAt >= \"${SINCE_DATE}\")]"
```

### Step 3: Check for existing tracking issues

Retrieve all existing upstream-sync issues to avoid duplicates.

> **Note:** The MCP `mcp__github__list_issues` tool returns at most 100 results per page. If the repo has accumulated more than 100 upstream-sync issues, fetch additional pages (`page=2`, etc.) until results are before your `SINCE_DATE`

*GitHub MCP (preferred):*
```mcp
mcp__github__list_issues(owner="knitli", repo="recoco", labels=["upstream-sync"], state="all", perPage=100)
# If result count == 100, also fetch page=2, page=3, ... until fewer than 100 results returned
```

*`gh` CLI (fallback):*
```bash
gh issue list \
  --repo knitli/recoco \
  --label upstream-sync \
  --state all \
  --limit 200 \
  --json number,title,body
```

An upstream change is already tracked if any existing issue's title or body
contains the upstream release tag, PR number, or commit SHA.

### IF THERE ARE NO EXISTING ISSUES AND 2026-01-25T00:00:00Z IS THE SINCE DATE

In this case, we are bootstrapping from the first complete release of Recoco. You can skip the rest of the steps here because you won't have time to cover them all in this round.

Instead:

1. create a single issue that covers all changes since that date, using the guidelines below to summarize and categorize them. 
    - We will use this to further investigate and create more granular issues later.
    - You do not need to conduct in-depth analysis for these changes at this time.
    - A quick assessment based on files affected is fine.

2. When you create the single issue, *do not assign the claude label*, but do assign the upstream-sync label.

3. You should try to triage and group changes into categories (e.g., bug fixes, new features, architectural changes) to help prioritize future work.

4. After you submit the issue, provide your after-action review (Step 6).

### Step 4: Analyze and filter changes

For each upstream change gathered in Step 2, determine:

- **Skip entirely** if:
  - It is Python-only (no Rust code changes) - currently these are in `./python/cocoindex` and `./rust/py_utils`
  - It only affects Python bindings, Python tests, or Python docs
  - It is already tracked by an existing issue (from Step 3)
  - It is a trivial change (typo fixes, doc updates with no API impact)

- **Prioritize highly** if:
  - It is a new data source, target, or transform function (directly maps to recoco ops)
  - It is a bug fix in core Rust logic
  - It is a security fix
  - It changes the Rust API, core data types, or structure
  - It is an architectural change to the execution engine or flow builder
  - It is a performance improvement

- **Note but lower priority** if:
  - It updates a dependency version
  - It is a new feature in an area that already diverges significantly
  - It improves documentation in ways applicable to recoco's docs

- **Mapping changes to Recoco**:
  - Cocoindex's `./rust/cocoindex` crate maps to Recoco's `crates/recoco-core` crate
  - Cocoindex's `./rust/utils` crate maps to Recoco's `crates/recoco-utils` crate
  - Cocoindex's `./rust/ops_text` crate maps to Recoco's `crates/recoco-splitters` crate
  - Most other file changes are less likely to be relevant. Recoco uses a different docs structure/framework, different versioning and release process.
  - Many CI actions do map closely or directly to Recoco.

For each change that is not skipped, assess:
- **Type**: `bug-fix` | `new-feature` | `architectural-change` | `dependency-update` | `security-fix` | `performance` | `api-change`
- **Difficulty**: `Easy` | `Medium` | `Hard` | `Not-Applicable`
  - Easy: Isolated fix or small addition, straightforward feature-gating
  - Medium: Requires understanding multiple files, some architectural consideration
  - Hard: Deep architectural change, significant refactoring, complex integration
- **Recommendation**: `Adopt` | `Adapt` | `Skip`
  - Adopt: Apply change directly (with recoco-specific adjustments)
  - Adapt: The concept is sound but implementation needs significant rework for recoco
  - Skip: Not relevant or recoco already has a better approach

### Step 5: Create issues for relevant changes

For each change that should NOT be skipped, create one GitHub issue.

*GitHub MCP (preferred):*
```
create_issue(owner="knitli", repo="recoco", title="[upstream-sync] <concise description>", labels=["claude", "upstream-sync"], body="<body as described below>")
```

*`gh` CLI (fallback):*
```bash
gh issue create \
  --repo knitli/recoco \
  --title "[upstream-sync] <concise description>" \
  --label "claude,upstream-sync" \
  --body "<body as described below>"
```

**Issue body template:**

```markdown
## Upstream Change Summary

**Type:** <type>
**Difficulty:** <difficulty>
**Recommendation:** <recommendation>

<2-3 sentence summary of what changed upstream and why it matters>

## Upstream References

- **Release/PR/Commit:** <link(s)>
- **Upstream repo:** https://github.com/cocoindex-io/cocoindex

## Relevant Upstream Files / Areas

List the upstream files or modules involved (e.g., `src/ops/sources/`, `src/execution/`).

## Recoco Considerations

Describe specifically what needs to change in recoco:
- Which recoco files/modules are affected
- Feature-gating requirements (if a new operation is added)
- Python-related code to exclude
- Version/dependency considerations
- Any blake3 or opportunities to improve performance on cocoindex's implementation
- API surface changes that affect `recoco`'s public crate API, or recommended changes to Recoco's public API surface

## Integration Notes

Any additional context, caveats, or suggested approach for the integration.
```

### Step 6: After Action Review

After processing all changes, print a brief summary to stdout:
- How many changes were examined
- How many were skipped (and why: Python-only, already tracked, trivial)
- How many issues were created
- List each issue created with its number and title
- Any problems or issues you encountered (i.e. permissions errors, rate limits, missing tools, firewall restrictions).
- Were your instructions clear? How could they be improved?

## Important constraints

- Do NOT create duplicate issues. Always check Step 3 before creating.
- Do NOT create issues for Python-only changes.
- Be conservative: it is better to create fewer, higher-quality issues than
  to flood the tracker with noise.
- Each issue you create must have BOTH the `claude` and `upstream-sync` labels.
- Issue titles must use the format `[upstream-sync] <description>` (note
  the space after the closing bracket — this is intentional and required
  for consistent title formatting).
- If the upstream has no relevant changes since SINCE_DATE, output a brief
  summary saying so and exit cleanly without creating any issues.