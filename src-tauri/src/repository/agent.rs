//! Agent、工具目录与工具权限的数据访问。

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};

use crate::interface::agent::{
    split_tool_id, AgentConfigInput, AgentInfo, AgentToolPermission, AgentToolPermissionInput,
    ToolGroup, ToolInfo, ToolPermission,
};
use crate::interface::error::ApiError;

use super::with_tx;

const AGENT_COLUMNS: &str = "id, name, description, icon, color, builtin";

/// 列出全部 Agent。
pub fn list_agents(conn: &Connection) -> Result<Vec<AgentInfo>, ApiError> {
    let mut stmt = conn.prepare(&format!("SELECT {AGENT_COLUMNS} FROM agent ORDER BY id"))?;
    let rows = stmt.query_map([], agent_from_row)?;
    let mut agents = Vec::new();
    for row in rows {
        agents.push(row?);
    }
    Ok(agents)
}

/// 按 id 读取单个 Agent。
pub fn get_agent(conn: &Connection, agent_id: i64) -> Result<Option<AgentInfo>, ApiError> {
    conn.query_row(
        &format!("SELECT {AGENT_COLUMNS} FROM agent WHERE id = ?1"),
        [agent_id],
        agent_from_row,
    )
    .optional()
    .map_err(ApiError::from)
}

/// 确认 Agent 存在，否则返回 `not_found`。
pub fn ensure_agent(conn: &Connection, agent_id: i64) -> Result<(), ApiError> {
    if get_agent(conn, agent_id)?.is_none() {
        return Err(ApiError::not_found(format!("Agent 不存在: {agent_id}")));
    }
    Ok(())
}

/// 名称是否已被占用（可排除指定 id）。
pub fn name_exists(
    conn: &Connection,
    name: &str,
    exclude_id: Option<i64>,
) -> Result<bool, ApiError> {
    let count: i64 = match exclude_id {
        Some(id) => conn.query_row(
            "SELECT COUNT(*) FROM agent WHERE name = ?1 AND id != ?2",
            rusqlite::params![name, id],
            |row| row.get(0),
        )?,
        None => conn.query_row(
            "SELECT COUNT(*) FROM agent WHERE name = ?1",
            [name],
            |row| row.get(0),
        )?,
    };
    Ok(count > 0)
}

/// 创建自定义 Agent（`builtin = 0`）。
pub fn insert_agent(
    conn: &Connection,
    name: &str,
    description: &str,
    icon: &str,
    color: &str,
    system_prompt: &str,
) -> Result<AgentInfo, ApiError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ApiError::invalid_input("Agent 名称不能为空"));
    }
    if name_exists(conn, name, None)? {
        return Err(ApiError::conflict(format!("Agent 名称已存在: {name}")));
    }

    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO agent (name, description, icon, color, system_prompt, builtin, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?6)",
        rusqlite::params![name, description, icon, color, system_prompt, now],
    )?;
    get_agent(conn, conn.last_insert_rowid())?
        .ok_or_else(|| ApiError::internal("新建 Agent 后无法读取"))
}

/// 更新 Agent 配置；`tool_permissions` 为 `Some` 时覆盖式写入工具权限。
pub fn update_agent(
    conn: &Connection,
    agent_id: i64,
    input: &AgentConfigInput,
) -> Result<AgentInfo, ApiError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(ApiError::invalid_input("Agent 名称不能为空"));
    }
    ensure_agent(conn, agent_id)?;
    if name_exists(conn, name, Some(agent_id))? {
        return Err(ApiError::conflict(format!("Agent 名称已存在: {name}")));
    }

    with_tx(conn, |c| {
        let now = Utc::now().to_rfc3339();
        c.execute(
            "UPDATE agent
                SET name = ?1,
                    description = ?2,
                    icon = COALESCE(?3, icon),
                    color = COALESCE(?4, color),
                    system_prompt = COALESCE(?5, system_prompt),
                    updated_at = ?6
              WHERE id = ?7",
            rusqlite::params![
                name,
                input.description,
                input.icon.as_deref(),
                input.color.as_deref(),
                input.system_prompt.as_deref(),
                now,
                agent_id
            ],
        )?;

        if let Some(items) = &input.tool_permissions {
            write_tool_permissions(c, agent_id, items)?;
        }

        get_agent(c, agent_id)?.ok_or_else(|| ApiError::internal("更新 Agent 后无法读取"))
    })
}

/// 删除 Agent；内置 Agent 受保护。
pub fn delete_agent(conn: &Connection, agent_id: i64) -> Result<(), ApiError> {
    let agent = get_agent(conn, agent_id)?
        .ok_or_else(|| ApiError::not_found(format!("Agent 不存在: {agent_id}")))?;
    if agent.builtin {
        return Err(ApiError::builtin_protected(format!(
            "内置 Agent 不可删除: {}",
            agent.name
        )));
    }
    conn.execute("DELETE FROM agent WHERE id = ?1", [agent_id])?;
    Ok(())
}

