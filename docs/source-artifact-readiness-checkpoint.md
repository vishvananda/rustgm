# Source artifacts, early readiness and anonymous-reference callers

Compiler snapshot `fa0f4e08` integrates the qualified A15 artifact/import proof,
LIB-004 early implementation readiness and LIB-005 reference-valued anonymous
constant callers. The original selected-reference program now produces `7 1`;
both API and CLI artifact-to-fresh-consumer programs produce `7`.
PA1–PA24 remain implemented drafts and PA25–PA44 remain planned. No original
upstream core, alloc or std artifact has been built.

## Functional qualification

All fifteen actual Cargo executable inventories run once, including ignored
native tests: 3,852 passes, no skipped cases. All 8,385 active text cases pass.
The inventory includes 1,897 required exact names; those names are a subset,
not extra passes. Formatting, file/architecture audit, all-target/all-feature
strict Clippy, harness, upstream provenance and backend checks pass. Independent
source PA7/8/12/19/20/21, native PA12/19/20/21, transitive metadata and all three
library-bound adapters pass.

The initial command hit its 36 GiB absolute output cap during the independent
PA19 source checks, after all Cargo/text checks completed. Guarded cleanup
retired 14,513 disposable artifacts, reclaiming 22.19 GiB of unshared blocks.
A remaining-only continuation uses the identical CLI and source/fixture hashes;
it repeats no Cargo or text cases and preserves the original failure receipt.
Known native output roots are cleaned between successful targets.

| Artifact | SHA-256 |
| --- | --- |
| CLI | `ae5a59fb22848005555dcc1795d96c8b2deb744bad4c77290e1545ee05585352` |
| Complete functional receipt | `42f848b7afa4450eeb47ddfaa07941f78947890dd962c766f98d960c052f02fd` |
| Functional checks | `376b215323280fac1ba158b518f8c760429a1bcd5255ec49fb1571eb28072651` |
| Full inventory | `b9d6ca8ad2924b45154b74a7efdb762eeaedd30349bf82e05e1f052f59dbe6d9` |

Evidence is in validation `obj/parallel/artifact-readiness-release-v1/` and
`artifact-readiness-continuation-v1/`. The continuation guard ends successfully
at 19,298,578,432 allocated bytes under its 24 GiB absolute cap.

## Contracts and retained requirements

Imported owners retain authenticated source descriptors and actual full-instance
identities. Their absence of a local syntax node is represented explicitly;
no fabricated NodeKey or automatic readiness grants permission. Fresh imported
body proof checks full signatures, inputs, regions, loans and cleanup before
publication/execution. Canonical member conformance and scoped visibility are
preserved. A15's borrowed Scope is 208 bytes; repeated borrowed lookups do not
repeat preparation, and measured typed trait-request/original widths remain
linear. Cold registry growth and retained storage are explicit costs.

Early source Copy readiness distinguishes pending from absent and invalid.
Completed declaration/header proof is retained separately from original-body
checking; a validated header cannot waive an invalid unused body or a value
cycle. The original reference-valued anonymous selector now keeps the actual
caller, selected operand, use identity and checked cached Code/body relationship.
The narrower qualified profile does not grant wider generic/imported anonymous
execution or generic early readiness.

Provided/default source bodies and their nonempty environments still require
separate authenticated transport. Ordinary Functions metadata does not infer
absent attributes, UB policy or source permissions. Primitive inherent coherence,
UB-check preservation, actual attribute consumers, nonempty formatting and
foreign macro transport remain required. See the [blocker tracker](library-build-blockers.md).

## Performance qualification

The frozen comparison against qualified public `ca54609a` checks 130 inputs:
121 byte-equal successful pairs, four supported typed/debug views with reviewed
own-output changes, and five newly supported inputs whose baseline rejects.
The two complete type views are exactly equal after explicit TypeId permutations;
the Rows debug view changes only the root name from `crate` to `minicore`.
Changed views and rejected baselines are excluded from speedup comparisons.
All 260 command observations are retained. No active text oracle changes.

One hundred native checks retain prior evidence only after authenticating equal
LowIR, client, contract, backend bridge and pinned backend bytes. Five actual
early-readiness/anonymous-reference programs execute freshly and pass. The
retained checks are not described as fresh executions.

