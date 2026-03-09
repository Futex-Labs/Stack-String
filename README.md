# Sstr: a stack allocated utf-8 string

This stack allocated string implementation attempts to balance efficiency and ergonomics.

Stack String only works on the nightly versions of the Rust compiler as it relies on generic constant expressions
for buffer concatenation.