/// 列出全部工具元数据，按 `(group, id)` 升序。
pub fn list_tools(conn: &Connection) -> Result<Vec<ToolInfo>, ApiError> {
    let mut stmt = conn.prepare(
        "SELECT group_name, id, name, description, parameters, returns, default_permission
           FROM tool
          ORDER BY group_name, id",
    )?;
    let rows = stmt.query_map([], raw_tool_from_row)?;
    let mut tools = Vec::new();
    for row in rows {
        tools.push(tool_from_raw(row?)?);
    }
    Ok(tools)
}

/// 列出全部工具组及组内工具数量，按组名升序。
pub fn list_tool_groups(conn: &Connection) -> Result<Vec<ToolGroup>, ApiError> {
    let mut stmt = conn.prepare(
        "SELECT group_name, COUNT(*)
           FROM tool
          GROUP BY group_name
          ORDER BY group_name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ToolGroup {
            name: row.get(0)?,
            tool_count: row.get::<_, i64>(1)? as u32,
        })
    })?;
    let mut groups = Vec::new();
    for row in rows {
        groups.push(row?);
    }
    Ok(groups)
}

/// 按 `(group, id)` 读取单个工具。
pub fn get_tool(conn: &Connection, group: &str, id: &str) -> Result<Option<ToolInfo>, ApiError> {
    let raw = conn
        .query_row(
            "SELECT group_name, id, name, description, parameters, returns, default_permission
               FROM tool
              WHERE group_name = ?1 AND id = ?2",
            rusqlite::params![group, id],
            raw_tool_from_row,
        )
        .optional()?;
    raw.map(tool_from_raw).transpose()
}

/// 判定 Agent 对某工具的生效权限；未配置等价于 `deny`。
pub fn resolve_permission(
    conn: &Connection,
    agent_id: i64,
    group: &str,
    id: &str,
) -> Result<ToolPermission, ApiError> {
    ensure_agent(conn, agent_id)?;
    if get_tool(conn, group, id)?.is_none() {
        return Err(ApiError::not_found(format!("工具不存在: {group}.{id}")));
    }

    let raw: Option<String> = conn
        .query_row(
            "SELECT permission FROM agent_tool
              WHERE agent_id = ?1 AND tool_group = ?2 AND tool_id = ?3",
            rusqlite::params![agent_id, group, id],
            |row| row.get(0),
        )
        .optional()?;

    match raw {
        Some(permission) => parse_permission(&permission),
        None => Ok(ToolPermission::Deny),
    }
}

/// 设置单个工具权限。
pub fn set_tool_permission(
    conn: &Connection,
    agent_id: i64,
    group: &str,
    id: &str,
    permission: ToolPermission,
) -> Result<AgentToolPermission, ApiError> {
    let tool = get_tool(conn, group, id)?
        .ok_or_else(|| ApiError::not_found(format!("工具不存在: {group}.{id}")))?;
    ensure_agent(conn, agent_id)?;

    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO agent_tool (agent_id, tool_group, tool_id, permission, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)
         ON CONFLICT (agent_id, tool_group, tool_id)
         DO UPDATE SET permission = excluded.permission, updated_at = excluded.updated_at",
        rusqlite::params![agent_id, group, id, permission.as_str(), now],
    )?;

    Ok(AgentToolPermission { tool, permission })
}

/// 覆盖式批量设置工具权限：先清空该 Agent 的全部记录，再写入给定项。
pub fn set_tool_permissions(
    conn: &Connection,
    agent_id: i64,
    items: &[AgentToolPermissionInput],
) -> Result<Vec<AgentToolPermission>, ApiError> {
    with_tx(conn, |c| write_tool_permissions(c, agent_id, items))
}

/// 覆盖式写入工具权限，不含事务，供上层组合调用。
pub(crate) fn write_tool_permissions(
    conn: &Connection,
    agent_id: i64,
    items: &[AgentToolPermissionInput],
) -> Result<Vec<AgentToolPermission>, ApiError> {
    ensure_agent(conn, agent_id)?;

    // 先校验全部工具引用并解析，再落库，保证失败时不产生半更新状态。
    let mut resolved: Vec<AgentToolPermission> = Vec::with_capacity(items.len());
    for item in items {
        let (group, id) = split_tool_id(&item.tool_id)?;
        let tool = get_tool(conn, &group, &id)?
            .ok_or_else(|| ApiError::not_found(format!("工具不存在: {}", item.tool_id)))?;
        match resolved
            .iter_mut()
            .find(|entry| entry.tool.group == tool.group && entry.tool.id == tool.id)
        {
            Some(existing) => existing.permission = item.permission,
            None => resolved.push(AgentToolPermission {
                tool,
                permission: item.permission,
            }),
        }
    }

    conn.execute(
        "DELETE FROM agent_tool WHERE agent_id = ?1",
        [agent_id],
    )?;

    let now = Utc::now().to_rfc3339();
    for entry in &resolved {
        conn.execute(
            "INSERT INTO agent_tool (agent_id, tool_group, tool_id, permission, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            rusqlite::params![
                agent_id,
                entry.tool.group,
                entry.tool.id,
                entry.permission.as_str(),
                now
            ],
        )?;
    }

    Ok(resolved)
}

