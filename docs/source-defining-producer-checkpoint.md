# Defining cleanup and source-provider inspection

This checkpoint checks const-trait default definitions against their actual
original bodies and environments, including live parameter cleanup. It also
collects complete raw source-provider rows from actual Checked/Proved results.
The source descriptor, codec and generic client execution remain disabled.

The qualified compiler/test snapshot is
`fc9f0313b2c9e19decd5ab33d13059afc0fb619b`. It combines defining cleanup
`46b25ef2` with source producer `39d2652f`, integrating the producer's net
prerequisites without replaying older cleanup code. An integration observer
follows the original Clone contract from the collector's Required state into
Prepared and Checked, verifies the same Environment owner, and observes the real
Store/region/loan result. It neither reconstructs a cleanup schedule nor supplies
an artificial proof. Subsequent publication edits affect documentation only.

## Functional qualification

One fresh release Cargo invocation, including ignored native cases, passes
**3,538 tests**. Its log contains all **87 exact required worker cases**.
All **8,382 PA1–PA24 text cases** pass. Formatting, file/architecture audit
(2,203 Rust files, zero findings), strict all-target/all-feature Clippy, harness,
upstream and backend checks pass. The audit retains its existing advisory for
nesting in `reservations.rs`; this is not an audit finding.
Affected independent source/native suites PA11–PA14 and PA16–PA24, seven
transitive-metadata native consumers and both final measurement adapters pass.
All 60 functional stages completed successfully.

| Component | Observed shared-machine wall time |
| --- | ---: |
| Full release regular/native command | 1,827.17 s |
| All active text cases | 19.35 s |
| Affected independent source checks | 250.80 s |
| Affected independent native checks | 411.25 s |
| Transitive-metadata native consumers | 1.97 s |

These are observed costs, not a controlled harness speedup. Functional cleanup
reclaimed **17.20 GiB** across 14,740 generated files and losslessly archived one
large diagnostic. Inputs, diagnostics, receipts and current comparison pins
remain. Retiring an older comparison pin now explicitly requires scoped cleanup:
automatic successful-run cleanup considers only files written during that run.

Evidence: `obj/parallel/defining-source-close-v1`.
Functional receipt SHA-256:
`9700b7c4d1bedbf4ff5b9be0d0a67c5b003070bee1d278bb121a10c460dde0df`.
Checks SHA-256:
`3d40d12970f6294361df837dc7a3277be488fae011e8064b036ab2ed06a62fbb`.
CLI SHA-256:
`b0b81f5cccd3a0b96da56660c2c6709ff01a17877c8cabde4b47f97c91e9e849`.
Cleanup receipt: `obj/cleanup/1792811295023654909.jsonl`, SHA-256
`b4fe4f0d24da263b9a853273be416ca7897f7bbad639f1eb43332f575d2939b3`.

## Representation and work

The defining domain is required only for actual const-trait provided bodies.
Preparation binds the exact function, source, original signature and immutable
Environment, then indexes declared cleanup premises once. The normal region and
loan path checks only active cleanup sites. Moved values and proved dropless
values create no destruction demand. Ordinary Copy and conditional Destruct use
distinct actual role identities; neither grants executable const permission.
Specialization clears the defining-only carrier.

The cold BuiltinRequirements record grows from 48 to 56 bytes. Existing Body,
Contract, Code, Value, Engine, Machine and State records remain unchanged.
Ordinary non-default contracts allocate no defining carrier. Retained worker
controls measure 181/535/1,951 preparation visits for 8/32/128 distinct parameter
subjects and 208/544/1,888 logical retained bytes. Adding 128 unrelated function
definitions at width 8 leaves these preparation counters unchanged. These are
algorithmic counters, not whole-frontend timing or allocator measurements.

Source row collection retains real declaration, body and binder identities,
canonical header prefixes, operation-specific applications and cleanup inputs.
It indexes scope/type graphs once, stages complete groups atomically, and borrows
the actual Checked/Proved result with the same immutable Contract. Completed empty
promotion analysis has no rows to transport; import must still run fresh proof.

## Performance qualification

