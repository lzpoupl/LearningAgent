//! Anki 工具实现：读写牌组与卡片，供 Agent 循环在用户授权后调用。
//!
//! 工具执行发生在 `with_conn` 已持有连接锁的同步上下文中，因此这里只使用
//! `repository::*` 与 `service::anki` 的模块级函数，绝不调用会再次加锁的服务方法。
//! `ToolOutcome.content` 只放业务载荷，`ok` / `error` 信封由循环统一包装。

use std::sync::Arc;

use chrono::Utc;
use serde_json::{json, Value};

use crate::interface::anki::{CardGrade, CardQuery, CardSearch, CardState};
use crate::interface::error::ApiError;
use crate::repository::{anki as card_repo, deck as deck_repo};
use crate::service::anki::{
    card_not_found, ensure_concrete_deck, grade_card_with, parse_id, resolve_deck_id,
};
use crate::service::scheduler::SchedulerRegistry;

use super::args::{limit, offset, optional_bool, optional_str, required_id, required_str};
use super::{Tool, ToolContext, ToolKey, ToolOutcome, ToolRegistry};

const GROUP: &str = "anki";

/// 本阶段实现的 anki 工具 id 清单，与 `000005.sql` 中的契约一一对应。
pub const TOOL_IDS: [&str; 10] = [
    "list_decks",
    "list_cards",
    "get_card",
    "search_cards",
    "add_deck",
    "add_card",
    "update_card",
    "move_card",
    "grade_card",
    "delete_card",
];

/// Anki 工具：一个实例对应一个工具 id。
pub struct AnkiTool {
    key: ToolKey,
    /// 仅 `grade_card` 需要；其余工具从 `ctx.config` 读取配置。
    schedulers: Arc<SchedulerRegistry>,
}

impl AnkiTool {
    pub fn new(id: &str, schedulers: Arc<SchedulerRegistry>) -> Self {
        Self {
            key: ToolKey {
                group: GROUP.to_string(),
                id: id.to_string(),
            },
            schedulers,
        }
    }

    fn list_decks(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let deck_path = optional_str(args, "deckPath")?.unwrap_or_else(|| "/".to_string());
        let deck_id = resolve_deck_id(ctx.conn, &deck_path)?;
        let decks = deck_repo::list_subdecks(ctx.conn, deck_id)?;
        let count = decks.len();
        Ok(json!({ "decks": decks, "count": count }))
    }

    fn list_cards(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let deck_path = required_str(args, "deckPath")?;
        let state = match optional_str(args, "state")? {
            Some(raw) => Some(parse_state(&raw)?),
            None => None,
        };
        let keyword = optional_str(args, "keyword")?;
        let due_only = optional_bool(args, "dueOnly")?.unwrap_or(false);
        let limit = limit(args)?;
        let offset = offset(args)?;

        let deck_id = resolve_deck_id(ctx.conn, &deck_path)?;
        let query = CardQuery {
            state,
            due_before: due_only.then(|| Utc::now().to_rfc3339()),
            due_after: None,
            keyword,
            // 多查一条用于判断 hasMore，截断后再返回。
            limit: Some(limit.saturating_add(1)),
            offset: Some(offset),
        };
        let cards = card_repo::list_cards(ctx.conn, deck_id, &query)?;
        let (cards, has_more) = paginate(cards, limit);
        let count = cards.len();
        Ok(json!({ "cards": cards, "count": count, "hasMore": has_more }))
    }

    fn get_card(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let card_id = required_id(args, "cardId")?;
        let id = parse_id(&card_id)?;
        let card = card_repo::get_card(ctx.conn, id)?.ok_or_else(|| card_not_found(&card_id))?;
        Ok(json!({ "card": card }))
    }

    fn search_cards(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let keyword = required_str(args, "keyword")?;
        let deck_path = optional_str(args, "deckPath")?;
        let limit = limit(args)?;
        let offset = offset(args)?;

        let search = CardSearch {
            deck_path,
            // 不暴露字段选择，正面与背面同时作为搜索范围。
            front: true,
            back: true,
            limit: Some(limit.saturating_add(1)),
            offset: Some(offset),
        };
        let cards = card_repo::search_cards(ctx.conn, &keyword, &search)?;
        let (cards, has_more) = paginate(cards, limit);
        let count = cards.len();
        Ok(json!({ "cards": cards, "count": count, "hasMore": has_more }))
    }

