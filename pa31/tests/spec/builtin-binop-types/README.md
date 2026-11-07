# Builtin binary operand types

Twelve no_core source-core libraries for rustc's builtin binary rule. When
neither operand is an unknown type and, under one shared reference, both are
integral or both floating (bitwise operators also two bools, comparisons any two
primitive scalars), the operand types are unified before an impl is selected.
Shifts relate only their result to the left operand. Each library has more than
one candidate impl, so trait selection alone is ambiguous.

The controls cover literal subtraction and multiplication in const arguments
and impl self-type lengths (core `array_impl_default` and portable-simd
`to_bytes` shapes), shift result and right-operand typing, floats, a reference
right operand, compound assignment, comparisons, two mismatched-width
opponents (E0308) and a user impl whose right operand is not numeric, which
still selects by trait. Outcomes were declared before the frozen rustc 1.98.0
oracle and before the implementation changed; both are recorded in
`outcomes.json`, with one fixture amendment made before its re-run.

The lang-item prefixes and the operator trait declarations are exact pinned
Rust core fragments with their attributes and comments, from the census
batch 2 controls and `spec/source-comparisons/equality/actual-equality.rs`.
They keep Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/binop_operand_types.rs` checks every case. See
`docs/parallel/census-batch3.md`.
