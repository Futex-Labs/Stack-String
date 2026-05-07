# Sstr: a stack allocated utf-8 string

Often we find ourselves allocating heap memory for strings that are never touched after their creation. In a vast
majority of these cases, a reasonable upper bound can be set and thus the size of the buffer can be known ahead of time. 
In other words, fixed size strings are well suited for the stack. This library was borne from the idea that a fixed 
size string implementation can have good ergonomics, compose well with other libraries, and make string usage more efficient.

# Usage 

Run the following command

```sh
cargo add sstr
```

or add the following line to your Cargo.toml

```toml
sstr = "0.3.0"
```

# Flags

All of the flags below can be used together, but please mind your dependency graph. No need to pull in unused code.

## Serde 

Stack String integrates with serde for serialization and serialization in various formats.
The serde dependency is optional, therefore, you must add `serde` as a feature to the sstr crate in your cargo.toml.

```toml
sstr = { version = "0.3.0", features = ["serde"] }
```

## SQLx

Stack String implements `sqlx::Encode`, `sqlx::Decode`, and `sqlx::Type` for all databases supported by SQLx.

### Postgres

```toml
sstr = { version = "0.3.0", features = ["sqlx-postgres"] }
```

### Sqlite

```toml
sstr = { version = "0.3.0", features = ["sqlx-sqlite"] }
```
### MySQL

```toml
sstr = { version = "0.3.0", features = ["sqlx-mysql"] }
```
