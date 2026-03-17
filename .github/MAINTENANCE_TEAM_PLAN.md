# Knitli Automated Maintenance Team — Architecture Plan

## Overview

A centralized, declarative CI/CD automation system hosted in `knitli/.github`,
serving all Knitli repos (currently: `recoco`, `codeweaver`, `thread`).

**Design philosophy:** YAML is infrastructure, not configuration. Writing YAML
should be a one-time cost for new trigger types; adding or modifying an agent
means editing a prompt file and a config entry — nothing else.

---

## The Core Problem: YAML Drift

Conventional GitHub Actions automation degrades because:
- Logic lives scattered across workflow YAML files
- "Adding a team member" means writing 50-200 lines of YAML
- Changing a prompt requires finding and editing YAML
- N repos × M agents = N×M workflow files to maintain

**Solution:** Separate three concerns that GitHub Actions conflates:

| Layer | What it is | How often it changes |
|---|---|---|
| Infrastructure | Composite action, reusable trigger workflows | Rarely |
| Configuration | `team.yml` declarative agent manifest | When adding/modifying agents |
| Behavior | `_prompts/*.md` prompt files | Often |

---

## Architecture: Four Layers

### Layer 1 — The Primitive: `claude-agent` Composite Action

A single composite action in `knitli/.github/.github/actions/claude-agent/`
that is the unit of composition for every team member. All agent boilerplate
lives here.

```
Inputs:
  prompt_file        Path to prompt markdown file (relative to repo root)
  profile            mise profile: minimal | reviewer | dev
  max_turns          default: 50
  allowed_tools      comma-separated Claude tool list
  output_label       (optional) label to add on completion
  extra_context      (optional) additional env vars as KEY=VALUE newline-separated

Steps (internal):
  1. actions/checkout
  2. setup-mise-env (called from .github repo)
  3. Inject extra_context into env
  4. anthropics/claude-code-action (prompt from prompt_file)
  5. Add output_label to current issue/PR (if specified)
```

This action is written once. Agents are configured by their inputs, not by
forking this action.

---

### Layer 2 — Trigger-Typed Reusable Workflows

One reusable workflow per trigger *type*. These are also written once. They
accept agent configuration as inputs and call `claude-agent`.

```
knitli/.github/.github/workflows/
  agent-on-pr-merge.yml      # trigger: pull_request closed+merged
  agent-on-label.yml         # trigger: issues/PRs labeled
  agent-on-schedule.yml      # trigger: schedule (cron)
  agent-on-ci-failure.yml    # trigger: workflow_run failed
  agent-on-push.yml          # trigger: push to branch
```

Example `agent-on-label.yml` interface:
```yaml
on:
  workflow_call:
    inputs:
      trigger_label: { required: true, type: string }
      prompt_file:   { required: true, type: string }
      profile:       { required: false, type: string, default: minimal }
      max_turns:     { required: false, type: number, default: 50 }
      allowed_tools: { required: true, type: string }
      output_label:  { required: false, type: string }
    secrets:
      CLAUDE_CODE_OAUTH_TOKEN: { required: true }
      PERSONAL_ACCESS_TOKEN:   { required: false }
```

These workflows form a stable API surface. They rarely need editing.

---

### Layer 3 — The Team Manifest: `team.yml`

**This is what humans edit.** One file in `knitli/.github/team.yml` defines all
agents across all repos.

