# LLVM declaration authority and scalar operations

The [source packet](../pa28/tests/spec/llvm-linkage/README.md) qualifies actual
`link_llvm_intrinsics` permission and reached scalar LLVM operations. The first
selected link_name on an actual foreign function or static requires the genuine
crate request or that item's macro-origin allowance. Captured caller items do
not inherit a forwarding macro's permission. Ignored attributes on Rust items,
cfg removal, ordinary unsafe contracts and constant-call restrictions retain
their independent meaning.

Reserved LLVM names are classified once in a bounded, paid cold symbol catalog.
An unknown unused declaration remains representable, but a reached unsupported
operation or storage use fails before ordinary C linkage. Typed unary shims
implement ctpop and bitreverse at8/16/32/64/128 bits and bswap at16/32/64/128 bits.
Alias normalization and equal-width signed/unsigned views preserve actual ABI
signatures. Logarithmic bit-group transforms use existing LowIR operations and
one cached shim per callable; the C++ backend is unchanged.

The real permission is bit64, so the crate-only feature set grows from one to
two u64 words (16 bytes, alignment8). Macro flags stay one byte, and ordinary
function/body records gain no feature field. No advisory request is used to
stand in for an operation capability. A pass-local foreign-owner index and one
macro allowance session avoid rescanning all declarations for each use.

Qualified evidence, with matching compiler/test source fingerprints:

- Private8, strict Clippy, formatting and architecture:240.37s; `obj/qualification/fast/1793776877633880888/receipt.json`, SHA-256 `60c59f5b6d937aeabbf76f0ee0452f937e55e676952da4c8a0348d3014fbc748`.
- Production9, strict Clippy, formatting, architecture, PA8 126/126 and PA10 81/81:184.98s; `obj/qualification/fast/1793777152645891662/receipt.json`, SHA-256 `ca05475824e0d79152a16af8467d3dc1170a7611483419932032f07d9a4a3687`.
- Native2, strict Clippy, formatting and architecture:56.65s; `obj/qualification/fast/1793777468608741610/receipt.json`, SHA-256 `d9381917d8a2b7ef4d8f4d6355e36295dac361e90bb49e5df2242488d610d5cf`. Unsigned, signed, mixed-sign and ordinary programs print1 through both RustGM/CPP and independent Rust; the preceding conditional-library native still prints7 9. The three LLVM programs exercise168 comparisons, including runtime C inputs.
- Independent oracle:86 source outcomes across Rust2021/2024 plus four native programs,2.82s; `obj/parallel/llvm-linkage-v1/reference-v3/receipt.json`, SHA-256 `86dc8f4d1417bed64c20c4c6ecdd073cea3d629a50e4c4f983dfd4651f531756`. Source predictions precede qualification; earlier versions retain unchanged cases.
- Joined CLI/configuration/harness/upstream/original-core qualification:33.03s; `obj/parallel/llvm-linkage-v1/qualification-v1/receipt.json`, SHA-256 `d1d0d0a7e429e47e9f291ab41b9ff5471bacc1171c8ae1af0c125f5138d8db7d`. All86 CLI outcomes agree, ordinary LowIR is byte-identical to the predecessor and all fifteen configuration constants match. All six configuration source pins are unchanged.

Immutable CLI SHA-256 `454eb435b569d8cd7e92408ea2212b1e2ce215e04b3cbe424106480334ed1bba`
clears the LLVM request boundary and reaches **LIB-138, unsized_fn_params at
original core/src/lib.rs165**. The original-core observation takes16.62s,
opens375 sources and verifies all722 original hashes. Options, explicit Allow
cap and finite14,500,624-work/1GiB-attempted/64MiB-retained limits are unchanged.
Core receipt `obj/parallel/llvm-linkage-v1/qualification-v1/core/result.json`,
SHA-256 `abf745466eeac05c498181beff91e308c2588bc21855afa28b133193ca440eca`.
A bounded debugger observation with freshly checked disassembly identifies the
request: `obj/parallel/llvm-linkage-v1/feature-probe-v1/receipt.json`, SHA-256
`232f11415408e1c3f9b70341e1d5a877e3912bc3d055f2eba5ea843a4288d3d1`.
This is an observed blocker, not an original library build pass.

The unoptimized ordinary128/512-function A/B check preserves frozen inputs,
40 observations, output equality, A/A calibration and four ABBA blocks per size.
Paired median B/A is1.112/1.056, with calibration max/min1.235/1.074.
Elapsed medians are37.82/40.83ms and124.83/127.62ms for predecessor/candidate.
These observations suggest possible modest overhead amid shared-host noise;
no speedup or absence of regression is claimed. Saved feature-access disassembly
shows additional unoptimized indexed two-word access checks. Feature lookups
remain constant work and kernel construction logarithmic in integer width.
Receipt `obj/parallel/llvm-linkage-v1/measurement-v1/receipt.json`, SHA-256
`a5a3b2b0e14cfdc2174e57a944cd21bf76c97ceeb739878840addd6d910b406e`.

Full predecessor qualification [41787601690](https://github.com/vishvananda/rustgm/actions/runs/41787601690)
on6f1e3eb3d passes every gate. Successor full qualification
[41795595365](https://github.com/vishvananda/rustgm/actions/runs/41795595365)
is running on compiler commit3d4b3ae9d; poll that exact handle.
Root works alone in the existing warm btrfs tree, under five-minute/two-thread/
16GiB/150GiB guards. Successful native outputs are pruned while inputs,
diagnostics, current immutable binaries, receipts and observations remain.

General LLVM target/vector/floating operations, immediate/zero-poison semantics,
address profiles and fresh imported identity/ABI consumers remain required.
The1077 textual upstream stdarch LLVM declarations are an inventory, not a count
of active blockers. PA28 is incomplete, as are PA25–PA44 and original
core/alloc/std artifacts. Next implement actual unsized parameter source,
unsized ownership and ABI/transport behavior under PA26/30/29/31; a feature
request cannot turn a by-value DST into an ordinary borrowed pointer.
