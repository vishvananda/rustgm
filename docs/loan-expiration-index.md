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
`171a3bda` are included without changing this compiler snapshot. Publication
checks and performance qualification of this optimization remain pending.
The predeclared comparison has 25 frozen inputs, uses immutable binaries,
pinned-rustc object qualification, output equality, A/A calibration, paired
ABBA blocks, RSS samples and instruction counts. Retain every observation.
Region seed collection and access-conflict scans remain separate costs; this
change does not claim linear whole-frontend complexity.
