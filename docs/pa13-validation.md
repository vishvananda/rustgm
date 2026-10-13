# PA13 validation

PA13 is implemented-draft for Rust 1.98.0, edition 2024, Linux x86-64, with the
existing abort/checked-overflow profile. The [handout](../pa13/README.md) owns
its exact selected-pattern vocabulary and execution boundaries; the
[manifest](../pa13/tests/manifest.json) declares every outcome before oracle
generation. PA14–PA16 remain responsible for their ownership/lifetime/loan
contracts. Shared-reference inspection never authorizes unchecked execution.

## Scope and independent controls

The 803 cases comprise 538 LowIR cases and 265 pattern inspections. There are
364 executable LowIR programs, 174 LowIR rejections, 170 successful inspections
and 95 inspection rejections. Cases cover binding modes, scalar and composite
coverage, constructor decisions, conditional bindings, guard retries, constant
patterns, strings, shared references and dynamic slices. Five packages explicitly
declare their auxiliary module files and preserve use-site identities.

All 803 cases pass independent pinned-rustc acceptance checks. Thirteen manifest
qualifications are explicit: five valid reference programs remain outside the
executable profile; two successful inspection cases still require lifetime or
loan rejection; and six borrowed non-reference constants follow the Reference's
implicit-dereference rule where rustc requires an explicit `&`. Explicit-reference
and constructor controls are also included. These are declared differences,
not silent acceptance-parity claims.

Native qualification executes 364 programs through the pinned CPPGM backend
and rustc. Of those, 348 have matching clean execution. Eight opposite-signed-zero
pairs (16 programs) independently assert the Reference's guard count of two and
rustc's count of one. The two members have opposite pass/panic expectations for
the compilers. Every such difference declares `native.rustc_outcome` and a reason;
neither compiler is skipped. Unqualified signals, aborts, timeouts and exit 86
still fail. Borrowed/text/slice inspections never count as native executions.

The native checker validates and builds LowIR with the immutable backend tools
in `backend.lock.json`, with no C++ source changes. A source assertion compares
full values by causing an intentional arithmetic panic on failure. Matching
stdout text is not a substitute for execution.

## Representation and invariants

The compiler consumes canonical types, resolved declaration identities and
selected operation/projection facts. Pattern nodes and fields live in flat
arenas. Composite coverage shares reduced decision graphs instead of expanding
or-pattern products, and dynamic slices specialize only lengths where tested
prefix/suffix positions alias. Ordinary node facts remain 80 bytes.

Guard execution keeps internal protected views, delays owning captures until
success, and resumes matching alternatives in written order. Cleanup shares
persistent storage tails. Imaginary ownership edges check bypassed paths while
executable lowering follows real edges. These internal checks are distinct from
pending user-reference loans.

Demanded constants share typed queries and flat scalar/product/text/place arenas.
Copy checks precede cached static reads; borrowing a non-Copy static does not
move it. Packed alignment survives projections and resets at a true referent.
Root coverage validity distinguishes owned values from dereferenced places,
including deferred constant-pattern binding checks. Deep references and repeated
products use bounded worklists and shared subpatterns.

The 347 debug and release tests include exhaustive small-domain comparisons,
manual scalar-rank/witness checks, sparse projection assertions, shared-pattern
checks, bounded adversarial inputs and a 2,048-level reference chain. A cyclic
static-pattern probe crashes pinned rustc; it is retained as an infrastructure
failure and tested only for RustGM's bounded diagnostic. It is not an independent
language rejection oracle.

## Frozen compiler and performance

The final compiler is `obj/pa13-work/reference-final-candidate`, SHA-256
`e730a7af794e7f8697f44ef194a50db75c311ecfd8553f8f351b320ff104793b`.
All 1,695 earlier assignment oracles remain byte-identical. Explicit regeneration
changes only their twelve compiler-source receipts. The final compiler also
reproduces byte-identical LowIR for all 372 prior PA13 native inputs, retaining
the original execution receipts and their documented qualifications.

Seven sequential CPU-10 runs retain all 378 timing observations, frozen inputs,
frozen binary hashes, A/A calibration and three ABBA blocks per scale. Four
existing workloads compare against the published text-pattern compiler; three
new reference workloads use candidate A/A for scaling. Complete rendered and
count outputs agree in every run. No local RustGM build/test overlaps timing.
The measured existing paths show no material median-time or peak-memory change;
wall-time noise does not justify a precise speedup claim.

The [buildout record](pa13-buildout.md#shared-reference-constants-and-place-queries)
contains the per-workload table and prior algorithm/performance checkpoints.
Reproduce the new workloads with `scripts/measure_lexer.py --stage patterns
--reference-pattern-input shared|places|chain`. The comparison generators for
ordinary LowIR, product/nominal constants and text patterns are in the same tool.

## Evidence and reproduction

Local retained evidence lives under `obj/pa13-work/` and `obj/pa13-course/`:
`reference-validation-summary.json`, `reference-final-*.log`,
`reference-final-native-equality.json`, `reference-perf-receipts.json`,
`reference-perf-verified.json`, `independent-complete.log` and `native.log`.
`obj/pa13-course/declaration-provenance.json` maps staged fixtures to the declared
maintainer controls or earlier independent execution receipts. The staged native
receipt records all commands, outcomes, manifest hash and immutable tool hashes.

The release checks are `make test-report lint audit-rust test-harness`,
`make test-upstream test-rustc`, debug/release Cargo tests and
`make test-native-pa13`.
Text references are generated only by explicit `make ref-test-pa13` after
independent qualification. Review the resulting sidecars with the declared
inputs and statuses; the manifest and oracles are published together.

The installed suite passes the root report: 2,498 cases across PA1–PA13. The
final root independent run passes PA11's 194 checks/180 executions, PA12's 97
checks/48 executions, and PA13's 538 checks/364 executions. All 803 PA13 rustc
acceptance checks pass with the thirteen declared qualifications. Rustfmt,
Clippy, the 216-file audit, 25 harness tests, placement and upstream verification
also pass. `obj/pa13-course/release-summary.json`, `release-report.log`,
`release-independent.log` and `obj/native/pa13/evidence.json` retain these results.
The installed and frozen compiler bytes were verified equal after validation.
