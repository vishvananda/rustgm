# Owned unsized parameter source contracts

The [PA26 packet](../pa26/tests/spec/unsized-parameters/README.md) implements the
genuine crate permission, well-formed unsized signature inputs and direct
parameter bindings. Internal macro allowances alone do not enable this feature.
Ordinary locals/results, nested bindings, captures and explicit Sized bounds
retain their independent requirements. Owned moves/loans use the existing
complete original-body proof. Parentheses preserve places; compound unsized
operands require Sized before any type-publication deduplication can erase the
source distinction.

A lazy pass-local index records actual direct parameter definitions once.
It is shared across body checks, bounded and prepaid through the existing source
contract ledger. Ordinary sized bindings do not build it. The genuine bit65
uses the existing two-word crate permission set; macro flags stay one byte and
ordinary syntax, Package, Resolution and Body records retain their sizes.
No borrowed-reference capability replaces unsized ownership.

Qualification:

- Private11, including all eight predecessor CI failures, strict Clippy, formatting and architecture:100.01s. `obj/qualification/fast/1793780970811858388/receipt.json`, SHA-256 `896faea7ba1601a2ad3dc84d40605fca382c6c8c1ce7bc506fea317ada39bc75`.
- Production48, strict Clippy, formatting, architecture and PA22 356/356:33.83s. `obj/qualification/fast/1793781208190912066/receipt.json`, SHA-256 `9df95d67996a7031a912afd62a54efdd97ada9b0e1cb17d2da761153ea518f3e`.
- Native2, strict Clippy, formatting and architecture:37.00s. `obj/qualification/fast/1793781107928445285/receipt.json`, SHA-256 `67df0f261cd9982e45735bcf45e9414a56bf8105c654606f36eec1803afc7c02`. The new actual ?Sized original specializes for Value and automatically destroys it, producing7 then9 through the PA30 C observer and independent Rust. The existing signed/unsigned/mixed LLVM matrix still passes168 comparisons and its ordinary control.
- Independent Rust:64 source outcomes and two compiled/executed native programs,2.35s. `obj/parallel/unsized-parameters-v1/reference-v3/receipt.json`, SHA-256 `036dba9191e7d879cdee8420f1907cd2ca3b1c3d956b23a64739757dec591ed7`.
- Joined public CLI/full-original-proof/configuration/harness/upstream/core qualification:36.69s. `obj/parallel/unsized-parameters-v1/qualification-v2/receipt.json`, SHA-256 `581f977c37cd1a24adc8400ddd1421e4a4b536bcb11c716d088e6b8d389953ac`.

31 of the32 source cases agree through complete original proofs in both editions
(62 observations). The unchanged Rust-positive str::len control has an explicit
required library-method gap, observed separately in both editions. The original
println native is independently positive and remains required standard-library
macro work; RustGM executes the C-observed counterpart. These boundaries are not
reported as Rust source-language rejections or passing positive implementations.
Concrete unsized ABI, allocation handoff, full metadata-aware destruction,
function addresses and imported ownership transport remain required. Imported
contracts keep mandatory Sized goals until actual transport proof is implemented.

The ordinary predecessor/control LowIR and all fifteen configuration constants
are byte-identical. All six configuration source pins are unchanged.
Immutable CLI SHA-256 `14fc5fcf6910eb9d48f0a7ea17d7b9f4bdb56715031b3f15aca953f79c2c26fa`
clears LIB-138 and reaches **LIB-139, with_negative_coherence at original
core/src/lib.rs166**. The fresh probe takes17.97s, opens375 sources and verifies
all722 original input hashes. The explicit Allow cap, options and finite
14,500,624-work/1GiB-attempted/64MiB-retained limits are unchanged.
Core receipt `obj/parallel/unsized-parameters-v1/qualification-v2/core/result.json`,
SHA-256 `b5019824eb3cbf9360e356d31a2b4388835287e0c518610c381ee5cd59433736`.
The bounded debugger uses freshly checked disassembly and verifies input hashes:
`obj/parallel/unsized-parameters-v1/feature-probe-v1/receipt.json`, SHA-256
`9ad57147a3230ddfa96388d5fc8b6c2225d9fbf134d5f63ba32ecdef72f5834f`.
The first joined attempt retained an infrastructure/setup failure caused by a
script variable overwriting the expected binary hash; V2 corrects that variable.
No compiler observation or source input was changed to hide that failure.

The immutable dev-fast A/B check retains40 ordinary observations, output equality,
A/A calibration and four ABBA blocks at128/512 functions. Median paired B/A is
1.077/0.981; calibration max/min is1.043/1.248. This does not establish a speedup
or absence of modest overhead. Four observations each at64/128/256/512 actual
owned-parameter functions take median40.26/74.05/131.65/233.04ms through complete
original proofs. Eight times the functions take5.79 times the elapsed time in
this sample. The index is constructed once per pass; there is no per-binding
scan of the crate. Receipt `obj/parallel/unsized-parameters-v1/measurement-v1/receipt.json`,
SHA-256 `3d2cff2cb485a2fc60b433dd85d85f01a6c7067bfbc2bd8874e3651f98800059`.

Full predecessor LLVM CI41795595365 failed eight stale cold-layout assertions:
the genuine bit64 expanded Features8→16, SourceCore184→192 and Source192→200.
The focused correction retains exact ordinary-record and one-byte macro checks;
all eight failures now pass. Other regular integration tests in that run pass;
subsequent course/native/debug/full gates were not established by that failed
run. Successor full qualification
[41801911308](https://github.com/vishvananda/rustgm/actions/runs/41801911308)
is terminal success on compiler commit975b32666, with every gate green.

Next inventory genuine negative-coherence use under PA20 solving/coherence,
PA31 source authority and PA29 imported assumptions. Do not classify it as an
advisory request or grant closed-world negative evidence without proof.
PA25–PA44, whole PA26 and original core/alloc/std artifacts remain unfinished.
Root stays alone in the existing warm btrfs tree under unchanged five-minute,
two-thread,16GiB output and150GiB reserve guards, with scoped output cleanup.
