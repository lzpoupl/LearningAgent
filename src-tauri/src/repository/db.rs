use std::path::Path;

use rusqlite::Connection;

include!(concat!(env!("OUT_DIR"), "\\migrations.rs"));

pub fn open<P: AsRef<Path>>(path: P) -> rusqlite::Result<Connection> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", true)?;
    migrate(&mut conn)?;
    Ok(conn)
}

pub fn migrate(conn: &mut Connection) -> rusqlite::Result<()> {

    for sql in MIGRATIONS.iter() {
        conn.execute_batch(sql)?;
    }

    Ok(())
}
