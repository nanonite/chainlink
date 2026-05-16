# Concurrent Issue Timers

## Decision

Chainlink timers are singleton per issue and concurrent across issues.

An issue can have at most one active `time_entries` row where `ended_at IS NULL`.
Different issues may each have an active timer at the same time.

## Invariant

SQLite enforces the invariant with a partial unique index:

```sql
CREATE UNIQUE INDEX idx_time_entries_active_issue
ON time_entries(issue_id)
WHERE ended_at IS NULL;
```

The application treats `timer start <issue_id>` as idempotent for an already
running issue. `timer stop <issue_id>` requires an explicit issue id because,
with concurrent timers, there is no safe global timer to infer.

## JSON Shape

Timer JSON output is always an array of active timer objects. A filtered status
request such as `timer show 42 --json` returns either a one-element array or an
empty array.

## Issue Status

Timer state is independent of issue status. A timer may be stopped while the
issue remains open, and later restarted as a new time entry.
