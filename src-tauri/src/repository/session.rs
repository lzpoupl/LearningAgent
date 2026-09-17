//! `session` / `message` 的数据访问与状态收敛。

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};

use crate::interface::error::ApiError;
use crate::interface::session::{MessageInfo, MessageRole, MessageStatus, SessionInfo, ToolCall};

/// 消息表的完整列顺序，与 `message_from_row` 保持一致。
const MESSAGE_COLUMNS: &str = "id, session_id, turn_id, role, content, tool_calls, tool_call_id, \
     tool_name, status, prompt_tokens, completion_tokens, created_at";

/// 会话查询：附带非 system 消息数与最近消息时间。
const SESSION_SELECT: &str = "SELECT s.id, s.agent_id, s.title, s.last_provider, s.last_model, \
     s.created_at, s.updated_at, \
     (SELECT COUNT(*) FROM message m WHERE m.session_id = s.id AND m.role <> 'system'), \
     (SELECT MAX(m.created_at) FROM message m WHERE m.session_id = s.id AND m.role <> 'system') \
     FROM session s";

/// 插入消息时的内部输入结构，避免为每个字段单开参数。
#[derive(Clone, Debug)]
pub struct NewMessage {
    pub turn_id: Option<String>,
    pub role: MessageRole,
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub tool_call_id: Option<String>,
    pub tool_name: Option<String>,
    pub status: MessageStatus,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
}

impl NewMessage {
    /// 默认插入一条 `complete` 状态的空消息。
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            turn_id: None,
            role,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: None,
            tool_name: None,
            status: MessageStatus::Complete,
            prompt_tokens: None,
            completion_tokens: None,
        }
    }

    pub fn with_turn_id(mut self, turn_id: impl Into<String>) -> Self {
        self.turn_id = Some(turn_id.into());
        self
    }

    pub fn with_status(mut self, status: MessageStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = tool_calls;
        self
    }

    pub fn with_tool_result(
        mut self,
        tool_call_id: impl Into<String>,
        tool_name: impl Into<String>,
    ) -> Self {
        self.tool_call_id = Some(tool_call_id.into());
        self.tool_name = Some(tool_name.into());
        self
    }

    pub fn with_usage(
        mut self,
        prompt_tokens: Option<i64>,
        completion_tokens: Option<i64>,
    ) -> Self {
        self.prompt_tokens = prompt_tokens;
        self.completion_tokens = completion_tokens;
        self
    }
}

/// `finish_message` 的增量补丁；`None` 字段保持原值。
#[derive(Clone, Debug, Default)]
pub struct MessagePatch {
    pub content: Option<String>,
    pub status: Option<MessageStatus>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
}

#[allow(dead_code)]
impl MessagePatch {
    pub fn complete(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            status: Some(MessageStatus::Complete),
            ..Self::default()
        }
    }

    pub fn status(status: MessageStatus) -> Self {
        Self {
            status: Some(status),
            ..Self::default()
        }
    }

    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls);
        self
    }

    pub fn with_usage(
        mut self,
        prompt_tokens: Option<i64>,
        completion_tokens: Option<i64>,
    ) -> Self {
        self.prompt_tokens = prompt_tokens;
        self.completion_tokens = completion_tokens;
        self
    }
}

// ---------- 会话 ----------

/// 列出会话，按更新时间倒序；`agent_id` 为 `None` 时返回全部。
pub fn list_sessions(
    conn: &Connection,
    agent_id: Option<i64>,
) -> Result<Vec<SessionInfo>, ApiError> {
    let mut stmt = conn.prepare(&format!(
        "{SESSION_SELECT} WHERE (?1 IS NULL OR s.agent_id = ?1) ORDER BY s.updated_at DESC, s.id DESC"
    ))?;
    let rows = stmt.query_map([agent_id], session_from_row)?;
    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row?);
    }
    Ok(sessions)
}

/// 按 id 读取单个会话。
pub fn get_session(conn: &Connection, session_id: i64) -> Result<Option<SessionInfo>, ApiError> {
    conn.query_row(
        &format!("{SESSION_SELECT} WHERE s.id = ?1"),
        [session_id],
        session_from_row,
    )
    .optional()
    .map_err(ApiError::from)
}

