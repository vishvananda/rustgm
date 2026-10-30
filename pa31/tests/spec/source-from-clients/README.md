# Imported source conversions

These required inputs consume the unchanged `source-from/nested-into-dot.rs`
provider in a fresh process and compiler store. `default.rs` must execute with
32; `explicit.rs` must execute with 64. Both must first compile with pinned Rust
1.98.0 using an actual separately built provider, then compile and execute with
RustGM's version-nine metadata. Provider and client sources are never replaced
by synthetic declarations or precomputed values. The accompanying contract
freezes the expected outcomes and input hashes before execution.

This joins PA31's real From identity to PA29's generic body and lifetime
transport. It does not close either whole assignment or build original core.
