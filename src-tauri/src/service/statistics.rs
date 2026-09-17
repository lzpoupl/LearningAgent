//! 卡片统计的业务服务。
//!
//! repository 只返回原始计数，本模块负责界面口径：按范围生成连续日期序列
//! 并补零、计算百分比与平均值、把分类计数展开成固定顺序的图例。

use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{Duration, Local, NaiveDate, Utc};

use crate::interface::anki::AnkiError;
use crate::interface::statistics::{
    AddedCardsStats, CardBreakdown, CardCategory, CardCategoryCount, DailyCount, ReviewHistoryStats,
    TimeRange, TodayProgress,
};
use crate::repository::anki as card_repo;
use crate::repository::statistics as stats_repo;

/// 卡片统计服务：只读、无状态，与 `AnkiService` 共享同一个数据库连接。
pub struct StatisticsService {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl StatisticsService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { db }
    }

    fn conn(&self) -> Result<MutexGuard<'_, rusqlite::Connection>, AnkiError> {
        self.db.lock().map_err(|_| AnkiError {
            code: "internal".into(),
            message: "数据库连接不可用".into(),
        })
    }

    /// 今日学习统计：已完成、待完成与卡片总数。
    pub fn today_progress(&self) -> Result<TodayProgress, AnkiError> {
        let conn = self.conn()?;
        let today = card_repo::today_local();
        let now = Utc::now().to_rfc3339();
        let counts = stats_repo::today_counts(&conn, &now)?;

        let pending_cards = counts.new_remaining + counts.due_remaining;
        let answered = counts.reviewed_cards + pending_cards;

        Ok(TodayProgress {
            date: today,
            reviewed_cards: counts.reviewed_cards,
            pending_cards,
            total_cards: counts.total_cards,
            new_remaining: counts.new_remaining,
            due_remaining: counts.due_remaining,
            completion_percent: percent(counts.reviewed_cards as f64, answered as f64),
        })
    }

    /// 卡片数量分布：四个分类与 `CardState` 一一对应。
    pub fn card_breakdown(&self) -> Result<CardBreakdown, AnkiError> {
        let conn = self.conn()?;
        let counts = stats_repo::category_counts(&conn)?;

        let total = counts.total();
        let mut categories: Vec<CardCategoryCount> = [
            (CardCategory::New, "新卡", counts.new),
            (CardCategory::Learning, "学习中", counts.learning),
            (CardCategory::Review, "复习中", counts.review),
            (CardCategory::Relearning, "重新学习", counts.relearning),
        ]
        .into_iter()
        .map(|(category, label, count)| CardCategoryCount {
            category,
            label: label.to_string(),
            count,
            percent: percent(count as f64, total as f64),
        })
        .collect();

        fix_rounding(&mut categories);

        Ok(CardBreakdown { total, categories })
    }

    /// 复习行为历史：按天的复习次数与四条统计脚注。
    pub fn review_history(&self, range: TimeRange) -> Result<ReviewHistoryStats, AnkiError> {
        let conn = self.conn()?;
        let today = Local::now().date_naive();
        let start = range_start(&conn, range, today)?;
        let from = start.to_string();

        let days = fill_days(start, today, &stats_repo::review_counts_by_day(&conn, &from)?);
        let total_reviews: u32 = days.iter().map(|day| day.count).sum();
        let studied_days = stats_repo::studied_day_count(&conn, &from)?;
        let elapsed_days = days.len() as u32;

        Ok(ReviewHistoryStats {
            range,
            days,
            total_reviews,
            studied_days,
            elapsed_days,
            studied_day_percent: percent(studied_days as f64, elapsed_days as f64),
            average_per_elapsed_day: round2(ratio(total_reviews as f64, elapsed_days)),
            average_per_studied_day: round2(ratio(total_reviews as f64, studied_days)),
        })
    }

    /// 新增卡片趋势：按天的新增数量与总数、日均。
    pub fn added_cards(&self, range: TimeRange) -> Result<AddedCardsStats, AnkiError> {
        let conn = self.conn()?;
        let today = Local::now().date_naive();
        let start = range_start(&conn, range, today)?;

        let days = fill_days(
            start,
            today,
            &stats_repo::added_counts_by_day(&conn, &start.to_string())?,
        );
        let total: u32 = days.iter().map(|day| day.count).sum();
        let elapsed_days = days.len() as u32;

        Ok(AddedCardsStats {
            range,
            days,
            total,
            elapsed_days,
            average_per_day: round2(ratio(total as f64, elapsed_days)),
        })
    }
}

/// 范围覆盖的自然天数（含今天）；`All` 由实际数据决定起点，返回 0。
fn range_days(range: TimeRange) -> i64 {
    match range {
        TimeRange::LastMonth => 30,
        TimeRange::LastThreeMonths => 90,
        TimeRange::LastYear => 365,
        TimeRange::All => 0,
    }
}

