use rusqlite::{Connection, Result as SqlResult, Transaction};
use thiserror::Error;

/// Migrate the database to the latest version.
/// Applies database migrations in order, starting from the current user_version.
///
/// This function is idempotent - running it multiple times will only apply
/// migrations that haven't been run yet. Each migration is executed in its own
/// transaction and the database's user_version pragma is updated to track
/// progress.
///
/// Will roll back to last good version on error.
pub fn migrate(
    conn: &mut Connection,
    migrations: &[fn(&Transaction) -> SqlResult<()>],
) -> Result<usize, MigrationError> {
    let current_version = get_user_version(conn)?;

    let mut last_successful_version = current_version;

    for (index, migration) in migrations.iter().enumerate() {
        let migration_version = index + 1;

        if migration_version > last_successful_version {
            let tx = conn.transaction()?;
            match migration(&tx) {
                Ok(()) => {
                    set_user_version(&tx, migration_version)?;
                    tx.commit()?;
                    last_successful_version = migration_version;
                }
                Err(error) => {
                    tx.rollback()?;
                    return Err(MigrationError {
                        version: last_successful_version,
                        error: error,
                    });
                }
            }
        }
    }

    Ok(last_successful_version)
}

/// Represents an error that occurred during database migrations.
#[derive(Debug, Error)]
#[error("Error performing migration. Rolled back to version {version}. Error: {error}")]
pub struct MigrationError {
    /// Last good version
    pub version: usize,
    /// Error that stopped completion of migrations
    pub error: rusqlite::Error,
}

impl From<rusqlite::Error> for MigrationError {
    fn from(error: rusqlite::Error) -> Self {
        MigrationError { version: 0, error }
    }
}

/// Returns the current user_version of the database.
pub fn get_user_version(conn: &Connection) -> SqlResult<usize> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    Ok(version as usize)
}

fn set_user_version(tx: &Transaction, version: usize) -> SqlResult<()> {
    tx.pragma_update(None, "user_version", version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn create_test_db() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    fn migration1(tx: &Transaction) -> SqlResult<()> {
        tx.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)", [])?;
        Ok(())
    }

    fn migration2(tx: &Transaction) -> SqlResult<()> {
        tx.execute("ALTER TABLE test ADD COLUMN name TEXT", [])?;
        Ok(())
    }

    fn failing_migration(_tx: &Transaction) -> SqlResult<()> {
        Err(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
            Some("Test error".to_string()),
        ))
    }

    #[test]
    fn test_empty_migrations() {
        let mut conn = create_test_db();
        let migrations: &[fn(&Transaction) -> SqlResult<()>] = &[];

        let result = migrate(&mut conn, migrations).unwrap();
        assert_eq!(result, 0);
        assert_eq!(get_user_version(&conn).unwrap(), 0);
    }

    #[test]
    fn test_single_migration() {
        let mut conn = create_test_db();

        let migrations: &[fn(&Transaction) -> SqlResult<()>] = &[migration1];
        let result = migrate(&mut conn, migrations).unwrap();

        assert_eq!(result, 1);
        assert_eq!(get_user_version(&conn).unwrap(), 1);
    }

    #[test]
    fn test_multiple_migrations() {
        let mut conn = create_test_db();

        let migrations: &[fn(&Transaction) -> SqlResult<()>] = &[migration1, migration2];
        let result = migrate(&mut conn, migrations).unwrap();

        assert_eq!(result, 2);
        assert_eq!(get_user_version(&conn).unwrap(), 2);
    }

    #[test]
    fn test_migration_failure_rollback() {
        let mut conn = create_test_db();

        let migrations: &[fn(&Transaction) -> SqlResult<()>] = &[migration1, failing_migration];
        let error =
            migrate(&mut conn, migrations).expect_err("Migrate should have returned an error");

        assert_eq!(error.version, 1);
        assert_eq!(get_user_version(&conn).unwrap(), 1);
    }

    #[test]
    fn test_idempotent_migrations() {
        let mut conn = create_test_db();

        let migrations: &[fn(&Transaction) -> SqlResult<()>] = &[migration1];

        let result1 = migrate(&mut conn, migrations).unwrap();
        assert_eq!(result1, 1);

        let result2 = migrate(&mut conn, migrations).unwrap();
        assert_eq!(result2, 1);
        assert_eq!(get_user_version(&conn).unwrap(), 1);
    }
}
