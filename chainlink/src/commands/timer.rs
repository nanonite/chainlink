use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::db::Database;
use crate::models::ActiveTimer;
use crate::utils::format_issue_id;

#[derive(Serialize)]
struct TimerView {
    issue_id: i64,
    issue: String,
    title: String,
    started_at: DateTime<Utc>,
    elapsed_seconds: i64,
}

pub fn start(db: &Database, issue_id: i64) -> Result<()> {
    // Verify issue exists
    let issue = match db.get_issue(issue_id)? {
        Some(i) => i,
        None => bail!("Issue {} not found", format_issue_id(issue_id)),
    };

    if db.get_active_timer_for_issue(issue_id)?.is_some() {
        println!(
            "Timer already running for issue {}",
            format_issue_id(issue_id)
        );
        return Ok(());
    }

    db.start_timer(issue_id)?;
    println!(
        "Started timer for {}: {}",
        format_issue_id(issue_id),
        issue.title
    );
    println!("Run 'chainlink timer stop {}' when done.", issue_id);

    Ok(())
}

pub fn stop(db: &Database, issue_id: i64) -> Result<()> {
    let timer = db.get_active_timer_for_issue(issue_id)?;
    let ActiveTimer {
        issue_id,
        started_at,
    } = match timer {
        Some(a) => a,
        None => bail!("No timer running. Start one with 'chainlink timer start <id>'."),
    };
    let duration = Utc::now().signed_duration_since(started_at);

    db.stop_timer(issue_id)?;

    let issue = db.get_issue(issue_id)?;
    let title = issue
        .map(|i| i.title)
        .unwrap_or_else(|| "(deleted)".to_string());

    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;

    println!("Stopped timer for {}: {}", format_issue_id(issue_id), title);
    println!("Time spent: {}h {}m {}s", hours, minutes, seconds);

    // Show total time for this issue
    let total = db.get_total_time(issue_id)?;
    let total_hours = total / 3600;
    let total_minutes = (total % 3600) / 60;
    println!(
        "Total time on this issue: {}h {}m",
        total_hours, total_minutes
    );

    Ok(())
}

pub fn status(db: &Database, issue_id: Option<i64>, json_output: bool) -> Result<()> {
    if let Some(issue_id) = issue_id {
        let active = db.get_active_timer_for_issue(issue_id)?;
        if json_output {
            let timers = match active {
                Some(timer) => vec![timer_view(db, timer)?],
                None => Vec::new(),
            };
            println!("{}", serde_json::to_string_pretty(&timers)?);
            return Ok(());
        }

        match active {
            Some(timer) => print_timer(db, timer)?,
            None => println!("No timer running for {}.", format_issue_id(issue_id)),
        }
        return Ok(());
    }

    list(db, json_output)
}

pub fn list(db: &Database, json_output: bool) -> Result<()> {
    let timers = db.get_active_timers()?;

    if json_output {
        let views = timers
            .into_iter()
            .map(|timer| timer_view(db, timer))
            .collect::<Result<Vec<_>>>()?;
        println!("{}", serde_json::to_string_pretty(&views)?);
        return Ok(());
    }

    if timers.is_empty() {
        println!("No timers running.");
        return Ok(());
    }

    for timer in timers {
        print_timer(db, timer)?;
    }

    Ok(())
}

fn print_timer(db: &Database, timer: ActiveTimer) -> Result<()> {
    let view = timer_view(db, timer)?;
    let duration = chrono::Duration::seconds(view.elapsed_seconds);
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;

    println!("Timer running: {} {}", view.issue, view.title);
    println!("Elapsed: {}h {}m {}s", hours, minutes, seconds);
    Ok(())
}

