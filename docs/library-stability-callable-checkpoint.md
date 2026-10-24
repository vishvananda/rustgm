# Local library stability and source callable execution

This checkpoint completes local regular API stability inventory, shares compact
source call contexts, executes closed source const callbacks, and lowers runtime
`const_eval_select` through the actual evaluated callback. It also gives actual
source Copy/Clone marker queries their callable formation obligations and resolves
source macro imports without creating bootstrap library declarations from names.

PA25–PA44 remain planned. No whole original core, alloc or std build succeeds.
Cross-crate source-role transport, broader const/default-body stability, const
trait default defining contexts, CTFE selector execution and scoped declarative
macro hygiene remain required. Their isolated worker drafts are not included.

## Functional qualification

Qualified compiler/test snapshot: `0cfcf32a8d81119bf039e968f099b00647ccbf4d`.
Release CLI SHA-256:
`966e8ca2446400276970a7d581f78afeba57d6e22f302bfb999c12fa9a2f61a9`.

All **3,477 regular and ignored maintainer cases** are qualified: **3,180 retained
passing cases in 14 unchanged targets**, including all 1,257 private library
invariants, plus **297 fresh cases in the complete frontend target**. This is
recovery from a failed full run, not a fresh full pass. The original full release
command ran every target and reported 3,475 passes and one stale assertion.

Unknown source feature requests now wait for the completed expanded declaration
inventory because an enabled local declaration may define a library feature.
The stale test expected rejection during loading. Its replacement verifies both
retention during loading/expansion and the exact rejection at inventory completion,
including failed-publication behavior. This correction changes only that test
file; production and every retained test binary/source hash are bound separately.

All **8,382 active text cases**, affected independent source/native suites PA17–PA24
and seven transitive-metadata native consumers pass. Formatting, architecture/file
audits, all-target/all-feature strict Clippy and harness checks pass. Unchanged
upstream/backend checks retain their earlier successful evidence; added benchmark
fixtures receive a fresh upstream/harness check before publication.

| Component | Observed shared-machine wall time |
| --- | ---: |
| Original full release/native command, including its failure | 1,671.27 s |
| Optimized compilation within that command | 15 min 33 s |
| Corrected entire frontend target, build and execution | 62.00 s |
| All active text cases | 17.68 s |
| Affected independent source checks | 197.36 s |
| Affected independent native checks | 240.85 s |

These are observed costs, not a controlled harness speedup. The original full
run is retained under `obj/parallel/library-stability-callable-close-v1`; recovery
is `library-stability-callable-close-recovery-v2` under the same parent. Recovery
checks SHA-256:
`1870d0c0628eaad7fcae77b74f4d148b43c39940e542762aba093344557cad32`.
The failed full log, exact target inventory and retained/fresh boundaries remain
separate. Diagnosed full-run cleanup reclaimed **17.17 GiB**; successful functional
cleanup reclaimed another 0.25 GiB, preserving inputs, diagnostics and current pins.

Earlier focused checks caught two test/harness issues before the full run. A
callable-lifetime test assigned a direct outlives condition to the trait-marker
proof lane; corrected controls retain genuine selected-impl lifetime debts and
exercise direct outlives checks in their owning well-formedness/borrow lane.
A direct test-binary batch omitted Cargo's package environment, making the
independent rustc half of the OptionEnv test fail. The batch runner now restores
explicit package metadata and manifest paths for inventory and execution, while
retaining inherited non-package environment and recording only package overrides.
Original failed receipts remain available; all affected controls pass after repair.

## Algorithmic contract

Library attributes are consumed during the configured owner walk. Indexed module
and lexical-owner policies avoid repeated crate scans and publish atomically with
source-role reservations. Ordinary source without library requests allocates no
library inventory. A library feature never grants a compiler language capability,
a trait proof or const-call permission.

Each source semantic body shares one immutable call-context descriptor; call sites
retain compact handles with exact owner checks. Closed callable execution uses the
existing real Invoke path and consumes fresh, site-specific conditional payments
on both cold and warm code reuse. Signature structure and cached code do not grant
another caller permission.

