use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::params;

use super::{parse_datetime, Database};
use crate::models::ActiveTimer;

impl Database {
    pub fn start_timer(&self, issue_id: i64) -> Result<i64> {
        if self.get_active_timer_for_issue(issue_id)?.is_some() {
            if let Some(timer_id) = self.get_active_timer_id(issue_id)? {
                return Ok(timer_id);
            }
        }

        let now = Utc::now().to_rfc3339();
        let rows = self.conn.execute(
            "INSERT OR IGNORE INTO time_entries (issue_id, started_at) VALUES (?1, ?2)",
            params![issue_id, now],
        )?;
        if rows == 1 {
            return Ok(self.conn.last_insert_rowid());
        }

        self.get_active_timer_id(issue_id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to start timer for issue {}", issue_id))
    }

    pub fn stop_timer(&self, issue_id: i64) -> Result<bool> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        let started_at: Option<String> = self
            .conn
            .query_row(
                "SELECT started_at FROM time_entries WHERE issue_id = ?1 AND ended_at IS NULL",
                [issue_id],
                |row| row.get(0),
            )
            .ok();

        if let Some(started) = started_at {
            let start_dt = DateTime::parse_from_rfc3339(&started)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(now);
            let duration = now.signed_duration_since(start_dt).num_seconds();

            let rows = self.conn.execute(
                "UPDATE time_entries SET ended_at = ?1, duration_seconds = ?2 WHERE issue_id = ?3 AND ended_at IS NULL",
                params![now_str, duration, issue_id],
            )?;
            Ok(rows > 0)
        } else {
            Ok(false)
        }
    }

    pub fn get_active_timer(&self) -> Result<Option<(i64, DateTime<Utc>)>> {
        Ok(self
            .get_active_timers()?
            .into_iter()
            .next()
            .map(|timer| (timer.issue_id, timer.started_at)))
    }

    pub fn get_active_timer_for_issue(&self, issue_id: i64) -> Result<Option<ActiveTimer>> {
        let result: Option<(i64, String)> = self
            .conn
            .query_row(
                "SELECT issue_id, started_at FROM time_entries WHERE issue_id = ?1 AND ended_at IS NULL ORDER BY id DESC LIMIT 1",
                [issue_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        Ok(result.map(|(id, started)| ActiveTimer {
            issue_id: id,
            started_at: parse_datetime(started),
        }))
    }

    pub fn get_active_timers(&self) -> Result<Vec<ActiveTimer>> {
        let mut stmt = self.conn.prepare(
            "SELECT issue_id, started_at FROM time_entries WHERE ended_at IS NULL ORDER BY id DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ActiveTimer {
                issue_id: row.get(0)?,
                started_at: parse_datetime(row.get::<_, String>(1)?),
            })
        })?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    fn get_active_timer_id(&self, issue_id: i64) -> Result<Option<i64>> {
        let result = self
            .conn
            .query_row(
                "SELECT id FROM time_entries WHERE issue_id = ?1 AND ended_at IS NULL ORDER BY id DESC LIMIT 1",
                [issue_id],
                |row| row.get(0),
            )
            .ok();
        Ok(result)
    }

    pub fn get_total_time(&self, issue_id: i64) -> Result<i64> {
        let total: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(duration_seconds), 0) FROM time_entries WHERE issue_id = ?1 AND duration_seconds IS NOT NULL",
                [issue_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(total)
    }
}
