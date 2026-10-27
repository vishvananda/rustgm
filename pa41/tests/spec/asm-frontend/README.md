# Source inline assembly reducer

`stream-ps.rs` preserves the complete `_mm_stream_ps` declaration/body and
64-bit `vps` macro from pinned Rust88d9 stdarch x86/sse.rs and x86/macros.rs.
The surrounding source-owned `core::arch::asm` declaration uses its actual
builtin form. The minimal vector declaration retains genuine `repr(simd)` and
four f32 lanes; it supplies no substitute scalar layout or Copy/ABI proof.

The first private expansion control checks typed templates/operands only. Complete
SIMD semantics, target-feature and asm body metadata, and native non-temporal
store emission remain separate required contracts. The locked backend cannot
emit this operation. No PA41 activation or whole-core acceptance is claimed.

The [vector join checkpoint](../../../../docs/source-asm-vector-join.md) also
checks this original function through genuine SIMD typing, actual target features,
ordinary original/instance proof and explicit demanded backend rejection. Native
non-temporal emission and assembly metadata transport remain required.