The exclusive matrix compares published `6b91df39` (compiler `a67564e2`) with the
qualified snapshot above. **All 55 inputs succeed on both compilers**; **54 have
byte-identical output** and one intentionally changed typed-facts view is measured
on the candidate alone. All **31 candidate native programs** pass preflight.
Four new complete-source cleanup width/noise inputs also have independently
qualified pinned-Rust metadata, object and native results. Each performs original
body proofs and full lowering, rather than only rendering type facts.

Comparable instruction changes range from **-0.47% to +1.15%**, with median
**+0.38%**. The largest median RSS increase is **514 KiB (4.62%)**. All
**20 growth checks** pass, with ratios from **1.10x to 3.64x** across the distinct
declared width/depth profiles. No instruction, RSS or scaling threshold fires.
The instruction threshold is 5%; RSS requires both more than 512 KiB and more
than 5% growth. Each scaling pair retains its own declared limit. These samples
do not establish a complexity bound for the whole frontend.

| Cleanup subjects / unrelated functions | Baseline instructions | Candidate instructions | Candidate median wall time |
| --- | ---: | ---: | ---: |
| 8 / 0 | 5,172,894 | 5,206,633 | 8.03 ms |
| 32 / 0 | 11,595,797 | 11,659,417 | 11.01 ms |
| 128 / 0 | 40,363,345 | 40,587,549 | 17.98 ms |
| 8 / 128 | 13,770,849 | 13,880,659 | 7.90 ms |

The two unrelated-declaration comparisons cost 3.43x and 2.67x whole-frontend
instructions. Parsing and checking the extra functions still costs work; only
the separately observed cleanup preparation counters are invariant.

Comparable paired wall-time medians range from **-6.31% to +9.67%**, with median
**+0.17%**. A/A spreads range from **1.05% to 71.59%**, with median **8.80%**.
These short shared-host observations do not establish a general throughput
improvement. Warmups, A/A calibration, five complete paired ABBA blocks, RSS and
Callgrind measurements are retained; input and output hashes remain unchanged.

The first performance attempt stopped at preflight because `provider-originals`
`--emit-types` has legitimately different universal-alias provenance. The new
default-body Self premise is prepared at original body `0:58`, before call
`0:56`; its two universal aliases are consequently allocated at the body instead
of lazily at that first call. Both bound-region equalities and the actual call
source remain intact. This is a source/provenance change, not merely renumbering.
The actual-body environment and original Clone proof controls pass. Both frozen
outputs and the source trace are preserved, without output normalization.

The second declaration explicitly separates baseline success from comparability.
It measures that one candidate view at 2,344,155 instructions and 4.42 ms, with
no paired ratio or speedup claim, and retains all 54 byte-equal comparisons.
The failed first preflight remains under `obj/parallel/defining-source-perf-v1`;
no timing or native stage ran there.

Completed performance evidence: `obj/parallel/defining-source-perf-v2`.
Results SHA-256:
`df510ca61239e15b0d980c95270735ae1eb5894e65d042bcf0ea547e42c196c3`.
Analysis SHA-256:
`0267bd2a73f05c6d8a111b3a7ec63d16c3d9e3ad4ffece17fce39fab21d0f159`.
Provenance review SHA-256:
`749f44940d06dfab827fe78ba58f461b565e27e73196cfe0cdadfdf5ec2d3bfe`.
Performance cleanup removed 58 generated files; receipt
`obj/cleanup/1792813378145265673.jsonl`, SHA-256
`0e4dccdd41d0377858a0d27e0cfc93a54fbb7be63d15be59f05ad3e47962ce69`.

## Remaining work

PA25–PA44 remain planned. Original core, alloc and std do not build yet. The
projected cleanup and reference-callback successors are separate worker drafts.
Scoped macro hygiene, authenticated source-body metadata and fresh client proofs,
remaining active library contracts, platform/runtime integration and full crate
acceptance remain required. This checkpoint grants no imported-original or
const-trait-member execution authority.

The detailed histories remain in
[defining cleanup preparation](pa31-defining-cleanup-preparation.md) and
[source-provider row inspection](pa29-source-producer-checkpoint.md).
