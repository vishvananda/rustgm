# Failed alias checks pay completed work

Full CI42805641905 failed one real private regression: the independently accepted
PA31 trait-fallback-leak source exhausted source method-query work.3056 other
private cases and all regular targets passed; later native/debug/independent
stages were skipped. A rejected inherent candidate had charged its lifetime
graph's entire allowance, leaving no producer budget for the valid trait fallback.

Compiler67b277950 returns a typed alias-check result with its completed work.
Plain successful and unsatisfied checks retain identical graph constraints and
errors. A semantic rejection pays the work actually performed. Rich graph,
projection-validation and genuine graph-budget failures preserve their former
conservative payment. Both quantified lookup and the closed universal-call
consumer reuse this result. No quota/factor increase, graph/evidence field growth,
proof suppression or backend change.

[PA21's typed reduction](../pa21/tests/spec/binders/probe-failure-work/README.md)
uses the unchanged independently checked PA21/PA31 fallback sources. A plain
rejection has identical nonzero work under allowances1024/32768/1000000.
Private1794311656331795406 passes5 exact checks, including the original CI
failure, exhaustive cycle/universe semantic parity, rich-graph parity and
width/context scaling. Production1794311817699595123 passes3 groups across
lookup, universal-call opposition and all new source upcasts. Selected/private
strict Clippy, format and architecture pass. Harness314/467 roots/44 stages and
upstream128/Reference126 pass; no fixture bytes or text oracle changed.

Source-bound immutable dev-fast CLI
8a653e24bffd5aa638ce8e0e9c8318dfc18673ba9eb9976af4662ef8f32f6081
binds67b277950 and an empty patch. Thirty A/A+twoABBA observations retain complete
ordinary static/correlated fallback semantics and74 ordinary outcomes. Ratios
0.961/1.037/1.011, A/A0.791/0.844/0.939; no timing claim. Evidence and the CI
excerpt are under obj/parallel/alias-query-work. Full joined successor remains
required. Original-core census progress/timeout and actual artifacts remain open.
