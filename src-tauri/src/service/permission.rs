//! 工具三级权限的判定与校验。

use rusqlite::Connection;

use crate::interface::agent::ToolPermission;
use crate::interface::error::ApiError;
use crate::repository::agent as repo;

use super::tool::ToolKey;

/// 判定 Agent 对工具的生效权限；未配置等价于 `deny`。
pub fn resolve(
    conn: &Connection,
    agent_id: i64,
    key: &ToolKey,
) -> Result<ToolPermission, ApiError> {
    repo::resolve_permission(conn, agent_id, &key.group, &key.id)
}

/// 供会话循环调用：`allow` 直接放行，`ask` 返回级别等待用户确认，`deny` 返回 `tool_denied`。
#[allow(dead_code)]
pub fn ensure_callable(
    conn: &Connection,
    agent_id: i64,
    key: &ToolKey,
) -> Result<ToolPermission, ApiError> {
    let permission = resolve(conn, agent_id, key)?;
    if permission == ToolPermission::Deny {
        return Err(ApiError::tool_denied(format!(
            "Agent {agent_id} 无权调用工具 {key}"
        )));
    }
    Ok(permission)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::db;

    fn key(raw: &str) -> ToolKey {
        ToolKey::parse(raw).unwrap()
    }

    #[test]
    fn resolves_seeded_permissions_and_unconfigured_deny() {
        let conn = db::open_in_memory().unwrap();

        assert_eq!(
            resolve(&conn, 1, &key("anki.list_decks")).unwrap(),
            ToolPermission::Allow
        );
        assert_eq!(
            resolve(&conn, 1, &key("anki.add_card")).unwrap(),
            ToolPermission::Ask
        );

        conn.execute("DELETE FROM agent_tool WHERE agent_id = 1", [])
            .unwrap();
        assert_eq!(
            resolve(&conn, 1, &key("anki.list_decks")).unwrap(),
            ToolPermission::Deny
        );
    }

    #[test]
    fn ensure_callable_rejects_denied_and_missing() {
        let conn = db::open_in_memory().unwrap();

        assert_eq!(
            ensure_callable(&conn, 1, &key("anki.add_card")).unwrap(),
            ToolPermission::Ask
        );

        conn.execute("DELETE FROM agent_tool WHERE agent_id = 1", [])
            .unwrap();
        assert_eq!(
            ensure_callable(&conn, 1, &key("anki.add_card"))
                .unwrap_err()
                .code,
            "tool_denied"
        );

        assert_eq!(
            ensure_callable(&conn, 999, &key("anki.add_card"))
                .unwrap_err()
                .code,
            "not_found"
        );
    }
}
