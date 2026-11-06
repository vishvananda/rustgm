# Nominal CoerceUnsized declarations and real consumers

The sixteen sources in `cases.tsv` and `predictions.json` were frozen before
candidate observation. Pinned Rust 1.98.0 independently matches every declared
outcome, including unchanged scalar prefixes, nested wrappers, PhantomData,
repr(C), packed structs and directed reference lifetimes. Distinct owners,
non-structs, missing recursive relations and zero/multiple changed fields reject.
The original `unsizing-roles/nominal-coerce-required.rs` remains an independent
positive goal consumer.

The declaration producer proves the actual changed field relation in the same
implementation environment. It supplies no value conversion or layout permission.
The complete original Coroutine result-coercion source remains unchanged under
`source-coroutines/full/result-coercion.rs`; actual wrapper conversion, metadata,
one-time value transfer, capture/Drop and native observers remain required.

The source role excerpts preserve attribution to the Rust Project Contributors,
MIT OR Apache-2.0, Rust commit 88d9e12ae178fab0fb5cc050a94da85685d449ea,
`library/core/src/{marker.rs,ops/unsize.rs,intrinsics/mod.rs}`. The nominal test
reducers are independently authored. This finite packet does not activate PA31.
