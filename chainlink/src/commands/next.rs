use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use super::epic::{self, IssueWithEpic};
use crate::db::Database;
use crate::lock_check::{self, LockStatus};
use crate::models::Issue;
use crate::utils::format_issue_id;

/// A ready, executable candidate for the "next" recommendation.
struct Candidate {
    issue: Issue,
    score: i32,
    /// Progress of the parent epic (closed, total) when this issue is a
    /// subissue of a container. `None` for top-level leaves.
    parent_progress: Option<(usize, usize)>,
}

/// Priority order for sorting (higher = more important)
fn priority_weight(priority: &str) -> i32 {
    match priority {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

/// Build the ranked list of executable work.
///
/// Containers (epics/parents with subissues) are never candidates themselves:
/// their unblocked subissues already appear in the ready set, so skipping the
/// container makes `next` recommend the actual unit of work instead of a
/// tracking issue.
fn select_candidates(db: &Database, chainlink_dir: &Path) -> Result<Vec<Candidate>> {
    let ready = db.list_ready_issues()?;
    let stats = epic::subissue_stats(db)?;
    let mut candidates: Vec<Candidate> = Vec::new();

    for issue in ready {
        // Skip containers — recommend their actionable subissues instead.
        if epic::stats_for(&stats, issue.id).is_epic() {
            continue;
        }

        // Best-effort: skip issues locked by other agents
        if let Ok(LockStatus::LockedByOther { stale: false, .. }) =
            lock_check::check_lock(chainlink_dir, issue.id)
        {
            continue;
        }

        let priority_score = priority_weight(&issue.priority) * 100;

        // Boost subissues of an epic that is already partially complete
        // (finish what you started).
        let parent_progress = issue.parent_id.and_then(|parent_id| {
            let parent = epic::stats_for(&stats, parent_id);
            if parent.is_epic() {
                Some((parent.closed(), parent.total))
            } else {
                None
            }
        });
        let progress_bonus = match parent_progress {
            Some((closed, total)) if closed > 0 && closed < total => 50,
            _ => 0,
        };

        candidates.push(Candidate {
            issue,
            score: priority_score + progress_bonus,
            parent_progress,
        });
    }

    // Sort by score descending; the sort is stable, so ties keep database
    // (id ascending) order.
    candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.score));

    Ok(candidates)
}

fn print_no_candidates(db: &Database) -> Result<()> {
    let ready = db.list_ready_issues()?;
    let stats = epic::subissue_stats(db)?;
    let containers: Vec<&Issue> = ready
        .iter()
        .filter(|issue| epic::stats_for(&stats, issue.id).is_epic())
        .collect();

    if containers.is_empty() {
        println!("No issues ready to work on.");
        println!(
            "Use 'chainlink list' to see all issues or 'chainlink blocked' to see blocked issues."
        );
    } else {
        println!("No unblocked leaf issues to work on.");
        println!("Every ready issue is an epic/parent with no unblocked subissue:");
        for issue in containers {
            let subissue = epic::stats_for(&stats, issue.id);
            println!(
                "  {} [{}] {} ({}/{} subissues complete)",
                format_issue_id(issue.id),
                issue.priority,
                issue.title,
                subissue.closed(),
                subissue.total
            );
        }
        println!(
            "Use 'chainlink issue tree' to inspect a parent, or 'chainlink blocked' to see blockers."
        );
    }

    Ok(())
}

pub fn run(db: &Database, chainlink_dir: &Path) -> Result<()> {
    let candidates = select_candidates(db, chainlink_dir)?;

    if candidates.is_empty() {
        return print_no_candidates(db);
    }

    // Recommend the top candidate.
    let top = &candidates[0];
    println!(
        "Next: {} [{}] {}",
        format_issue_id(top.issue.id),
        top.issue.priority,
        top.issue.title
    );

    if let Some(parent_id) = top.issue.parent_id {
        if let (Some((closed, total)), Ok(Some(parent))) =
            (top.parent_progress, db.get_issue(parent_id))
        {
            println!(
                "       Part of epic {} [{}] {} ({}/{} complete)",
                format_issue_id(parent_id),
                parent.priority,
                parent.title,
                closed,
                total
            );
        }
    }

    if let Some(desc) = &top.issue.description {
        if !desc.is_empty() {
            let preview: String = desc.chars().take(80).collect();
            let suffix = if desc.chars().count() > 80 { "..." } else { "" };
            println!("       {}{}", preview, suffix);
        }
    }

    println!();
    println!("Run: chainlink session work {}", top.issue.id);

    // Show runners-up if any
    if candidates.len() > 1 {
        println!();
        println!("Also ready:");
        for candidate in candidates.iter().skip(1).take(3) {
            let parent_note = match candidate.issue.parent_id {
                Some(parent_id) => format!(" (subissue of {})", format_issue_id(parent_id)),
                None => String::new(),
            };
            println!(
                "  {} [{}] {}{}",
                format_issue_id(candidate.issue.id),
                candidate.issue.priority,
                candidate.issue.title,
                parent_note
            );
        }
    }

    Ok(())
}

