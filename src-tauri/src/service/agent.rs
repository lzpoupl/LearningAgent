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
}
