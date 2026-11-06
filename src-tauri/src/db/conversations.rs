use crate::db::{now_ms, uuid, Db};
use crate::error::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMessage {
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
}

fn row_to_conversation(row: &rusqlite::Row) -> rusqlite::Result<Conversation> {
    Ok(Conversation {
        id: row.get(0)?,
        title: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
    })
}

fn row_to_message(row: &rusqlite::Row) -> rusqlite::Result<ConversationMessage> {
    Ok(ConversationMessage {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        model: row.get(4)?,
        created_at: row.get(5)?,
    })
}

pub fn create_conversation(db: &Db, title: String) -> AppResult<Conversation> {
    let id = uuid();
    let now = now_ms();
    db.conn.execute(
        "INSERT INTO conversations (id, title, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?3)",
        rusqlite::params![id, title, now],
    )?;
    let mut stmt = db.conn.prepare(
        "SELECT id, title, created_at, updated_at FROM conversations WHERE id = ?1")?;
    let mut rows = stmt.query_map([id], row_to_conversation)?;
    rows.next()
        .transpose()?
        .ok_or_else(|| crate::error::AppError::Db("insert vanished".into()))
}

pub fn add_message(db: &Db, input: NewMessage) -> AppResult<ConversationMessage> {
    let id = uuid();
    let now = now_ms();
    db.conn.execute(
        "INSERT INTO conversation_messages (id, conversation_id, role, content, model, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, input.conversation_id, input.role, input.content, input.model, now],
    )?;
    let mut stmt = db.conn.prepare(
        "SELECT id, conversation_id, role, content, model, created_at
         FROM conversation_messages WHERE id = ?1")?;
    let mut rows = stmt.query_map([id], row_to_message)?;
    rows.next()
        .transpose()?
        .ok_or_else(|| crate::error::AppError::Db("insert vanished".into()))
}

pub fn list_messages(db: &Db, conversation_id: &str) -> AppResult<Vec<ConversationMessage>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, conversation_id, role, content, model, created_at
         FROM conversation_messages WHERE conversation_id = ?1 ORDER BY created_at ASC")?;
    let rows = stmt.query_map([conversation_id], row_to_message)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn delete_conversation(db: &Db, id: &str) -> AppResult<()> {
    db.conn.execute("DELETE FROM conversations WHERE id = ?1", [id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_with_messages_cascades() {
        let db = Db::open_in_memory().unwrap();
        let c = create_conversation(&db, "demo".into()).unwrap();
        add_message(&db, NewMessage {
            conversation_id: c.id.clone(), role: "user".into(),
            content: "hello".into(), model: Some("gpt-4o-mini".into()),
        }).unwrap();
        assert_eq!(list_messages(&db, &c.id).unwrap().len(), 1);
        delete_conversation(&db, &c.id).unwrap();
        assert_eq!(list_messages(&db, &c.id).unwrap().len(), 0);
    }
}
