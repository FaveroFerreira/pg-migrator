use std::io;

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[cfg(feature = "postgres")]
    #[error("postgres error: {0}")]
    Postgres(#[from] postgres::Error),

    #[cfg(feature = "tokio-postgres")]
    #[error("postgres error: {0}")]
    Postgres(#[from] tokio_postgres::Error),

    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("migration error: {0}")]
    Migration(String),

    #[error("migration error: {0}")]
    MigrationParse(#[from] regex::Error),
}
