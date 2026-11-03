# Source MaybeUninit native-execution checkpoint

This PA31 runtime checkpoint qualifies the three native observers from the
existing [MaybeUninit packet](../pa31/tests/spec/source-maybe-uninit/README.md).
Original Rust/C bytes and independently declared outcomes are unchanged. Real
source methods preserve mutation, owned values and destruction; an uninhabited
assertion calls the actual source `panic_nounwind` function, named `fail` in the
packet. This is additional execution evidence, not a PA31 release or an original
core/alloc/std artifact. General PA29 nominal transport and fresh imported
consumers remain required.

| Observer | Candidate output | Exit |
| --- | --- | --- |
| native-values | `8` | 0 |
| native-drop | `drop 7`, then `value 7 count 1` | 0 |
| native-uninhabited | attempted to instantiate uninhabited type `Void` | 23 |

The independent Rust receipt remains
`2c9311175a142cdf7f73bf06110fcfcc80d372f34b230cd29068bd8c3e5de84a`.
No stub panic, fallback source body or external Rust frontend implements the
candidate behavior. CPPGM remains only the external LowIR-to-native backend.

## Producer and execution boundaries

`panic_nounwind` appends language-item ordinal 82; all prior ordinals remain
unchanged. The local execution profile authenticates the actual module function,
current source header/body and completed safe Rust `(&'static str) -> !`
signature without written generics or predicates. Its body follows ordinary
instance demand and lowering. The pinned Rust collector demands the hook even
for a concrete inhabited assertion; only an uninhabited assertion calls it.
Declaration admission supplies neither constant-call nor unsafe permission.
The pinned primary sources are `rustc_monomorphize/src/collector.rs`
(`7130846f8316a0618149500c792733df527a5fd4e7e6623b8381b8ff7bd54c41`)
and `rustc_codegen_ssa/src/mir/block.rs`
(`13d6d31caf4c1693ec26c4fa4d8385361d816413b1c9fbc77a71b914face653b`).

Concrete source `size_of`, `align_of` and `assert_inhabited` share completed
layout data in a cold optional catalog. Each reached direct Call still checks
its current proved body, complete signature/result, empty argument list and
actual evaluated callable target. A warm target does not authorize another
caller. Ordinary sources without runtime plans return before inspecting values
or allocating a map. Queries pay attempted work even on cache hits and after
exhaustion; growth is charged before publication through the existing source
contract payer. The 128-query control retains no additional capacity.

False assertions build initialized static UTF-8 text from completed types and
resolved nominal paths. The source hook receives it through the ordinary checked
argument writer and actual fat-string ABI. The existing C/object argument path
also consumes that writer; tuple callback ABI fences remain. Source and native
controls cover scalar calls, nullable record carriers, host objects, wrappers,
NoDrop mutation and owned destruction.

The current diagnostic spelling profile covers primitive/never, pattern base,
array, slice, reference, pointer, tuple and local nominal/foreign types. General
function/closure/opaque spellings, signed const formatting and complete lifetime
display parity remain required. Reified/unwind intrinsic paths and general
imported panic contracts also remain open. This checkpoint does not claim those
profiles or complete `type_name`.

## Structural storage improvement

Retained provider declarations use checked off-by-one `NonZeroU32` identities
for definition/optional parent and preserve the original `NodeKey`. Public
declaration data and metadata formats do not change. Zero and absent parents
stay distinct; maximum unrepresentable IDs reject instead of wrapping.

Measured sizes on the qualified Linux x86-64 host:

| Record | Before on main | Candidate |
| --- | ---: | ---: |
| Source catalog | 1840 | 1608 |
| Retained provider role table | 1640 | 1328 |
| Imported role table | 1384 | 1400 |
| Role-binding enum | 1640 | 1400 |
| Reservation producer | 2544 | 2560 |
| Declarations / Resolution / Package | 7088 / 2032 / 256 | 7088 / 2032 / 256 |

