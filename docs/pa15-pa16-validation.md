# PA15–PA16 release validation

PA15 publishes lifetime-contract proofs; PA16 publishes loan proofs and checked
reference execution. PA17 destruction remains incomplete. This release does not
claim full Rust conformance or remove the later const/trait/unsized/unsafe owners.

## Assignment and implementation evidence

PA15 contains 191 cases: 124 accepted region views and 67 rejections. They cover
lifetime declaration/elision, early/late binding, explicit and implied outlives,
variance, higher-ranked callables, casts, nominal bounds and local/referent
provenance. PA16 contains 292 cases: 105 accepted loan views, 75 rejections and
112 executable LowIR cases. The native cases declare 107 clean successes and
five runtime panics. All acceptance expectations agree with pinned rustc.

Fixtures promote existing original RustGM maintainer vectors into required
assignment manifests. `maintainer_origin` records the source. Expected outcomes
came from those already independently checked vectors, before reference output
was generated. Eighteen additional PA15 provenance controls retain the distinction
between a parameter's storage and its referent. No upstream bytes or provenance
records changed.

The new `--emit-regions`/`--count-regions` modes expose the retained directed
contract graph, parameter/inference identities and placeholder universes.
Assumptions cannot become obligations or vice versa. The new
`--emit-loans`/`--count-loans` modes expose canonical paths, generated loans,
CFG points, sparse runtime regions and reached loan/activation states. The latter
records come from the final checked transfer pass, rather than a second solver.
Executable verification does not allocate inspection snapshots.

Three new maintainer tests check a hand-written exact graph view with duplicate
requirements, the distinction between contract and loan validity, count/full
agreement, process-to-process determinism, absence of partial failure output,
normal/inspection proof-count equality and activation at the consuming call.
Twelve declared sources are independently checked with rustc in that CLI test.
Existing region tests independently compare small graphs with exhaustive
set-inclusion models; existing loan controls cover last uses, guards, storage,
loops, reborrows and two-phase conflicts.

A structural review of all 371 proof-view cases checks statuses/streams, graph
identity and numeric edge order, counted record totals, valid sparse region
point masks, reachable loan-state points and activation as a subset of live
loans. Inspection cases are never counted as executable comparisons.

## Earlier assignment contracts

Twelve obsolete executable boundary inputs moved to their earliest proof owner:
PA11's constant/static/reference cases, PA12's nominal-lifetime/nested-reference
cases, and PA13's seven `execute-*` reference cases. Nominal lifetime declaration
and local-reference escape belong to PA15; the remaining ten belong to PA16.
The source cases remain present, with new declared acceptance where the complete
compiler now implements the formerly pending capability. Additional native
vectors demand the actual reference and persistent-storage operations.

The other 59 previously reviewed oracle differences stay with their original
inspection owners. They record directed coercions instead of incorrectly equating
lifetimes, fresh inferred reference identities, selected reborrow facts and the
specific mutable-constant-memory boundary diagnostic. The PA9/PA10/PA13 handouts
now describe those records. PA14's two changed reference-copy views retain the
same move/copy counts while using the actual inferred types/value IDs.

Regeneration used the explicit course generator after independent expected
outcomes were checked. Earlier output changes are confined to the 71 reviewed
cases (including the twelve moves); there are no unrelated output changes.
Source/fixture/manifest provenance is recorded by the generated receipts.
Historical validation reports describe their original release counts; the
current manifests and handouts are authoritative.

## CI repair

The failed GitHub run at `3ee9fb9` stopped in
`named_constants_and_static_storage_execute_like_rustc` because hosted CI had no
`backend.local.json`. Six maintainer tests had begun requiring the qualified
CPPGM binaries from the default Cargo suite, contrary to the documented hosted
frontend boundary. They are now explicitly ignored by that suite and run through
`make test-maintainer-native`, also included in `make test-native`. The native
target first qualifies the pinned backend; missing or changed binaries fail.