/// 确认会话存在，否则返回 `not_found`。
pub fn ensure_session(conn: &Connection, session_id: i64) -> Result<SessionInfo, ApiError> {
    get_session(conn, session_id)?
        .ok_or_else(|| ApiError::not_found(format!("会话不存在: {session_id}")))
}

/// 新建会话；`title` 为占位标题，首条用户消息到达时再自动截取。
pub fn insert_session(
    conn: &Connection,
    agent_id: i64,
    title: &str,
) -> Result<SessionInfo, ApiError> {
    crate::repository::agent::ensure_agent(conn, agent_id)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO session (agent_id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        rusqlite::params![agent_id, title, now],
    )?;
    let session_id = conn.last_insert_rowid();
    get_session(conn, session_id)?.ok_or_else(|| ApiError::internal("新建会话后无法读取"))
}

/// 重命名会话。
pub fn update_title(
    conn: &Connection,
    session_id: i64,
    title: &str,
) -> Result<SessionInfo, ApiError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(ApiError::invalid_input("会话名称不能为空"));
    }
    let now = Utc::now().to_rfc3339();
    let affected = conn.execute(
        "UPDATE session SET title = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![title, now, session_id],
    )?;
    if affected == 0 {
        return Err(ApiError::not_found(format!("会话不存在: {session_id}")));
    }
    ensure_session(conn, session_id)
}

/// 刷新会话更新时间；会话标题与模型写入已顺带刷新，保留给需要单独 touch 的调用方。
#[allow(dead_code)]
pub fn touch_session(conn: &Connection, session_id: i64) -> Result<(), ApiError> {
    let now = Utc::now().to_rfc3339();
    let affected = conn.execute(
        "UPDATE session SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, session_id],
    )?;
    if affected == 0 {
        return Err(ApiError::not_found(format!("会话不存在: {session_id}")));
    }
    Ok(())
}

/// 记录本会话最近一轮实际调用的 provider / model。
pub fn set_session_model(
    conn: &Connection,
    session_id: i64,
    provider: &str,
    model: &str,
) -> Result<(), ApiError> {
    let now = Utc::now().to_rfc3339();
    let affected = conn.execute(
        "UPDATE session SET last_provider = ?1, last_model = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![provider, model, now, session_id],
    )?;
    if affected == 0 {
        return Err(ApiError::not_found(format!("会话不存在: {session_id}")));
    }
    Ok(())
}

/// 删除会话；消息随外键级联删除。
pub fn delete_session(conn: &Connection, session_id: i64) -> Result<(), ApiError> {
    let affected = conn.execute("DELETE FROM session WHERE id = ?1", [session_id])?;
    if affected == 0 {
        return Err(ApiError::not_found(format!("会话不存在: {session_id}")));
    }
    Ok(())
}

// ---------- 消息 ----------

/// 读取会话的全部消息（按 id 升序）；读取前把残留的 `streaming` 收敛为 `interrupted`。
pub fn list_messages(conn: &Connection, session_id: i64) -> Result<Vec<MessageInfo>, ApiError> {
    interrupt_streaming(conn, session_id)?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {MESSAGE_COLUMNS} FROM message WHERE session_id = ?1 ORDER BY id"
    ))?;
    let rows = stmt.query_map([session_id], raw_message_from_row)?;
    let mut messages = Vec::new();
    for row in rows {
        messages.push(message_from_raw(row?)?);
    }
    Ok(messages)
}

/// 追加一条消息。
pub fn append_message(
    conn: &Connection,
    session_id: i64,
    new: &NewMessage,
) -> Result<MessageInfo, ApiError> {
    ensure_session(conn, session_id)?;
    let now = Utc::now().to_rfc3339();
    let tool_calls = serialize_tool_calls(&new.tool_calls)?;
    conn.execute(
        "INSERT INTO message (session_id, turn_id, role, content, tool_calls, tool_call_id, \
         tool_name, status, prompt_tokens, completion_tokens, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
        rusqlite::params![
            session_id,
            new.turn_id,
            new.role.as_str(),
            new.content,
            tool_calls,
            new.tool_call_id,
            new.tool_name,
            new.status.as_str(),
            new.prompt_tokens,
            new.completion_tokens,
            now
        ],
    )?;
    get_message(conn, conn.last_insert_rowid())
}