fn agent_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentInfo> {
    Ok(AgentInfo {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        icon: row.get(3)?,
        color: row.get(4)?,
        builtin: row.get::<_, i64>(5)? != 0,
    })
}

/// 工具原始行：Schema 与权限先保持字符串，稍后解析。
struct RawTool {
    group: String,
    id: String,
    name: String,
    description: String,
    parameters: String,
    returns: String,
    default_permission: String,
}

fn raw_tool_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawTool> {
    Ok(RawTool {
        group: row.get(0)?,
        id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        parameters: row.get(4)?,
        returns: row.get(5)?,
        default_permission: row.get(6)?,
    })
}

fn tool_from_raw(raw: RawTool) -> Result<ToolInfo, ApiError> {
    Ok(ToolInfo {
        group: raw.group,
        id: raw.id,
        name: raw.name,
        description: raw.description,
        parameters: parse_schema("parameters", &raw.parameters)?,
        returns: parse_schema("returns", &raw.returns)?,
        default_permission: parse_permission(&raw.default_permission)?,
    })
}

fn parse_schema(field: &str, raw: &str) -> Result<serde_json::Value, ApiError> {
    serde_json::from_str(raw)
        .map_err(|e| ApiError::internal(format!("工具 {field} Schema 无法解析: {e}")))
}

