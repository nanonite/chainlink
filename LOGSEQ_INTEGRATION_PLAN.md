# Chainlink × Logseq Integration Plan

Branch: `feature/logseq-integration`
Chainlink issues: #3 (parent), #4 (Track 1), #5 (Track 2)

---

## Background

Logseq is a local-first knowledge base that stores data as Markdown files (classic version) or a local SQLite DB (new DB version). The goal is to give Chainlink's issue/session data a user-friendly UI inside Logseq without duplicating the source of truth — Chainlink's `.chainlink/issues.db` remains authoritative.

**Sync direction**: One-way only. Chainlink → Logseq. Logseq is a read-only view layer.
**Trigger**: Manual (`chainlink logseq export`). No auto-sync or daemon.
**Target**: Classic Logseq (Markdown files on disk), not the new DB version.

---

## What Chainlink Captures (data shape)

| Entity | Key fields |
|--------|-----------|
| Issues | id, title, description, status, priority, parent_id, labels, created_at, closed_at |
| Subissues | parent_id linking to parent issue |
| Comments | content, kind (note / agent breadcrumb), created_at |
| Sessions | started_at, ended_at, active_issue_id, handoff_notes, agent_id |
| Time entries | per-issue duration_seconds |
| Relations | blocker / dependency links between issues |
| Milestones | name, description, open/closed |
| Labels | flexible string tags |

Token usage is excluded — too implementation-specific for a knowledge base view.

---

## Track 1 — `chainlink logseq` Subcommand (Phase 1, implement now)

### New commands

```
chainlink logseq export              # full sync of all issues + sessions
chainlink logseq export --id <n>     # refresh a single issue page
chainlink logseq config --graph-dir <path>   # persist graph path to config
chainlink logseq config --show       # print current config
```

### Config storage

Stored in `.chainlink/config.toml` (new file, created on first `config` call):

```toml
[logseq]
graph_dir = "/Users/you/logseq/graphs/my-graph"
```

### Output file layout

All files land inside the Logseq graph's `pages/` directory. Logseq renders `___` as `/` in the UI, giving a clean namespace hierarchy.

```
<graph_dir>/pages/
  chainlink___dashboard.md         ← open issues summary, active timers, current session
  chainlink___sessions.md          ← full session history with handoff notes
  chainlink___issues___001.md      ← one page per issue (zero-padded id)
  chainlink___issues___002.md
  ...
```

### Issue page format

```markdown
---
title: Chainlink Issue #001
tags: chainlink
---

priority:: medium
status:: open
labels:: bug, backend
parent:: [[chainlink/issues/003]]
created:: 2026-04-29
closed::

- TODO Fix the sync race condition
  - **Description:** Timers occasionally fire out of order under concurrent load
  - **Time logged:** 2h 15m
  - **Blocks:** [[chainlink/issues/004]]
  - **Comments**
    - 2026-05-01 · [agent] Root cause in sync.rs:44 — see commit abc123
    - 2026-05-02 · [agent] Started concurrent timer tests
```

Closed issues use `DONE` instead of `TODO`. Logseq natively renders these as task markers.

### Dashboard page format

```markdown
---
title: Chainlink Dashboard
tags: chainlink
---

- **Last export:** 2026-05-19 16:30 UTC
- **Session:** #2 (started 2026-05-19 16:01, working on #3)

## Open Issues

| Priority | Issue | Labels | Time |
|----------|-------|--------|------|
| high     | [[chainlink/issues/003]] Add logseq integration | logseq | 0m |
| medium   | [[chainlink/issues/002]] test issue | — | 0m |

## Active Timers

(none)

## Recent Sessions

| # | Started | Ended | Issue | Notes |
|---|---------|-------|-------|-------|
| 2 | 2026-05-19 | — | #3 | — |
| 1 | 2026-04-28 | 2026-04-28 | — | — |
```

This is the file the future Logseq plugin will render — no separate data feed needed.

### Sessions page format

```markdown
---
title: Chainlink Sessions
tags: chainlink
---

## Session #2 — 2026-05-19

- **Started:** 2026-05-19 16:01 UTC
- **Working on:** [[chainlink/issues/003]]
- **Agent:** claude-sonnet-4-6
- **Handoff notes:** (in progress)

## Session #1 — 2026-04-28

- **Started:** 2026-04-28 23:24 UTC
- **Ended:** 2026-04-28 23:24 UTC
- **Handoff notes:** —
```

### Implementation notes

- New Rust module: `src/commands/logseq.rs`
- Config module: `src/config.rs` (new — reads/writes `.chainlink/config.toml` via `toml` crate)
- Export logic reads directly from `Database` — no intermediate JSON step
- File writes are idempotent: full overwrite on each export
- Zero-pad issue IDs to 4 digits for stable lexicographic sort in the pages list
- Filenames must replace spaces with `___` per Logseq convention — verify exact escaping rules before implementation

---

## Track 2 — `logseq-plugin-chainlink` (Phase 2, revisit after Track 1)

### Revised approach (updated from original plan)

Rather than building a separate kanban data pipeline, the plugin renders the **dashboard markdown page** that Track 1 already produces. This keeps the plugin simple and stateless — it doesn't need its own data source.

The plugin registers a custom slash command `/chainlink-dashboard` that inserts a renderer macro block. When Logseq renders that block, the plugin:

1. Reads the path to `chainlink___dashboard.md` from plugin settings
2. Parses the open issues table from that file
3. Renders it as a Kanban board (columns: `Open` / `Closed`, grouped by priority)

### Data flow

```
chainlink logseq export
    writes → chainlink___dashboard.md
                  ↑ plugin reads this file
                  ↓ renders Kanban in Logseq UI
```

No HTTP server. No live DB access. The plugin is a pure markdown parser + renderer.

### Plugin structure (for future reference)

```
logseq-plugin/
  package.json          # @logseq/libs dependency
  src/
    index.ts            # plugin entry, registers slash command + renderer
    parser.ts           # parses chainlink___dashboard.md open issues table
    kanban.ts           # React component: Kanban columns + cards
  index.html            # plugin entry point (Logseq requirement)
  README.md
```

### Card layout (sketch)

```
[ Open                          ] [ Closed                      ]
┌─────────────────────────────┐   ┌────────────────────────────┐
│ #003 Add logseq integration │   │ #001 Write NO_HOOKS notes  │
│ HIGH  · logseq              │   │ LOW                        │
└─────────────────────────────┘   └────────────────────────────┘
│ #002 test issue             │
│ MEDIUM                      │
└─────────────────────────────┘
```

Cards link to their `[[chainlink/issues/NNN]]` Logseq page on click.

---

## Sequencing

1. Track 1 fully implemented and tested
2. Verify the dashboard markdown is readable and useful on its own
3. Revisit Track 2 — decide if the plugin is still needed or if the dashboard page suffices

---

## Open questions (for Track 2 revisit)

- Does the dashboard page alone provide enough value in Logseq without the kanban renderer?
- Should the plugin ship inside the chainlink repo or as a separate package?
- File-watching for auto-refresh: feasible with Logseq's plugin API? Or require manual page reload?
