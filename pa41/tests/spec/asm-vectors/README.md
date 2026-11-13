# Assembly vector opponents

`predictions.json`, `stream-predictions.json` and `copy-predictions.json` inventory
all 32 inputs and freeze their hashes/outcomes before the independent metadata
observations. Original `_mm_stream_ps` and vps source derives from pinned Rust88d9
stdarch x86/sse.rs and x86/macros.rs; preserve NOTICE and the original reducer in
`../asm-frontend/stream-ps.rs`. The valid stream observer uses genuine core __m128.

The candidate consumes supported exact 128-bit numeric XMM values while retaining
actual Copy, initialization, place and safety obligations. Since the
[ymm operand checkpoint](../../../../docs/pa41-ymm-operands.md), `ymm-no-feature`
fails at the `avx` check, and the independently accepted `ymm-body-feature` types
and stops at the initial assembly profile. These controls provide no AVX execution, metadata or backend permission. PA41 is planned;
see [the checkpoint](../../../../docs/source-asm-vector-join.md).