Thirty-seven predeclared changed-path and ordinary controls receive exclusive
A/A calibration, ABBA wall samples, instruction profiles and original RSS samples.
Thirty-one are comparable. Their instruction changes range from -0.193% to
+4.838%, median +1.291%; none crosses the 5% instruction review threshold.
All eighteen focused growth controls pass. The actual early-readiness 32→128
instruction ratio is 3.085x. These are bounded compiler workloads, not a claim
about whole-core compile time. Every wall/RSS observation remains recorded.

Twenty original RSS changes cross +512 KiB/+5%, ranging from 590 to 1,068 KiB.
A predeclared seven-pair stratified review covers both executable pairs, an
ordinary control, the largest original delta, reached-call/instance extremes,
alias width and a separate reference-caller path. It takes exactly one Massif
profile, one GDB exit mapping and one executable-size observation per A/B/input:
fourteen observations, 42 commands. All original outputs remain exact.

| Input | Recorded heap maximum delta (bytes) | Recorded total maximum delta (bytes) | Exit compiler-file RSS delta (KiB) |
| --- | ---: | ---: | ---: |
| ordinary-scalar | +331 | -688 | +612 |
| library-owner-width-128 | 0 | -192 | +336 |
| payment-repeated-reached-8 | +24 | -896 | +696 |
| payment-repeated-reached-128 | +460 | +40 | +680 |
| payment-distinct-instances-128 | +24 | +40 | +812 |
| opening-subjects-128-noise-0 | +16 | -768 | +728 |
| reference-caller-scaling-8 | +542 | +2200 | +696 |

Massif retains its default 1% peak approximation; small recorded differences are
not precise retained-allocation counts. The CLI's allocated executable text grows
915,472 bytes; the source adapter grows 381,560. Their data/bss also grow. Separate
GDB observations show additional compiler-file residency, with small recorded
heap changes across the selected dimensions. This supports accepting a bounded
resident-code cost for the feature checkpoint. It does not numerically explain
every original sample or establish zero memory regression. All twenty original
triggers remain visible; seven reviewed pairs are not claimed as twenty pairs.

The separate immutable A14→A15 artifact-Scope comparison retains 160 timed
observations plus eight warmups across four workloads, with exact LowIR/metadata
outputs. Median CPU increases are 8.92% cold, 2.18% warm64, 5.36% distinct and
8.33% ordinary-control. A fixed eight Callgrind/eight Massif follow-up measures
instruction increases of 0.517%, 0.150%, 0.915% and 0.00535%, respectively. The
one-imported-instance paths add about 82,000 instructions; the two-instance path
adds 162,492. Recorded heap maximum deltas are +80/+484/+80/+1704 bytes, with
Massif's approximation retained. No timing or RSS sample is rerun or replaced.
The ordinary timing increase's cause remains unproved; simulated cache counters
do not establish it. Retain these timing costs without a blanket nonregression
verdict or an unsupported claim of equivalent added semantic work.

Evidence: validation `artifact-readiness-perf-v2/` retains the strict initial
output stop; `artifact-readiness-perf-v4/` holds the reviewed preflight and all
measurements; `artifact-readiness-memory-v1/` holds bounded memory attribution.
Worker A's `lib007/scope-perf-attribution-handoff-v1.json` joins the exact original
timing inputs, profiles and limitations. Earlier setup/path failures are retained
separately, with no compiler case or timing repeated to hide them.

| Artifact | SHA-256 |
| --- | --- |
| Performance results | `4740cb8c74a440182477749a396eafdaa750d847ed9e97af49a452b89f9cfbcd` |
| Performance analysis | `c62f6eac90b54d7c1a6a703c59ff1d681662ee0e9c2eb5da4219ffc667414a5f` |
| Memory results | `94fc8d0f2091e72fce409bd8032109273ff556e18b1aac5fa7f5f04be0a8c6ee` |
| Memory analysis | `eafe7b7d8883736db6ea23462b5c730535fed63586b2b6a7ca56ce61a076f62f` |
| A14→A15 attribution handoff | `a4566d9aa1d402fb82bcfb7c12bb492710193ed65698c397459c7e044e67d8ef` |
