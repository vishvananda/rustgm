# Reached constant selector execution

This checkpoint executes the actual selected const callback of a source-declared
`const_eval_select`. The interpreter transfers evaluated tuple operands through
ordinary function entry and suspends for the selected body when needed. It does
not execute the selector placeholder or demand the unselected runtime callback.
Proved static callable mappings also survive callbacks returned through source
functions.

PA25–PA44 remain planned. No whole original core, alloc or std build passes.
Nonempty callback binders, remaining caller-dependent lifetime entry, nonempty
conditional-F payment, imported callbacks and const-trait member execution remain
explicit boundaries. The later reference-callback and default-defining-context
worker checkpoints are separate and are not included here.

## Functional qualification

Qualified compiler/test snapshot: `a67564e2ebd0875c653c4250db237b1510320155`.
Release CLI SHA-256:
`2af87d0e130e955f0512d37b1691d80ea3b8b34a42de6dc36043635c24946a32`.

The complete fresh release regular/native Cargo run passes **3,488 cases**;
**8,382 PA1–PA24 text cases** pass. Fresh formatting, architecture/file audits
(2,152 Rust files, zero findings), all-target/all-feature strict Clippy, harness,
upstream and backend checks pass. Affected independent source/native suites
PA11–PA14 and PA16–PA24 pass, as do seven transitive-metadata native consumers.
The source and edition measurement adapters link the same qualified release
library and have separate pinned hashes.

| Component | Observed shared-machine wall time |
| --- | ---: |
| Full release regular/native command | 1,690.30 s |
| All active text cases | 18.23 s |
| Affected independent source checks | 307.31 s |
| Affected independent native checks | 456.23 s |
| Transitive-metadata native consumers | 2.67 s |

These are observed costs, not a controlled harness speedup. The functional gate
completed all 60 stages successfully. Its cleanup reclaimed **17.20 GiB** of
unshared generated blocks across 14,740 files and losslessly archived one large
diagnostic. Inputs, diagnostics, receipts and current comparison pins remain.
The scoped development history retains the original Index/E0608 hypothesis
disagreements, the initial lint failure and the mapped-return failure; none is
rewritten as a passing observation. The mapped-return repair consumes the actual
solved static proof, while the independently valid no-static-proof opponent
continues to report its unsupported lifetime context.

Functional evidence: `obj/parallel/ctfe-selector-close-v1`.
Checks SHA-256:
`63b3cdbbf99b5242d3187498afbcc4ec30ea6db2e0805bdb8ddeced8f4599003`.
Functional receipt SHA-256:
`f67a13c89cffa89be4ffa76797c999cf289de270c71fd74460dd12846b46f4e7`.

## Algorithmic contract

The reached adapter authenticates the actual Code, Call, selector, evaluated F,
full instance and owned tuple. Existing common function demand and entry perform
execution; suspension preserves the original destination and fresh site-specific
payments. Warm cached F code never supplies another caller's permission.

Executable publication consumes one owned snapshot prepared after actual region
and loan proof. The existing sparse solved-callable mapping moves into Code;
an empty static solution scans no values and allocates no mapping. Original-only
proof creates no executable Code. Attempted work and retained snapshot payload
remain charged even on failure, and publication validates mode and index storage.
Matching later body headers cannot replace the proved instructions.

Measured hot records remain Body448, Code304, Frame248, State448, Machine4728,
Engine1696 and Value24 bytes. The cold CheckedInput and Prepared records are
320 and 312 bytes. These are record layouts, not whole-heap measurements.
The isolated source controls retain bounded origin indexing, zero warm chain
traversal, no pointer-only origin allocation and fresh payment per reached call.

## Performance qualification

The exclusive matrix freezes **51 inputs** against published compiler `0cfcf32a`
(main `b8fea59e`): **48 byte-identical A/B comparisons** and **three candidate-only
constant-selector widths**. All **27 declared candidate native programs** pass
before measurement. The three new inputs independently pass pinned Rust metadata,
object and C-linked native checks with output `7 1`. The old compiler rejects
these inputs, so they have no A/B speedup claim.

Comparable instruction changes range from **-0.20% to
+1.34%**, with median **+0.18%**.
The largest median RSS increase is **556 KiB**. All
**18** declared growth comparisons pass their individual thresholds;
observed instruction ratios range from **1.10x to 3.64x**.
No instruction, RSS or scaling investigation threshold fires. Ratios include the
separately declared owner-depth and width profiles; they do not prove whole-frontend
linear complexity. Unrelated declaration cost is reported separately in analysis.
The RSS threshold requires both more than 512 KiB and more than 5% growth; the
instruction threshold is 5%. Each scaling pair retains its declared limit.

| Family | Sizes | Candidate instructions |
| --- | --- | --- |
| Reached source payments | 8 / 32 / 128 | 2,886,670 / 5,796,770 / 17,378,436 |
| Closed source callbacks | 8 / 32 / 128 | 6,445,101 / 15,226,131 / 50,606,086 |
| Mapped runtime selectors | 8 / 32 / 128 | 10,098,594 / 26,243,853 / 90,648,668 |
| New constant selectors | 8 / 32 / 128 | 7,757,139 / 16,574,992 / 51,977,009 |

The new constant-selector widths take median **5.77 / 8.83 / 16.64 ms** respectively in
this run. Comparable paired wall-time medians range from **-12.45% to
+9.39%**, with median **-2.00%**. A/A spreads
range from **1.18% to 76.41%**, with median
**7.52%**. These short shared-host observations do
not establish a general wall-time improvement. Warmups, A/A calibration, five
paired ABBA blocks, RSS and Callgrind measurements are retained without favorable
retries; input and output hashes are checked throughout.

Performance evidence: `obj/parallel/ctfe-selector-perf-v1`.
Results SHA-256:
`e1eb28f6e12e30a834a7c8c491331e7e9022132e1721a4c68ffffb08baf8283c`.
Analysis SHA-256:
`bdaaf0d157705e30bf128a2ef345a791288612c3229d3ee8bc5d95b004c3b09b`.

The [source implementation record](pa31-const-select-ctfe-progress.md) contains
the exact source contracts, retained failures, private ownership controls and
remaining callback profiles. Scoped macros and authenticated source-library
metadata remain required before original core and its downstream crate graph.
