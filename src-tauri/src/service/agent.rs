//! Agent 配置与工具权限编排。

use std::sync::{Arc, Mutex, MutexGuard};

use crate::config::ConfigHandle;
use crate::interface::agent::{
    AgentConfigInput, AgentInfo, AgentToolPermission, AgentToolPermissionInput, ToolGroup, ToolInfo,
    ToolPermission,
};
use crate::interface::error::ApiError;
use crate::repository::agent as repo;

use super::permission;
use super::tool::{ToolKey, ToolRegistry};

/// 新建 Agent 未指定图标 / 配色时使用的默认值，与迁移中的默认值一致。
const DEFAULT_ICON: &str = "AI";
const DEFAULT_COLOR: &str = "";

/// 测试环境专用 Agent 的名称；仅在 debug 构建中播种。
#[cfg(debug_assertions)]
const DEBUG_AGENT_NAME: &str = "测试环境 Agent";

/// 测试环境专用 Agent 的系统提示词，明确其职责与权限范围。
#[cfg(debug_assertions)]
const DEBUG_AGENT_PROMPT: &str = "\
你是测试环境专用的调试 Agent，职责是验证 Agent 配置、工具权限与对话编排等后端能力。\
你拥有全部工具的 allow 权限，可以直接调用任意工具完成端到端测试，无需等待用户确认。\
回答时请明确说明当前处于测试环境，并聚焦于测试目标与验证结论。";

/// Agent 服务：持有数据库连接、工具注册表与配置句柄。
#[allow(dead_code)]
pub struct AgentService {
    db: Arc<Mutex<rusqlite::Connection>>,
    registry: Arc<ToolRegistry>,
    config: ConfigHandle,
}

impl AgentService {
    pub fn new(
        db: Arc<Mutex<rusqlite::Connection>>,
        registry: Arc<ToolRegistry>,
        config: ConfigHandle,
    ) -> Self {
        Self {
            db,
            registry,
            config,
        }
    }

