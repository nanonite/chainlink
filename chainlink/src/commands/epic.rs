//! Shared helpers for epic/parent-aware issue output.
//!
//! An "epic" here is any issue that has at least one subissue — a container
//! rather than an executable unit of work. `next` uses this to avoid
//! recommending containers, and `ready`/`next` expose the metadata so callers
//! can filter without guessing from the title text.

use std::collections::HashMap;

use anyhow::Result;
use serde::Serialize;

use crate::db::Database;
use crate::models::Issue;

/// Aggregate subissue counts for a parent issue.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SubissueStats {
    pub total: usize,
    pub open: usize,
}

impl SubissueStats {
    /// An issue is a container (epic) when it has any subissues at all,
    /// regardless of their status.
    pub fn is_epic(&self) -> bool {
        self.total > 0
    }

    /// Number of closed subissues.
    pub fn closed(&self) -> usize {
        self.total - self.open
    }
}

/// An issue flattened with epic metadata, used for `--json` output.
#[derive(Debug, Serialize)]
pub struct IssueWithEpic {
    #[serde(flatten)]
    pub issue: Issue,
    pub is_epic: bool,
    pub subissue_count: usize,
    pub open_subissue_count: usize,
}

impl IssueWithEpic {
    pub fn new(issue: Issue, stats: SubissueStats) -> Self {
        Self {
            issue,
            is_epic: stats.is_epic(),
            subissue_count: stats.total,
            open_subissue_count: stats.open,
        }
    }

    /// Convenience constructor that derives the stats from `issue`'s own
    /// subissues. Callers that already have a stats map should use `new`.
    pub fn from_issue(issue: Issue, stats: &HashMap<i64, SubissueStats>) -> Self {
        let subissue = stats_for(stats, issue.id);
        Self::new(issue, subissue)
    }
}

/// Compute `parent_id -> SubissueStats` for every issue in the database.
///
/// This is a single pass over all issues, so callers can enrich a list without
/// issuing a query per issue.
pub fn subissue_stats(db: &Database) -> Result<HashMap<i64, SubissueStats>> {
    let all = db.list_issues(Some("all"), None, None)?;
    let mut stats: HashMap<i64, SubissueStats> = HashMap::new();
    for issue in all {
        if let Some(parent_id) = issue.parent_id {
            let entry = stats.entry(parent_id).or_default();
            entry.total += 1;
            if issue.status == "open" {
                entry.open += 1;
            }
        }
    }
    Ok(stats)
}

/// Look up stats for a single issue, returning zero counts when it has no
/// subissues (or is absent from the map).
pub fn stats_for(stats: &HashMap<i64, SubissueStats>, id: i64) -> SubissueStats {
    stats.get(&id).copied().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_db() -> (Database, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        (db, dir)
    }

    #[test]
    fn test_stats_empty() {
        let (db, _dir) = setup_test_db();
        let stats = subissue_stats(&db).unwrap();
        assert!(stats.is_empty());
    }

    #[test]
    fn test_stats_counts_total_and_open() {
        let (db, _dir) = setup_test_db();
        let parent = db.create_issue("Epic", None, "high").unwrap();
        let child1 = db
            .create_subissue(parent, "Child 1", None, "medium")
            .unwrap();
        db.create_subissue(parent, "Child 2", None, "medium")
            .unwrap();
        db.close_issue(child1).unwrap();

        let stats = subissue_stats(&db).unwrap();
        let s = stats_for(&stats, parent);
        assert_eq!(s.total, 2);
        assert_eq!(s.open, 1);
        assert_eq!(s.closed(), 1);
        assert!(s.is_epic());
    }

    #[test]
    fn test_stats_leaf_is_not_epic() {
        let (db, _dir) = setup_test_db();
        let leaf = db.create_issue("Leaf", None, "medium").unwrap();
        let stats = subissue_stats(&db).unwrap();
        assert!(!stats_for(&stats, leaf).is_epic());
    }

    #[test]
    fn test_stats_includes_closed_subissues() {
        let (db, _dir) = setup_test_db();
        let parent = db.create_issue("Epic", None, "high").unwrap();
        let child = db.create_subissue(parent, "Child", None, "medium").unwrap();
        db.close_issue(child).unwrap();

        let stats = subissue_stats(&db).unwrap();
        let s = stats_for(&stats, parent);
        assert_eq!(s.total, 1);
        assert_eq!(s.open, 0);
        assert!(s.is_epic());
    }

    #[test]
    fn test_issue_with_epic_reports_metadata() {
        let (db, _dir) = setup_test_db();
        let parent = db.create_issue("Epic", None, "high").unwrap();
        db.create_subissue(parent, "Child", None, "medium").unwrap();

        let stats = subissue_stats(&db).unwrap();
        let issue = db.get_issue(parent).unwrap().unwrap();
        let enriched = IssueWithEpic::from_issue(issue, &stats);
        assert!(enriched.is_epic);
        assert_eq!(enriched.subissue_count, 1);
        assert_eq!(enriched.open_subissue_count, 1);
    }
}
