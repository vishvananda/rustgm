# Selected place sessions belong to one body

Compiler6c4ae3d04 clears body-local selected indexing/dereference/field offsets
and mutable-place marks at fresh function and constant entry. CallTypes already
cleared its method rows there, while Places retained bare NodeIds and old offsets.
A NodeId reused by another file could overwrite the wrong row or panic. Current
body refinement still uses its original actual selected calls.

[PA26's typed state reduction](../pa26/tests/spec/place-session/README.md) reuses
its active shared-index input and models the completed-body boundary. Both exact
private checks1794307955484501906 pass; production1794308086619661798 passes
three existing groups covering mutable outcomes, inherited output and index
captures. Native1794308190203763582 passes21 actual Rust/CPPGM program comparisons.
Strict selected/private Clippy, formatting and architecture pass. Harness314,
467 integration roots/44 stages and unchanged upstream128/Reference126 pass.
The initial function test left an unfinished signature in its synthetic old
state; its corrected seed matches actual finalization. The failed attempt stays
recorded; no output oracle changed.

This joins source method-probe ownerf5d0d07e8, which pays the existing producer
ledger instead of capping a crate-wide observation. PA19's ordinary cumulative
cap and each source probe's local bound stay1000000. Four private checks
1794307159868921493 and three production method-selection checks
1794307307431956788 pass. Thirty source-method A/A+twoABBA observations preserve
complete const/plain method semantics and74 ordinary outcomes, ratios
1.028/1.029/0.996, A/A0.930/0.971/1.005. Thirty joined indexing observations
preserve complete shared/nested LowIR and74 ordinary outcomes, ratios
1.026/1.071/0.981, A/A0.847/0.747/0.960. No throughput claim.

The intermediate immutableCLIeca74fc3 clears the previous method guard but
panics in dot_calls at96.96s; this is an infrastructure failure, not a census.
Final CLI61c983f6ab6fed31f4e4d51f03d5c2e3b88f0cfb555184998fb692f5e2533a86
binds committed6c4ae3d04 and an empty patch. Original722 hashes/profile/compiler
limits remain fixed. Original-core census attempts time out at120.97s and170.75s,
without a table; neither is a passing observation or original artifact.

The observer now accepts an explicit5..170s observation window, default120,
recording it in binding and result. Hard worker190s and local300s feedback
limits stay fixed; this operational window is separate from compiler quotas.
A focused harness test checks parser bounds, outer-deadline clamping and actual
worker provenance. All314 harness tests pass. An interrupted immutable115s GDB
snapshot shows body projection normalization calling trait candidate selection.
This single snapshot is attribution, not proof of a bottleneck. Full body
observation remains pending; use the last completed119-failure table for shared
cause reductions meanwhile. Do not raise the window again to hide a timeout.

Full predecessor42800653535 succeeded every stage. A full joined successor
remains required. Raw evidence is under obj/parallel/source-probe-budget and
obj/parallel/place-session, including both failed census attempts and snapshot.
No backend bridge/contract changes, original artifact or assignment release.