/// 以 `streaming` 状态开始一条助手消息。
pub fn begin_assistant_message(
    conn: &Connection,
    session_id: i64,
    turn_id: &str,
) -> Result<MessageInfo, ApiError> {
    append_message(
        conn,
        session_id,
        &NewMessage::new(MessageRole::Assistant, "")
            .with_turn_id(turn_id)
            .with_status(MessageStatus::Streaming),
    )
}

/// 追加流式内容增量，返回更新后的内容长度。
pub fn append_content(conn: &Connection, message_id: i64, delta: &str) -> Result<i64, ApiError> {
    let now = Utc::now().to_rfc3339();
    let affected = conn.execute(
        "UPDATE message SET content = content || ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![delta, now, message_id],
    )?;
    if affected == 0 {
        return Err(ApiError::not_found(format!("消息不存在: {message_id}")));
    }
    let length: i64 = conn.query_row(
        "SELECT length(content) FROM message WHERE id = ?1",
        [message_id],
        |row| row.get(0),
    )?;
    Ok(length)
}

/// 定稿一条消息：写入完整内容、状态、工具调用与用量。
pub fn finish_message(
    conn: &Connection,
    message_id: i64,
    patch: &MessagePatch,
) -> Result<MessageInfo, ApiError> {
    let existing = get_message(conn, message_id)?;

    let content = patch.content.clone().unwrap_or(existing.content);
    let status = patch.status.unwrap_or(existing.status);
    let tool_calls = patch.tool_calls.clone().unwrap_or(existing.tool_calls);
    let prompt_tokens = patch.prompt_tokens.or(existing.prompt_tokens);
    let completion_tokens = patch.completion_tokens.or(existing.completion_tokens);
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE message SET content = ?1, status = ?2, tool_calls = ?3, prompt_tokens = ?4, \
         completion_tokens = ?5, updated_at = ?6 WHERE id = ?7",
        rusqlite::params![
            content,
            status.as_str(),
            serialize_tool_calls(&tool_calls)?,
            prompt_tokens,
            completion_tokens,
            now,
            message_id
        ],
    )?;

    get_message(conn, message_id)
}

/// 把该会话残留的 `streaming` 消息收敛为 `interrupted`，返回受影响行数。
pub fn interrupt_streaming(conn: &Connection, session_id: i64) -> Result<u32, ApiError> {
    let now = Utc::now().to_rfc3339();
    let affected = conn.execute(
        "UPDATE message SET status = 'interrupted', updated_at = ?1 \
         WHERE session_id = ?2 AND status = 'streaming'",
        rusqlite::params![now, session_id],
    )?;
    Ok(affected as u32)
}

/// 按 id 读取单条消息。
pub fn get_message(conn: &Connection, message_id: i64) -> Result<MessageInfo, ApiError> {
    let raw = conn
        .query_row(
            &format!("SELECT {MESSAGE_COLUMNS} FROM message WHERE id = ?1"),
            [message_id],
            raw_message_from_row,
        )
        .optional()?
        .ok_or_else(|| ApiError::not_found(format!("消息不存在: {message_id}")))?;
    message_from_raw(raw)
}

fn session_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionInfo> {
    Ok(SessionInfo {
        id: row.get(0)?,
        agent_id: row.get(1)?,
        title: row.get(2)?,
        last_provider: row.get(3)?,
        last_model: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        message_count: row.get::<_, i64>(7)? as u32,
        last_message_at: row.get(8)?,
    })
}

/// 消息原始行：角色、状态与工具调用先保持字符串，稍后解析。
struct RawMessage {
    id: i64,
    session_id: i64,
    turn_id: Option<String>,
    role: String,
    content: String,
    tool_calls: Option<String>,
    tool_call_id: Option<String>,
    tool_name: Option<String>,
    status: String,
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    created_at: String,
}

