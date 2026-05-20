use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::ChainlinkConfig;
use crate::db::Database;
use crate::models::{Issue, Session};

const DASHBOARD_FILE: &str = "chainlink___dashboard.md";
const SESSIONS_FILE: &str = "chainlink___sessions.md";

pub fn configure_graph_dir(chainlink_dir: &Path, graph_dir: PathBuf) -> Result<()> {
    let mut config = ChainlinkConfig::load(chainlink_dir)?;
    config.logseq.graph_dir = Some(graph_dir);
    config.save(chainlink_dir)?;
    println!(
        "Saved Logseq graph directory to {}",
        chainlink_dir.join("config.toml").display()
    );
    Ok(())
}

pub fn show_config(chainlink_dir: &Path) -> Result<()> {
    let config = ChainlinkConfig::load(chainlink_dir)?;
    match config.logseq.graph_dir {
        Some(path) => println!("logseq.graph_dir = {}", path.display()),
        None => println!("logseq.graph_dir is not configured"),
    }
    Ok(())
}

pub fn export(db: &Database, chainlink_dir: &Path, issue_id: Option<i64>) -> Result<()> {
    let config = ChainlinkConfig::load(chainlink_dir)?;
    let graph_dir = config.logseq.graph_dir.ok_or_else(|| {
        anyhow::anyhow!(
            "Logseq graph directory is not configured. Run `chainlink logseq config --graph-dir <path>` first."
        )
    })?;
    let pages_dir = graph_dir.join("pages");
    fs::create_dir_all(&pages_dir)
        .with_context(|| format!("Failed to create {}", pages_dir.display()))?;

    match issue_id {
        Some(id) => {
            let issue = db.require_issue(id)?;
            write_issue_page(db, &pages_dir, &issue)?;
            println!(
                "Exported issue {} to {}",
                issue_ref(id),
                issue_file(&pages_dir, id).display()
            );
        }
        None => {
            let issues = db.list_issues(Some("all"), None, None)?;
            for issue in &issues {
                write_issue_page(db, &pages_dir, issue)?;
            }
            write_dashboard_page(db, &pages_dir, &issues)?;
            write_sessions_page(db, &pages_dir)?;
            println!(
                "Exported {} issues to {}",
                issues.len(),
                pages_dir.display()
            );
        }
    }

    Ok(())
}

fn write_issue_page(db: &Database, pages_dir: &Path, issue: &Issue) -> Result<()> {
    let labels = db.get_labels(issue.id)?;
    let comments = db.get_comments(issue.id)?;
    let time_logged = db.get_total_time(issue.id)?;
    let blocking = db.get_blocking(issue.id)?;
    let blockers = db.get_blockers(issue.id)?;
    let marker = if issue.status == "closed" {
        "DONE"
    } else {
        "TODO"
    };

    let mut md = String::new();
    md.push_str("---\n");
    md.push_str(&format!("title: Chainlink Issue {}\n", issue_ref(issue.id)));
    md.push_str("tags: chainlink\n");
    md.push_str("---\n\n");
    md.push_str(&format!("priority:: {}\n", issue.priority));
    md.push_str(&format!("status:: {}\n", issue.status));
    md.push_str(&format!("labels:: {}\n", labels_or_dash(&labels)));
    if let Some(parent_id) = issue.parent_id {
        md.push_str(&format!("parent:: {}\n", issue_link(parent_id)));
    } else {
        md.push_str("parent::\n");
    }
    md.push_str(&format!(
        "created:: {}\n",
        issue.created_at.format("%Y-%m-%d")
    ));
    md.push_str(&format!(
        "closed:: {}\n\n",
        issue
            .closed_at
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_default()
    ));

    md.push_str(&format!("- {} {}\n", marker, escape_line(&issue.title)));
    if let Some(description) = issue
        .description
        .as_deref()
        .filter(|d| !d.trim().is_empty())
    {
        md.push_str(&format!(
            "  - **Description:** {}\n",
            escape_line(description)
        ));
    }
    md.push_str(&format!(
        "  - **Time logged:** {}\n",
        format_duration(time_logged)
    ));
    if !blockers.is_empty() {
        md.push_str(&format!("  - **Blocked by:** {}\n", links_join(&blockers)));
    }
    if !blocking.is_empty() {
        md.push_str(&format!("  - **Blocks:** {}\n", links_join(&blocking)));
    }
    if !comments.is_empty() {
        md.push_str("  - **Comments**\n");
        for comment in comments {
            md.push_str(&format!(
                "    - {} - [{}] {}\n",
                comment.created_at.format("%Y-%m-%d %H:%M"),
                escape_line(&comment.kind),
                escape_line(&comment.content)
            ));
        }
    }

    fs::write(issue_file(pages_dir, issue.id), md).context("Failed to write issue page")
}

