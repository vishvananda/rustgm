# Scoped macro field identity (LIB-002)

This source-only work starts at combined `1df52349`. The independent hypotheses
in `pa31/tests/spec/declarative-macros/field-contract.json` are declared before
execution. They predict nine accepted source/native programs and six rejections;
none has been observed. Any disagreement must retain its original bytes and
expectation. Existing qualified item/path/privacy fixtures do not establish
field semantics. The clean anonymous-selector checkpoint `379006ad` remains on
`pa28-anonymous-select`, outside this branch.

## Ownership and publication interface

Root owns item/path/import/glob/restricted-visibility lookup and the shared
container adjustment. C owns this first aggregate-field slice. Method candidate
retention, associated-member indexes and impl-to-trait matching are a distinct
follow-up coordinated with B's conformance work.

`Field`, `Variant`, `FieldId`, `Name`, downstream operations, `Body`, `Value` and
`Type` keep their existing fields and identities. Ordinary tuple and root named
fields keep the existing `(VariantId, Name)` index. A lazy boxed index retains
only non-root `(VariantId, NameId, ItemContext) -> FieldId` entries. Contexts come
from the actual declaration identifier spans, not field type syntax, container
name context or expansion keywords. Library and external source-free fields
remain root-only until contextual transport is implemented.

Proposed storage entry points are `Members::insert_contextual`, receiving a
source field group plus its aligned declaration contexts, and
`Members::field_in`, receiving an already adjusted item context. They do not
perform adjustment or confer privacy. Missing contextual entries cannot fall
back to root fields. Numeric fields reject non-root contexts. Duplicate checking
uses the complete identity, and failed publication leaves all existing field
and variant indexes intact. The ordinary `insert` and `field` paths remain
available for root-only callers. `has_contextual_fields` supports the whole
provider export frontier without scanning fields on each export.

Both scalar metadata export and authored source export must reject retained
contextual field indexes before creating a descriptor, including when a captured
record name is root-context. Wire names presently carry only spelling, and may
not silently erase a field context. This is rejection of an unsupported metadata
profile, not rejection of the Rust source language.

The storage draft re-exports the existing mark-arena `ItemContext` through
`resolve::model`; semantic code consumes that resolved identity without depending
on the expansion phase. It adds no adjustment cache. Cold `Members` and its
inline `Declarations` owner are predicted to grow by `size_of::<usize>()` for
the optional index handle, from the qualified 7064-byte `Declarations` baseline.
This arithmetic is a declared layout expectation, not an observed measurement.
The existing collection, clone, mutable iterator, iterator and progression
layout controls retain that explicit expectation alongside their hot-row checks.

## Selection and privacy seam (awaiting root's pinned contract)

The reviewed shared signature is
`Report::adjust_item(container: DefId, context: ItemContext, caller: ModuleId)`
returning `ItemAdjustment { context: ItemContext, privacy: ModuleId }`.
The use context comes from `Package::item_context` at the actual identifier span;
functional update uses the actual expression provenance. C will use both
components in record initialization, record patterns and
field access. Actual container introduction and its captured name context are
independent facts. Captured field tokens do not inherit literal-token privacy.
The shorthand value/binding path remains local-normalized independently of its
field-name path. Functional update needs the pinned expression-site privacy
rule for omitted fields; no arbitrary explicit field token substitutes for it.

Root's cache belongs to the existing boxed expansion-mark arena, keyed by the
normalized context and actual container introduction. C adds no cache. Actual
enum variant provenance remains retained even though current variant collection
inherits the enum introduction. No spelling lookup supplies an introduction.

No lookup consumer is enabled until that shared interface is available. Storage
preparation alone grants no scoped macro expansion, field selection or access.

## Required controls and measurement

The declared source cases cover literal and captured construction/pattern/access,
shorthand locals, same-spelling captured/literal fields, definition-site versus
caller private reads and updates, actual foreign containers, exact duplicates,
and ordinary record/tuple controls. Independently observed metadata/object/native
outcomes must precede candidate acceptance expectations.

Private storage controls will cover root-only no-allocation, exact contextual
lookup, distinct complete identities, wrong variant/owner, duplicate rollback,
mixed root/context publication, tuple context rejection and source-free import
opponents. Export controls must reject contextual fields even when public type
names are root-context. Measure actual `Members` cold growth separately from
unchanged `Field`/`Variant` and hot layouts, plus retained map payload and lookup
work at widths 8/32/128 with an ordinary root control. No build, probe or heavy
qualification is queued by this declaration.
