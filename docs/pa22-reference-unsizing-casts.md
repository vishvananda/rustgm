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

The worker delivered the owning PA22 fixtures and manifest separately from
compiler changes. Its checkpoint qualified against the unchanged active PA22
suite; reference generation and installation remained integrator work.

The reviewable payload is
`obj/parallel/pa26/slice-casts/fixture-payload-v2/payload.patch`, with SHA-256
`696362870f7ef81f439f3553f52b6d8b04ddf722bae8d8092010881473e42e5f`.
It adds the 28 sources under `pa22/tests/spec/casts/reference-unsizing` and
their explicit manifest entries, taking the manifest to 356 cases.
All 18 positive cases declare native execution. The receipt retains each
source hash, base/proposed manifest hashes and the independent outcomes.
`git apply --check` succeeded against the unchanged PA22 base.

The integrated default-release compiler at `9314f2ef` independently qualifies
all 356 PA22 sources and all 120 executable/134 compile-only native comparisons.
Its production sources are exactly `b0d85e28`; the later commit corrects only
the volatile test's pointer-registry count after adding the two copy operations.
The compiler binary SHA-256 is
`228f9496bb6d9424bfd2ce89fb70cda6c6a63ef7c80358e827ee4f9df9650a05`.

Evidence in the main checkout's `obj/parallel/pa22-casts-publication-v1/`
records the isolated fixture tree, independent source/native runs and oracle
review. Every new three-stream oracle equals the corresponding independently
qualified frontend output. All 18 positive LowIR outputs equal the exact bytes
compiled and executed by the native checker. The ten negatives reach actual
type, permission, lifetime or loan failures, with no unsupported-feature result
accepted. All 984 earlier sidecars remain byte-for-byte unchanged.

The legacy full receipt lacked per-case contracts, so the complete 356-case
receipt was regenerated after independent checks. The first generation attempt
rejected an evidence directory outside the isolated tree's own `obj/`; its
failure is retained. Correcting that path required no compiler, source, outcome
or oracle edits. The successful generation and exact text comparison are
retained separately, along with the 28-case installation and review receipts.

Remaining PA26 work includes concrete str and structural comparisons, elided
reference-impl lifetime admission, dot-call RHS inference, unsuffixed primitive
assignment-method inference, remaining primitive reference/unary forms, Deref,
iteration, question-mark, derives and formatting. This bounded cast fix does
not replace those requirements or expand PA30.
