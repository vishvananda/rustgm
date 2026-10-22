# PA26 fixture development

PA26 remains planned. `operator-outcomes.json` declares the draft operator and
shared-index outcomes before reference generation; it is not an executed course
manifest. Its inputs belong to the `spec/operators` family. The maintainer
`language_operators` integration test independently qualifies these outcomes with
the pinned rustc, checks full typed lowering, and runs native controls with both
rustc and the external CPPGM LowIR backend.

The controls distinguish different RHS and result types, generic bounds, aliases,
spelling-only lookalikes, method conformance, moved operands, shared-index writes
and receiver loans during index argument evaluation. Native programs additionally
check operand evaluation order and actual selected method execution.

No text sidecars are published yet. Release requires the remaining PA26 families,
reviewed complete outcomes and references, the cumulative source/native suites,
and the assignment activation reviewed by the integrator.

`library-type-outcomes.json` declares the separate `spec/library-types` family.
The maintainer `library_nominals` test qualifies enum construction and matching,
aliases and namespace shadowing, private fields, conditional Copy and destructor
lifetime obligations. Native controls execute both enum variants and compare
nested `ManuallyDrop` cleanup suppression with ordinary field destruction.
These draft declarations do not activate PA26 or publish text references.

`environment-outcomes.json` declares `spec/environment` controls for option_env's
canonical Option result, macro aliases/shadowing, prelude controls, eager argument
validation, const initializers and rejected syntax contexts. Provider snapshot
and error checks are in `option_environment`; its ignored native test compares
present, absent, shadowed and constant payload programs with pinned rustc and
the external CPPGM backend. These declarations remain draft fixtures.

`dot-method-outcomes.json` declares `spec/dot-methods` controls. The
`language_operators::methods` tests independently qualify source outcomes,
exercise full typed lowering and compare accepted native programs with rustc.
The separate scaling invariant checks cached visibility work and receiver probes.
These tests do not constitute an activated course manifest or text references.

`comparison-outcomes.json` declares the canonical equality family in
`spec/comparison`. The comparison module of `language_operators` independently
qualifies all 52 outcomes and executes 31 accepted controls through both pinned
compilers. Its CPPGM-dependent native test is explicitly ignored by default.
Reference forwarding and derive execution are Rust-valid implementation
boundaries, not language rejection controls. See `docs/pa26-comparisons.md`.

`ordering-outcomes.json` declares 42 `spec/ordering` outcomes for canonical
PartialOrd/Ord, including 28 implemented native programs and one Rust-valid
Ord-helper boundary. `language_operators::ordering` independently qualifies the
source matrix; its explicitly ignored CPPGM native test executes 27 controls.
The no-prelude program also has retained standalone native qualification.
See `docs/pa26-ordering.md` for contracts, evidence and remaining scope.

`ord-helper-outcomes.json` declares 28 `spec/ord-helpers` outcomes: 21 native
programs (three checked failures) and seven rejections. The extrema module of
`language_operators` qualifies the source matrix and its explicitly ignored
native test compares execution with Rust and the external CPPGM backend.
The unchanged earlier `ordering/ord-helpers.rs` control is now accepted/native.
See `docs/pa26-ord-helpers.md`; Ordering's own trait implementations remain pending.


`ordering-impl-outcomes.json` declares the canonical `spec/ordering-impls`
family: 21 implemented positive programs, seven language rejections and two
Rust-valid reference/array boundaries. The ranks module of language_operators
qualifies all 30 source outcomes; its explicitly ignored native test compares
20 selected programs with Rust and CPPGM. The no-prelude program also has
standalone direct-backend qualification. Exact schema/instance and compact
representation controls live beside the existing primitive factory invariants.


`shared-iteration-outcomes.json` declares 26 `spec/iteration/shared-collections`
outcomes: 18 accepted native programs and eight language rejections. The
shared_iterators module of language_operators qualifies the unchanged sources;
its explicitly ignored backend test compares execution with pinned Rust. These
include empty/ZST and non-Copy elements, source/item lifetimes, qualified/reified
lookup, one-time formation and ordinary Drop observers. The Clone-bound case
does not invoke clone; actual `Iter::clone`, mutable/owned collections and ranges
remain required. See `docs/pa26-shared-iteration.md`.
