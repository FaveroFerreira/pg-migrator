use std::path::Path;

#[cfg(feature = "postgres")]
use postgres::Client;
#[cfg(feature = "tokio-postgres")]
use tokio_postgres::Client;

#[cfg(any(feature = "postgres", feature = "tokio-postgres"))]
use crate::error::BoxError;

mod error;
mod fs;
mod migration;

const DEFAULT_MIGRATIONS_TABLE: &str = "__migrations";

pub struct PostgresMigrator<P: AsRef<Path>> {
    migrations_path: P,
    migrations_table: String,
    ignore_missing_migrations: bool,
}

impl<P: AsRef<Path>> PostgresMigrator<P> {
    pub fn new(migrations_path: P) -> Self {
        Self {
            migrations_path,
            migrations_table: DEFAULT_MIGRATIONS_TABLE.to_string(),
            ignore_missing_migrations: false,
        }
    }

    pub fn migrations_table<T: Into<String>>(mut self, migrations_table: T) -> Self {
        self.migrations_table = migrations_table.into();
        self
    }

    pub fn ignore_missing_migrations(mut self, ignore_missing: bool) -> Self {
        self.ignore_missing_migrations = ignore_missing;
        self
    }

    #[cfg(feature = "postgres")]
    pub fn migrate(&self, pg: &mut Client) -> Result<(), BoxError> {
        let migrations = fs::load_migrations(self.migrations_path.as_ref())?;

        migration::ensure_migrations_table_exists(pg, &self.migrations_table)?;

        let applied = migration::validate_applied(pg, &self.migrations_table, &migrations)?;

        for migration in migrations {
            if applied.contains(&migration.version) {
                continue;
            }

            migration::apply(pg, &self.migrations_table, &migration)?;
        }

        Ok(())
    }

    #[cfg(feature = "tokio-postgres")]
    pub async fn migrate(&self, pg: &mut Client) -> Result<(), BoxError> {
        let migrations = fs::load_migrations(self.migrations_path.as_ref())?;

        migration::ensure_migrations_table_exists(pg, &self.migrations_table).await?;

        let applied = migration::validate_applied(pg, &self.migrations_table, &migrations).await?;

        for migration in migrations {
            if applied.contains(&migration.version) {
                continue;
            }

            migration::apply(pg, &self.migrations_table, &migration).await?;
        }

        Ok(())
    }
}