fn write_dashboard_page(db: &Database, pages_dir: &Path, issues: &[Issue]) -> Result<()> {
    let now = Utc::now();
    let current_session = db.get_current_session()?;
    let active_timers = db.get_active_timers()?;
    let recent_sessions = db.list_sessions(Some(5))?;
    let mut open_issues: Vec<&Issue> = issues.iter().filter(|i| i.status == "open").collect();
    let mut closed_issues: Vec<&Issue> = issues.iter().filter(|i| i.status == "closed").collect();
    open_issues.sort_by_key(|issue| (priority_rank(&issue.priority), issue.id));
    closed_issues
        .sort_by_key(|issue| std::cmp::Reverse(issue.closed_at.unwrap_or(issue.updated_at)));
    closed_issues.truncate(12);

    let mut md = String::new();
    md.push_str("---\n");
    md.push_str("title: Chainlink Dashboard\n");
    md.push_str("tags: chainlink\n");
    md.push_str("---\n\n");
    md.push_str(&format!("exported:: {}\n", format_datetime(now)));
    md.push_str(&format!("open:: {}\n", open_issues.len()));
    md.push_str(&format!("closed-recent:: {}\n", closed_issues.len()));
    md.push_str(&format!("active-timers:: {}\n\n", active_timers.len()));

    md.push_str("- **Last export:** ");
    md.push_str(&format_datetime(now));
    md.push('\n');
    md.push_str("- **Current session:** ");
    md.push_str(&session_summary(current_session.as_ref()));
    md.push('\n');
    md.push_str(&format!("- **Open issues:** {}\n", open_issues.len()));
    md.push_str(&format!("- **Active timers:** {}\n\n", active_timers.len()));

    md.push_str("## Open Issues\n\n");
    if open_issues.is_empty() {
        md.push_str("- No open issues\n");
    } else {
        for issue in open_issues {
            write_dashboard_issue_block(&mut md, db, issue)?;
        }
    }

    md.push_str("\n## Recently Closed\n\n");
    if closed_issues.is_empty() {
        md.push_str("- No recently closed issues\n");
    } else {
        for issue in closed_issues {
            write_dashboard_issue_block(&mut md, db, issue)?;
        }
    }

    md.push_str("\n## Active Timers\n\n");
    if active_timers.is_empty() {
        md.push_str("- None\n");
    } else {
        for timer in active_timers {
            md.push_str(&format!(
                "- {} started {}\n",
                issue_link(timer.issue_id),
                format_datetime(timer.started_at)
            ));
        }
    }

    md.push_str("\n## Recent Sessions\n\n");
    if recent_sessions.is_empty() {
        md.push_str("- No sessions\n");
    } else {
        for session in recent_sessions {
            md.push_str(&format!(
                "- Session #{} - {}\n",
                session.id,
                session.started_at.format("%Y-%m-%d")
            ));
            md.push_str(&format!(
                "  started:: {}\n",
                format_datetime(session.started_at)
            ));
            md.push_str(&format!(
                "  ended:: {}\n",
                session
                    .ended_at
                    .map(format_datetime)
                    .unwrap_or_else(|| "-".to_string())
            ));
            if let Some(issue_id) = session.active_issue_id {
                md.push_str(&format!("  issue:: {}\n", issue_link(issue_id)));
            }
            md.push_str(&format!(
                "  notes:: {}\n",
                escape_line(session.handoff_notes.as_deref().unwrap_or("-"))
            ));
        }
    }

    fs::write(pages_dir.join(DASHBOARD_FILE), md).context("Failed to write dashboard page")
}

fn write_dashboard_issue_block(md: &mut String, db: &Database, issue: &Issue) -> Result<()> {
    let labels = db.get_labels(issue.id)?;
    let time = db.get_total_time(issue.id)?;
    let marker = if issue.status == "closed" {
        "DONE"
    } else {
        "TODO"
    };
    md.push_str(&format!(
        "- {} {} {}\n",
        marker,
        issue_link(issue.id),
        escape_line(&issue.title)
    ));
    md.push_str(&format!("  id:: {}\n", issue.id));
    md.push_str(&format!("  priority:: {}\n", issue.priority));
    md.push_str(&format!("  status:: {}\n", issue.status));
    md.push_str(&format!("  labels:: {}\n", labels_or_dash(&labels)));
    md.push_str(&format!("  time:: {}\n", format_duration(time)));
    if let Some(parent_id) = issue.parent_id {
        md.push_str(&format!("  parent:: {}\n", issue_link(parent_id)));
    }
    Ok(())
}

