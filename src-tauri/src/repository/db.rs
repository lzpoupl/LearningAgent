use std::path::Path;

use rusqlite::Connection;

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "anki_schema",
    sql: r#"
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
            created_at  TEXT    NOT NULL,
            updated_at  TEXT    NOT NULL,
            FOREIGN KEY (deck_id) REFERENCES deck (id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_card_deck ON card (deck_id);
        CREATE INDEX IF NOT EXISTS idx_card_state ON card (state);
        CREATE INDEX IF NOT EXISTS idx_card_due ON card (due_at);
    "#,
}, Migration {
    version: 2,
    name: "card_scheduling",
    sql: r#"
        ALTER TABLE card ADD COLUMN algorithm TEXT NOT NULL DEFAULT 'sm2';
        ALTER TABLE card ADD COLUMN scheduler_state TEXT;
    "#,
}];

pub fn open<P: AsRef<Path>>(path: P) -> rusqlite::Result<Connection> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", true)?;
    migrate(&mut conn)?;
    Ok(conn)
}

pub fn migrate(conn: &mut Connection) -> rusqlite::Result<()> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    for migration in MIGRATIONS.iter().filter(|m| m.version > current) {
        let tx = conn.transaction()?;
        tx.execute_batch(migration.sql)?;
        tx.pragma_update(None, "user_version", migration.version)?;
        tx.commit()?;
    }

    Ok(())
}
