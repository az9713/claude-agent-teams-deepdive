use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS scan_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS file_fingerprints (
            path TEXT PRIMARY KEY,
            mtime INTEGER NOT NULL,
            size INTEGER NOT NULL,
            hash TEXT
        );

        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path TEXT NOT NULL,
            line INTEGER NOT NULL,
            col INTEGER NOT NULL,
            tag TEXT NOT NULL,
            message TEXT NOT NULL,
            author TEXT,
            issue TEXT,
            priority TEXT,
            context_line TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_todos_file ON todos(file_path);
    ",
    )?;
    Ok(())
}