    /// 获取数据库连接锁；锁在方法内部持有，命令层无需关心并发细节。
    fn conn(&self) -> Result<MutexGuard<'_, rusqlite::Connection>, ApiError> {
        self.db
            .lock()
            .map_err(|_| ApiError::internal("数据库连接不可用"))
    }

    // ---------- Agent ----------

    pub fn list_agents(&self) -> Result<Vec<AgentInfo>, ApiError> {
        let conn = self.conn()?;
        repo::list_agents(&conn)
    }

    pub fn get_agent(&self, agent_id: i64) -> Result<AgentInfo, ApiError> {
        let conn = self.conn()?;
        repo::get_agent(&conn, agent_id)?
            .ok_or_else(|| ApiError::not_found(format!("Agent 不存在: {agent_id}")))
    }

    pub fn create_agent(&self, input: AgentConfigInput) -> Result<AgentInfo, ApiError> {
        let conn = self.conn()?;
        crate::repository::with_tx(&conn, |c| {
            let agent = repo::insert_agent(
                c,
                &input.name,
                &input.description,
                input.icon.as_deref().unwrap_or(DEFAULT_ICON),
                input.color.as_deref().unwrap_or(DEFAULT_COLOR),
                input.system_prompt.as_deref().unwrap_or(""),
            )?;
            if let Some(items) = &input.tool_permissions {
                repo::write_tool_permissions(c, agent.id, items)?;
            }
            repo::get_agent(c, agent.id)?.ok_or_else(|| ApiError::internal("新建 Agent 后无法读取"))
        })
    }

    pub fn update_agent(
        &self,
        agent_id: i64,
        input: AgentConfigInput,
    ) -> Result<AgentInfo, ApiError> {
        let conn = self.conn()?;
        repo::update_agent(&conn, agent_id, &input)
    }

    pub fn delete_agent(&self, agent_id: i64) -> Result<(), ApiError> {
        let conn = self.conn()?;
        repo::delete_agent(&conn, agent_id)
    }

    // ---------- 工具与权限 ----------

    pub fn list_tools(&self) -> Result<Vec<ToolInfo>, ApiError> {
        let conn = self.conn()?;
        repo::list_tools(&conn)
    }

    pub fn list_tool_groups(&self) -> Result<Vec<ToolGroup>, ApiError> {
        let conn = self.conn()?;
        repo::list_tool_groups(&conn)
    }

    /// 工具目录全集 + 该 Agent 的逐项生效级别，未配置的工具显示为 `deny`。
    pub fn get_tool_permissions(&self, agent_id: i64) -> Result<Vec<AgentToolPermission>, ApiError> {
        let conn = self.conn()?;
        repo::ensure_agent(&conn, agent_id)?;
        let tools = repo::list_tools(&conn)?;

        let mut permissions = Vec::with_capacity(tools.len());
        for tool in tools {
            let level = repo::resolve_permission(&conn, agent_id, &tool.group, &tool.id)?;
            permissions.push(AgentToolPermission {
                tool,
                permission: level,
            });
        }
        Ok(permissions)
    }

    pub fn set_tool_permission(
        &self,
        agent_id: i64,
        tool_id: &str,
        permission: ToolPermission,
    ) -> Result<AgentToolPermission, ApiError> {
        let key = ToolKey::parse(tool_id)?;
        let conn = self.conn()?;
        repo::set_tool_permission(&conn, agent_id, &key.group, &key.id, permission)
    }

    /// 覆盖式批量设置工具权限。
    pub fn set_tool_permissions(
        &self,
        agent_id: i64,
        items: Vec<AgentToolPermissionInput>,
    ) -> Result<Vec<AgentToolPermission>, ApiError> {
        let conn = self.conn()?;
        repo::set_tool_permissions(&conn, agent_id, &items)
    }

    pub fn resolve_tool_permission(
        &self,
        agent_id: i64,
        tool_id: &str,
    ) -> Result<ToolPermission, ApiError> {
        let key = ToolKey::parse(tool_id)?;
        let conn = self.conn()?;
        permission::resolve(&conn, agent_id, &key)
    }

    // ---------- 测试环境 ----------

    /// 播种测试环境专用 Agent，并授予全部工具 `allow` 权限。
    ///
    /// 仅在 debug 构建中可用；名称已存在时直接返回既有 Agent。测试环境使用
    /// 内存数据库，正常启动流程下每次都会重新创建。
    #[cfg(debug_assertions)]
    pub fn ensure_debug_agent(&self) -> Result<AgentInfo, ApiError> {
        let conn = self.conn()?;
        if let Some(existing) = repo::list_agents(&conn)?
            .into_iter()
            .find(|agent| agent.name == DEBUG_AGENT_NAME)
        {
            return Ok(existing);
        }

        crate::repository::with_tx(&conn, |c| {
            let agent = repo::insert_agent(
                c,
                DEBUG_AGENT_NAME,
                "测试环境专用 Agent，拥有全部工具权限",
                DEFAULT_ICON,
                DEFAULT_COLOR,
                DEBUG_AGENT_PROMPT,
            )?;

            let items: Vec<AgentToolPermissionInput> = repo::list_tools(c)?
                .into_iter()
                .map(|tool| AgentToolPermissionInput {
                    tool_id: format!("{}.{}", tool.group, tool.id),
                    permission: ToolPermission::Allow,
                })
                .collect();
            repo::write_tool_permissions(c, agent.id, &items)?;

            repo::get_agent(c, agent.id)?
                .ok_or_else(|| ApiError::internal("播种测试环境 Agent 后无法读取"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::repository::db;

    fn service() -> AgentService {
        let conn = db::open_in_memory().unwrap();
        AgentService::new(
            Arc::new(Mutex::new(conn)),
            Arc::new(ToolRegistry::new()),
            ConfigHandle::new(AppConfig::default()),
        )
    }

    #[test]
    fn list_and_get_seeded_agents() {
        let service = service();
        let agents = service.list_agents().unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(service.get_agent(agents[0].id).unwrap().name, agents[0].name);
        assert_eq!(service.get_agent(999).unwrap_err().code, "not_found");
    }

    #[test]
    fn create_and_update_agent_include_permissions() {
        let service = service();

        let created = service
            .create_agent(AgentConfigInput {
                name: "计算机 Agent".into(),
                description: "操作系统与算法".into(),
                icon: Some("terminal".into()),
                color: None,
                system_prompt: None,
                tool_permissions: Some(vec![AgentToolPermissionInput {
                    tool_id: "anki.list_decks".into(),
                    permission: ToolPermission::Deny,
                }]),
            })
            .unwrap();
        assert!(!created.builtin);
        assert_eq!(created.icon, "terminal");
        assert_eq!(created.color, DEFAULT_COLOR);
        assert_eq!(
            service
                .resolve_tool_permission(created.id, "anki.list_decks")
                .unwrap(),
            ToolPermission::Deny
        );

        let updated = service
            .update_agent(
                created.id,
                AgentConfigInput {
                    name: "计算机".into(),
                    description: "操作系统与算法".into(),
                    icon: None,
                    color: None,
                    system_prompt: Some("你是计算机学习助手。".into()),
                    tool_permissions: Some(vec![AgentToolPermissionInput {
                        tool_id: "anki.list_decks".into(),
                        permission: ToolPermission::Allow,
                    }]),
                },
            )
            .unwrap();
        assert_eq!(updated.icon, "terminal");
        assert_eq!(
            service
                .resolve_tool_permission(created.id, "anki.list_decks")
                .unwrap(),
            ToolPermission::Allow
        );

        service.delete_agent(created.id).unwrap();
        assert_eq!(service.get_agent(created.id).unwrap_err().code, "not_found");
    }

    #[test]
    fn tool_catalog_and_permission_matrix() {
        let service = service();

        assert_eq!(service.list_tools().unwrap().len(), 14);
        assert_eq!(service.list_tool_groups().unwrap().len(), 3);

        let matrix = service.get_tool_permissions(1).unwrap();
        assert_eq!(matrix.len(), 14);
        assert!(matrix
            .iter()
            .any(|entry| entry.tool.id == "add_card"
                && entry.permission == ToolPermission::Ask));

        assert_eq!(
            service.get_tool_permissions(999).unwrap_err().code,
            "not_found"
        );
    }

    #[test]
    fn set_tool_permissions_overwrites_and_validates_ids() {
        let service = service();

        let written = service
            .set_tool_permissions(
                1,
                vec![AgentToolPermissionInput {
                    tool_id: "anki.add_card".into(),
                    permission: ToolPermission::Deny,
                }],
            )
            .unwrap();
        assert_eq!(written.len(), 1);

        let matrix = service.get_tool_permissions(1).unwrap();
        assert!(matrix
            .iter()
            .all(|entry| entry.tool.id == "add_card" || entry.permission == ToolPermission::Deny));
        assert!(matrix
            .iter()
            .any(|entry| entry.tool.id == "list_decks" && entry.permission == ToolPermission::Deny));

        assert_eq!(
            service
                .set_tool_permission(1, "no-dot", ToolPermission::Allow)
                .unwrap_err()
                .code,
            "invalid_input"
        );
        assert_eq!(
            service
                .set_tool_permission(1, "anki.nope", ToolPermission::Allow)
                .unwrap_err()
                .code,
            "not_found"
        );
        assert_eq!(
            service
                .resolve_tool_permission(1, "missing.tool")
                .unwrap_err()
                .code,
            "not_found"
        );
    }

    #[test]
    fn builtin_agent_is_protected() {
        let service = service();
        assert_eq!(service.delete_agent(1).unwrap_err().code, "builtin_protected");
    }

    #[cfg(debug_assertions)]
    #[test]
    fn debug_agent_seeds_with_full_tool_permissions() {
        let service = service();

        let agent = service.ensure_debug_agent().unwrap();
        assert_eq!(agent.name, DEBUG_AGENT_NAME);
        assert!(!agent.builtin);
        assert_eq!(service.get_agent(agent.id).unwrap().name, DEBUG_AGENT_NAME);

        let matrix = service.get_tool_permissions(agent.id).unwrap();
        assert_eq!(matrix.len(), 14);
        assert!(matrix
            .iter()
            .all(|entry| entry.permission == ToolPermission::Allow));

        // 重复调用返回既有 Agent，不会因名称唯一约束而失败。
        assert_eq!(service.ensure_debug_agent().unwrap().id, agent.id);
        assert_eq!(service.list_agents().unwrap().len(), 3);
    }
}
