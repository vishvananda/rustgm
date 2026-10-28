# Original-core offset_of requirement

Combined CLI836e21 passes the prior formatting construction guard and exposes
the next original-core failure: `expected semicolon or end of block` at
core/mem/mod.rs1619, inside the real offset_of macro. The call at
iter/adapters/filter_map.rs101 is `offset_of!(Option<B>, Some.0)`. All722 pins
remain unchanged;371 files open in7.28s. No core artifact exists.

The source macro must stay unchanged. It emits
`const { builtin # offset_of($Container, $($fields)+) }`. Add typed builtin syntax
with original type and field captures. Preserve each selected container,
variant/field ordinal, visibility and substituted type. A variant name alone
has no field offset; pointers/references do not imply autodereferencing.

PA31/source-runtime owns the genuine source syntax/feature boundary; PA28's
intrinsic closure and requirement28.6 own the safe, const-only source intrinsic
`offset_of<T: PointeeSized>(variant: u32, field: u32) -> usize`. PA12 supplies
real aggregate/variant/niche layout; PA18 and PA25 retain generic substitution
and deferred inline-constant execution. Put source-profile fixtures in PA31.

Reuse actual reached-call authentication and checked u32 operands. Existing
aggregate/Layout caches prepare the concrete container once; then indexed
variant/field offsets are constant-time. Return this reached call's actual
offset. The Instance-only size/align result cache cannot distinguish two fields
and must remain unchanged. No new offset-result cache or proof owner is needed.
Existing `InlineInstance` retains generic Option<B> until concrete B is known.

Builtin syntax checks the actual name token's feature authority, including its
genuine macro allowance. Enum offsets separately require the crate's real
offset_of_enum request; do not infer it from a macro allowance. Freeze that
opponent against the pinned compiler before implementation.

The first executable exit includes the exact original macro and generic
Option<B>/Some.0, both nonzero tagged offsets and niche offsets, two fields
sharing one callable Instance, nested/packed/tuple/union paths, capture/privacy
and invalid-ordinal controls. Measure widths1/8/32 and ordinary no-offset
storage. Preserve unsupported DST/static-alignment and ordinary stable macro
routing requirements. A struct-only implementation or zero offset cannot close
the observed case.

Root observation: `obj/parallel/bootstrap-constructors-integration-v1/observed/`.
Read-only primary-source/API plan,38 bound hashes:
`obj/parallel/assignment-integration-review-v1/offset-of-v1/PLAN.md` in public
checkout; index SHA-256
`ef53b7cc11ba692c3bfceac92a2bb13a505aa8ccc3f027c60e096efd18a1222e`.

The [qualified syntax integration](offset-syntax-checkpoint.md) now advances the
unchanged original macro: CLI35835492 opens372 files, then rejects the real
FieldId alternate/zero-padded34-wide LowerHex format. Current-source capture and
hard-gate controls pass; full field selection and reached CTFE remain required.
The exact57-source Rust oracle packet and nine syntax setup controls live under
pa31/tests/spec/source-offset-of, with original disagreements preserved.

The independently qualified `pa31/tests/spec/source-offset-of/provider/` now gives
an executable no_core exit without std println: actual intrinsic bytes and exact
original macro, explicit sizing hierarchy, source Option and same-instance Fields
queries. Eight commands across both editions all pass with `1 4 8 0 0 8 16`.
Use this as the first genuine candidate source/instance/CTFE/native target; preserve
all other57 originals, nine syntax controls and complete DST/runtime requirements.
