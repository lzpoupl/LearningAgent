//! Anki 卡片统计的接口类型。
//!
//! 统计只读取 `card` 与 `review_log` 两张表，所有「天」均按本地时区日期
//! （`YYYY-MM-DD`）切分，与 Anki 的统计口径保持一致。

use serde::{Deserialize, Serialize};

/// 图表时间范围。
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TimeRange {
    /// 近 30 天（含今天）。
    LastMonth,
    /// 近 90 天（含今天）。
    LastThreeMonths,
    /// 近 365 天（含今天）。
    LastYear,
    /// 自首个有数据的日期起。
    All,
}

/// 卡片分类，与 `CardState` 一一对应，顺序即图例顺序。
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CardCategory {
    /// 新卡。
    New,
    /// 学习中。
    Learning,
    /// 复习中。
    Review,
    /// 重新学习。
    Relearning,
}

/// 今日学习统计：首页环形图与统计页第一张卡片共用。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TodayProgress {
    /// 本地日期 `YYYY-MM-DD`。
    pub date: String,
    /// 今日已作答的去重卡片数。
    pub reviewed_cards: u32,
    /// 此刻仍需作答的卡片数。
    pub pending_cards: u32,
    /// 卡片总数（含暂停与搁置）。
    pub total_cards: u32,
    /// 待作答卡片中未学习的数量。
    pub new_remaining: u32,
    /// 待作答卡片中已到期需复习的数量。
    pub due_remaining: u32,
    /// 今日完成度：已完成 /（已完成 + 待完成）。
    pub completion_percent: f64,
}

/// 单个分类的数量与占比。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardCategoryCount {
    pub category: CardCategory,
    /// 展示名，如「新卡」。
    pub label: String,
    pub count: u32,
    /// 占 `CardBreakdown::total` 的百分比。
    pub percent: f64,
}

/// 卡片数量分布。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardBreakdown {
    pub total: u32,
    /// 固定顺序：新卡、学习中、复习中、重新学习。
    pub categories: Vec<CardCategoryCount>,
}

/// 条形图中的一天。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DailyCount {
    /// 本地日期 `YYYY-MM-DD`。
    pub day: String,
    /// 距今天数：`0` 为今天，`-N` 为 N 天前。
    pub days_ago: i32,
    pub count: u32,
}

/// 复习行为历史。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReviewHistoryStats {
    pub range: TimeRange,
    /// 范围内每一天，无数据的天补零。
    pub days: Vec<DailyCount>,
    pub total_reviews: u32,
    /// 学习天数：范围内有作答记录的天数。
    pub studied_days: u32,
    /// 范围内包含今天在内的自然天数。
    pub elapsed_days: u32,
    /// 学习天数占比。
    pub studied_day_percent: f64,
    /// 平均值（包含未学习天数）。
    pub average_per_elapsed_day: f64,
    /// 平均值（只计实际学习天数）。
    pub average_per_studied_day: f64,
}

/// 新增卡片趋势。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddedCardsStats {
    pub range: TimeRange,
    /// 范围内每一天，无新增的天补零。
    pub days: Vec<DailyCount>,
    pub total: u32,
    /// 范围内包含今天在内的自然天数。
    pub elapsed_days: u32,
    pub average_per_day: f64,
}