Runtime selector demand and emission share actual checked operand origins. A
bounded forward index follows Constrain values once, retaining full mapped targets
and ordinary ownership/destruction proofs. Pointer-only bodies avoid that index.
The unselected const callback retains its obligations without demanding its body.
Callable Copy/Clone marker queries use the existing caller-local formation DAG;
marker evidence supplies neither an executable Clone body nor const execution.

## Performance qualification

The exclusive matrix freezes **48 inputs** against the preceding published
compiler: **26 byte-equal A/B comparisons** and **22 candidate-only measurements**.
All **24 declared candidate native programs** pass before timing. Candidate-only
acceptance is not a speedup over a failing baseline. The source and edition
adapters use identical driver bytes for both compiler versions and the real
expansion, resolution, proof and lowering paths.

Comparable instruction changes range from **−0.69% to +0.69%**, with median
**+0.24%**. The largest median RSS increase is **346 KiB**. All fourteen declared
4x growth comparisons are **1.58x–3.64x**; the two 2x owner-depth comparisons are
**1.10x–1.16x**. No instruction, RSS or growth investigation threshold fires.
Adding 128 unrelated declarations to the payment control costs 3.41x for the
whole frontend; this is reported separately from the bounded per-use machinery.

| Changed-path family | Sizes | Candidate instructions |
| --- | --- | --- |
| Reached source payments | 8 / 32 / 128 | 2,872,896 / 5,754,994 / 17,220,338 |
| Closed source callbacks | 8 / 32 / 128 | 6,432,361 / 15,192,112 / 50,451,004 |
| Library owner width | 8 / 32 / 128 | 960,598 / 2,065,035 / 6,469,819 |
| Library repeated uses | 8 / 32 / 128 | 835,610 / 1,323,685 / 3,256,185 |
| Library owner depth | 8 / 16 / 32 | 688,792 / 754,525 / 877,877 |
| Mapped runtime selector | 8 / 32 / 128 | 10,092,596 / 26,214,885 / 90,526,547 |

Paired wall-time medians range from **−19.33% to +12.37%**, with median **+1.56%**.
A/A spreads range from **0.70% to 78.96%**, with median **8.01%**. These short
shared-host observations do not establish a general wall-time improvement.
Warmups, calibration, five ABBA blocks, RSS and Callgrind observations are retained
without retries for a favorable result.

Final evidence: `obj/parallel/library-stability-callable-perf-v3`.
Results SHA-256:
`c4f50e94dab86778e91734652b66adcd2b63f8a70c8b0466aa77d18baf31c3d5`.
Analysis SHA-256:
`e33b78ac94f0a325b904c2e13c02d92773e36b3d2c93bd989d8e1913ce77b280`.


## Retained benchmark preparation corrections

V1 stopped before declaration or observations because the frozen selector contract
hash preceded the append-only `item-reuse` fixture. Exact review established that
all existing inputs and policy were unchanged. V2 then ran all 48 command preflights
and stopped on four macro inputs: their expansion/resolution contract passed, but
the selected CLI types view additionally required the actual source sizing traits.
No native preflight or timing ran in either failed matrix.

The separately declared complete-provider variants preserve every original macro
fixture byte, request `lang_items`, and add real PointeeSized/MetaSized/Sized
headers. All eight pinned Rust metadata/object observations pass; no native claim
is attached to them. V3 replaces only those four inputs and preserves the other
44 rows, comparison scope and measurement policy. The original fixtures, contracts
and failed preflight observations remain unchanged. The original `contract-v2`
mention of CLI types is a retained inaccurate view hypothesis; its actual passing
candidate API is full expansion plus resolution.

Feature contracts: [local stability](pa31-local-stability.md),
[closed const callbacks](pa31-const-callable-execution.md),
[runtime callback selection](pa31-runtime-const-select.md), and
[callable formation](pa31-source-copy-signatures.md).
