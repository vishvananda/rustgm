# RustGM Programming Assignment 19: Traits and methods

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA18. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

## Required implementation

Trait and inherent impls, method lookup, bounds, receiver adjustments.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

Extend `dev/rustgm --emit-semantics -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

## Tests to develop with the reference

autoderef/autoref; qualified calls; bounds; ambiguous methods.

Add spec fixtures, positive/negative controls and cross-feature regressions in
the earliest owning assignment. Use the course reference for text oracles and
pinned rustc for independent language checks. From PA11, execute emitted LowIR
with the configured CPPGM backend. Do not require new future-stage behavior
implicitly. An empty suite or a not-implemented result cannot pass.

## Completion and review

Publish the semantic and lowering boundary, tests/manifest.json and reference
sidecars together. Run all active earlier suites, lint, file/architecture audits,
and affected scaling measurements. Record independent checks before setting
status active in `assignments.json` and adding the stage to the root report.
Student export is a later operation: this repository owns the full solution.

## Maintainer inspection contract (draft)

The maintainer now develops `--emit-semantics` and `--count-semantics` as typed
inspection modes. They run declaration and body inference, including method-name
selection and trait application proofs. Region, initialization, move, loan and
concrete-instance proof still belong to the complete `--emit-lowir` pipeline.
A successful semantic view is not execution authorization. The course release
must pair these inspection fixtures with full-proof LowIR controls and execution.

The proposed version-one text starts with `semantic-crate 1`, followed by the
existing canonical `type` table and typed `def` records. It then reports:

- `associated-owner` and `associated-member`: dense inventory identity, owner kind,
  Self declaration/type, source, safety, trait name and receiver shape.
- `parameter`: declaration owner, binding, canonical argument and optional default.
- `predicate`: owner/source and subject with a builtin, outlives or trait bound.
  Trait bounds include their complete canonical argument list.
- `impl` and `impl-method`: validated trait implementation and its method mapping,
  including whether the selected body is the trait's default.
- `method-call`: source, selected method and callable, receiver dereferences,
  optional borrow type, converted receiver type and inherent/bound/lexical origin.
- `inherent-path` and `trait-path`: selected declaration and explicit/implicit
  qualification, including parameter-bound origin and the written Self source.
- `implementation-use`: selected implementation owner and complete arguments.

The existing region-variable, equality, coercion and outlives rows follow. New
`type-outlives` rows retain their body, source, type and shorter region. Constraints
marked `pending` have not passed the later region/loan proof.

`eof owners=N members=N parameters=N predicates=N impls=N calls=N paths=N impl-uses=N`
closes the view. Count mode emits that same counts line without `eof `. Dense
inventories preserve declaration order. Maps are sorted by declaration identity
or `(file,node)` source identity only at this output boundary. Per-owner vectors
preserve elaboration order. Canonical types, arguments and regions reuse the
existing type-view spelling. Work counters and hash-table iteration order are
excluded. Successful output ends in a newline and has empty stderr; diagnosed
semantic rejection has exit 1 and empty stdout. Output failures are errors.

This is still a planned assignment: the maintainer view does not replace the
required fixture manifest, independent checks, complete handout and release.
