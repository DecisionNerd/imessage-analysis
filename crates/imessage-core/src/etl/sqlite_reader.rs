use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{Error, Result};

/// A single row from the messages table with all joins resolved.
#[derive(Debug)]
pub struct MessageRow {
    pub message_id: i64,
    pub is_from_me: i64,
    pub text: Option<String>,
    pub attributed_body: Option<Vec<u8>>,
    pub handle_id: i64,
    pub associated_message_type: i64,
    pub expressive_send_style_id: Option<String>,
    pub thread_originator_guid: Option<String>,
    pub balloon_bundle_id: Option<String>,
    pub is_audio_message: i64,
    pub unix_ts: i64,
    pub chat_id: Option<i64>,
    pub contact_info: Option<String>,
}

/// All participants in a given chat: handle_ids and their contact strings.
#[derive(Debug, Default)]
pub struct ChatMembership {
    pub handle_ids: Vec<i64>,
    pub contact_infos: Vec<String>,
}

pub struct RawData {
    pub messages: Vec<MessageRow>,
    /// Keyed by chat_id.
    pub chat_members: HashMap<i64, ChatMembership>,
    /// High-watermark ROWID seen in this load.
    pub max_message_rowid: i64,
}

pub fn read_all(db_path: &Path) -> Result<RawData> {
    read_since(db_path, 0)
}

pub fn read_since(db_path: &Path, since_rowid: i64) -> Result<RawData> {
    if !db_path.exists() {
        return Err(Error::DbNotFound {
            path: db_path.display().to_string(),
        });
    }

    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| {
            let reason = e.to_string();
            // SQLite returns "unable to open database file" when macOS denies access.
            // Guide the user to Full Disk Access rather than showing a raw SQLite error.
            if reason.contains("unable to open") || reason.contains("permission denied") {
                Error::DbAccessDenied {
                    path: db_path.display().to_string(),
                    reason: "Permission denied — grant Full Disk Access to your terminal:\n  System Settings → Privacy & Security → Full Disk Access".to_string(),
                }
            } else {
                Error::DbAccessDenied {
                    path: db_path.display().to_string(),
                    reason,
                }
            }
        })?;

    let messages = load_messages(&conn, since_rowid)?;
    let chat_members = load_chat_members(&conn)?;
    let max_message_rowid = messages
        .iter()
        .map(|m| m.message_id)
        .max()
        .unwrap_or(since_rowid);

    Ok(RawData {
        messages,
        chat_members,
        max_message_rowid,
    })
}

fn load_messages(conn: &Connection, since_rowid: i64) -> Result<Vec<MessageRow>> {
    let sql = "
        SELECT
            m.ROWID                         AS message_id,
            COALESCE(m.is_from_me, 0)       AS is_from_me,
            m.text,
            m.attributedBody,
            COALESCE(m.handle_id, 0)        AS handle_id,
            COALESCE(m.associated_message_type, 0) AS associated_message_type,
            m.expressive_send_style_id,
            m.thread_originator_guid,
            m.balloon_bundle_id,
            COALESCE(m.is_audio_message, 0) AS is_audio_message,
            CAST((CASE WHEN m.date > 1000000000000 THEN m.date / 1000000000 ELSE m.date END + 978307200) AS INTEGER) AS unix_ts,
            cmj.chat_id,
            h.id                            AS contact_info
        FROM message m
        LEFT JOIN (
            SELECT message_id, MIN(chat_id) AS chat_id
            FROM chat_message_join
            GROUP BY message_id
        ) cmj ON m.ROWID = cmj.message_id
        LEFT JOIN handle h ON m.handle_id = h.ROWID
        WHERE m.ROWID > ?1
        ORDER BY m.ROWID
    ";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([since_rowid], |row| {
        Ok(MessageRow {
            message_id: row.get(0)?,
            is_from_me: row.get(1)?,
            text: row.get(2)?,
            attributed_body: row.get(3)?,
            handle_id: row.get(4)?,
            associated_message_type: row.get(5)?,
            expressive_send_style_id: row.get(6)?,
            thread_originator_guid: row.get(7)?,
            balloon_bundle_id: row.get(8)?,
            is_audio_message: row.get(9)?,
            unix_ts: row.get(10)?,
            chat_id: row.get(11)?,
            contact_info: row.get(12)?,
        })
    })?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(row?);
    }
    Ok(messages)
}

fn load_chat_members(conn: &Connection) -> Result<HashMap<i64, ChatMembership>> {
    let sql = "
        SELECT chj.chat_id, h.ROWID AS handle_id, h.id AS contact_info
        FROM chat_handle_join chj
        JOIN handle h ON chj.handle_id = h.ROWID
        ORDER BY chj.chat_id
    ";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,    // chat_id
            row.get::<_, i64>(1)?,    // handle_id
            row.get::<_, String>(2)?, // contact_info
        ))
    })?;

    let mut map: HashMap<i64, ChatMembership> = HashMap::new();
    for row in rows {
        let (chat_id, handle_id, contact_info) = row?;
        let entry = map.entry(chat_id).or_default();
        entry.handle_ids.push(handle_id);
        entry.contact_infos.push(contact_info);
    }
    Ok(map)
}