fn raw_message_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawMessage> {
    Ok(RawMessage {
        id: row.get(0)?,
        session_id: row.get(1)?,
        turn_id: row.get(2)?,
        role: row.get(3)?,
        content: row.get(4)?,
        tool_calls: row.get(5)?,
        tool_call_id: row.get(6)?,
        tool_name: row.get(7)?,
        status: row.get(8)?,
        prompt_tokens: row.get(9)?,
        completion_tokens: row.get(10)?,
        created_at: row.get(11)?,
    })
}

fn message_from_raw(raw: RawMessage) -> Result<MessageInfo, ApiError> {
    let role = MessageRole::parse(&raw.role)
        .ok_or_else(|| ApiError::internal(format!("未知的消息角色: {}", raw.role)))?;
    let status = MessageStatus::parse(&raw.status)
        .ok_or_else(|| ApiError::internal(format!("未知的消息状态: {}", raw.status)))?;
    let tool_calls = match raw.tool_calls.as_deref() {
        Some(text) if !text.is_empty() => serde_json::from_str(text)
            .map_err(|e| ApiError::internal(format!("工具调用无法解析: {e}")))?,
        _ => Vec::new(),
    };

    Ok(MessageInfo {
        id: raw.id,
        session_id: raw.session_id,
        turn_id: raw.turn_id,
        role,
        content: raw.content,
        tool_calls,
        tool_call_id: raw.tool_call_id,
        tool_name: raw.tool_name,
        status,
        prompt_tokens: raw.prompt_tokens,
        completion_tokens: raw.completion_tokens,
        created_at: raw.created_at,
    })
}

