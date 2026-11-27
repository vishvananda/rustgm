# Fixed integer registers and register views

The outcome manifest declares expected native results before any oracle run.
The original `CpuidResult` declaration and the complete tail of `__cpuid_count`
(from its four output declarations through the return) are inserted verbatim.
Only the wrapper is authored: a generic parameter forces fresh-consumer code.
The omitted prefix is the SGX-only panic branch, inactive on the course target.
The full source remains in `../../upstream/rust/cpuid.rs`, pinned to Rust 1.98.0
88d9e12ae178fab0fb5cc050a94da85685d449ea; see `third_party/sources.json` and
`shared/RUST-NOTICE` for licensing and checksums.

An independent C CPUID query checks every output for three leaf/subleaf pairs.
A full-width RBX canary checks that the original 64-bit scratch view preserves
the caller register despite its 32-bit output type. A signed inlateout and an
ordinary scalar control have separately declared values. Wrong argument types
and v13 export retain semantic/profile rejections. No backend opcode is replaced.

Discarded-inout controls are accepted by pinned Rust and rejected by both source
profiles. Their typed write flag remains present; neither the producer nor the
native lowerer may silently turn an unsupported discarded result into an input.
A split fixed input/late output pair shares EAX legally, in both operand orders;
its two native controls return 32 and 42. This tests the allowed alias branch
alongside forged overlapping-input rejection.

The original SSE2 `_mm_stream_si32` function, including its target feature,
macro template and `:e` view, is inserted verbatim from pinned `sse2.rs`. The full
stream runtime case retains expected success and four exact store/sentinel
checks in `runtime_client.rs` and `runtime_observe.c`. Pinned CPPGM does not
implement `movnti`, so that execution remains unqualified; no opcode is changed.
The qualified native controls use `mov` with the 32-bit view on five u64
inouts, checking each low-half pattern and requiring a zero upper half.