    fn add_deck(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let deck_path = required_str(args, "deckPath")?;
        if deck_repo::split_path(&deck_path).is_empty() {
            return Err(ApiError::invalid_path("牌组路径不能为空"));
        }
        // 与 `create_deck` 的幂等语义区分：工具层把「已存在」显式报为冲突。
        if deck_repo::resolve_deck(ctx.conn, &deck_path)?.is_some() {
            return Err(ApiError::conflict(format!("牌组已存在: {deck_path}")));
        }
        let normalized = deck_repo::create_deck(ctx.conn, &deck_path)?;
        Ok(json!({ "deckPath": normalized }))
    }

    fn add_card(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let deck_path = required_str(args, "deckPath")?;
        let front = required_str(args, "front")?;
        let back = required_str(args, "back")?;

        let deck_id = resolve_deck_id(ctx.conn, &deck_path)?;
        ensure_concrete_deck(deck_id, &deck_path)?;
        let algorithm = ctx.config.anki().scheduler.algorithm;
        let id =
            card_repo::create_card_with_algorithm(ctx.conn, deck_id, &front, &back, &algorithm)?;
        Ok(json!({ "cardId": id.to_string() }))
    }

    fn update_card(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let card_id = required_id(args, "cardId")?;
        let front = optional_str(args, "front")?;
        let back = optional_str(args, "back")?;
        if front.is_none() && back.is_none() {
            return Err(ApiError::invalid_input("参数 front 与 back 至少提供一个"));
        }

        let id = parse_id(&card_id)?;
        card_repo::update_card_content(ctx.conn, id, front, back)?;
        Ok(json!({ "cardId": card_id }))
    }

    fn move_card(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let card_id = required_id(args, "cardId")?;
        let target_path = required_str(args, "targetDeckPath")?;

        let id = parse_id(&card_id)?;
        let target_id = resolve_deck_id(ctx.conn, &target_path)?;
        ensure_concrete_deck(target_id, &target_path)?;
        card_repo::move_card(ctx.conn, id, target_id)?;

        let deck_path = deck_repo::deck_path(ctx.conn, target_id)?;
        Ok(json!({ "cardId": card_id, "deckPath": deck_path }))
    }

    fn grade_card(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let card_id = required_id(args, "cardId")?;
        let grade = parse_grade(&required_str(args, "grade")?)?;

        let outcome =
            grade_card_with(ctx.conn, &self.schedulers, ctx.config, &card_id, grade)?;
        serde_json::to_value(outcome)
            .map_err(|e| ApiError::internal(format!("序列化作答结果失败: {e}")))
    }

    fn delete_card(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let card_id = required_id(args, "cardId")?;
        let id = parse_id(&card_id)?;
        card_repo::delete_card(ctx.conn, id)?;
        Ok(json!({ "cardId": card_id, "deleted": true }))
    }
}

impl Tool for AnkiTool {
    fn key(&self) -> ToolKey {
        self.key.clone()
    }

    fn execute(
        &self,
        ctx: &ToolContext<'_>,
        arguments: Value,
    ) -> Result<ToolOutcome, ApiError> {
        let content = match self.key.id.as_str() {
            "list_decks" => self.list_decks(ctx, &arguments)?,
            "list_cards" => self.list_cards(ctx, &arguments)?,
            "get_card" => self.get_card(ctx, &arguments)?,
            "search_cards" => self.search_cards(ctx, &arguments)?,
            "add_deck" => self.add_deck(ctx, &arguments)?,
            "add_card" => self.add_card(ctx, &arguments)?,
            "update_card" => self.update_card(ctx, &arguments)?,
            "move_card" => self.move_card(ctx, &arguments)?,
            "grade_card" => self.grade_card(ctx, &arguments)?,
            "delete_card" => self.delete_card(ctx, &arguments)?,
            other => {
                return Err(ApiError::tool_unavailable(format!(
                    "工具尚未实现: anki.{other}"
                )))
            }
        };
        Ok(ToolOutcome::new(content))
    }
}

/// 注册全部 anki 工具；应用启动时调用一次。
pub fn register(registry: &mut ToolRegistry, schedulers: Arc<SchedulerRegistry>) {
    for id in TOOL_IDS {
        registry.register(Arc::new(AnkiTool::new(id, schedulers.clone())));
    }
}

fn parse_state(raw: &str) -> Result<CardState, ApiError> {
    match raw {
        "new" => Ok(CardState::New),
        "learning" => Ok(CardState::Learning),
        "review" => Ok(CardState::Review),
        "relearning" => Ok(CardState::Relearning),
        other => Err(ApiError::invalid_input(format!("参数 state 取值无效: {other}"))),
    }
}