/// Payload for `next --json`.
#[derive(Serialize)]
struct NextJson {
    next: Option<IssueWithEpic>,
    /// Parent epic of the recommendation, when it is a subissue.
    parent: Option<IssueWithEpic>,
    /// Other ready, executable issues (unranked beyond their score order).
    also_ready: Vec<IssueWithEpic>,
}

pub fn run_json(db: &Database, chainlink_dir: &Path) -> Result<()> {
    let candidates = select_candidates(db, chainlink_dir)?;
    let stats = epic::subissue_stats(db)?;

    let (next, parent, also_ready) = match candidates.split_first() {
        Some((top, rest)) => {
            let parent = match top.issue.parent_id {
                Some(parent_id) => db
                    .get_issue(parent_id)?
                    .map(|issue| IssueWithEpic::from_issue(issue, &stats)),
                None => None,
            };
            let also_ready = rest
                .iter()
                .map(|candidate| IssueWithEpic::from_issue(candidate.issue.clone(), &stats))
                .collect();
            (
                Some(IssueWithEpic::from_issue(top.issue.clone(), &stats)),
                parent,
                also_ready,
            )
        }
        None => (None, None, Vec::new()),
    };

    let payload = NextJson {
        next,
        parent,
        also_ready,
    };
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::tempdir;

    fn setup_test_db() -> (Database, std::path::PathBuf, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        let chainlink_dir = dir.path().join(".chainlink");
        std::fs::create_dir_all(&chainlink_dir).unwrap();
        (db, chainlink_dir, dir)
    }

    #[test]
    fn test_priority_weight_critical() {
        assert_eq!(priority_weight("critical"), 4);
    }

    #[test]
    fn test_priority_weight_high() {
        assert_eq!(priority_weight("high"), 3);
    }

    #[test]
    fn test_priority_weight_medium() {
        assert_eq!(priority_weight("medium"), 2);
    }

    #[test]
    fn test_priority_weight_low() {
        assert_eq!(priority_weight("low"), 1);
    }

    #[test]
    fn test_priority_weight_unknown() {
        assert_eq!(priority_weight("unknown"), 0);
    }

    #[test]
    fn test_run_no_issues() {
        let (db, cl, _dir) = setup_test_db();
        run(&db, &cl).unwrap();
        let ready = db.list_ready_issues().unwrap();
        assert!(ready.is_empty());
    }

    #[test]
    fn test_run_with_issues() {
        let (db, cl, _dir) = setup_test_db();
        let id = db.create_issue("Issue 1", None, "high").unwrap();

        run(&db, &cl).unwrap();
        let ready = db.list_ready_issues().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, id);
    }

    #[test]
    fn test_run_prioritizes_higher() {
        let (db, cl, _dir) = setup_test_db();
        db.create_issue("Low priority", None, "low").unwrap();
        let critical_id = db
            .create_issue("Critical priority", None, "critical")
            .unwrap();
        db.create_issue("Medium priority", None, "medium").unwrap();

        let candidates = select_candidates(&db, &cl).unwrap();
        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0].issue.id, critical_id);
        assert_eq!(priority_weight("critical"), 4);
        assert!(priority_weight("critical") > priority_weight("low"));
        assert!(priority_weight("critical") > priority_weight("medium"));
    }

    #[test]
    fn test_run_skips_epic_and_recommends_subissue() {
        let (db, cl, _dir) = setup_test_db();
        let epic = db.create_issue("EPIC: big thing", None, "high").unwrap();
        let child = db
            .create_subissue(epic, "Actual work", None, "medium")
            .unwrap();

        let candidates = select_candidates(&db, &cl).unwrap();
        assert_eq!(candidates.len(), 1, "epic must not be a candidate");
        assert_eq!(candidates[0].issue.id, child);
    }

    #[test]
    fn test_run_prefers_leaf_over_higher_priority_epic() {
        let (db, cl, _dir) = setup_test_db();
        let epic = db
            .create_issue("EPIC: critical container", None, "critical")
            .unwrap();
        db.create_subissue(epic, "Blocked work", None, "low")
            .unwrap();
        let blocker = db.create_issue("Blocker", None, "low").unwrap();
        let blocked_child = db.get_subissues(epic).unwrap()[0].id;
        db.add_dependency(blocked_child, blocker).unwrap();
        let standalone = db.create_issue("Standalone leaf", None, "medium").unwrap();

        let candidates = select_candidates(&db, &cl).unwrap();
        let ids: Vec<i64> = candidates.iter().map(|c| c.issue.id).collect();
        assert!(!ids.contains(&epic), "epic must never be recommended");
        assert!(ids.contains(&standalone));
        assert_eq!(candidates[0].issue.id, standalone);
    }

    #[test]
    fn test_run_no_candidates_when_all_epics() {
        let (db, cl, _dir) = setup_test_db();
        let epic = db.create_issue("EPIC: all blocked", None, "high").unwrap();
        let child = db.create_subissue(epic, "Child", None, "medium").unwrap();
        let blocker = db.create_issue("Blocker", None, "medium").unwrap();
        db.add_dependency(child, blocker).unwrap();
        // Blocker is a leaf, so it is a valid candidate; the epic and its
        // blocked child are not.
        let candidates = select_candidates(&db, &cl).unwrap();
        let ids: Vec<i64> = candidates.iter().map(|c| c.issue.id).collect();
        assert_eq!(ids, vec![blocker]);
    }

    #[test]
    fn test_parent_progress_bonus_prefers_in_progress_epic() {
        let (db, cl, _dir) = setup_test_db();
        let epic = db.create_issue("Epic", None, "medium").unwrap();
        let child_done = db.create_subissue(epic, "Done", None, "medium").unwrap();
        let child_ready = db.create_subissue(epic, "Ready", None, "medium").unwrap();
        db.close_issue(child_done).unwrap();
        let standalone = db.create_issue("Standalone", None, "medium").unwrap();

        let candidates = select_candidates(&db, &cl).unwrap();
        // Both medium; the subissue of a partially-done epic gets the bonus.
        assert_eq!(candidates[0].issue.id, child_ready);
        assert!(candidates.iter().any(|c| c.issue.id == standalone));
    }

    #[test]
    fn test_run_skips_blocked() {
        let (db, cl, _dir) = setup_test_db();
        let blocker = db.create_issue("Blocker", None, "high").unwrap();
        let blocked = db.create_issue("Blocked", None, "critical").unwrap();
        db.add_dependency(blocked, blocker).unwrap();

        run(&db, &cl).unwrap();
        let ready = db.list_ready_issues().unwrap();
        assert!(
            !ready.iter().any(|i| i.id == blocked),
            "Blocked issue should not be in ready list"
        );
        assert!(
            ready.iter().any(|i| i.id == blocker),
            "Blocker should be in ready list"
        );
    }

    #[test]
    fn test_run_all_issues_closed() {
        let (db, cl, _dir) = setup_test_db();
        let id = db.create_issue("Done", None, "medium").unwrap();
        db.close_issue(id).unwrap();

        run(&db, &cl).unwrap();
        let ready = db.list_ready_issues().unwrap();
        assert!(
            ready.is_empty(),
            "Closed issues should not appear in ready list"
        );
    }

    #[test]
    fn test_run_json_no_candidates_is_valid_json() {
        let (db, cl, _dir) = setup_test_db();
        run_json(&db, &cl).unwrap();
    }

    #[test]
    fn test_run_json_with_subissue() {
        let (db, cl, _dir) = setup_test_db();
        let epic = db.create_issue("Epic", None, "high").unwrap();
        db.create_subissue(epic, "Child", None, "medium").unwrap();

        run_json(&db, &cl).unwrap();
        let candidates = select_candidates(&db, &cl).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].issue.parent_id, Some(epic));
    }

    proptest! {
        #[test]
        fn prop_priority_weight_valid(priority in "low|medium|high|critical") {
            let weight = priority_weight(&priority);
            prop_assert!((1..=4).contains(&weight));
        }

        #[test]
        fn prop_run_never_panics(count in 0usize..5) {
            let (db, cl, _dir) = setup_test_db();
            for i in 0..count {
                db.create_issue(&format!("Issue {}", i), None, "medium").unwrap();
            }
            let result = run(&db, &cl);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn prop_next_never_recommends_container(count in 1usize..5) {
            let (db, cl, _dir) = setup_test_db();
            for i in 0..count {
                let parent = db
                    .create_issue(&format!("Epic {}", i), None, "high")
                    .unwrap();
                db.create_subissue(parent, "Child", None, "medium").unwrap();
            }
            let candidates = select_candidates(&db, &cl).unwrap();
            for candidate in candidates {
                prop_assert!(
                    candidate.issue.parent_id.is_some(),
                    "container {} was recommended",
                    candidate.issue.id
                );
            }
        }
    }
}
