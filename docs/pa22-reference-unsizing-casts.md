# Array-reference to slice coercion casts

This follow-up closes the Rust-positive explicit array-reference to slice `as`
gap recorded by the slice-length checkpoint. It uses the existing ordinary
reference coercion and metadata path. There is no new Cast enum, pointer cast
engine, library identity, source spelling check or lowering permission.

Preparation recognizes actual Reference<Array/ArrayTerm> to Reference<Slice>
shapes, with ArrayInfer used only during candidate discovery. It runs the
ordinary coercion, retaining element and lifetime relations, source mutability,
reborrow selection and metadata. Validation then requires the solved operand
type to equal the annotated target and checks the exact retained adjustment on
the actual unadjusted array-reference child. Group/Attributed wrappers already
propagate this adjustment to their child. A bounded walk through only those
transparent wrappers, charged to the existing reference-work budget, preserves
that selection without evaluating the operand again. Incorrect child targets,
kinds and raw types fail the private validator controls.

The ordinary Builder applies the selected operand adjustment before the
existing Cast::Pointer. That cast already lowers through Constrain and forwards
constant pointer values without inventing metadata. Shared/mutable permission,
covariance/invariance, lifetime formation and normal region/ownership/loan checks
remain independent. ArrayTerm specialization preserves its concrete bound;
slice contents and match-based length controls need no library method.

The independently declared matrix has 28 cases: 18 accepts and 10 rejects.
Pinned Rust object generation qualifies concrete/shared/mutable/mutable-to-shared,
generic/alias, empty/ZST, constant/generic-constant, non-Copy, evaluation-once and
reborrow positives. Opposing element, permission, lifetime, invariance, nested,
reverse and active-loan controls reject. Grouped positives and grouped lifetime
or permission negatives are separately qualified. CPPGM-native programs compare
actual contents, mutation, length patterns and evaluation count with Rust.

The unchanged two original length/pattern programs are now retained as
`pa26/tests/spec/slice-length/{byte-array-cast-pattern,empty-byte-array-cast-pattern}.rs`,
with the original hashes recorded in the length checkpoint. Their latest feature
is PA26 len; the isolated match-based cast matrix belongs at PA22. They are
positive source/native controls rather than substitutions for the earlier failed
programs.

Receipts under `obj/parallel/pa26/slice-casts` retain independent declarations and
all failed observations. First-v2 exposed a validator reading the already
adjusted type as its raw input; second-v3 corrected that and passed all initial
24 source/16 native cases. Declared-v2 independently qualifies four wrapper
controls and retains their before-fix frontend outcomes. Third-v5 passes all
28 source/18 native cases, two nonzero private controls and library Clippy.
The private layout test confirms unchanged TypeId 4, Kind 24, Body Value 24 and
NodeFacts 80 sizes. No record or enum grows.

Finish-v6 retained a filename-stem audit failure; the new maintainer table was
renamed into its owning coercion_casts module. Finish-v7 then passed strict
release all-target/all-feature Clippy, audit 1538/0, layout, pinned upstream,
all four old/new pointer-cast groups including their native runs, and all three
length groups (51 source cases, 32 natives and six retained rest positives).
Both original byte-cast sources now pass with unchanged bytes and hashes.
Active PA1–24 text is 8253/8255 exact, including all 328 PA22 and all 467
published PA24 cases. Only the two already-known base PA20 owner failures
remain; root c47a8e88 independently repairs them. No old cast diagnostic or
oracle changed. The final binary SHA-256 is
`0eabab4f84ef27d6d59c9f145d1052f45d077ac209213c804de52d3bf0a7c9c3`;
finish-v7/source-map.json retains production file hashes with digest
`520daa19899ad7f843547087b6a5abb8d154c94545989528c4961ff28cd78507`.
The final guard closed successfully and cleaned disposable outputs.

The owning PA22 fixture
and manifest payload is separate: its declared sources and outcomes will be
integrated with root-reviewed generated sidecars and provenance. The compiler
checkpoint qualifies against the unchanged active PA22 suite. Neither payload
layout nor new text oracles is claimed complete here, and no references or
assignment activation are changed by the worker.

The reviewable payload is
`obj/parallel/pa26/slice-casts/fixture-payload-v2/payload.patch`, with SHA-256
`696362870f7ef81f439f3553f52b6d8b04ddf722bae8d8092010881473e42e5f`.
It adds the 28 sources under `pa22/tests/spec/casts/reference-unsizing` and
their explicit manifest entries, taking the proposed manifest to 356 cases.
All 18 positive cases declare native execution. The receipt retains each
source hash, base/proposed manifest hashes and the independent outcomes.
`git apply --check` succeeds against the unchanged PA22 base. This is not a
claim that the missing generated sidecars or reference receipt are ready.

Remaining PA26 work includes concrete str and structural comparisons, elided
reference-impl lifetime admission, dot-call RHS inference, unsuffixed primitive
assignment-method inference, remaining primitive reference/unary forms, Deref,
iteration, question-mark, derives and formatting. This bounded cast fix does
not replace those requirements or expand PA30.
