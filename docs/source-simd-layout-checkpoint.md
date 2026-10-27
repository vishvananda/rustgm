# Source SIMD layout checkpoint

This bounded PA41 checkpoint adds nominal `repr(simd)` layout and the C128
object boundary. It does not complete PA41, stdarch, or target-feature execution.
The source pin is Rust 1.98.0, commit
88d9e12ae178fab0fb5cc050a94da85685d449ea; see NOTICE and the authored reducers
in `pa41/tests/spec/source-simd/`.

`Representation::Simd` keeps the genuine one-array struct field at offset zero.
The shared `Definitions::simd_shape` query reads existing prepared fields and
returns NotSimd, Pending, or the actual lane/count. It performs no substitution,
retains no cache, and grants no feature, CPU, callable, or imported permission.
Generic declaration checking remains separate from demanded lane storage.
Primitive scalar lanes and scalar Rust/transparent product wrappers are covered;
broader scalar enum/union lane monomorphizations remain unsupported.

The pinned repr gate uses the whole attribute span. Captured meta/hint forms
inside the actual allowed macro wrapper accept; the initial contrary predictions
remain retained. Cfg-disabled repr does not demand the feature. `simd_ffi`
applies to direct SIMD types in foreign-block declarations. Extern-C definitions
and nested repr(C) wrappers do not acquire a stronger gate.

Rust arguments and returns use the existing indirect object path. Aligned C128
vectors use the existing CPPGM host-object SSE/SSEUP capability. Larger vectors
reject before the generic C memory fallback, including inside larger wrappers.
No backend, TypeKind, layout Shape, Body, Report, or persistent owner field was
added. SIMD records explicitly reject old metadata export; no new wire tag or
imported permission was introduced. Unadjusted/LLVM intrinsic declarations,
SIMD arithmetic and typed target-feature safety remain separate required work.

Qualification is bound to compiler commit 0c934fb4a706b0fc8cacefe3faa5398c90089bca:

- Six exact private controls, private strict Clippy, format and global audit:
  receipt `1793116733692734182`, 113.94 seconds.
- Same-source production/CLI strict Clippy and ten lexer controls, format and
  global audit: receipt `1793116851099885112`, 179.78 seconds.
- One authenticated no-build existing composite-class control passes. It proves
  ordinary flat scalar fields retain no composite cache and warm plans retain
  their prior accounting. Plans/Passing/Object/Plan measure 88/16/16/40 bytes.
- The actual no_core F/D/I provider reaches checked bodies and LowIR, with a
  typed private Rust call using two pointers and void result, and C SSE/SSEUP.
  Candidate native execution is pending the integrator's combined CLI.

Actual retained enclosing widths are Repr20, Definition36, Definitions264,
Layout48, Layouts152, TypeKind24, Body448 and Report3040 bytes. No predecessor
observation is invented for a width lacking one. For 1/8/32/128 distinct SIMD
nominals, ABI work is 12/96/384/1536 and the existing logical retained-byte
counter is 286/1728/6672/26448. These are accounting values, not allocator
measurements. Layout work is 8/43/163/643; actual layout-record capacities are
4/16/64/256, and offset/variant capacities are 4/8/32/128. Warm ABI queries do
not grow retained accounting.

Independent Rustc observations retain every source, hypothesis, warning and raw
stream. The C observer sees
`1.25 -2.00 3.50 4.00 5.50 -6.25 7 -8 3.50` followed by LF in both editions.
A separately demanded generic scalar wrapper/bool layout returns 4/4; a product
lane rejects. Three f32 lanes have size/alignment16/16, packed3xf32 has12/4,
and an explicit align8 SIMD remains16/16. Earlier typing/style/spelling-test
failures and the prior five-pass checkpoint remain historical evidence.

The immutable handoff, 2,942 Git-verified source fingerprints, raw index and
frozen native runner/plan are under
`obj/parallel/pa41/source-simd-design-v1/qualified-v1/`. Native must lower the
same source through the real CLI and locked object bridge; hand-authored LowIR
is not a substitute. Typed target-feature rows follow on the accepted combined
base, preserving distinct language call safety and ASM effective features.

## Combined integration and native copy qualification

Normal merge `41258fdc` integrates this source with the typed assembly compiler.
Twenty-three combined private controls pass in250.90s. Three separately repaired
CI guards pass in124.48s: the body layout mirror now includes the existing
assembly payload; marker ownership and duplicate attributes have separate
negative controls; feature-request checks distinguish real inline assembly from
global assembly and ordinary same-named macros. These are test repairs.
Production/CLI strict Clippy, formatting, architecture, lexer10, feature-request5
and all120 PA8 text cases pass in38.23s, receipt1793119857069645726. The combined
Body is456 bytes, including the separately introduced8-byte assembly payload.

Immutable combined CLI SHA-256 is
`1b5d6518ee7aa179a640f6461b1928e47a7a3fb99c2cf75acae42300bced2e04`.
An actual native copy/ABI consumer matches pinned Rust through the locked CPPGM
object backend: `1.25 -2.00 3.50 4.00 5.50 -6.25 7 -8 3.50\n`. All nine commands
pass in6.06s. The complete authored Rust provider prefix is unchanged; an
explicit entry calls the original C observer through `observer-bridge.c`. Rustc
uses the same Rust input as a library object with `observer-entry.c`; RustGM
uses its genuine executable entry shim. No source bodies or LowIR are replaced.
Evidence is under
`obj/parallel/lib045-root-implementation-v1/simd-integration/native-probe/`;
results SHA-256 is
`3eca7cced1d2c2e1fa0080d7abb3480f4ff1aa01ed90130a5c9dc9d39778b63f`.

The earlier runner's missing-entry and duplicate-entry failures remain recorded.
The unchanged library-mode source still rejects unsupported source stability
metadata; SIMD record transport also remains explicitly unsupported. Thus this
is native copy/ABI qualification with an explicit entry, not a complete source
library artifact, assembly execution or whole stdarch build. Full CI40537834822
failed the three stale guards above before course/native checks; the focused
repairs do not retroactively turn that run into a pass.
