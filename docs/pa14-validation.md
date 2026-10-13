# PA14 validation

PA14 is implemented-draft for Rust 1.98.0, edition 2024, Linux x86-64. The
[handout](../pa14/README.md) specifies the ownership view and its proof boundary;
the [manifest](../pa14/tests/manifest.json) declares every outcome before oracle
generation. The implementation already existed beneath the earlier executable
profile. This release adds the formal course contract and independent fixtures
without changing compiler source or weakening pending lifetime/loan checks.

## Declared controls

The 124 cases comprise 112 ownership inspections (58 acceptances and 54
rejections) and twelve executable LowIR programs. The initial 74 controls come
from the maintainer's independently checked move/derive matrix. Fifty additional
cases exercise immutable/mutable initialization joins, loop storage, divergence,
index evaluation, static reads, disjoint and overlapping pattern moves,
imaginary ownership paths and delayed guard capture. All 124 agree with pinned
rustc with no declared acceptance difference.

The twelve executable controls assert full values across branch and loop
reinitialization, nested partial moves, storage on continue, struct updates,
Copy derives, disjoint array/enum selection, temporary moves and guard success.
They execute cleanly through both the qualified CPPGM backend and rustc.
Ownership-view acceptance alone never counts as execution evidence.

The seven clusters contain 21 move cases, 28 initialization cases, 18 array
cases, five struct-update cases, 28 Copy/Clone cases, twelve pattern interactions
and twelve execution controls. Every input is manifest-listed. These are authored
course fixtures, not new upstream imports.

## Proof and invariants

Ownership consumes typed CFG actions and selected projections. Flat move paths
track only demanded fields/indices/rests; array intervals share overlap bits and
do not expand from declared length. Initialization retains a shallow parent bit
so writing fields cannot create an uninitialized or wholly moved parent. The
fixed point intersects must-initialized state, unites may-written state and
queues changed successors. Storage boundaries reset local state. Imaginary
successors participate in the proof, while executable lowering follows real
edges. Internal guard views retain their separate protected-place/liveness check.

Copy/Clone queries cache positive and negative type properties, stop at pointer
and reference boundaries and do not infer properties from size/layout. The
existing debug/release invariants exercise deep queries and paths, failed-query
isolation, huge sparse arrays, overlapping selected intervals and linear storage
cleanup. The inspection and executable APIs run the same proof; the executable
API releases each body's inspection records before lowering.

These remain owned-value proofs. PA15 lifetime contracts, PA16 user-reference
loans and PA17 destruction are not implemented by this view. Static writes,
reborrowing and user trait implementations retain their explicit boundaries.

## Frozen scaling measurement

The compiler bytes are unchanged from PA13's final candidate, SHA-256
`e730a7af794e7f8697f44ef194a50db75c311ecfd8553f8f351b320ff104793b`.
Run `scripts/measure_lexer.py --stage ownership --candidate BINARY` to reproduce
the generator: independent functions each move a field, reinitialize it on
both branches and consume the initialized fields. The measurement varies
function count, not the complexity of a single body's fixed point.

Run `obj/perf/1791870054766248325/` retains 54 observations on CPU 10. Both
snapshots are the same immutable compiler. Every scale retains warmups, four
A/A calibration samples and three ABBA blocks, with frozen input hashes and
complete rendered/count output equality. No local build/test overlaps timing.

| Functions | Candidate paired median ms | Paired median peak KiB | A/A calibration range ms |
| ---: | ---: | ---: | ---: |
| 4,000 | 273.22 | 57,452 | 273.88–279.98 |
| 16,000 | 1,244.38 | 219,838 | 1,154.94–1,595.76 |
| 64,000 | 4,755.13 | 868,024 | 5,450.48–8,388.58 |

Memory scales with the retained crate/inspection facts. Wall-time calibration
shows considerable host noise, especially at 64k, and is not a speedup or
regression comparison. Sparse representation and work-budget assertions remain
independent of timings. `obj/pa14-course/ownership-perf-verified.json` records
verification of all 54 observations and all binary/input/output hashes.

## Evidence and reproduction

Local staged receipts are `obj/pa14-course/expanded-independent.log`,
`native.log` and `obj/native/staged/evidence.json` beneath that same directory.
The installed suite uses the standard root harness and native checker, with
explicit `make ref-test-pa14` oracle generation after independent qualification.
Run `make test-report lint audit-rust test-harness test-upstream`,
`make test-rustc` and `make test-native-pa14` for the course release checks.

The installed root report passes 2,622 cases across PA1–PA14. PA14 passes all
124 rustc checks and twelve native executions. Rustfmt, Clippy, the 216-file
audit, 25 harness tests, placement, 24 upstream fixtures and 126 Reference files
also pass. All fourteen source receipts match the unchanged compiler tree, and
no earlier oracle changes. `obj/pa14-course/release-summary.json`, `release.log`,
`installed-independent.log`, `installed-native.log` and
`obj/native/pa14/evidence.json` retain these results. Debug/release compiler
invariants are unchanged from PA13's 347-test runs and successful hosted CI.