fn write_sessions_page(db: &Database, pages_dir: &Path) -> Result<()> {
    let sessions = db.list_sessions(None)?;
    let mut md = String::new();
    md.push_str("---\n");
    md.push_str("title: Chainlink Sessions\n");
    md.push_str("tags: chainlink\n");
    md.push_str("---\n\n");

    for session in sessions {
        md.push_str(&format!(
            "## Session #{} - {}\n\n",
            session.id,
            session.started_at.format("%Y-%m-%d")
        ));
        md.push_str(&format!(
            "- **Started:** {}\n",
            format_datetime(session.started_at)
        ));
        if let Some(ended_at) = session.ended_at {
            md.push_str(&format!("- **Ended:** {}\n", format_datetime(ended_at)));
        }
        if let Some(issue_id) = session.active_issue_id {
            md.push_str(&format!("- **Working on:** {}\n", issue_link(issue_id)));
        }
        if let Some(agent_id) = session.agent_id.as_deref() {
            md.push_str(&format!("- **Agent:** {}\n", escape_line(agent_id)));
        }
        md.push_str(&format!(
            "- **Handoff notes:** {}\n\n",
            escape_line(session.handoff_notes.as_deref().unwrap_or("-"))
        ));
    }

    fs::write(pages_dir.join(SESSIONS_FILE), md).context("Failed to write sessions page")
}

fn session_summary(session: Option<&Session>) -> String {
    match session {
        Some(session) => {
            let issue = session
                .active_issue_id
                .map(|id| format!(", working on {}", issue_ref(id)))
                .unwrap_or_default();
            format!(
                "#{} (started {}{})",
                session.id,
                session.started_at.format("%Y-%m-%d %H:%M"),
                issue
            )
        }
        None => "-".to_string(),
    }
}

fn issue_file(pages_dir: &Path, id: i64) -> PathBuf {
    pages_dir.join(format!("chainlink___issues___{:04}.md", id))
}

fn issue_link(id: i64) -> String {
    format!("[[chainlink/issues/{:04}]]", id)
}

fn issue_ref(id: i64) -> String {
    format!("#{}", id)
}

fn labels_or_dash(labels: &[String]) -> String {
    if labels.is_empty() {
        "-".to_string()
    } else {
        labels.join(", ")
    }
}

fn links_join(ids: &[i64]) -> String {
    ids.iter()
        .map(|id| issue_link(*id))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M UTC").to_string()
}

fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    match (hours, minutes) {
        (0, 0) => "0m".to_string(),
        (0, m) => format!("{}m", m),
        (h, 0) => format!("{}h", h),
        (h, m) => format!("{}h {}m", h, m),
    }
}

fn priority_rank(priority: &str) -> u8 {
    match priority {
        "critical" => 0,
        "high" => 1,
        "medium" => 2,
        "low" => 3,
        _ => 4,
    }
}

fn escape_line(value: &str) -> String {
    value.replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup() -> (Database, tempfile::TempDir, tempfile::TempDir) {
        let chainlink_dir = tempdir().unwrap();
        let db = Database::open(&chainlink_dir.path().join("issues.db")).unwrap();
        let graph_dir = tempdir().unwrap();
        (db, chainlink_dir, graph_dir)
    }

    #[test]
    fn export_writes_dashboard_sessions_and_issue_pages() {
        let (db, chainlink_dir, graph_dir) = setup();
        configure_graph_dir(chainlink_dir.path(), graph_dir.path().to_path_buf()).unwrap();
        let id = db
            .create_issue("Logseq export", Some("Write markdown pages"), "high")
            .unwrap();
        db.add_label(id, "logseq").unwrap();
        db.add_comment(id, "First pass works", "result").unwrap();

        export(&db, chainlink_dir.path(), None).unwrap();

        let pages = graph_dir.path().join("pages");
        let issue = fs::read_to_string(pages.join("chainlink___issues___0001.md")).unwrap();
        assert!(issue.contains("priority:: high"));
        assert!(issue.contains("- TODO Logseq export"));
        assert!(issue.contains("[result] First pass works"));
        assert!(pages.join(DASHBOARD_FILE).exists());
        assert!(pages.join(SESSIONS_FILE).exists());
    }

    #[test]
    fn export_single_issue_does_not_require_dashboard_write() {
        let (db, chainlink_dir, graph_dir) = setup();
        configure_graph_dir(chainlink_dir.path(), graph_dir.path().to_path_buf()).unwrap();
        let id = db.create_issue("Single issue", None, "medium").unwrap();

        export(&db, chainlink_dir.path(), Some(id)).unwrap();

        let pages = graph_dir.path().join("pages");
        assert!(pages.join("chainlink___issues___0001.md").exists());
        assert!(!pages.join(DASHBOARD_FILE).exists());
    }

    #[test]
    fn export_requires_configured_graph() {
        let (db, chainlink_dir, _graph_dir) = setup();
        let err = export(&db, chainlink_dir.path(), None).unwrap_err();
        assert!(err
            .to_string()
            .contains("graph directory is not configured"));
    }
}