fn serialize_tool_calls(tool_calls: &[ToolCall]) -> Result<Option<String>, ApiError> {
    if tool_calls.is_empty() {
        return Ok(None);
    }
    serde_json::to_string(tool_calls)
        .map(Some)
        .map_err(|e| ApiError::internal(format!("工具调用无法序列化: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::db;

    fn setup() -> Connection {
        db::open_in_memory().unwrap()
    }

    fn tool_call() -> ToolCall {
        ToolCall {
            id: "call-1".to_string(),
            tool_id: "anki.list_decks".to_string(),
            arguments: serde_json::json!({ "parent": null }),
        }
    }

    #[test]
    fn session_crud_round_trip() {
        let conn = setup();

        let session = insert_session(&conn, 1, "新会话").unwrap();
        assert_eq!(session.agent_id, 1);
        assert_eq!(session.title, "新会话");
        assert_eq!(session.message_count, 0);
        assert_eq!(session.last_message_at, None);
        assert_eq!(session.last_provider, None);
        assert_eq!(session.last_model, None);

        let renamed = update_title(&conn, session.id, "概率论").unwrap();
        assert_eq!(renamed.title, "概率论");
        assert_eq!(
            update_title(&conn, session.id, "  ").unwrap_err().code,
            "invalid_input"
        );

        set_session_model(&conn, session.id, "edgee", "claude").unwrap();
        let updated = get_session(&conn, session.id).unwrap().unwrap();
        assert_eq!(updated.last_provider.as_deref(), Some("edgee"));
        assert_eq!(updated.last_model.as_deref(), Some("claude"));

        let all = list_sessions(&conn, None).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, session.id);
        assert!(list_sessions(&conn, Some(2)).unwrap().is_empty());

        update_title(&conn, 999, "x").unwrap_err();
        set_session_model(&conn, 999, "p", "m").unwrap_err();
        touch_session(&conn, 999).unwrap_err();

        delete_session(&conn, session.id).unwrap();
        assert!(get_session(&conn, session.id).unwrap().is_none());
        assert_eq!(
            delete_session(&conn, session.id).unwrap_err().code,
            "not_found"
        );
    }

    #[test]
    fn messages_append_stream_and_finish() {
        let conn = setup();
        let session = insert_session(&conn, 1, "新会话").unwrap();

        let system = append_message(
            &conn,
            session.id,
            &NewMessage::new(MessageRole::System, "你是助手"),
        )
        .unwrap();
        assert_eq!(system.role, MessageRole::System);
        assert_eq!(system.status, MessageStatus::Complete);

        let user = append_message(
            &conn,
            session.id,
            &NewMessage::new(MessageRole::User, "什么是导数？").with_turn_id("t-1"),
        )
        .unwrap();
        assert_eq!(user.turn_id.as_deref(), Some("t-1"));

        let assistant = begin_assistant_message(&conn, session.id, "t-1").unwrap();
        assert_eq!(assistant.status, MessageStatus::Streaming);
        // 返回值为字符数（与 SQLite length() 一致），供调用方判断落库节奏。
        assert_eq!(append_content(&conn, assistant.id, "导数是").unwrap(), 3);
        assert_eq!(append_content(&conn, assistant.id, "变化率").unwrap(), 6);

        let finished = finish_message(
            &conn,
            assistant.id,
            &MessagePatch::complete("导数是变化率").with_usage(Some(10), Some(4)),
        )
        .unwrap();
        assert_eq!(finished.content, "导数是变化率");
        assert_eq!(finished.status, MessageStatus::Complete);
        assert_eq!(finished.prompt_tokens, Some(10));
        assert_eq!(finished.completion_tokens, Some(4));

        // 会话聚合：system 快照不计入消息数。
        let session = get_session(&conn, session.id).unwrap().unwrap();
        assert_eq!(session.message_count, 2);
        assert!(session.last_message_at.is_some());
    }

    #[test]
    fn message_tool_calls_round_trip() {
        let conn = setup();
        let session = insert_session(&conn, 1, "新会话").unwrap();

        let assistant = append_message(
            &conn,
            session.id,
            &NewMessage::new(MessageRole::Assistant, "")
                .with_turn_id("t-1")
                .with_tool_calls(vec![tool_call()]),
        )
        .unwrap();
        assert_eq!(assistant.tool_calls.len(), 1);
        assert_eq!(assistant.tool_calls[0].tool_id, "anki.list_decks");

        let tool = append_message(
            &conn,
            session.id,
            &NewMessage::new(MessageRole::Tool, "{\"ok\":true}")
                .with_turn_id("t-1")
                .with_tool_result("call-1", "anki.list_decks"),
        )
        .unwrap();
        assert_eq!(tool.tool_call_id.as_deref(), Some("call-1"));
        assert_eq!(tool.tool_name.as_deref(), Some("anki.list_decks"));
    }

    #[test]
    fn interrupt_streaming_converges_leftovers() {
        let conn = setup();
        let session = insert_session(&conn, 1, "新会话").unwrap();
        let streaming = begin_assistant_message(&conn, session.id, "t-1").unwrap();

        assert_eq!(interrupt_streaming(&conn, session.id).unwrap(), 1);

        let messages = list_messages(&conn, session.id).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, streaming.id);
        assert_eq!(messages[0].status, MessageStatus::Interrupted);
        // 再次读取不会重复收敛。
        assert_eq!(interrupt_streaming(&conn, session.id).unwrap(), 0);
    }

    #[test]
    fn deleting_session_cascades_messages() {
        let conn = setup();
        let session = insert_session(&conn, 1, "新会话").unwrap();
        append_message(&conn, session.id, &NewMessage::new(MessageRole::User, "hi")).unwrap();

        delete_session(&conn, session.id).unwrap();
        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM message", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 0);
    }

    #[test]
    fn missing_session_and_message_are_reported() {
        let conn = setup();
        assert_eq!(
            append_message(&conn, 999, &NewMessage::new(MessageRole::User, "hi"))
                .unwrap_err()
                .code,
            "not_found"
        );
        assert_eq!(
            append_content(&conn, 999, "x").unwrap_err().code,
            "not_found"
        );
        assert_eq!(
            finish_message(&conn, 999, &MessagePatch::complete("x"))
                .unwrap_err()
                .code,
            "not_found"
        );
    }
}
