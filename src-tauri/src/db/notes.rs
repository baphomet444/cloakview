use crate::db::Db;
use crate::error::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub archived: bool,
    pub folder_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewNote {
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub folder_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteFilter {
    pub folder_id: Option<String>,
    #[serde(default)]
    pub archived: bool,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    let tags_raw: String = row.get(3)?;
    Ok(Note {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        tags: serde_json::from_str(&tags_raw).unwrap_or_default(),
        favorite: row.get::<_, i64>(4)? != 0,
        archived: row.get::<_, i64>(5)? != 0,
        folder_id: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub fn create(db: &Db, input: NewNote) -> AppResult<Note> {
    let id = uuid();
    let now = now_ms();
    db.conn.execute(
        "INSERT INTO notes (id, title, content, tags, folder_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        rusqlite::params![id, input.title, input.content,
            serde_json::to_string(&input.tags)?, input.folder_id, now],
    )?;
    get(db, &id)?.ok_or_else(|| crate::error::AppError::Db("insert vanished".into()))
}

pub fn get(db: &Db, id: &str) -> AppResult<Option<Note>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, title, content, tags, favorite, archived, folder_id, created_at, updated_at
         FROM notes WHERE id = ?1")?;
    let mut rows = stmt.query_map([id], row_to_note)?;
    Ok(rows.next().transpose()?)
}

pub fn list(db: &Db, filter: NoteFilter) -> AppResult<Vec<Note>> {
    let mut sql = String::from(
        "SELECT id, title, content, tags, favorite, archived, folder_id, created_at, updated_at
         FROM notes WHERE archived = ?1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(filter.archived as i64)];
    if filter.folder_id.is_some() {
        sql.push_str(" AND folder_id = ?2");
        params.push(Box::new(filter.folder_id.clone().unwrap()));
    }
    sql.push_str(" ORDER BY updated_at DESC");
    let mut stmt = db.conn.prepare(&sql)?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(params_refs.as_slice(), row_to_note)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn update(db: &Db, note: &Note) -> AppResult<Note> {
    let now = now_ms();
    db.conn.execute(
        "UPDATE notes SET title=?2, content=?3, tags=?4, favorite=?5, archived=?6,
         folder_id=?7, updated_at=?8 WHERE id=?1",
        rusqlite::params![note.id, note.title, note.content,
            serde_json::to_string(&note.tags)?, note.favorite as i64,
            note.archived as i64, note.folder_id, now],
    )?;
    get(db, &note.id)?.ok_or_else(|| crate::error::AppError::Db("update vanished".into()))
}

pub fn delete(db: &Db, id: &str) -> AppResult<()> {
    db.conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
    Ok(())
}

fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("{:x}-{:x}", nanos, std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_list_update_delete() {
        let db = Db::open_in_memory().unwrap();
        let n = create(&db, NewNote {
            title: "t".into(), content: "c".into(), tags: vec!["a".into()], folder_id: None,
        }).unwrap();
        assert_eq!(list(&db, NoteFilter::default()).unwrap().len(), 1);

        let mut n2 = n.clone();
        n2.title = "t2".into();
        n2.favorite = true;
        let n2 = update(&db, &n2).unwrap();
        assert!(n2.favorite);
        assert!(n2.updated_at >= n.updated_at);

        delete(&db, &n.id).unwrap();
        assert!(list(&db, NoteFilter::default()).unwrap().is_empty());
    }

    #[test]
    fn archived_filter_separates() {
        let db = Db::open_in_memory().unwrap();
        let mut n = create(&db, NewNote {
            title: "x".into(), content: "".into(), tags: vec![], folder_id: None,
        }).unwrap();
        n.archived = true;
        update(&db, &n).unwrap();
        assert_eq!(list(&db, NoteFilter::default()).unwrap().len(), 0);
        assert_eq!(list(&db, NoteFilter { archived: true, ..Default::default() }).unwrap().len(), 1);
    }
}
