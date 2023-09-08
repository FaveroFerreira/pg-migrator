# Pg-Migrator

![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)
[![Docs.rs](https://docs.rs/pg-migrator/badge.svg)](https://docs.rs/pg-migrator)

Pg-Migrator is a simple, macro-free, crate for running migrations on Postgres databases.


## Usage

### Quickstart

For `rust-postgres` use:

```toml
[dependencies]
pg-migrator = { version = "0.1.0", features = ["postgres"] }
```

For `tokio-postgres` use:

```toml
[dependencies]
pg-migrator = { version = "0.1.0", features = ["tokio-postgres"] }
```

Then, create your Postgres/Tokio Postgres connection as always and run the migrations:


```rust
use postgres::{Client, NoTls, Error};

fn main() {
    let mut client = Client::connect("postgresql://postgres:postgres@localhost/library", NoTls).unwrap();

    PostgresMigrator::new("./migrations")
        .migrate(conn)
        .unwrap();
}
```