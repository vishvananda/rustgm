# Sparse loan expiration

The capture scaling profile at compiler snapshot `68d3a32e` showed repeated
loan pruning inside the borrow proof. The precise-capture input grew from
10.857 million instructions at width 32 to 70.396 million at width 128.
This change indexes region exits once, then removes only the loan masks that
expire at each consecutive point. It does not change capture semantics.

The index consumes retained sparse region words, groups loans by region and
word, and records only live-to-dead transitions. Full extents need no events.
Block entries, joins, jumps and unvalidated births retain the previous exact
membership check. Regeneration and two-phase activation remain separate from
region liveness. Construction charges the existing loan work budget and caps
retained event entries at 500,000.

Three new private invariants compare against independent point sets across
holes, word boundaries, jumps, joins, regeneration and activation; check full
extents and final points; and bound continuous-region index/pruning work at
widths 8, 32, 128 and 512. The existing occupied-state invariant also passes.
The corrected focused gate passed strict release all-target/all-feature Clippy,
the file audit (1,334 files, zero findings), layout, eight borrow/closure targets,
selected native groups and all active PA1–23 byte comparisons. Evidence is in
`obj/parallel/loan-expirations/gate-v2/complete.json`; it retains the exact
source map and compiler hash. An earlier audit rejected a duplicate generic
test filename; its failed receipt remains beside the corrected gate.

The independently reviewed zero-repeat oracle and test corrections in
`171a3bda` are included without changing this compiler snapshot. Combined
publication checks remain pending. The completed performance comparison uses
25 predeclared frozen inputs, immutable binaries, pinned-rustc object qualification,
output equality, A/A calibration, five paired ABBA blocks, RSS samples and
instruction counts. Every observation is retained in
`obj/parallel/loan-expirations/measurement-v1/`.

The baseline hash is
`76a94721c579c2e008d0c6b1d3b873c0fb96058d975292cc608c709781c3bff2`
and the candidate hash is
`7b0f40c9984b545fa81c441da0e98ee7c1dc7913b2e2bd4830e9d42d26eb7d9f`.
The candidate is the isolated optimization at `a06d873e`, before later closure
output, nested capture and atomic compare-exchange integrations. All 24 accepted
inputs have byte-identical output. The independently valid live-borrows input at
width 512 exceeds the existing value-liveness seed budget in both binaries;
`unsupported.json` records that unmet input separately, with both diagnostics.

| Input | Instruction change | Paired wall median | A/A spread |
| --- | ---: | ---: | ---: |
| Precise captures, 32 | -8.90% | +4.68% | 3.26% |
| Precise captures, 128 | -20.73% | -11.90% | 3.09% |
| Precise captures, 512 | -29.92% | -26.88% | 2.20% |
| Live borrows, 32 | -23.83% | -7.91% | 6.38% |
| Live borrows, 128 | -30.23% | -26.17% | 1.59% |
| Ordinary, 8 | -0.10% | +10.90% | 1.60% |
| Ordinary, 128 | +0.11% | +1.49% | 5.68% |
| Shared reborrows, 512 | +0.01% | +0.80% | 4.91% |

The study demonstrates reduced work and time for the large expiration-heavy
inputs, with mixed wall-time results elsewhere. Several small inputs slowed
beyond their A/A spread despite nearly unchanged instruction counts; this run
does not establish wall-time neutrality for ordinary code. Other families'
instruction changes range from -0.43% to +0.68%. Median RSS differences across
the accepted matrix range from -674 to +432 KiB. The exclusive resource lease
excluded other RustGM jobs during measurement, not unrelated host activity.
Region seed collection and access-conflict scans remain separate costs; this
change does not claim linear whole-frontend complexity.