fn parse_permission(raw: &str) -> Result<ToolPermission, ApiError> {
    ToolPermission::parse(raw)
        .ok_or_else(|| ApiError::internal(format!("未知的工具权限: {raw}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::db;

    fn setup() -> Connection {
        db::open_in_memory().unwrap()
    }

    fn permission(tool_id: &str, permission: ToolPermission) -> AgentToolPermissionInput {
        AgentToolPermissionInput {
            tool_id: tool_id.to_string(),
            permission,
        }
    }

    #[test]
    fn migration_seeds_builtin_agents_tools_and_grants() {
        let conn = setup();

        let agents = list_agents(&conn).unwrap();
        assert_eq!(agents.len(), 2);
        assert!(agents.iter().all(|agent| agent.builtin));
        assert_eq!(agents[0].id, 1);

        let tools = list_tools(&conn).unwrap();
        assert_eq!(tools.len(), 14);
        assert_eq!(tools[0].group, "anki");
        assert_eq!(tools[0].id, "add_card");
        assert_eq!(tools[0].default_permission, ToolPermission::Ask);
        assert_eq!(tools[0].parameters, serde_json::json!({}));

        let groups = list_tool_groups(&conn).unwrap();
        assert_eq!(
            groups
                .iter()
                .map(|group| (group.name.as_str(), group.tool_count))
                .collect::<Vec<_>>(),
            vec![("anki", 10), ("asset", 3), ("user", 1)]
        );

        // 内置授权直接采用默认级别：新增牌组为 ask，列出牌组为 allow。
        assert_eq!(
            resolve_permission(&conn, 1, "anki", "add_card").unwrap(),
            ToolPermission::Ask
        );
        assert_eq!(
            resolve_permission(&conn, 1, "anki", "list_decks").unwrap(),
            ToolPermission::Allow
        );
    }

    #[test]
    fn agent_crud_round_trip_and_name_conflict() {
        let conn = setup();

        let agent = insert_agent(&conn, "操作系统 Agent", "课程知识", "term", "#333", "prompt")
            .unwrap();
        assert!(!agent.builtin);
        assert_eq!(get_agent(&conn, agent.id).unwrap().unwrap().name, "操作系统 Agent");
        assert!(name_exists(&conn, "操作系统 Agent", None).unwrap());
        assert!(!name_exists(&conn, "操作系统 Agent", Some(agent.id)).unwrap());
        assert_eq!(
            insert_agent(&conn, "操作系统 Agent", "", "x", "", "").unwrap_err().code,
            "conflict"
        );

        let updated = update_agent(
            &conn,
            agent.id,
            &AgentConfigInput {
                name: "操作系统".into(),
                description: "新的描述".into(),
                icon: None,
                color: Some("#fff".into()),
                system_prompt: None,
                tool_permissions: None,
            },
        )
        .unwrap();
        assert_eq!(updated.name, "操作系统");
        assert_eq!(updated.description, "新的描述");
        // 未提供的字段沿用原值。
        assert_eq!(updated.icon, "term");
        assert_eq!(updated.color, "#fff");

        delete_agent(&conn, agent.id).unwrap();
        assert!(get_agent(&conn, agent.id).unwrap().is_none());
        assert_eq!(delete_agent(&conn, agent.id).unwrap_err().code, "not_found");
    }

    #[test]
    fn builtin_agent_cannot_be_deleted() {
        let conn = setup();
        assert_eq!(delete_agent(&conn, 1).unwrap_err().code, "builtin_protected");
        assert_eq!(delete_agent(&conn, 999).unwrap_err().code, "not_found");
    }

    #[test]
    fn permission_defaults_to_deny_and_upserts() {
        let conn = setup();

        // 内置授权里没有的操作默认拒绝。
        assert_eq!(
            resolve_permission(&conn, 1, "user", "ask_question").unwrap(),
            ToolPermission::Allow
        );
        conn.execute("DELETE FROM agent_tool WHERE agent_id = 1", [])
            .unwrap();
        assert_eq!(
            resolve_permission(&conn, 1, "anki", "add_card").unwrap(),
            ToolPermission::Deny
        );

        let entry =
            set_tool_permission(&conn, 1, "anki", "add_card", ToolPermission::Allow).unwrap();
        assert_eq!(entry.permission, ToolPermission::Allow);
        assert_eq!(
            resolve_permission(&conn, 1, "anki", "add_card").unwrap(),
            ToolPermission::Allow
        );

        set_tool_permission(&conn, 1, "anki", "add_card", ToolPermission::Deny).unwrap();
        assert_eq!(
            resolve_permission(&conn, 1, "anki", "add_card").unwrap(),
            ToolPermission::Deny
        );

        assert_eq!(
            resolve_permission(&conn, 1, "missing", "tool").unwrap_err().code,
            "not_found"
        );
        assert_eq!(
            resolve_permission(&conn, 999, "anki", "add_card")
                .unwrap_err()
                .code,
            "not_found"
        );
    }

    #[test]
    fn set_tool_permissions_overwrites_and_validates() {
        let conn = setup();

        let written = set_tool_permissions(
            &conn,
            1,
            &[
                permission("anki.add_card", ToolPermission::Deny),
                permission("anki.list_decks", ToolPermission::Ask),
            ],
        )
        .unwrap();
        assert_eq!(written.len(), 2);

        // 覆盖式：原有授权全部被替换。
        let remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM agent_tool WHERE agent_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 2);

        // 重复工具引用时后者生效，且不会触发主键冲突。
        let written = set_tool_permissions(
            &conn,
            1,
            &[
                permission("anki.add_card", ToolPermission::Allow),
                permission("anki.add_card", ToolPermission::Deny),
            ],
        )
        .unwrap();
        assert_eq!(written.len(), 1);
        assert_eq!(written[0].permission, ToolPermission::Deny);

        // 非法引用与不存在的工具都整体失败，且不破坏已有数据。
        assert_eq!(
            set_tool_permissions(&conn, 1, &[permission("nope", ToolPermission::Allow)])
                .unwrap_err()
                .code,
            "invalid_input"
        );
        assert_eq!(
            set_tool_permissions(&conn, 1, &[permission("anki.nope", ToolPermission::Allow)])
                .unwrap_err()
                .code,
            "not_found"
        );
        let remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM agent_tool WHERE agent_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 1);
    }

    #[test]
    fn update_agent_writes_permissions_atomically() {
        let conn = setup();

        let updated = update_agent(
            &conn,
            1,
            &AgentConfigInput {
                name: "数学 Agent".into(),
                description: "数学问题、公式推导与解题思路".into(),
                icon: None,
                color: None,
                system_prompt: None,
                tool_permissions: Some(vec![permission(
                    "anki.add_card",
                    ToolPermission::Deny,
                )]),
            },
        )
        .unwrap();
        assert_eq!(updated.id, 1);
        assert_eq!(
            resolve_permission(&conn, 1, "anki", "add_card").unwrap(),
            ToolPermission::Deny
        );

        // 引用不存在的工具时整体回滚：授权与名称都保持原样。
        let err = update_agent(
            &conn,
            1,
            &AgentConfigInput {
                name: "改个名字".into(),
                description: "".into(),
                icon: None,
                color: None,
                system_prompt: None,
                tool_permissions: Some(vec![permission("anki.nope", ToolPermission::Allow)]),
            },
        )
        .unwrap_err();
        assert_eq!(err.code, "not_found");
        assert_eq!(get_agent(&conn, 1).unwrap().unwrap().name, "数学 Agent");
        assert_eq!(
            resolve_permission(&conn, 1, "anki", "add_card").unwrap(),
            ToolPermission::Deny
        );
    }
}