```yaml
# knitli/.github/team.yml

agents:

  # ── Docs Pipeline ──────────────────────────────────────────────────────────

  docs-assessor:
    description: Analyzes merged PR changes for documentation impact
    trigger: pr_merge
    branches: [main]
    prompt: _prompts/docs-assessor.md
    profile: minimal
    max_turns: 50
    tools: [Bash, Glob, Grep, Read, WebFetch]
    output_label: "docs:write"

  docs-writer:
    description: Updates site docs and docstrings based on assessor output
    trigger: label
    trigger_label: "docs:write"
    prompt: _prompts/docs-writer.md
    profile: dev
    max_turns: 100
    tools: [Bash, Edit, Glob, Grep, Read, Write]
    output_label: "docs:edit"

  docs-editor:
    description: Style, voice, and consistency review; edits branch directly
    trigger: label
    trigger_label: "docs:edit"
    prompt: _prompts/docs-editor.md
    profile: minimal
    max_turns: 50
    tools: [Bash, Edit, Glob, Grep, Read]
    output_label: "docs:factcheck"

  docs-factchecker:
    description: Validates doc accuracy against actual codebase
    trigger: label
    trigger_label: "docs:factcheck"
    prompt: _prompts/docs-factchecker.md
    profile: minimal
    max_turns: 50
    tools: [Bash, Glob, Grep, Read]
    output_label: "docs:ready"

  # ── CI/CD Health ───────────────────────────────────────────────────────────

  ci-failure-responder:
    description: Diagnoses failing workflows and opens a fix PR
    trigger: ci_failure
    prompt: _prompts/ci-failure-responder.md
    profile: dev
    max_turns: 100
    tools: [Bash, Edit, Glob, Grep, Read, Write, WebFetch]

  # ── Upstream Sync ──────────────────────────────────────────────────────────
  # (already exists in recoco; candidate for migration here)

  upstream-sync:
    description: Weekly check for relevant upstream cocoindex changes
    trigger: schedule
    cron: "0 9 * * 1"
    prompt: _prompts/upstream-sync.md
    profile: minimal
    max_turns: 50
    tools: [Bash, WebFetch]
    repos: [recoco]   # only runs for specific repos, not all
```

**Adding a new team member:**
1. Write `_prompts/new-agent.md`
2. Add an entry to `team.yml`
3. Run the generator (see Layer 4)

No YAML editing beyond the 10-line team.yml entry.

---

### Layer 4 — The Generator

A Python script `tools/generate-callers.py` that reads `team.yml` and emits
thin caller workflow YAML for each repo.

```
knitli/.github/
  tools/
    generate-callers.py       # reads team.yml, writes caller workflows
    templates/
      caller-on-label.yml.j2
      caller-on-pr-merge.yml.j2
      caller-on-schedule.yml.j2
      caller-on-ci-failure.yml.j2
```

The generated output for each repo is a directory of thin callers:

```yaml
# generated: recoco/.github/workflows/_agent-docs-assessor.yml
# DO NOT EDIT — generated from knitli/.github/team.yml
# To modify this agent, edit team.yml and re-run tools/generate-callers.py

name: "Agent: Docs Assessor"
on:
  pull_request:
    types: [closed]
    branches: [main]
jobs:
  run:
    if: github.event.pull_request.merged == true
    uses: knitli/.github/.github/workflows/agent-on-pr-merge.yml@main
    secrets: inherit
    with:
      prompt_file: _prompts/docs-assessor.md
      profile: minimal
      max_turns: 50
      allowed_tools: "Bash,Glob,Grep,Read,WebFetch"
      output_label: "docs:write"
```

The generator can be invoked:
- Locally: `python tools/generate-callers.py --repo recoco --output ../recoco/.github/workflows/`
- In CI: A workflow in `.github` that runs on `team.yml` changes and opens a
  PR updating callers in each affected repo

The `# DO NOT EDIT` header makes it clear these files are generated. Humans
edit `team.yml`; the generator handles the YAML.

---

## Prompt Architecture

Prompts live in `_prompts/` and follow a consistent structure. Since the
Claude Code Action supports a `predefined_skills` parameter pointing to skill
files in `.claude/commands/`, shared skills can be installed from the `.github`
repo by the `claude-agent` composite action before invoking Claude.

```
knitli/.github/
  _prompts/
    docs-assessor.md
    docs-writer.md
    docs-editor.md
    docs-factchecker.md
    ci-failure-responder.md
    upstream-sync.md
  .claude/
    commands/
      # Shared skills available to all agents
      assess-pr-impact.md
      update-docs.md
      validate-accuracy.md
```

The composite action copies `.claude/commands/` from `.github` into the
working directory before invoking Claude, making shared skills available
regardless of which repo the agent is running in.

---

## The Docs Pipeline in Detail

```
1. PR merged to main in any repo
   └─ _agent-docs-assessor.yml fires
      └─ Assessor:
           - reads PR diff (gh pr diff)
           - reads current docs (site/src/content/docs/)
           - reads code comments in changed files
           - creates branch: docs/auto-update-{pr-number}
           - writes ASSESSMENT.md to branch:
               mapped table of change → affected doc sections
               filepath:line_number references throughout
           - opens draft PR with label docs:write

2. label docs:write added
   └─ _agent-docs-writer.yml fires
      └─ Writer:
           - reads ASSESSMENT.md
           - updates site/src/content/docs/ per assessment
           - updates code comments in changed files
           - commits to same branch
           - updates PR description
           - adds label docs:edit

3. label docs:edit added
   └─ _agent-docs-editor.yml fires
      └─ Editor:
           - reads updated docs
           - reviews for voice/style consistency with existing docs
           - edits directly on branch (no new PR)
           - adds label docs:factcheck

4. label docs:factcheck added
   └─ _agent-docs-factchecker.yml fires
      └─ Fact-checker:
           - reads final docs
           - validates every claim against actual source code
           - approves PR or leaves review comments
           - adds label docs:ready

5. label docs:ready
   └─ Sits until release; human merges
      (could auto-merge at release tag as a future enhancement)
```

