# Source destruction, body origins and host build checkpoint

This checkpoint advances original-library compilation without activating a
planned assignment. It retains typed source/imported body and proof origins,
resolves value `Self` through its actual impl constructor, validates actual
source Drop/Destruct headers and explicit-call restrictions, and executes
source `needs_drop` through checked calls and completed registration. Source
Copy preparation retains the exact public and owned execution signatures.
Package expansion now carries lexical edition context; the public driver still
exposes only the qualified Rust 2024 profile. PA34 also gains shared host rlib
and script scheduling, with immediate DEP metadata and actual generated inputs.

Original core/alloc/std builds remain incomplete. Generic body transport is
still implementation work beyond the retained ownership/origin prerequisites.
Actual Freeze and runtime/constant drop glue, full source Drop/Pin behavior,
source-sysroot import, remaining edition semantics and target build execution
remain required. A host Cargo comparison is not target crate acceptance.

## Frozen qualification

Qualified checkout: `c3b292b39467c2d4266899d0164e89c5fcc53611`.
The immutable release CLI SHA-256 is
`d4772a584e3187472f0db191fdc4b56ca90faccd246478b6d1675acdf8dceaf7`.
Its production sources are unchanged from `8d1d9e8c`: subsequent changes correct
five test expectations and add the separately qualified Python host executor.

All 3,270 regular and ignored maintainer tests are qualified. The initial full
release command passed 3,265 and failed five aggregate size assertions expecting
Declarations 7,040 rather than 7,048. The eight-byte increase is the reviewed
SourceCatalog destruction-completion identity (416 to 424); other measured
ordinary records retained their sizes. Only those five expected constants
changed. The complete release library rerun passed all 1,117 tests; the 2,153
passing integration tests from all 13 unchanged groups were retained under exact
production source hashes. The failed run remains evidence, not a passing run.

All 8,382 active text cases pass. PA19 includes 20 newly qualified `Self`
constructor cases with independently reviewed references. Affected independent
source and native suites PA16–PA24 pass, as do strict all-target/all-feature
release Clippy, formatting, the 1,986-file architecture audit, 159 Python harness
tests, registration/layout, upstream bytes and the existing backend gate.
Unchanged independent source observations for the new families are retained;
the merged release executes their actual candidate native consumers.

The host graph additionally passes a real pinned offline Cargo comparison:
six host compilations, five script runs and identical generated source, with no
candidate target compilation. It preserves separate host/target contexts and
features, direct renames, own generated configuration and immediate metadata.
At 8/32/128 shared consumers, harness counters verify one launch per host node,
two package validations and three full-receipt writes. Per-task receipts and
append-only completion records grow with the graph.

| Qualification component | Observed time |
| --- | ---: |
| Initial complete Cargo command, including compilation | 26m 01.6s |
| Optimized compilation within that command | 14m 12s |
| Corrected complete library rerun, including compilation | 6m 23.7s |
| All active text cases | 19.14s |
| Affected independent source suites | 228.61s |
| Affected independent native suites | 269.17s |
| Merged real host graph comparison | 2.57s |

These shared-machine observations do not establish a controlled harness
speedup. The extra library rebuild demonstrates why record-size dependencies
belong in focused checks before a release build. No integration group was rerun
merely because a test constant changed. A subsequent orchestration attempt
correctly failed when a nested storage guard refused a lock; its already-passed
static/harness checks were retained under identical sources, and the host runner
was invoked directly inside the existing outer guard.

Raw records live in `obj/parallel/source-destruction-editions-combined-v1/`,
`source-destruction-editions-close-v1/` and `source-destruction-editions-close-v2/`.
The final functional receipt SHA-256 is
`f887722145ad0439bd9ec6512682164b452da1d95cf0c313cd62fb3565b12090`.
After measurements, guarded cleanup removed 9,823 generated files (16.97 GiB
unshared blocks) and losslessly archived one large diagnostic. Current comparison
executables, original/generated inputs, hashes and observations remain.

## Performance

Eight common inputs compare byte-for-byte against the published source-traits
CLI `f520dc5f488d13a95e3502a36c8973069d58471de6ac54d729a912ed84bd2d38`.
The frozen protocol uses AABB warmups, AAAA calibration, five ABBA blocks,
three executions per observation, separate RSS measurements and Callgrind.
An exclusive RustGM lease prevents overlap with our workers; unrelated machine
load remains. Every observation is retained.

| Common input | Instructions | Median wall | A/A spread | RSS delta |
| --- | ---: | ---: | ---: | ---: |
| Copy calls 8 | −0.17% | +11.77% | 20.99% | −496 KiB |
| Copy calls 32 | −0.21% | +7.24% | 7.98% | −168 KiB |
| Copy calls 128 | −0.35% | +7.51% | 13.60% | −522 KiB |
| ordinary scalar | −0.59% | +8.00% | 5.05% | −326 KiB |
| opaque width 128/repeat 128 | −0.73% | +6.59% | 11.51% | −54 KiB |
| ordinary string | −0.10% | +9.98% | 5.91% | −96 KiB |
| generic empty 128 | +0.08% | +3.20% | 0.78% | −32 KiB |
| ordinary 512 | +1.48% | +3.87% | 5.92% | −978 KiB |

The wall medians increased; this is not a throughput improvement claim. A/A
spread and unrelated load limit attribution, and some wall differences exceed
their A/A spread. The declared instruction/RSS gates did not trigger, but those
gates do not prove equal wall performance. The measurements retain that limit
rather than discarding or rerunning inconvenient observations.

The 13 exact kernel inputs are rejected by the old compiler and accepted by the
candidate constants view; they have candidate-only measurements, not A/B
speedups. Width and reached-call count vary independently over 8/32/128, with
64/1,024 unrelated nominal controls, an unused intrinsic and 128 primitive calls.
For each isolated 4× width/call increase, whole-compiler instruction ratios range
from 1.02× to 3.24×. The separate query invariants retain 29/101/389 reached steps
and 388/772/2,308 logical bytes by width, unchanged by repetitions or unrelated
nominals. Ordinary unused declarations allocate no query; 128 primitive calls
share one four-step answer. This distinguishes total parsing/typechecking work
from the reachable structural query and does not claim global linear complexity.

No predeclared trigger fired: common-input instructions >5%, RSS growth both
>512KiB and>5%, or >6× instructions for an isolated 4× dimension increase.
The 13 sources retain exact independent Rust object/native evidence; merged
release native controls also pass. This does not establish generic library
artifact export or source-sysroot acceptance.

Raw declaration/results are in
`obj/parallel/source-destruction-editions-perf-v1/`.
Analysis SHA-256 is
`a5691bd501647fc33e41990571780592ad6b9b3c9a6c5046ff73badca77b3eca`;
measurement receipt SHA-256 is
`90a52c16bac7cb0137455047c5ab06165b8128b2aa6aefc281adf3eedb3e768e`.
Later publication edits are documentation only.
