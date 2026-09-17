-- Anki 卡片统计：复习历史表。
-- 本文件只在 user_version 从 2 推进到 3 时执行一次，无需幂等。

-- 复习历史：每次作答一条，是复习图表与「今日已完成」的唯一数据源。
CREATE TABLE review_log (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id       INTEGER NOT NULL,
    deck_id       INTEGER NOT NULL,           -- 记录作答时所在牌组，不设外键以保留历史
    grade         TEXT    NOT NULL
                  CHECK (grade IN ('again', 'hard', 'good', 'easy')),
    prev_state    TEXT    NOT NULL
                  CHECK (prev_state IN ('new', 'learning', 'review', 'relearning')),
    next_state    TEXT    NOT NULL
                  CHECK (next_state IN ('new', 'learning', 'review', 'relearning')),
    duration_ms   INTEGER NOT NULL DEFAULT 0, -- 预留：作答耗时
    reviewed_at   TEXT    NOT NULL,           -- RFC3339（UTC），精确时间
    review_date   TEXT    NOT NULL,           -- 本地日期 YYYY-MM-DD，按天聚合用
    FOREIGN KEY (card_id) REFERENCES card (id) ON DELETE CASCADE
);

CREATE INDEX idx_review_log_date ON review_log (review_date);
CREATE INDEX idx_review_log_card ON review_log (card_id);