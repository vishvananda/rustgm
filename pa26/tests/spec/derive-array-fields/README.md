# Typed constant children in derived field types

These inputs freeze ordinary Rust predictions before candidate execution. Literal,
named and compound array counts retain their original typed AST and literal identity.
The incompatible count and non-Copy payload inputs remain language negatives.

The compiler copies only the selected field subtree. Inline stored integer, boolean,
character and byte values receive fresh literal-arena IDs. Deferred token literals
remain deferred and retain their pending-materialization IDs. Stored values backed
by a decoded byte pool remain an explicit unsupported copy boundary. Empty pure
constant-expression blocks, unary/binary operations and casts preserve their children;
this does not admit statement-bearing constant blocks or arbitrary item subtrees.

The contract and pinned Rust observations precede candidate execution. The private
controls cover failed growth, invalid IDs, stale decoded handles, deferred transfer,
8/32/128 selected fields, unrelated-owner noise and actual pending-splice accounting.
