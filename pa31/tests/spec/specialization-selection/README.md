# Specialization selection

Nine no_core source-core libraries for rustc 1.98's `min_specialization`
selection rule. Rustc's winnowing drops an applicable impl candidate that
another applicable candidate specializes, so the most specific impl is
selected without a finished graph. `assoc_def` then reads an associated type
that the selected impl omits from its nearest defining ancestor, with the
arguments translated through each specializing relation.

The three positive controls follow original core. `step-by-inherited` is the
`StepBy` shape: a concrete `Range<u8>` child omits `type Item` and names
`Self::Item` in its own signatures, while a generic `I` sees only the parent.
`fuse-marker` is the `Fuse` shape, with a child conditioned on an unsafe
specialization marker (`FusedIterator`). `zip-chain` is the `Zip` shape, with
a three-level chain over two specialization traits.

The six opponents are an overlap whose where-clauses (`T: A`, `T: B`) do not
imply each other (E0119), an override of a member that its parent does not
mark `default` (E0520), a child that adds an ordinary trait bound and so is not
always applicable ("cannot specialize on trait"), an override of the parent's
final associated type (E0520), an inherited type used at the wrong type
(E0308), and `default type`, which needs full `specialization` (E0658).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
implementation change. The fixture amendment, which oracle v1 required for the
positive controls, was made before oracle v2 ran. Both are recorded in
`outcomes.json`.

The lang-item prefix is an exact pinned Rust core fragment from
`spec/fn-sugar-impl-parameters/`, taken from the original core `marker.rs` and
`clone.rs`. The case bodies are reduced from the original core
`iter/adapters/step_by.rs`, `fuse.rs`, `zip.rs` and `iter/traits/marker.rs`.
They keep Rust's MIT/Apache-2.0 attribution; see the root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/candidate_winnowing.rs` checks every case.
