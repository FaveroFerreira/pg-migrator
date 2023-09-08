//! # File System module
//!
//! This module contains all the logic for interacting with the file system.
//!
//! We need to interact with the file system to load the SQL migrations and to load the SQL queries.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::Hasher;
use std::path::Path;

use regex::Regex;

use crate::error::BoxError;

pub(crate) struct MigrationFile {
    pub version: String,
    pub description: String,
    pub sql: String,
    pub checksum: String,
}

pub(crate) fn load_migrations<P: AsRef<Path>>(
    migrations_path: P,
) -> Result<Vec<MigrationFile>, BoxError> {
    // Regex for extracting the version and description from the filename
    // Example: V001__CREATE_USERS_TABLE.sql
    let re = Regex::new(r"^V(\d+)__(.+)\.sql$")?;

    let dir = fs::read_dir(migrations_path)?;
    let mut migration_files = Vec::new();

    for entry in dir {
        let entry = entry?;
        let filename = entry.file_name();
        let sql = fs::read_to_string(entry.path())?;

        let Some(captures) = re.captures(filename.to_str().unwrap()) else {
            continue;
        };

        let mut hasher = DefaultHasher::new();
        hasher.write(sql.as_bytes());

        let version = captures.get(1).unwrap().as_str();
        let description = captures.get(2).unwrap().as_str();
        let checksum = format!("{:x}", hasher.finish());

        migration_files.push(MigrationFile {
            version: version.to_string(),
            description: description.to_string(),
            sql,
            checksum,
        });
    }

    migration_files.sort_by(|a, b| a.version.cmp(&b.version));

    Ok(migration_files)
}