An isolated checkout without backend configuration passes all 463 pre-view
frontend tests and reports six ignored native tests. This is an explicit suite
selection, not a successful native result. The complete new suite passes 466
frontend tests in both debug and release profiles. All six native tests also
pass locally against the unchanged pinned backend.

CI compares the current compiler directly with committed course text in
`make test`. It also checks fixture discovery and manifest/fixture provenance.
It no longer regenerates oracles and then rejects changed compiler-origin hashes:
those hashes describe the compiler that generated an oracle, and unchanged
oracles legitimately retain earlier provenance. Duplicate `test-report` execution
was removed from the same build step. No test-result exception list or automatic
oracle update was added.

## Completed local checks

- All 3,093 PA1–PA16 course cases match committed stdout/stderr/status exactly.
- PA9–PA16 independent rustc checks pass, including the new proof suites.
- All 112 PA16 native comparisons pass through rustc and CPPGM. The promoted
  overflow fixture explicitly allows the compile-time arithmetic-overflow lint
  so its declared runtime panic remains the property under test.
- Debug/release maintainer suites pass (466 tests each); all six explicit native
  maintainer tests pass. This is 472 distinct maintainer tests, not their sum
  across repeated profiles.
- Formatting, Clippy all/pedantic with warnings denied, the 304-file audit with
  zero findings, all 25 harness tests and backend smoke pass.
- Upstream verification preserves all 24 fixture checksums and all 126 pinned
  Rust Reference files. No new compiler lint allowances or audit exceptions.

Logs, immutable binaries and measurements are retained under `obj/pa15-work/`
with the `proofviews-*` and `paN-proofviews-*` prefixes. The independently declared
fixture migration is recorded in `assignment-migration.json` there.

## Performance evidence

The frozen candidate was compared with the preceding `3ee9fb9` binary using
three scales per workload, A/A calibration and three paired ABBA blocks. All
270 observations are retained, including warmups and outliers. Source, binary
and input hashes were verified after measurement; full outputs and count outputs
agree between binaries at every scale. No other local test or build ran during
these measurements. Timing is wall time on a shared host, not a hardware-counter
gate or a claim of a speedup.

| Existing workload, largest scale | Baseline median | Candidate median | Candidate median RSS |
| --- | ---: | ---: | ---: |
| Ownership, 64,000 items | 5,673.33 ms | 5,820.29 ms | 949,388 KiB |
| Guard retries, 16,000 items | 1,554.48 ms | 1,520.59 ms | 296,784 KiB |
| Thin-reference lowering, 4,096 items | 474.75 ms | 488.36 ms | 76,778 KiB |

Largest-scale median changes are +2.59%, -2.18% and +2.87%, respectively;
observed timing ranges overlap. Smaller scales also vary in both directions.
There is no consistent regression across scales, and retained RSS is comparable.
The raw observations remain available rather than discarding noisy measurements.

The new modes have no predecessor flag, so their scaling runs use two immutable
copies of the candidate. At 256/1,024/4,096 input units, region-view medians are
11.78/37.30/166.15 ms with 6,454/12,774/37,410 KiB median RSS; loan-view medians are
27.85/109.62/457.11 ms with 9,230/24,026/82,626 KiB. All reported region, loan and
thin-reference counters grow linearly over these scales. Normal executable
verification keeps inspection snapshots unallocated; the optional trace is
bounded to two million 24-byte records.

The candidate SHA-256 is
`abc0be79acb5f34ee5bbdb19ea98daf86fa3ac3172849ef5e679d74c4c283ada`;
the baseline is
`e89959a537430345622fa3b4f4dc70443faa76592ad04c083b400b6705d5525f`.
`obj/pa15-work/proofviews-v1-perf-receipts.json` indexes the five complete
`obj/perf/*/samples.json` receipts and their raw `observations.jsonl` files.
`proofviews-v1-perf-verified.json` records the summaries; the matching source
hashes and immutable binary identities are retained beside it.