The provider/imported difference is 72 bytes, within the unchanged 256-byte
bound. No ordinary Body, Code, frame or type-property record gains a field.
Compaction also removes the old large-enum Clippy expectation. Existing tests
that forge parent and current callable-role data still reject through the new
representation; no proof or size limit was relaxed.

## Qualification

All local gates use the guarded unoptimized warm profile, two threads, 16 GiB
absolute output cap, 150 GiB reserve and a five-minute deadline. The full release
and assignment qualification stays in CI.

| Gate | Result | Seconds |
| --- | --- | ---: |
| 28 exact affected private tests, strict private Clippy, formatting/architecture | pass | 163.00 |
| 105 source outcomes, strict production Clippy and PA12 125/125 | pass | 174.59 |
| Five exact native test groups, including the three new observers and existing ABI controls | pass | 44.40 |
| 70 CLI outcomes, upstream/harness and unchanged original-core probe | pass | 47.03 |

The three fast receipts are
`obj/qualification/fast/1793717955673948726`,
`obj/qualification/fast/1793718136899135914` and
`obj/qualification/fast/1793718329809593346`.
Successful native checks reclaim their executables/objects while keeping inputs,
LowIR, literal observations and cleanup receipts.

Immutable candidate CLI:
`f5ae84ebd2b553aa521f2c0c0a7139c165e9ed7fcfc73a8a0e3c46b0f76b13bf`.
Joined receipt:
`1b9b433dc43e0e360134a5bc0adfeee0d55ce69f946f2a56da7ff73e67031ebe`.
Evidence lives under `obj/parallel/maybe-uninit-native-v1/`.

The original-core probe takes 20.67 seconds with 375 source-loading opens, all
722 input hashes and all six configuration implementation hashes unchanged.
It retains cap-lints=allow and finite limits of 14,500,624 work steps, 1 GiB
attempted storage and 64 MiB retained storage. The next source blocker remains
LIB-124: genuine `TransmuteFrom`/structured `Assume` and its coinductive validity,
layout and lifetime relation in `mem/transmutability.rs:87` (file 59 byte 3207).
These opens are not compiled files, and no original crate artifact was produced.

## Compile-time observations

Immutable main/candidate binaries and frozen inputs were compared with A/A
calibration and paired ABBA blocks. Every output byte matches. Each input has
66 retained wall-time observations; these are unoptimized development timings,
not release throughput claims.

| Input | Main mean, ms | Candidate mean, ms | Median paired ratio | A/A ratios |
| --- | ---: | ---: | ---: | --- |
| ordinary source | 13.18 | 13.07 | 0.992 | 0.970–1.048 |
| 32 reference selectors | 182.88 | 189.93 | 1.039 | 1.008–1.036 |
| opaque union | 16.80 | 16.93 | 1.007 | 1.020–1.101 |

Wall receipt:
`a154baf82f7f0fc69c8ab7a1ede9dad4184f7514abb796d7fe33c577b807160e`.
No compile-time speedup is established. The repeated-reference result shows a
small measured slowdown close to its calibration variation; keep that observation
alongside the verified structural storage reduction and bounded-work controls.

A follow-up Linux user-instruction measurement retains two A/A pairs and two
ABBA blocks per frozen input, again with equal outputs. Candidate/main median
instruction ratios are 1.00179 (ordinary), 1.00583 (reference selectors) and
1.00026 (opaque union). The reference case adds about 0.58% instructions; the
wall-time difference is larger than the measured extra instruction work. The
counter receipt is
`dfa659ee7da6c26bee3bc6be4bc8e666308adab3d3944deef3a3492a60fdc243`.
These finite controls establish neither zero overhead nor a broad release
performance result. Preserve their raw observations for subsequent comparison.

The public predecessor `fedfef216` passed full CI
[41650945582](https://github.com/vishvananda/rustgm/actions/runs/41650945582).
This checkpoint's full CI is recorded separately when available; focused gates
do not stand in for assignment release.
