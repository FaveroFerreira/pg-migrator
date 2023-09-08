use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use chrono::Utc;
#[cfg(feature = "postgres")]
use postgres::Client;
#[cfg(feature = "tokio-postgres")]
use tokio_postgres::Client;

use crate::error::BoxError;
use crate::fs::MigrationFile;

const CREATE_MIGRATIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS {{migrations_table}} (
    version VARCHAR(255) PRIMARY KEY,
    description VARCHAR(255) NOT NULL,
    sql TEXT NOT NULL,
    applied_at TIMESTAMP WITH TIME ZONE NOT NULL,
    checksum VARCHAR(255) NOT NULL
)
"#;

#[cfg(feature = "postgres")]
pub(crate) fn ensure_migrations_table_exists(
    pg: &mut Client,
    migrations_table: &str,
) -> Result<(), BoxError> {
    let query = CREATE_MIGRATIONS_TABLE.replace("{{migrations_table}}", migrations_table);

    let _ = pg.execute(&query, &[])?;

    Ok(())
}

#[cfg(feature = "tokio-postgres")]
pub(crate) async fn ensure_migrations_table_exists(
    pg: &mut Client,
    migrations_table: &str,
) -> Result<(), BoxError> {
    let query = CREATE_MIGRATIONS_TABLE.replace("{{migrations_table}}", migrations_table);

    let _ = pg
        .execute(&query, &[&migrations_table])
        .await?;
    Ok(())
}

const SELECT_MIGRATIONS: &str = "SELECT * FROM {{migrations_table}}";

#[cfg(feature = "postgres")]
pub(crate) fn validate_applied(
    pg: &mut Client,
    migrations_table: &str,
    fs_migrations: &[MigrationFile],
) -> Result<Vec<String>, BoxError> {
    let query = SELECT_MIGRATIONS.replace("{{migrations_table}}", migrations_table);

    let applied_migrations = pg.query(&query, &[])?;

    let mut applied_versions = Vec::with_capacity(applied_migrations.len());

    for applied_migration in applied_migrations {
        let version = applied_migration.try_get::<_, String>("version")?;
        let checksum = applied_migration.try_get::<_, String>("checksum")?;

        let fs_migration = fs_migrations
            .iter()
            .find(|m| m.version == version)
            .ok_or_else(|| format!("migration {} not found", version))?;

        if fs_migration.checksum != checksum {
            return Err(format!("checksum mismatch for migration {}", fs_migration.version).into());
        }

        applied_versions.push(version);
    }

    Ok(applied_versions)
}

#[cfg(feature = "tokio-postgres")]
pub(crate) async fn validate_applied(
    pg: &mut Client,
    migrations_table: &str,
    fs_migrations: &[MigrationFile],
) -> Result<Vec<String>, BoxError> {
    let query = SELECT_MIGRATIONS.replace("{{migrations_table}}", migrations_table);

    let applied_migrations = pg.query(&query, &[]).await?;

    let mut applied_versions = Vec::with_capacity(applied_migrations.len());

    for applied_migration in applied_migrations {
        let version = applied_migration.try_get::<_, String>("version")?;
        let checksum = applied_migration.try_get::<_, String>("checksum")?;

        let fs_migration = fs_migrations
            .iter()
            .find(|m| m.version == version)
            .ok_or_else(|| format!("migration {} not found", version))?;

        if fs_migration.checksum != checksum {
            return Err(format!("checksum mismatch for migration {}", fs_migration.version).into());
        }

        applied_versions.push(version);
    }

    Ok(applied_versions)
}

const INSERT_MIGRATION: &str = r#"
    INSERT INTO {{migrations_table}} (version, description, sql, applied_at, checksum)
    VALUES ($1, $2, $3, $4, $5)
"#;

#[cfg(feature = "postgres")]
pub(crate) fn apply(
    pg: &mut Client,
    migrations_table: &str,
    migration: &MigrationFile,
) -> Result<(), BoxError> {
    let mut hasher = DefaultHasher::new();
    hasher.write(migration.sql.as_bytes());

    let checksum = format!("{:x}", hasher.finish());

    let applied_at = Utc::now();

    let mut tx = pg.transaction()?;

    tx.batch_execute(&migration.sql)?;

    let query = INSERT_MIGRATION.replace("{{migrations_table}}", migrations_table);

    tx.execute(
        &query,
        &[
            &migration.version,
            &migration.description,
            &migration.sql,
            &applied_at,
            &checksum,
        ],
    )?;

    tx.commit()?;

    Ok(())
}

#[cfg(feature = "tokio-postgres")]
pub(crate) async fn apply(
    pg: &mut Client,
    migrations_table: &str,
    migration: &MigrationFile,
) -> Result<(), BoxError> {
    let mut hasher = DefaultHasher::new();
    hasher.write(migration.sql.as_bytes());

    let checksum = format!("{:x}", hasher.finish());

    let applied_at = Utc::now();

    let tx = pg.transaction().await?;

    tx.batch_execute(&migration.sql).await?;

    let query = INSERT_MIGRATION.replace("{{migrations_table}}", migrations_table);

    tx.execute(
        &query,
        &[
            &migration.version,
            &migration.description,
            &migration.sql,
            &applied_at,
            &checksum,
        ],
    )
    .await?;

    tx.commit().await?;

    Ok(())
}
