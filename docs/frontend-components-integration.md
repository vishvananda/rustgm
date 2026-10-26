# Integrated source writer and specialization verifier

This integrates the private parsed-expression writer and the real specialization
verifier with the already-qualified Self producer and fresh consumer constructor.
It does not enable source Assert, specialization execution, imported Provided
bodies or typed alias expansion. No original core/alloc/std artifact exists.

The [writer checkpoint](source-assert-implementation.md) borrows actual typed
arenas and opaque fragments, emits a bounded document, then computes selected
widths and box/break layout without reparsing source or diagnostic text. The
private Assert condition can supply its default message through that path.
Its tested spelling/layout coverage is not an exhaustive compatibility claim
for Rust pretty printing, and the actual panic/operator/runtime consumers remain
required. Ordinary compilation does not invoke the staged writer.

The [verifier checkpoint](specialization-graph-design.md) now runs real overlap
comparison, declaration implication, forest closure, Min rules, source marker
authority, complete conformance and immediate-boundary finality. Its graph owns
the actual source/arena identities and cumulative shared budget. Unsupported
overlap execution still fails before analyzed facts or export can escape.
SelectedMethod dispatch, inherited-body conditions, both RPIT consumers and
native behavior remain required; a verified forest is not execution evidence.

The shared algorithms avoid repeated whole-graph work. Capped multiplicity uses
one indexed canonical DAG and topological path counts rather than a traversal
per parameter or a parameter map per node. Ordinary Sole shares the same edge
enumerator. Typed structural well-formedness shares the nominal traversal while
preserving actual Sized and outlives rules. A failed counted traversal rolls back
only its newly visited keys, retaining earlier successes and cumulative debt.
Environment substitution and lazy assumption indexing prepay actual collection,
Arc and fingerprint growth; an index publishes only after success. Ordinary
wrappers and hot type/shape/environment fields remain unchanged.

Root independently authenticates the writer run40256288761 and verifier
run40256578419 archives and every2669/2689 source fingerprint against their
qualified commits. The shared Self run40256280863 and constructor40253944977
remain separate component evidence. No dirty successor code is copied.

[Fresh integrated run40259432326](https://github.com/vishvananda/rustgm/actions/runs/40259432326)
passes six exact groups: full Self formals, genuine consumer ownership, both new
writer groups, counted formation/failure recovery and the complete verifier
group. Both strict Clippy modes, the global architecture audit, formatting,
metadata and harness/upstream checks pass. The helper takes108.91 seconds,
including an unoptimized test build; actual tests take0.032 seconds. It runs no
release, native, full private or course-text sweep. Unchanged component controls
are retained from their recorded runs rather than labelled freshly executed.

All2700 source fingerprints match before/after, and no formatting delta needs
importing. Input source is `da23d9ba650da136f13c7a68c8657febaf455ad8`.
Receipt SHA is `2968447d760ebc8ed7c328af82114a6580625496c0b8a86829511469724be850`;
binding SHA is `cf3f1e905e8aea8e03e416bd1be2f2b9e8c741e61f97a48545755a2c2f6fad39`.
No local compiler build, formatter, probe or executable hydration was used.
