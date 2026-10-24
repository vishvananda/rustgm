# Scoped macro field identity (LIB-002)

The preserved field draft is now ported onto integration `96103c07` in the
integrator worktree. The original four contracts and all 24 source files remain
unchanged. Their [independent observations](../pa31/tests/spec/declarative-macros/field-observations.json)
now contain 48 metadata/object results and twelve matching native programs.
One original positive prediction is false: `field-functional-update-definition`
rejects with E0451 because its captured base grants no authority for omitted
private fields. The separate complete-explicit update remains accepted.

The first oracle invocation omitted library crate mode and produced E0601 for
every input. It qualifies no field behavior, including negative statuses.
The corrected run retains that failure and changes only the invocation profile
and native driver. The foreign opaque case rejects earlier at an associated
method (E0599); the separately declared unrelated-container case reaches the
intended field boundary (E0609). No original outcome was rewritten.

## Ownership and publication interface

Root owns item/path/import/glob/restricted-visibility lookup and the shared
container adjustment. The integrator now owns the preserved aggregate-field slice while C advances source builtins. Method candidate
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
the optional index handle, from the measured 7032-byte `Declarations` baseline after canonical member matching.
This arithmetic is a declared layout expectation, not an observed measurement.
The existing collection, clone, mutable iterator, iterator and progression
layout controls retain that explicit expectation alongside their hot-row checks.

## Selection and privacy seam

The shared API source `3d0cea1b`, cherry-picked as `623a46bf`, provides
`Report::adjust_item(container: DefId, context: ItemContext, caller: ModuleId)`
returning `ItemAdjustment { context: ItemContext, privacy: ModuleId }`.
The use context comes from `Package::item_context` at the actual identifier span.
The source draft consumes both components in record initialization, record
patterns and field access. Actual container introduction and its captured name context are
independent facts. Captured field tokens do not inherit literal-token privacy.
The shorthand value/binding path remains local-normalized independently of its
field-name path. The pinned privacy visitor uses **base-expression** provenance
for omitted update fields, separately from each explicit field's identifier.
The draft first marks explicit fields, then checks only unmentioned fields using
the actual base node span. A complete explicit initializer transfers no omitted
field and does not demand base privacy adjustment.

Positional field identity remains numeric. Direct `.0` access takes privacy
from that actual numeric token. Tuple patterns preserve the resolved constructor
permission (already the intersection of field visibility), and their extra
semantic guard uses one adjustment from the actual final constructor path
identifier. Typed Path, Applied and Qualified suffixes retain that identifier;
captured child bindings do not supply privacy. No transient field work record
or hot body record grows to carry these spans: completion reads retained syntax.

Root's cache belongs to the existing boxed expansion-mark arena, keyed by the
normalized context and actual container introduction. C adds no cache. Actual
enum variant provenance remains retained even though current variant collection
inherits the enum introduction. No spelling lookup supplies an introduction.

The consumer source is unqualified and does not enable scoped macro expansion.
Shared adjustment and storage preparation alone grant no expansion or access.

## Required controls and measurement

The initial fifteen declared source cases cover literal and captured construction/pattern/access,
shorthand locals, same-spelling captured/literal fields, definition-site versus
caller private reads and updates, actual foreign containers, exact duplicates,
and ordinary record/tuple controls. Independently observed metadata/object/native
outcomes must precede candidate acceptance expectations. Six additional hypotheses
in `field-provenance-contract.json` oppose literal and captured update bases,
constructor paths and numeric field tokens. Two more in
`field-update-explicit-contract.json` distinguish explicit field authority from
omitted transfer authority. The original fifteen predictions remain unchanged;
the original `field-functional-update-definition` predicted success now looks
suspect from the pinned base-expression privacy rule, but has not been observed.
Keep any eventual disagreement as original hypothesis evidence.

`field-container-contract.json` separately declares an unrelated-container
opponent using a captured public factory function name. The earlier
`field-foreign-opaque-denied` also calls literal `Record::new`, so associated
method hygiene could reject it before the intended field boundary. Both inputs
and their original predictions remain intact. The four contracts now declare
24 hypotheses (13 predicted native positives and 11 predicted rejections),
without an oracle invocation or an observed outcome.

Private storage controls will cover root-only no-allocation, exact contextual
lookup, distinct complete identities, wrong variant/owner, duplicate rollback,
mixed root/context publication, tuple context rejection and source-free import
opponents. Export controls must reject contextual fields even when public type
names are root-context. Measure actual `Members` cold growth separately from
unchanged `Field`/`Variant` and hot layouts, plus retained map payload and lookup
work at widths 8/32/128 with an ordinary root control. Four additional private
consumer controls parse actual legacy-generated source, supply opaque context
only on actual retained field-token origins and use recorded real introductions.
They check declaration/initializer/pattern/access FieldId agreement, actual
metadata refusal with a root-named contextual record, unrelated container-context
rejection, and literal versus captured privacy. They are structural controls,
not end-to-end macro2 acceptance or independent Rust observations.

Eventual focused selection includes the four storage controls, four consumer
controls, all five shared adjustment invariants and the five existing cold/hot
layout controls. Candidate static/private/integration qualification remains required after the port.
The independent oracle above does not enable or qualify scoped macro invocation.

## Current evidence and next acceptance

Earlier paragraphs retain the original preimplementation hypotheses; the current
observation supersedes their statements that the corpus is unobserved. Native
outputs cover literal/captured field identity, shorthand, distinct same-spelling
fields, provider-private reads, numeric fields, constructor provenance and
complete versus omitted update fields. Candidate source execution still requires
scoped macro invocation; private typed-consumer controls are an intermediate
check, not a replacement for that final source/native qualification.