Labels are the pipeline state. Any stage can be re-run by removing and
re-adding its trigger label. The pipeline can be paused by removing a label.

---

## Future Team Members

| Agent | Trigger | Value |
|---|---|---|
| `ci-failure-responder` | `workflow_run` failed | Opens fix PR with diagnosis |
| `dependency-validator` | Dependabot PR opened | Validates breaking changes, auto-merges safe bumps |
| `release-drafter` | Tag pushed | Generates changelog from commits + PR bodies |
| `stale-issue-triager` | Weekly schedule | Re-labels, pings, or closes stale issues |
| `security-scanner` | PR opened | Reviews for OWASP issues beyond CodeQL |

---

## Cross-Repo Permissions Summary

| Operation | Token needed | Available via |
|---|---|---|
| Write to calling repo (e.g. recoco) | `recoco` GITHUB_TOKEN | Automatic in reusable workflow |
| Read upstream public repos | Public, no token | `PERSONAL_ACCESS_TOKEN` for rate limits |
| Create PRs in calling repo | `recoco` GITHUB_TOKEN | Automatic |
| Write to `.github` repo from another repo | `.github` write token | Org-level PAT secret |
| Multi-repo writes (future) | GitHub App token | Custom App (not needed yet) |

Org-level secrets needed (set once in knitli org settings):
- `CLAUDE_CODE_OAUTH_TOKEN` — already set
- `PERSONAL_ACCESS_TOKEN` — already set

No new secrets needed for the initial docs pipeline build.

---

## Build Order

### Phase 1 — Infrastructure foundation
1. Migrate `setup-mise-env` from `recoco` to `knitli/.github` (update reference in recoco)
2. Write `claude-agent` composite action
3. Write four trigger-typed reusable workflows
4. Write generator script + Jinja2 templates

### Phase 2 — Docs pipeline
5. Write prompt files for all four docs pipeline roles
6. Add all four agents to `team.yml`
7. Run generator to produce caller workflows for recoco
8. Test end-to-end with a real PR

### Phase 3 — CI failure responder
9. Write `ci-failure-responder.md` prompt
10. Add to `team.yml`, generate callers

### Phase 4 — Generator automation
11. Add workflow to `.github` that runs generator on `team.yml` changes
    and opens PRs in affected repos with updated callers
    (makes the system fully self-maintaining)

---

## File Map (end state)

```
knitli/.github/
  team.yml                                   # THE manifest — edit here
  _prompts/
    docs-assessor.md
    docs-writer.md
    docs-editor.md
    docs-factchecker.md
    ci-failure-responder.md
    upstream-sync.md                         # migrated from recoco
  .claude/
    commands/                                # shared skills
  .github/
    actions/
      claude-agent/action.yml               # the primitive
      setup-mise-env/action.yml             # migrated from recoco
    workflows/
      cla.yml                               # existing
      agent-on-pr-merge.yml                 # reusable trigger wrappers
      agent-on-label.yml
      agent-on-schedule.yml
      agent-on-ci-failure.yml
      generate-callers.yml                  # Phase 4: auto-runs generator
  tools/
    generate-callers.py
    templates/
      *.yml.j2

knitli/recoco/.github/
  workflows/
    # Generated files (DO NOT EDIT):
    _agent-docs-assessor.yml
    _agent-docs-writer.yml
    _agent-docs-editor.yml
    _agent-docs-factchecker.yml
    _agent-ci-failure-responder.yml
    _agent-upstream-sync.yml                # replaces upstream-sync.yml
    # Hand-maintained:
    ci.yml
    claude.yml
    release.yml
    cla.yml
    gemini-*.yml
    claude-code-review.yml
  actions/
    setup-mise-env -> REMOVED (use knitli/.github version)
  _upstream_agent_prompt.md -> REMOVED (moved to knitli/.github/_prompts/)
```
