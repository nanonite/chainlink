use anyhow::{bail, Result};

use super::epic::{self, IssueWithEpic};
use crate::db::Database;
use crate::utils::{format_issue_id, truncate};

pub fn block(db: &Database, issue_id: i64, blocker_id: i64) -> Result<()> {
    // Check if both issues exist
    db.require_issue(issue_id)?;
    db.require_issue(blocker_id)?;

    if issue_id == blocker_id {
        bail!("An issue cannot block itself");
    }

    if db.add_dependency(issue_id, blocker_id)? {
        println!(
            "Issue {} is now blocked by {}",
            format_issue_id(issue_id),
            format_issue_id(blocker_id)
        );
    } else {
        println!("Dependency already exists");
    }
    Ok(())
}

pub fn unblock(db: &Database, issue_id: i64, blocker_id: i64) -> Result<()> {
    if db.remove_dependency(issue_id, blocker_id)? {
        println!(
            "Removed: {} no longer blocked by {}",
            format_issue_id(issue_id),
            format_issue_id(blocker_id)
        );
    } else {
        println!("No such dependency found");
    }
    Ok(())
}

pub fn list_blocked(db: &Database) -> Result<()> {
    let issues = db.list_blocked_issues()?;

    if issues.is_empty() {
        println!("No blocked issues.");
        return Ok(());
    }

    println!("Blocked issues:");
    for issue in issues {
        let blockers = db.get_blockers(issue.id)?;
        let blocker_strs: Vec<String> = blockers.iter().map(|b| format_issue_id(*b)).collect();
        println!(
            "  {:<5} {} (blocked by: {})",
            format_issue_id(issue.id),
            truncate(&issue.title, 40),
            blocker_strs.join(", ")
        );
    }

    Ok(())
}

/// Ready issues enriched with epic metadata, in database order.
pub fn ready_with_epic(db: &Database) -> Result<Vec<IssueWithEpic>> {
    let issues = db.list_ready_issues()?;
    let stats = epic::subissue_stats(db)?;
    Ok(issues
        .into_iter()
        .map(|issue| IssueWithEpic::from_issue(issue, &stats))
        .collect())
}

pub fn list_ready(db: &Database) -> Result<()> {
    let issues = ready_with_epic(db)?;

    if issues.is_empty() {
        println!("No ready issues.");
        return Ok(());
    }

    println!("Ready issues (no blockers):");
    for entry in issues {
        let epic_note = if entry.is_epic {
            let closed = entry.subissue_count - entry.open_subissue_count;
            format!(
                " [epic: {}/{} subissues done]",
                closed, entry.subissue_count
            )
        } else {
            String::new()
        };
        println!(
            "  {:<5} {:8} {}{}",
            format_issue_id(entry.issue.id),
            entry.issue.priority,
            entry.issue.title,
            epic_note
        );
    }

    Ok(())
}

