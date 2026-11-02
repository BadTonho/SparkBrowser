use rusqlite::{Connection, OptionalExtension};

#[derive(Clone, Copy)]
pub enum RunCheckpoint {
    Restored,
    CleanExit,
}
pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "session-recovery-run-state-v1", |conn| {
        conn.execute_batch("CREATE TABLE browser_run_state(singleton INTEGER PRIMARY KEY CHECK(singleton=1), clean_exit INTEGER NOT NULL CHECK(clean_exit IN(0,1)), restoration_confirmed INTEGER NOT NULL CHECK(restoration_confirmed IN(0,1)))").map_err(|e|e.to_string())
    })
}
/// Call only after acquiring the OS profile lock, before reading or changing the session.
pub fn begin_run(conn: &Connection) -> Result<bool, String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let previous: Option<bool> = tx
        .query_row(
            "SELECT clean_exit FROM browser_run_state WHERE singleton=1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO browser_run_state VALUES(1,0,0) ON CONFLICT(singleton) DO UPDATE SET clean_exit=0,restoration_confirmed=0", []).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(previous == Some(false))
}
/// Participates in the transaction that saves the complete session and split layouts.
pub fn checkpoint(conn: &Connection, checkpoint: RunCheckpoint) -> Result<(), String> {
    let sql = match checkpoint {
        RunCheckpoint::Restored => "UPDATE browser_run_state SET restoration_confirmed=1 WHERE singleton=1 AND clean_exit=0",
        RunCheckpoint::CleanExit => "UPDATE browser_run_state SET clean_exit=1 WHERE singleton=1 AND restoration_confirmed=1",
    };
    if conn.execute(sql, []).map_err(|e| e.to_string())? != 1 {
        return Err("recovery.invalid_run_state".into());
    }
    Ok(())
}
/// A Windows installer can return after its pre-exit hook if launching it fails.
pub fn resume_run(conn: &Connection) -> Result<(), String> {
    if conn
        .execute(
            "UPDATE browser_run_state SET clean_exit=0 WHERE singleton=1",
            [],
        )
        .map_err(|e| e.to_string())?
        != 1
    {
        return Err("recovery.invalid_run_state".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_launch_crash_clean_exit_and_cancelled_installer_have_distinct_markers() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        assert!(!begin_run(&conn).unwrap());
        assert!(checkpoint(&conn, RunCheckpoint::CleanExit).is_err());
        assert!(begin_run(&conn).unwrap());
        checkpoint(&conn, RunCheckpoint::Restored).unwrap();
        checkpoint(&conn, RunCheckpoint::CleanExit).unwrap();
        assert!(!begin_run(&conn).unwrap());
        checkpoint(&conn, RunCheckpoint::Restored).unwrap();
        checkpoint(&conn, RunCheckpoint::CleanExit).unwrap();
        resume_run(&conn).unwrap();
        assert!(begin_run(&conn).unwrap());
    }
}
