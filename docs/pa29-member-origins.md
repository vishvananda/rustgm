# Source-free nominal member identities

The member table distinguishes `Origin::Source(NodeKey)`,
`Origin::Library(NodeKey)` and `Origin::External(DefId)`. Source and library
origins preserve their actual diagnostic locations. Only source declarations
enter the existing syntax-node-to-variant index; a library registration at the
same diagnostic location cannot replace that source identity. External members
retain the remapped canonical nominal definition and have no syntax or diagnostic
node. Absence is explicit, rather than a fabricated file/node pair.

`Members::insert_external` publishes one named record field group with ordinal
zero. Every field must carry the same external owner, a named field identity and
its completed type/access facts. Duplicate publication, duplicate field names,
foreign field owners and mixed source/library/external provenance fail before
completed field/variant arrays are appended. The existing field-to-variant and
field-to-ordinal indexes derive `Identity { owner, variant, field }`; wire
transport must remap the owner through `ExternalDef` and retain the ordinals,
never transmit the local `FieldId` or `VariantId`.

Source-only constructor factories and pending discriminant diagnostics require
a real diagnostic origin. They do not invent a source location for an external
record. Record field lookup, construction, patterns, physical layout and owned
field destruction already consume typed selections and field types. Coverage
witnesses can name a source-free record through its canonical definition.

This prerequisite publishes no new metadata wire profile and enables no imported
constructor body, trait implementation, const permission or execution proof.
The next transport slice will cover closed nongeneric named record declarations,
including field visibility and complete layout/storage facts. References in
those declarations are static-only. Tuple/unit factories, enums, generic and
lifetime-parameterized declarations, and unsupported semantic contracts such as
Drop/Copy/trait implementations remain explicit boundaries until their typed
contracts are transported. The PA29 assignment remains planned.

Focused API invariants distinguish source/library indexing, preserve external
owner/field order and access, and reject malformed publication without leaving
completed rows. Existing source, library, constructor, discriminant and layout
controls qualify the provenance migration. Evidence is retained under
`obj/parallel/pa29-member-origins/`; cross-crate source/native qualification
belongs to the following nominal transport checkpoint.

Qualification: all-target/all-feature check, three private provenance invariants
and 23 affected source/layout/constructor/library/discriminant groups pass.
The affected batch explicitly includes both backend-dependent native groups.
Strict all-target/all-feature release Clippy passes. Architecture audit reports
1,397 files and zero findings; layout and upstream verification pass. The first
audit found a seventh `member_*` filename; the invariant was renamed to
`provenance_members.rs` and its tests, lint and audits passed again. The failed
audit remains retained and is not counted as passing.

The final guarded receipt is
`obj/parallel/resources/d8c04d5a80264ff180be5bd0798aa722.json` (exit 0).
Source digest and per-file hashes are retained in
`obj/parallel/pa29-member-origins/source-sha256.json`. Wire profiles zero through
two have no encoding or decoding changes. No cross-crate nominal/native support,
performance neutrality, full release matrix, oracle generation or assignment
activation is claimed by this prerequisite. Inline `Origin` replaces the stored
node pair without adding a separate per-field heap allocation; aggregate member
storage changes still require measurement with the combined transport candidate.