fn parse_grade(raw: &str) -> Result<CardGrade, ApiError> {
    match raw {
        "again" => Ok(CardGrade::Again),
        "hard" => Ok(CardGrade::Hard),
        "good" => Ok(CardGrade::Good),
        "easy" => Ok(CardGrade::Easy),
        other => Err(ApiError::invalid_input(format!("参数 grade 取值无效: {other}"))),
    }
}

/// 截断为 limit 条并给出 `hasMore`；调用方需多查一条。
fn paginate<T>(mut items: Vec<T>, limit: u32) -> (Vec<T>, bool) {
    let has_more = items.len() > limit as usize;
    if has_more {
        items.truncate(limit as usize);
    }
    (items, has_more)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, ConfigHandle};
    use crate::repository::db;

    fn config() -> ConfigHandle {
        ConfigHandle::new(AppConfig::default())
    }

    fn schedulers() -> Arc<SchedulerRegistry> {
        Arc::new(SchedulerRegistry::new())
    }

    fn call(
        tool: &AnkiTool,
        conn: &rusqlite::Connection,
        config: &ConfigHandle,
        args: Value,
    ) -> Result<Value, ApiError> {
        let ctx = ToolContext {
            conn,
            agent_id: 1,
            session_id: 1,
            config,
        };
        tool.execute(&ctx, args).map(|outcome| outcome.content)
    }

    fn run(
        id: &str,
        conn: &rusqlite::Connection,
        config: &ConfigHandle,
        args: Value,
    ) -> Result<Value, ApiError> {
        let tool = AnkiTool::new(id, schedulers());
        call(&tool, conn, config, args)
    }

    fn make_deck(conn: &rusqlite::Connection, path: &str) -> i64 {
        let normalized = deck_repo::create_deck(conn, path).unwrap();
        deck_repo::resolve_deck(conn, &normalized).unwrap().unwrap()
    }

    fn create_card(conn: &rusqlite::Connection, deck_id: i64, front: &str, back: &str) -> i64 {
        card_repo::create_card_with_algorithm(conn, deck_id, front, back, "sm2").unwrap()
    }

    #[test]
    fn register_covers_all_tool_ids() {
        let mut registry = ToolRegistry::new();
        register(&mut registry, schedulers());

        let mut ids: Vec<&str> = registry
            .keys()
            .filter(|key| key.group == GROUP)
            .map(|key| key.id.as_str())
            .collect();
        ids.sort_unstable();

        let mut expected = TOOL_IDS.to_vec();
        expected.sort_unstable();
        assert_eq!(ids, expected);
    }

    #[test]
    fn unknown_id_reports_unavailable() {
        let conn = db::open_in_memory().unwrap();
        let tool = AnkiTool::new("not_a_tool", schedulers());
        let err = call(&tool, &conn, &config(), json!({})).unwrap_err();
        assert_eq!(err.code, "tool_unavailable");
    }

    #[test]
    fn list_decks_lists_direct_subdecks() {
        let conn = db::open_in_memory().unwrap();
        let config = config();
        make_deck(&conn, "/parent/child");
        make_deck(&conn, "/parent/other");

        let scoped = run("list_decks", &conn, &config, json!({ "deckPath": "/parent" })).unwrap();
        assert_eq!(scoped["count"], 2);
        let paths: Vec<&str> = scoped["decks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|deck| deck["path"].as_str().unwrap())
            .collect();
        assert_eq!(paths, vec!["/parent/child", "/parent/other"]);

        // deckPath 缺省为根。
        let root = run("list_decks", &conn, &config, json!({})).unwrap();
        assert_eq!(root["count"], 1);
        assert_eq!(root["decks"][0]["path"], "/parent");

        assert_eq!(
            run("list_decks", &conn, &config, json!({ "deckPath": "/missing" }))
                .unwrap_err()
                .code,
            "not_found"
        );
    }

    #[test]
    fn list_cards_filters_by_state_keyword_and_due_only() {
        let conn = db::open_in_memory().unwrap();
        let config = config();
        let deck = make_deck(&conn, "/d");
        let new_card = create_card(&conn, deck, "apple", "fruit");
        let due_card = create_card(&conn, deck, "banana", "fruit");
        conn.execute(
            "UPDATE card SET state = 'review', due_at = '2020-01-01T00:00:00+00:00' WHERE id = ?1",
            [due_card],
        )
        .unwrap();

        let by_state = run("list_cards", &conn, &config, json!({ "deckPath": "/d", "state": "review" }))
            .unwrap();
        assert_eq!(by_state["count"], 1);
        assert_eq!(by_state["cards"][0]["id"], due_card.to_string());

        let by_keyword =
            run("list_cards", &conn, &config, json!({ "deckPath": "/d", "keyword": "apple" })).unwrap();
        assert_eq!(by_keyword["count"], 1);
        assert_eq!(by_keyword["cards"][0]["id"], new_card.to_string());

        // dueOnly 排除 due_at IS NULL 的新卡。
        let due_only =
            run("list_cards", &conn, &config, json!({ "deckPath": "/d", "dueOnly": true })).unwrap();
        assert_eq!(due_only["count"], 1);
        assert_eq!(due_only["cards"][0]["id"], due_card.to_string());

        assert_eq!(
            run("list_cards", &conn, &config, json!({ "deckPath": "/d", "state": "nope" }))
                .unwrap_err()
                .code,
            "invalid_input"
        );
    }

    #[test]
    fn list_cards_pages_and_clamps_limit() {
        let conn = db::open_in_memory().unwrap();
        let config = config();
        let deck = make_deck(&conn, "/d");
        for index in 0..205 {
            create_card(&conn, deck, &format!("front {index}"), "back");
        }

        let clamped =
            run("list_cards", &conn, &config, json!({ "deckPath": "/d", "limit": 1000 })).unwrap();
        assert_eq!(clamped["count"], 200);
        assert_eq!(clamped["hasMore"], true);

        let page = run(
            "list_cards",
            &conn,
            &config,
            json!({ "deckPath": "/d", "limit": 10, "offset": 200 }),
        )
        .unwrap();
        assert_eq!(page["count"], 5);
        assert_eq!(page["hasMore"], false);
    }

    #[test]
    fn get_card_accepts_string_and_numeric_id() {
        let conn = db::open_in_memory().unwrap();
        let config = config();
        let deck = make_deck(&conn, "/d");
        let id = create_card(&conn, deck, "f", "b");

        let by_string = run("get_card", &conn, &config, json!({ "cardId": id.to_string() })).unwrap();
        assert_eq!(by_string["card"]["front"], "f");

        let by_number = run("get_card", &conn, &config, json!({ "cardId": id })).unwrap();
        assert_eq!(by_number["card"]["id"], id.to_string());

        assert_eq!(
            run("get_card", &conn, &config, json!({})).unwrap_err().code,
            "invalid_input"
        );
        assert_eq!(
            run("get_card", &conn, &config, json!({ "cardId": 999_999 }))
                .unwrap_err()
                .code,
            "not_found"
        );
    }

    #[test]
    fn search_cards_spans_decks_and_honors_scope() {
        let conn = db::open_in_memory().unwrap();
        let config = config();
        let one = make_deck(&conn, "/one");
        let two = make_deck(&conn, "/two");
        create_card(&conn, one, "shared", "x");
        create_card(&conn, two, "shared", "y");

        let all = run("search_cards", &conn, &config, json!({ "keyword": "shared" })).unwrap();
        assert_eq!(all["count"], 2);

        let scoped = run(
            "search_cards",
            &conn,
            &config,
            json!({ "keyword": "shared", "deckPath": "/one" }),
        )
        .unwrap();
        assert_eq!(scoped["count"], 1);
        assert_eq!(scoped["cards"][0]["deckPath"], "/one");

        assert_eq!(
            run("search_cards", &conn, &config, json!({})).unwrap_err().code,
            "invalid_input"
        );
    }

    #[test]
    fn add_deck_creates_and_conflicts() {
        let conn = db::open_in_memory().unwrap();
        let config = config();

        let created = run("add_deck", &conn, &config, json!({ "deckPath": "/a//b/" })).unwrap();
        assert_eq!(created["deckPath"], "/a/b");

        assert_eq!(
            run("add_deck", &conn, &config, json!({ "deckPath": "/a/b" }))
                .unwrap_err()
                .code,
            "conflict"
        );
        assert_eq!(
            run("add_deck", &conn, &config, json!({ "deckPath": "/" }))
                .unwrap_err()
                .code,
            "invalid_path"
        );
        assert_eq!(
            run("add_deck", &conn, &config, json!({ "deckPath": "" }))
                .unwrap_err()
                .code,
            "invalid_path"
        );
    }

    #[test]
    fn add_card_validates_deck_and_uses_configured_algorithm() {
        let conn = db::open_in_memory().unwrap();

        assert_eq!(
            run(
                "add_card",
                &conn,
                &config(),
                json!({ "deckPath": "/", "front": "f", "back": "b" })
            )
            .unwrap_err()
            .code,
            "invalid_path"
        );
        assert_eq!(
            run(
                "add_card",
                &conn,
                &config(),
                json!({ "deckPath": "/missing", "front": "f", "back": "b" })
            )
            .unwrap_err()
            .code,
            "not_found"
        );
        assert_eq!(
            run(
                "add_card",
                &conn,
                &config(),
                json!({ "deckPath": "/d", "front": "f" })
            )
            .unwrap_err()
            .code,
            "invalid_input"
        );

        let mut app = AppConfig::default();
        app.anki.scheduler.algorithm = "fsrs".to_string();
        let config = ConfigHandle::new(app);
        make_deck(&conn, "/d");
        let created = run(
            "add_card",
            &conn,
            &config,
            json!({ "deckPath": "/d", "front": "f", "back": "b" }),
        )
        .unwrap();
        let id: i64 = created["cardId"].as_str().unwrap().parse().unwrap();
        let schedule = card_repo::find_schedule(&conn, id).unwrap().unwrap();
        assert_eq!(schedule.algorithm, "fsrs");
    }

    #[test]
    fn update_card_requires_a_field_and_updates_content() {
        let conn = db::open_in_memory().unwrap();
        let config = config();
        let deck = make_deck(&conn, "/d");
        let id = create_card(&conn, deck, "old", "back");

        assert_eq!(
            run("update_card", &conn, &config, json!({ "cardId": id }))
                .unwrap_err()
                .code,
            "invalid_input"
        );

        let updated =
            run("update_card", &conn, &config, json!({ "cardId": id, "front": "new" })).unwrap();
        assert_eq!(updated["cardId"], id.to_string());

        let card = card_repo::get_card(&conn, id).unwrap().unwrap();
        assert_eq!(card.front, "new");
        assert_eq!(card.back, "back");
    }

    #[test]
    fn move_card_moves_and_validates_target() {
        let conn = db::open_in_memory().unwrap();
        let config = config();
        let from = make_deck(&conn, "/from");
        make_deck(&conn, "/to");
        let id = create_card(&conn, from, "f", "b");

        let moved = run(
            "move_card",
            &conn,
            &config,
            json!({ "cardId": id, "targetDeckPath": "/to" }),
        )
        .unwrap();
        assert_eq!(moved["deckPath"], "/to");
        assert_eq!(
            card_repo::get_card(&conn, id).unwrap().unwrap().deck_path,
            "/to"
        );

        assert_eq!(
            run(
                "move_card",
                &conn,
                &config,
                json!({ "cardId": id, "targetDeckPath": "/" })
            )
            .unwrap_err()
            .code,
            "invalid_path"
        );
        assert_eq!(
            run(
                "move_card",
                &conn,
                &config,
                json!({ "cardId": 999_999, "targetDeckPath": "/to" })
            )
            .unwrap_err()
            .code,
            "not_found"
        );
    }

    #[test]
    fn grade_card_appends_review_log() {
        let conn = db::open_in_memory().unwrap();
        let config = config();
        let deck = make_deck(&conn, "/d");
        let id = create_card(&conn, deck, "f", "b");

        let outcome =
            run("grade_card", &conn, &config, json!({ "cardId": id, "grade": "good" })).unwrap();
        assert_eq!(outcome["cardId"], id.to_string());
        assert_eq!(outcome["state"], "review");
        assert!(outcome["dueAt"].is_string());

        let logs: i64 = conn
            .query_row("SELECT COUNT(*) FROM review_log", [], |row| row.get(0))
            .unwrap();
        assert_eq!(logs, 1);

        assert_eq!(
            run("grade_card", &conn, &config, json!({ "cardId": id, "grade": "nope" }))
                .unwrap_err()
                .code,
            "invalid_input"
        );
        assert_eq!(
            run("grade_card", &conn, &config, json!({ "cardId": id }))
                .unwrap_err()
                .code,
            "invalid_input"
        );
    }

    #[test]
    fn delete_card_removes_card() {
        let conn = db::open_in_memory().unwrap();
        let config = config();
        let deck = make_deck(&conn, "/d");
        let id = create_card(&conn, deck, "f", "b");

        let deleted = run("delete_card", &conn, &config, json!({ "cardId": id.to_string() })).unwrap();
        assert_eq!(deleted["deleted"], true);
        assert!(card_repo::get_card(&conn, id).unwrap().is_none());

        assert_eq!(
            run("delete_card", &conn, &config, json!({ "cardId": id }))
                .unwrap_err()
                .code,
            "not_found"
        );
    }
}
