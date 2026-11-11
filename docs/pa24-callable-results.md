# Callable outputs before structural consumers

Original core's IndexRange::fold consumes
`self.try_fold(init, NeverShortCircuit::wrap_mut_2(f)).0`. Frozen ordinary probes
reduce the missing result constraints to generic FnOnce/FnMut/Fn returning a
tuple through `drive(f).0`. Nominal, opaque, const and source-language roles are
unnecessary to reproduce it. Explicit result/type arguments, function pointers
and custom Make::Output controls already pass. PA24 is the earliest owner.
Actual core's const/source FnMut/Try companions remain required.

The [47-case packet](../pa24/tests/spec/callable-results/README.md) declares
35 accepted executable programs and 12 rejections before implementation or oracle
generation. All outcomes independently agree with pinned Rust1.98.0, edition2024,
aborting panics and checked overflow. All35 positives independently execute actual
CHECK assertions. The packet includes all three call traits, tuple/named/array/
callable consumers, associated results, opaque Self/named/free adapters, references,
moves, quantified inputs, mutable forwarding and width8/32/128 with128 unrelated
types. Rejections preserve field/input/result/kind, move, lifetime and quantifier
requirements. No input is silently reclassified as an unsupported negative.

Complete a call's arguments, then reuse the existing owner-indexed argument
refinement to progress only that owner's ready trait requirements. Keep suspended
queries, notifications and full evidence for ordinary final proof. Existing
inference certificates supply readiness without interning or scanning unresolved
suffixes. Inspect that summary once, rather than rescanning wide result prefixes
after every goal. Register projections introduced by selected outputs with the
existing body normalizer before structural consumers. No hot fields, global
cache/type scan, compiler limit, LowIR or backend change is introduced.

The first focused run passed45/47 cases and the width/noise invariant. It exposed
one associated output needing normalization and a tuple out-of-range access that
reported an internal identity error. The corrected run
1794358768672527277 passes7exact tests in121.64s: all47 complete LowIR outcomes,
width/noise, scoped callable assumptions, projected parameter patterns, erased
callables and existing source-closure/actual Iterator bodies. Strict selected
production/CLI/test Clippy, full formatting and architecture pass.

Inference work at widths8/32/128 is6229/21541/82789, trait selections33/105/393;
canonical work stays25. Projection, unsizing and method counters remainzero.
The128 unrelated types add exactlyzero body work at every width. A new private
certificate/no-interning/budget guard remains pending in full CI; the cold private
harness is not rebuilt locally after its known five-minute limit.

Native receipt1794359073388122651 passes all35 new Rust/CPPGM programs in55.37s.
The preceding native batch1794358997144576224 passes both existing native groups
(15 projected-parameter and17 source-closure programs), but fails the new group's
filename-derived crate name. The corrected test removes `.rs` from that name;
no production change or source-fixture change was needed.

PA12 already owns the unchanged `reject/tuple-field.rs` reduction. Its Rust
rejection remains1; tuple access now checks the selected arity before child lookup.
Only that predeclared diagnostic reference may change from an internal identity
error to `tuple field index is out of range`. New PA24 references, complete affected
text checks, provenance review, immutable measurements and a fresh original-core
observation remain pending at this implementation snapshot. No original
core/alloc/std artifact is claimed.
