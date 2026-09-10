use serde::{Deserialize, Serialize};

/// 卡片记忆状态（与具体调度算法无关）
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CardState {
    New,
    Learning,
    Review,
    Relearning,
}

/// 作答等级：重来 / 困难 / 良好 / 简单
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CardGrade {
    Again,
    Hard,
    Good,
    Easy,
}

/// 牌组
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Deck {
    pub path: String,
    pub name: String,
    pub card_count: u32,
    pub subdeck_count: u32,
    pub created_at: String,
}

/// 卡片
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub deck_path: String,
    pub front: String,
    pub back: String,
    pub state: CardState,
    pub due_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 获取卡片列表的查询条件
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardQuery {
    pub state: Option<CardState>,
    pub due_before: Option<String>,
    pub due_after: Option<String>,
    pub keyword: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// 搜索卡片（跨牌组/按牌组）
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardSearch {
    pub deck_path: Option<String>,
    pub front: bool,
    pub back: bool,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// 新建卡片输入
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewCard {
    pub deck_path: String,
    pub front: String,
    pub back: String,
}

/// 更新卡片内容
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCardContent {
    pub front: Option<String>,
    pub back: Option<String>,
}

/// 作答/重置结果（调度字段后续确定）
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReviewOutcome {
    pub card_id: String,
    pub state: CardState,
    pub due_at: Option<String>,
}

/// 复习选项：按当前调度策略预演四种作答等级各自的下次复习安排。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReviewOption {
    pub grade: CardGrade,
    pub interval_label: String,
    pub due_at: Option<String>,
}

/// 接口错误
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnkiError {
    pub code: String,
    pub message: String,
}
