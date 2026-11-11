//! Past interviews, kept for the history window (CloakView Pro). A finished
//! `session::FinishedSession` is stored whole: the entries travel as one JSON
//! column, which keeps the schema to one table and makes search a single LIKE
//! over the title and the entries.

use crate::db::{uuid, Db};
use crate::error::AppResult;
use crate::session::{Entry, FinishedSession};
use serde::{Deserialize, Serialize};

/// What the history list shows per interview. The counts come from the stored
/// entries, so a corrupt row is skipped rather than listed wrong.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub heard: usize,
    pub suggestions: usize,
}

/// One interview, whole, for the detail view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    pub id: String,
    pub title: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub entries: Vec<Entry>,
}

fn counts(entries: &[Entry]) -> (usize, usize) {
    (
        entries.iter().filter(|e| matches!(e, Entry::Heard { .. })).count(),
        entries.iter().filter(|e| matches!(e, Entry::Suggestion { .. })).count(),
    )
}

fn parse_entries(raw: &str) -> Option<Vec<Entry>> {
    serde_json::from_str(raw).ok()
}

/// Persists a finished interview. Called from `finish_session` for CloakView Pro.
pub fn save(db: &Db, session: &FinishedSession) -> AppResult<SessionSummary> {
    let id = uuid();
    let title = crate::session::title(session.started_ms);
    db.conn.execute(
        "INSERT INTO sessions (id, title, started_at, ended_at, entries)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            id,
            title,
            session.started_ms,
            session.ended_ms,
            serde_json::to_string(&session.entries)?
        ],
    )?;
    let (heard, suggestions) = counts(&session.entries);
    Ok(SessionSummary {
        id,
        title,
        started_at: session.started_ms,
        ended_at: session.ended_ms,
        heard,
        suggestions,
    })
}

/// Every interview, newest first. `query` matches the title and the entries
/// text (SQLite LIKE is case-insensitive for ASCII); `None` lists all.
pub fn list(db: &Db, query: Option<&str>) -> AppResult<Vec<SessionSummary>> {
    let mut sql = String::from("SELECT id, title, started_at, ended_at, entries FROM sessions");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(q) = query.map(str::trim).filter(|q| !q.is_empty()) {
        sql.push_str(" WHERE title LIKE ?1 OR entries LIKE ?1");
        params.push(Box::new(format!("%{q}%")));
    }
    sql.push_str(" ORDER BY started_at DESC");
    let mut stmt = db.conn.prepare(&sql)?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (id, title, started_at, ended_at, raw) = row?;
        // A row whose entries no longer parse is skipped, not shown broken.
        let Some(entries) = parse_entries(&raw) else { continue };
        let (heard, suggestions) = counts(&entries);
        out.push(SessionSummary { id, title, started_at, ended_at, heard, suggestions });
    }
    Ok(out)
}

pub fn get(db: &Db, id: &str) -> AppResult<Option<SessionDetail>> {
    let mut stmt = db
        .conn
        .prepare("SELECT id, title, started_at, ended_at, entries FROM sessions WHERE id = ?1")?;
    let mut rows = stmt.query_map([id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;
    let Some(row) = rows.next().transpose()? else { return Ok(None) };
    let (id, title, started_at, ended_at, raw) = row;
    let entries = parse_entries(&raw).unwrap_or_default();
    Ok(Some(SessionDetail { id, title, started_at, ended_at, entries }))
}

pub fn delete(db: &Db, id: &str) -> AppResult<()> {
    db.conn.execute("DELETE FROM sessions WHERE id = ?1", [id])?;
    Ok(())
}

/// For tests and the diag log, never the content.
#[cfg(test)]
pub fn count(db: &Db) -> AppResult<usize> {
    let n: i64 = db.conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;
    Ok(n as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finished(start: i64) -> FinishedSession {
        FinishedSession {
            started_ms: start,
            ended_ms: start + 38 * 60_000,
            entries: vec![
                Entry::Heard { ts: start + 12_000, text: "Tell me about your last project.".into(), mine: false },
                Entry::Heard { ts: start + 16_000, text: "Sure, one moment.".into(), mine: true },
                Entry::Suggestion {
                    ts: start + 20_000,
                    prompt: "Tell me about your last project.".into(),
                    text: "I led a rewrite.".into(),
                },
            ],
            auto_note_id: None,
        }
    }

    #[test]
    fn save_list_get_delete() {
        let db = Db::open_in_memory().unwrap();
        let saved = save(&db, &finished(1_700_000_000_000)).unwrap();
        assert_eq!(saved.heard, 2);
        assert_eq!(saved.suggestions, 1);
        assert!(saved.title.starts_with("Interview "));

        let all = list(&db, None).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, saved.id);

        let detail = get(&db, &saved.id).unwrap().expect("stored session");
        assert_eq!(detail.entries, finished(1_700_000_000_000).entries);

        delete(&db, &saved.id).unwrap();
        assert!(list(&db, None).unwrap().is_empty());
        assert!(get(&db, &saved.id).unwrap().is_none());
    }

    #[test]
    fn newest_first() {
        let db = Db::open_in_memory().unwrap();
        save(&db, &finished(1_700_000_000_000)).unwrap();
        save(&db, &finished(1_700_000_100_000)).unwrap();
        let all = list(&db, None).unwrap();
        assert!(all[0].started_at > all[1].started_at);
    }

    #[test]
    fn search_matches_title_and_entry_text_case_insensitively() {
        let db = Db::open_in_memory().unwrap();
        let saved = save(&db, &finished(1_700_000_000_000)).unwrap();

        assert_eq!(list(&db, Some("rewrite")).unwrap().len(), 1, "entry text");
        assert_eq!(list(&db, Some(&saved.title)).unwrap().len(), 1, "title");
        assert_eq!(list(&db, Some("REWRITE")).unwrap().len(), 1, "case-insensitive");
        assert!(list(&db, Some("kubernetes")).unwrap().is_empty());
        // Blank means "no filter", not "match nothing".
        assert_eq!(list(&db, Some("   ")).unwrap().len(), 1);
    }

    #[test]
    fn a_row_with_unparseable_entries_is_skipped_not_fatal() {
        let db = Db::open_in_memory().unwrap();
        save(&db, &finished(1_700_000_000_000)).unwrap();
        db.conn
            .execute(
                "INSERT INTO sessions (id, title, started_at, ended_at, entries)
                 VALUES ('bad', 'x', 0, 1, '{not json')",
                [],
            )
            .unwrap();
        let all = list(&db, None).unwrap();
        assert_eq!(all.len(), 1, "the corrupt row is skipped");
        // ...and it is still deletable.
        delete(&db, "bad").unwrap();
        assert_eq!(count(&db).unwrap(), 1);
    }

    #[test]
    fn entries_serialize_with_a_kind_tag() {
        let entry = Entry::Heard { ts: 1, text: "hola".into(), mine: false };
        assert_eq!(
            serde_json::to_value(&entry).unwrap(),
            serde_json::json!({"kind": "heard", "ts": 1, "text": "hola", "mine": false})
        );
        let entry = Entry::Suggestion { ts: 2, prompt: "q".into(), text: "a".into() };
        assert_eq!(
            serde_json::to_value(&entry).unwrap(),
            serde_json::json!({"kind": "suggestion", "ts": 2, "prompt": "q", "text": "a"})
        );
    }
}
