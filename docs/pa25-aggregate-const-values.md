# Aggregate const-value foundation and core dependency

Original core's `TransmuteFrom<Src, const ASSUME: Assume = { Assume::NOTHING }>`
requires a genuine structured const argument. `Assume` is a real four-boolean
struct with source-owned trait implementations, not a compiler bit mask. PA25
owns evaluation/value production; PA21 owns typed generic arguments, PA29 owns
transport and PA31 owns the actual source language bindings. All these parent
requirements remain open.

This checkpoint implements the canonical data representation only. Source
declarations still reject aggregate const-parameter types, the generic evaluator
still publishes scalar arguments, and the source transmutability relation is
unimplemented. Neither core admission nor a whole assignment is completed.

## Representation

`ConstKind::Aggregate` holds a dense canonical node identity alongside existing
scalar and parameter cases. Ordinary scalar bits are never repurposed as object
IDs. A cold optional owner beneath the already optional argument storage retains
typed products and immediate canonical child IDs. Nominal type, enum variant,
field order and scalar type all participate in identity. Nominal field checking
belongs to the semantic producer; these data constructors establish no feature,
`ConstParamTy`, layout, visibility or caller permission.

Parents are appended only after their complete children exist. Interning hashes
immediate IDs and compares complete slices on collisions, so equality reuses
canonical identities rather than recursively expanding a value. A 128-level
binary shared product retains 128 nodes and 256 child edges. Canonical repeated
arrays retain one child even at length one billion. Explicit uniform arrays and
compressed repeats normalize to the same identity; empty arrays do not retain an
irrelevant repeated element.

The producer charges validation, collision probes and prospective capacity
growth before publication. Warm lookups charge work and allocate no additional
payload. Bad identities, sequence types, unclosed types and failed payments
cannot become canonical value nodes. Reusable field slices may survive a later
failed publication after their allocation has been paid; they grant no value or
semantic authority. Node/edge limits remain finite.

The payload belongs to its const-value producer. It is excluded from unrelated
trait-input storage charges and argument-cache invalidations. Existing canonical
constant IDs still participate in their normal store extent. No field is added
to the outer type store, declaration result, body or execution frame. The cold
argument-storage owner gains one optional pointer. Existing scalar APIs remain
scalar; an aggregate cannot masquerade as an array length or scalar VM operand.
Old metadata formats remain closed to aggregate arguments.

## Focused evidence

`obj/qualification/fast/1793720963537416621` passes five exact private tests in
121.09 seconds, with strict private Clippy, formatting and architecture checks.
The controls cover nominal/variant/type identity, uniform and compressed arrays,
deep sharing, bounded warm work, unchanged unrelated trait storage, failed payment
and malformed input. The ordinary-source catalog control remains present.

`obj/qualification/fast/1793721130316649825` passes nine existing product/operand
tests, strict production Clippy and PA21 758/758 in 202.68 seconds. This includes
independent Rust comparisons, generic operand substitution, primitive const
kinds, checked pattern borrows and bounded repeats. Every local batch uses the
existing five-minute guard, two threads, 16 GiB output cap, 150 GiB reserve and
warm-cache cleanup. No release build or broad native sweep is added here.

## Remaining implementation order

1. Canonicalize completed evaluator values using their actual typed field recipes,
   source-owned nominal variant and initialized values. Visit each unique
   `(type, value)` once, preserve repeat compression and pay scratch/retention.
   Do not parse source expressions again or reconstruct values from rendered text.
2. Let semantic/VM/lowering readers consume the canonical graph without repeatedly
   rebuilding its complete product tree. A compact semantic value referring to
   a canonical const ID is the proposed handoff; it is not implemented by this
   checkpoint. Preserve exact storage type, initialization, borrowing and caller
   proofs. Audit the existing `Constant::argument` consumers in semantic queries,
   CFG promotion, the constant machine and native lowering together.
3. Qualify aggregate parameter declarations using the actual feature context and
   source `ConstParamTy`/structural-equality evidence. Separate header type identity
   from delayed implementation/default readiness, including the original forward
   reference from `TransmuteFrom` to `Assume`. Publish evaluated `ConstId` results
   from the existing suspended generic-argument evaluator. Add independently
   checked PA25 source fixtures before enabling the language profile; retain
   missing-feature, fake-marker, changed-field and lifetime opponents.
4. Implement genuine `transmute_trait`/`transmute_opts` declaration authority and
   the coinductive transmutability relation over source/destination layouts,
   validity sets, padding, references and lifetimes under the actual `Assume`
   value. Reinterpreting one value successfully is not proof for all values of a
   source type. Preserve the actual unsafe trait and source default method;
   registering the language-item spellings alone does not close LIB-124.
5. Transport actual nominal declarations, structured constants, generic defaults
   and body inputs through a versioned PA29 profile with deterministic identities
   and fresh consumers. Remap type/value/variant identities once at the boundary;
   old scalar-only formats must not encode aggregate IDs as scalar bits. Original
   core, alloc and std artifacts and all assignment release requirements remain.
