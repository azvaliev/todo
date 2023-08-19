# todo!

`todo!` is an app for managing todo lists.

It keeps a running list of your incomplete todos, and shows those which have been completed today

## Getting Started

**Requirements**
- [rust](https://www.rust-lang.org/learn/get-started)
- [sqlx cli](https://crates.io/crates/sqlx-cli)

### Environment

Copy `.env.development` to `.env`.
`DATABASE_URL` is for sqlx compile time query checking

### Database

Migrations are run & created via [sqlx cli](https://crates.io/crates/sqlx-cli)

#### Initialize and migrate database

```bash
touch db.sqlite
sqlx migrate run
```

### Running app

```bash
cargo run
```
