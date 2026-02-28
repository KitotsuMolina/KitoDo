use rusqlite::Connection;

const MIGRATIONS: [&str; 6] = [
    include_str!("../../migrations/001_init.sql"),
    include_str!("../../migrations/002_indexes.sql"),
    include_str!("../../migrations/003_qol_sort_recurrence.sql"),
    include_str!("../../migrations/004_qol_sort_recurrence_indexes.sql"),
    include_str!("../../migrations/005_rebuild_tasks_fk.sql"),
    include_str!("../../migrations/006_indexes_hardening.sql"),
];

pub fn pending_versions(conn: &Connection) -> rusqlite::Result<Vec<i64>> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS _migrations (
          version INTEGER PRIMARY KEY,
          applied_at TEXT NOT NULL
        );
        ",
    )?;

    let mut pending = Vec::new();
    for (idx, _) in MIGRATIONS.iter().enumerate() {
        let version = (idx + 1) as i64;
        let already_applied = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM _migrations WHERE version = ?1)",
            [version],
            |row| row.get::<_, i64>(0),
        )?;
        if already_applied == 0 {
            pending.push(version);
        }
    }
    Ok(pending)
}

pub fn run(conn: &Connection) -> rusqlite::Result<()> {
    let pending = pending_versions(conn)?;
    if pending.is_empty() {
        return Ok(());
    }

    for version in pending {
        let sql = MIGRATIONS[(version - 1) as usize];
        conn.execute_batch(sql)?;
        conn.execute(
            "INSERT INTO _migrations(version, applied_at) VALUES(?1, datetime('now'))",
            [version],
        )?;
    }

    Ok(())
}