/// 范围的起始日期；`All` 取首个有数据的日期，无数据时取今天。
fn range_start(
    conn: &rusqlite::Connection,
    range: TimeRange,
    today: NaiveDate,
) -> Result<NaiveDate, AnkiError> {
    match range {
        TimeRange::All => match stats_repo::first_activity_day(conn)? {
            Some(day) => Ok(parse_day(&day)?.min(today)),
            None => Ok(today),
        },
        _ => Ok(today - Duration::days(range_days(range) - 1)),
    }
}

/// 解析 `YYYY-MM-DD` 形式的日期。
fn parse_day(raw: &str) -> Result<NaiveDate, AnkiError> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d").map_err(|e| AnkiError {
        code: "internal".into(),
        message: format!("无法解析日期 {raw}: {e}"),
    })
}

/// 把聚合结果铺成从 `from` 到 `to`（含两端）的连续日期序列，缺失的天补零。
fn fill_days(from: NaiveDate, to: NaiveDate, counts: &[(String, u32)]) -> Vec<DailyCount> {
    let mut days = Vec::new();
    let mut current = from;
    while current <= to {
        let key = current.to_string();
        let count = counts
            .iter()
            .find(|(day, _)| day == &key)
            .map(|(_, count)| *count)
            .unwrap_or(0);
        days.push(DailyCount {
            day: key,
            days_ago: (current - to).num_days() as i32,
            count,
        });
        current += Duration::days(1);
    }
    days
}

/// 百分比，保留一位小数；分母为 0 时返回 0。
fn percent(value: f64, total: f64) -> f64 {
    if total <= 0.0 {
        return 0.0;
    }
    (value / total * 1000.0).round() / 10.0
}

/// 平均值；天数为 0 时返回 0。
fn ratio(value: f64, days: u32) -> f64 {
    if days == 0 {
        0.0
    } else {
        value / days as f64
    }
}