/// JSON variant of [`list_ready`]. Each entry is the issue plus `is_epic`,
/// `subissue_count`, and `open_subissue_count` so callers can distinguish
/// containers from executable work without parsing the title.
pub fn list_ready_json(db: &Database) -> Result<()> {
    let enriched = ready_with_epic(db)?;
    println!("{}", serde_json::to_string_pretty(&enriched)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::tempdir;

    fn setup_test_db() -> (Database, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        (db, dir)
    }

    // Block function tests
    #[test]
    fn test_block_success() {
        let (db, _dir) = setup_test_db();
        let issue1 = db.create_issue("Issue 1", None, "medium").unwrap();
        let issue2 = db.create_issue("Issue 2", None, "medium").unwrap();

        block(&db, issue1, issue2).unwrap();
        let blockers = db.get_blockers(issue1).unwrap();
        assert!(
            blockers.contains(&issue2),
            "Issue 2 should be a blocker of Issue 1"
        );
    }

    #[test]
    fn test_block_nonexistent_issue() {
        let (db, _dir) = setup_test_db();
        let issue = db.create_issue("Issue", None, "medium").unwrap();

        let result = block(&db, 99999, issue);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_block_nonexistent_blocker() {
        let (db, _dir) = setup_test_db();
        let issue = db.create_issue("Issue", None, "medium").unwrap();

        let result = block(&db, issue, 99999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_block_self() {
        let (db, _dir) = setup_test_db();
        let issue = db.create_issue("Issue", None, "medium").unwrap();

        let result = block(&db, issue, issue);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot block itself"));
    }

    #[test]
    fn test_block_duplicate() {
        let (db, _dir) = setup_test_db();
        let issue1 = db.create_issue("Issue 1", None, "medium").unwrap();
        let issue2 = db.create_issue("Issue 2", None, "medium").unwrap();

        block(&db, issue1, issue2).unwrap();
        block(&db, issue1, issue2).unwrap(); // Should succeed, print "already exists"
        let blockers = db.get_blockers(issue1).unwrap();
        assert_eq!(
            blockers.len(),
            1,
            "Duplicate block should not create second dependency"
        );
        assert!(blockers.contains(&issue2));
    }

    // Unblock function tests
    #[test]
    fn test_unblock_success() {
        let (db, _dir) = setup_test_db();
        let issue1 = db.create_issue("Issue 1", None, "medium").unwrap();
        let issue2 = db.create_issue("Issue 2", None, "medium").unwrap();
        db.add_dependency(issue1, issue2).unwrap();

        unblock(&db, issue1, issue2).unwrap();
        let blockers = db.get_blockers(issue1).unwrap();
        assert!(
            blockers.is_empty(),
            "Blocker should be removed after unblock"
        );
    }

    #[test]
    fn test_unblock_nonexistent_dependency() {
        let (db, _dir) = setup_test_db();
        let issue1 = db.create_issue("Issue 1", None, "medium").unwrap();
        let issue2 = db.create_issue("Issue 2", None, "medium").unwrap();

        // Should succeed gracefully even when no dependency exists
        unblock(&db, issue1, issue2).unwrap();
        let blockers = db.get_blockers(issue1).unwrap();
        assert!(blockers.is_empty(), "No blockers should exist");
    }

    // List blocked tests
    #[test]
    fn test_list_blocked_empty() {
        let (db, _dir) = setup_test_db();

        list_blocked(&db).unwrap();
        let blocked = db.list_blocked_issues().unwrap();
        assert!(blocked.is_empty());
    }

    #[test]
    fn test_list_blocked_with_issues() {
        let (db, _dir) = setup_test_db();
        let issue1 = db.create_issue("Blocked issue", None, "medium").unwrap();
        let issue2 = db.create_issue("Blocker", None, "medium").unwrap();
        db.add_dependency(issue1, issue2).unwrap();

        list_blocked(&db).unwrap();
        let blocked = db.list_blocked_issues().unwrap();
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].id, issue1);
    }

    #[test]
    fn test_list_blocked_multiple_blockers() {
        let (db, _dir) = setup_test_db();
        let blocked = db.create_issue("Blocked", None, "medium").unwrap();
        let blocker1 = db.create_issue("Blocker 1", None, "medium").unwrap();
        let blocker2 = db.create_issue("Blocker 2", None, "medium").unwrap();
        db.add_dependency(blocked, blocker1).unwrap();
        db.add_dependency(blocked, blocker2).unwrap();

        list_blocked(&db).unwrap();
        let blockers = db.get_blockers(blocked).unwrap();
        assert_eq!(blockers.len(), 2);
        assert!(blockers.contains(&blocker1));
        assert!(blockers.contains(&blocker2));
    }

    // List ready tests
    #[test]
    fn test_list_ready_empty() {
        let (db, _dir) = setup_test_db();

        list_ready(&db).unwrap();
        let ready = db.list_ready_issues().unwrap();
        assert!(ready.is_empty());
    }

    #[test]
    fn test_list_ready_with_issues() {
        let (db, _dir) = setup_test_db();
        let id = db.create_issue("Ready issue", None, "medium").unwrap();

        list_ready(&db).unwrap();
        let ready = db.list_ready_issues().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, id);
    }

    #[test]
    fn test_list_ready_excludes_blocked() {
        let (db, _dir) = setup_test_db();
        let blocked = db.create_issue("Blocked", None, "high").unwrap();
        let blocker = db.create_issue("Blocker", None, "medium").unwrap();
        db.add_dependency(blocked, blocker).unwrap();

        let ready = db.list_ready_issues().unwrap();
        assert!(!ready.iter().any(|i| i.id == blocked));
        assert!(ready.iter().any(|i| i.id == blocker));
    }

    #[test]
    fn test_list_ready_excludes_closed() {
        let (db, _dir) = setup_test_db();
        let issue = db.create_issue("Closed issue", None, "medium").unwrap();
        db.close_issue(issue).unwrap();

        let ready = db.list_ready_issues().unwrap();
        assert!(!ready.iter().any(|i| i.id == issue));
    }

    #[test]
    fn test_ready_with_epic_flags_container() {
        let (db, _dir) = setup_test_db();
        let epic = db.create_issue("Epic", None, "high").unwrap();
        let child = db.create_subissue(epic, "Child", None, "medium").unwrap();

        let rows = ready_with_epic(&db).unwrap();
        let epic_row = rows.iter().find(|r| r.issue.id == epic).unwrap();
        let child_row = rows.iter().find(|r| r.issue.id == child).unwrap();

        assert!(epic_row.is_epic);
        assert_eq!(epic_row.subissue_count, 1);
        assert_eq!(epic_row.open_subissue_count, 1);

        assert!(!child_row.is_epic);
        assert_eq!(child_row.subissue_count, 0);
    }

    #[test]
    fn test_list_ready_variants_run_on_empty_db() {
        let (db, _dir) = setup_test_db();
        list_ready(&db).unwrap();
        list_ready_json(&db).unwrap();
        assert!(ready_with_epic(&db).unwrap().is_empty());
    }

    // Integration tests
    #[test]
    fn test_block_unblock_roundtrip() {
        let (db, _dir) = setup_test_db();
        let issue1 = db.create_issue("Issue 1", None, "medium").unwrap();
        let issue2 = db.create_issue("Issue 2", None, "medium").unwrap();

        block(&db, issue1, issue2).unwrap();
        let blocked = db.list_blocked_issues().unwrap();
        assert!(blocked.iter().any(|i| i.id == issue1));

        unblock(&db, issue1, issue2).unwrap();
        let blocked = db.list_blocked_issues().unwrap();
        assert!(!blocked.iter().any(|i| i.id == issue1));
    }

    #[test]
    fn test_closing_blocker_unblocks() {
        let (db, _dir) = setup_test_db();
        let blocked = db.create_issue("Blocked", None, "high").unwrap();
        let blocker = db.create_issue("Blocker", None, "medium").unwrap();
        db.add_dependency(blocked, blocker).unwrap();

        // Blocked issue should not be ready
        let ready = db.list_ready_issues().unwrap();
        assert!(!ready.iter().any(|i| i.id == blocked));

        // Close the blocker
        db.close_issue(blocker).unwrap();

        // Now blocked issue should be ready
        let ready = db.list_ready_issues().unwrap();
        assert!(ready.iter().any(|i| i.id == blocked));
    }

    proptest! {
        #[test]
        fn truncate_respects_limit(s in ".{10,100}", max_chars in 5usize..50) {
            let result = truncate(&s, max_chars);
            assert!(result.chars().count() <= max_chars);
        }

        #[test]
        fn prop_block_creates_dependency(title1 in "[a-zA-Z ]{1,20}", title2 in "[a-zA-Z ]{1,20}") {
            let (db, _dir) = setup_test_db();
            let issue1 = db.create_issue(&title1, None, "medium").unwrap();
            let issue2 = db.create_issue(&title2, None, "medium").unwrap();

            block(&db, issue1, issue2).unwrap();
            let blockers = db.get_blockers(issue1).unwrap();
            prop_assert!(blockers.contains(&issue2));
            let blocked = db.list_blocked_issues().unwrap();
            prop_assert!(blocked.iter().any(|i| i.id == issue1));
        }
    }
}
