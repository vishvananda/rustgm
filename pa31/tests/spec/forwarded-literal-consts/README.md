# Original core swap_prefix reducer

`swap-prefix.rs` retains the exact macro definition/invocation slice from
Rust revision 88d9e12ae178fab0fb5cc050a94da85685d449ea,
`library/core/src/ptr/mod.rs`, around line 1491. Attribution is retained in the
source and repository NOTICE. The wrapper supplies genuine pointer/array types;
it does not implement or qualify core's swap operation.

Pinned Rust 1.98 accepts the complete reducer in editions 2021 and 2024. The
maintainer control sends it through the actual crate expansion parser and checks
three literal captures and actual generic applications. The macro transcription
view alone does not parse the emitted function body and is not this acceptance
proof. Original core/allocation/borrow/native behavior remains required.
