# Sstr: a stack allocated utf-8 string

This stack allocated string implementation attempts to balance efficiency and ergonomics.

Stack String only works on the nightly versions of the Rust compiler as it relies on generic constant expressions
for buffer concatenation.

# Flags

Stack String integrates with serde for serialization and serialization in various formats.
The serde dependency is optional, therefore, you must add `serde` as a feature to the sstr crate in your cargo.toml.

```
sstr = { version = "0.1.0", features = ["serde"] }
```
