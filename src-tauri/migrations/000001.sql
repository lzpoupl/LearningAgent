-- 基础 Anki 表结构：牌组与卡片。
CREATE TABLE IF NOT EXISTS deck (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id   INTEGER NOT NULL DEFAULT 0,
    name        TEXT    NOT NULL,
    created_at  TEXT    NOT NULL,
    UNIQUE (parent_id, name)
);

CREATE INDEX IF NOT EXISTS idx_deck_parent ON deck (parent_id);

CREATE TABLE IF NOT EXISTS card (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    deck_id     INTEGER NOT NULL,
    front       TEXT    NOT NULL,
    back        TEXT    NOT NULL,
    state       TEXT    NOT NULL DEFAULT 'new'
                CHECK (state IN ('new', 'learning', 'review', 'relearning')),
    due_at      TEXT,
    algorithm        TEXT NOT NULL DEFAULT 'sm2',
    scheduler_state  TEXT,
    created_at  TEXT    NOT NULL,
    updated_at  TEXT    NOT NULL,
    FOREIGN KEY (deck_id) REFERENCES deck (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_card_deck ON card (deck_id);
CREATE INDEX IF NOT EXISTS idx_card_state ON card (state);
CREATE INDEX IF NOT EXISTS idx_card_due ON card (due_at);