/// 保留两位小数。
fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// 四舍五入会带来尾差，把差额补给数量最多的分类，保证占比合计为 100.0%。
fn fix_rounding(categories: &mut [CardCategoryCount]) {
    let sum: f64 = categories.iter().map(|item| item.percent).sum();
    if sum == 0.0 {
        return;
    }
    let diff = ((100.0 - sum) * 10.0).round() / 10.0;
    if diff == 0.0 {
        return;
    }
    if let Some(target) = categories.iter_mut().max_by_key(|item| item.count) {
        target.percent = ((target.percent + diff) * 10.0).round() / 10.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::anki::{CardGrade, CardState};
    use crate::repository::{db, deck};

    fn service() -> (StatisticsService, Arc<Mutex<rusqlite::Connection>>, i64) {
        let conn = db::open_in_memory().unwrap();
        let path = deck::create_deck(&conn, "/统计").unwrap();
        let deck_id = deck::resolve_deck(&conn, &path).unwrap().unwrap();
        let db = Arc::new(Mutex::new(conn));
        (StatisticsService::new(db.clone()), db, deck_id)
    }

    fn add_card(db: &Arc<Mutex<rusqlite::Connection>>, deck_id: i64, state: &str) -> i64 {
        let conn = db.lock().unwrap();
        let id = card_repo::create_card_with_algorithm(&conn, deck_id, "f", "b", "sm2").unwrap();
        conn.execute(
            "UPDATE card SET state = ?1 WHERE id = ?2",
            rusqlite::params![state, id],
        )
        .unwrap();
        id
    }

    fn log_review(db: &Arc<Mutex<rusqlite::Connection>>, deck_id: i64, card_id: i64, day: &str) {
        let conn = db.lock().unwrap();
        card_repo::insert_review_log(
            &conn,
            &card_repo::ReviewLogEntry {
                card_id,
                deck_id,
                grade: CardGrade::Good,
                prev_state: CardState::Review,
                next_state: CardState::Review,
                duration_ms: 0,
                reviewed_at: format!("{day}T04:00:00+00:00"),
                review_date: day.to_string(),
            },
        )
        .unwrap();
    }

    #[test]
    fn breakdown_lists_four_states_in_order() {
        let (service, db, deck_id) = service();
        add_card(&db, deck_id, "new");
        add_card(&db, deck_id, "learning");
        add_card(&db, deck_id, "review");
        add_card(&db, deck_id, "relearning");

        let breakdown = service.card_breakdown().unwrap();
        assert_eq!(breakdown.total, 4);
        assert_eq!(breakdown.categories.len(), 4);

        let labels: Vec<&str> = breakdown
            .categories
            .iter()
            .map(|item| item.label.as_str())
            .collect();
        assert_eq!(labels, vec!["新卡", "学习中", "复习中", "重新学习"]);
        assert_eq!(breakdown.categories[0].category, CardCategory::New);
        assert_eq!(breakdown.categories[3].category, CardCategory::Relearning);
        assert!(breakdown.categories.iter().all(|item| item.count == 1));
        assert!(breakdown.categories.iter().all(|item| item.percent == 25.0));
    }

    #[test]
    fn breakdown_keeps_percent_sum_at_one_hundred() {
        let (service, db, deck_id) = service();
        add_card(&db, deck_id, "new");
        add_card(&db, deck_id, "review");
        add_card(&db, deck_id, "review");

        let breakdown = service.card_breakdown().unwrap();
        assert_eq!(breakdown.total, 3);
        assert_eq!(breakdown.categories[0].percent, 33.3);
        assert_eq!(breakdown.categories[2].percent, 66.7);

        let sum: f64 = breakdown.categories.iter().map(|item| item.percent).sum();
        assert_eq!(sum, 100.0);
    }

    #[test]
    fn empty_library_reports_zero_without_nan() {
        let (service, _, _) = service();

        let today = service.today_progress().unwrap();
        assert_eq!(today.total_cards, 0);
        assert_eq!(today.pending_cards, 0);
        assert_eq!(today.completion_percent, 0.0);

        let breakdown = service.card_breakdown().unwrap();
        assert_eq!(breakdown.total, 0);
        assert_eq!(breakdown.categories.len(), 4);
        assert!(breakdown.categories.iter().all(|item| item.percent == 0.0));

        let history = service.review_history(TimeRange::LastMonth).unwrap();
        assert_eq!(history.days.len(), 30);
        assert_eq!(history.total_reviews, 0);
        assert_eq!(history.studied_day_percent, 0.0);
        assert_eq!(history.average_per_studied_day, 0.0);

        let added = service.added_cards(TimeRange::All).unwrap();
        assert_eq!(added.days.len(), 1);
        assert_eq!(added.average_per_day, 0.0);
    }

    #[test]
    fn today_progress_counts_reviewed_and_pending() {
        let (service, db, deck_id) = service();
        let due = add_card(&db, deck_id, "review");
        add_card(&db, deck_id, "new");
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "UPDATE card SET due_at = '2020-01-01T00:00:00+00:00' WHERE id = ?1",
                [due],
            )
            .unwrap();
        }
        log_review(&db, deck_id, due, &card_repo::today_local());

        let today = service.today_progress().unwrap();
        assert_eq!(today.reviewed_cards, 1);
        assert_eq!(today.new_remaining, 1);
        assert_eq!(today.due_remaining, 1);
        assert_eq!(today.pending_cards, 2);
        assert_eq!(today.total_cards, 2);
        assert_eq!(today.completion_percent, 33.3);
    }

    #[test]
    fn review_history_fills_missing_days_and_averages() {
        let (service, db, deck_id) = service();
        let card_id = add_card(&db, deck_id, "review");
        let today = Local::now().date_naive();
        let yesterday = (today - Duration::days(1)).to_string();
        let today_text = today.to_string();
        log_review(&db, deck_id, card_id, &yesterday);
        log_review(&db, deck_id, card_id, &today_text);
        log_review(&db, deck_id, card_id, &today_text);

        let history = service.review_history(TimeRange::LastMonth).unwrap();
        assert_eq!(history.days.len(), 30);
        assert_eq!(history.days[0].days_ago, -29);
        assert_eq!(history.days[29].days_ago, 0);
        assert_eq!(history.days[0].day, (today - Duration::days(29)).to_string());
        assert_eq!(history.total_reviews, 3);
        assert_eq!(history.studied_days, 2);
        assert_eq!(history.elapsed_days, 30);
        assert_eq!(history.studied_day_percent, 6.7);
        assert_eq!(history.average_per_elapsed_day, 0.1);
        assert_eq!(history.average_per_studied_day, 1.5);

        // 全部时间以首个记录日为起点。
        let all = service.review_history(TimeRange::All).unwrap();
        assert_eq!(all.days.len(), 2);
        assert_eq!(all.total_reviews, 3);
        assert_eq!(all.average_per_elapsed_day, 1.5);
    }

    #[test]
    fn added_cards_trends_from_first_card() {
        let (service, db, deck_id) = service();
        add_card(&db, deck_id, "new");

        let added = service.added_cards(TimeRange::LastYear).unwrap();
        assert_eq!(added.days.len(), 365);
        assert_eq!(added.total, 1);
        assert_eq!(added.days[364].days_ago, 0);
        assert_eq!(added.average_per_day, 0.0);

        let all = service.added_cards(TimeRange::All).unwrap();
        assert_eq!(all.days.len(), 1);
        assert_eq!(all.total, 1);
        assert_eq!(all.average_per_day, 1.0);
    }
}