fn timer_view(db: &Database, timer: ActiveTimer) -> Result<TimerView> {
    let issue = db.get_issue(timer.issue_id)?;
    let title = issue
        .map(|i| i.title)
        .unwrap_or_else(|| "(deleted)".to_string());
    Ok(TimerView {
        issue_id: timer.issue_id,
        issue: format_issue_id(timer.issue_id),
        title,
        started_at: timer.started_at,
        elapsed_seconds: Utc::now()
            .signed_duration_since(timer.started_at)
            .num_seconds(),
    })
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
    fn test_start_timer() {
        let (db, _dir) = setup_test_db();
        let id = db.create_issue("Test issue", None, "medium").unwrap();

        let result = start(&db, id);
        assert!(result.is_ok());

        let active = db.get_active_timer_for_issue(id).unwrap();
        assert!(active.is_some());
    }

    #[test]
    fn test_start_nonexistent_issue() {
        let (db, _dir) = setup_test_db();

        let result = start(&db, 99999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_start_timer_already_running() {
        let (db, _dir) = setup_test_db();
        let id = db.create_issue("Test issue", None, "medium").unwrap();

        start(&db, id).unwrap();
        let result = start(&db, id);
        assert!(result.is_ok());
        assert_eq!(db.get_active_timers().unwrap().len(), 1);
    }

    #[test]
    fn test_start_timer_different_issue_running() {
        let (db, _dir) = setup_test_db();
        let id1 = db.create_issue("Issue 1", None, "medium").unwrap();
        let id2 = db.create_issue("Issue 2", None, "medium").unwrap();

        start(&db, id1).unwrap();
        let result = start(&db, id2);
        assert!(result.is_ok());
        let active = db.get_active_timers().unwrap();
        assert_eq!(active.len(), 2);
        assert!(active.iter().any(|timer| timer.issue_id == id1));
        assert!(active.iter().any(|timer| timer.issue_id == id2));
    }

    #[test]
    fn test_stop_timer() {
        let (db, _dir) = setup_test_db();
        let id = db.create_issue("Test issue", None, "medium").unwrap();

        start(&db, id).unwrap();
        let result = stop(&db, id);
        assert!(result.is_ok());

        assert!(db.get_active_timer_for_issue(id).unwrap().is_none());
    }

    #[test]
    fn test_stop_one_timer_leaves_other_issue_running() {
        let (db, _dir) = setup_test_db();
        let id1 = db.create_issue("Issue 1", None, "medium").unwrap();
        let id2 = db.create_issue("Issue 2", None, "medium").unwrap();

        start(&db, id1).unwrap();
        start(&db, id2).unwrap();
        stop(&db, id1).unwrap();

        assert!(db.get_active_timer_for_issue(id1).unwrap().is_none());
        assert!(db.get_active_timer_for_issue(id2).unwrap().is_some());
    }

    #[test]
    fn test_stop_no_timer() {
        let (db, _dir) = setup_test_db();

        let result = stop(&db, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No timer running"));
    }

    #[test]
    fn test_status_no_timer() {
        let (db, _dir) = setup_test_db();

        let result = status(&db, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_status_with_timer() {
        let (db, _dir) = setup_test_db();
        let id = db.create_issue("Test issue", None, "medium").unwrap();

        start(&db, id).unwrap();
        let result = status(&db, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_timer_workflow() {
        let (db, _dir) = setup_test_db();
        let id = db.create_issue("Test issue", None, "medium").unwrap();

        start(&db, id).unwrap();
        status(&db, None, false).unwrap();
        stop(&db, id).unwrap();

        let active = db.get_active_timers().unwrap();
        assert!(active.is_empty());
    }

    #[test]
    fn test_start_stop_roundtrip_keeps_singleton_per_issue() {
        let (db, _dir) = setup_test_db();
        let id = db.create_issue("Issue", None, "medium").unwrap();

        start(&db, id).unwrap();
        start(&db, id).unwrap();

        assert_eq!(db.get_active_timers().unwrap().len(), 1);
        assert!(db.get_active_timer_for_issue(id).unwrap().is_some());

        stop(&db, id).unwrap();
        assert!(db.get_active_timer_for_issue(id).unwrap().is_none());
    }
}
