# Closed named-record metadata task

This is a bounded implementation plan, not an implemented profile or assignment
completion claim. It can run independently of PA25's associated-initializer
caller/result work. The existing prerequisite is
[member origins](pa29-member-origins.md); implementation must preserve real source
and library nodes, and use `Origin::External(remapped_owner)` for imported fields.

The first family transports closed nongeneric named records used by ordinary
Rust function signatures. Fields may use supported scalars, other supported
records and static shared references. Default Rust representation only is a
reasonable first family: unsupported representation attributes, generics,
Drop/Copy/trait contracts, interior-mutability policies and unsupported field
types must diagnose explicitly. Tuple/unit factories and enum discriminants need
separate typed transport. Missing contracts must never silently become ordinary
records. References with early/late nominal parameters are a subsequent family.

## Typed wire boundary

Preserve exact schema-one profile zero/one/two bytes. Introduce profile three
only when a reachable named-record declaration is present; reject an empty
nominal profile and unknown tags. Header target/language/profile IDs keep their
fixed meanings. Existing scalar values and signature lifetime slot rules remain
unchanged, including unused signature binder slots and valid static outputs.

Proposed neutral records in `support::crates`:

- `RecordRef(u16)`: an artifact-local checked ordinal, never a frontend ID.
- `Record { definition: ExternalDef, public: bool, fields: Box<[RecordField]> }`.
- `RecordField { name: String, ty: TypeRef, public: bool }`, in declaration order.
- `TypeNode::Record(RecordRef)`, retaining a compact copyable graph node.
- A stable `ExternalKind::Record` tag, distinct from functions/constants.

Resolve the exact table placement before implementation: current TypeGraph is
signature-local, while record fields need reusable roots. Profile three can use
one artifact-level type table with ordered function and field roots, leaving the
older signature-local encodings intact. Bound slots remain interpreted against
each function's declared binder; record field roots must contain static regions
only. Record nodes are leaves in the type graph. A separate by-value nominal
edge check rejects impossible storage cycles; reference edges must not be
mistaken for by-value recursion. All definition identities are sorted by their
complete crate identity/path/kind, then explicitly remapped.

Validate lengths, combined table/name/node quotas, depth, duplicate fields,
foreign owners, invalid ordinals, unreachable nodes, malformed roots and exact
canonical ordering before resolver or inference allocation. Decode failures must
leave no partly published interface. Retain malformed/truncated controls and
cross-kind same-path collisions. Do not append local FieldId/VariantId/TypeId,
DefId or SymbolNameId bytes to the artifact.

## Import and export consumers

`resolve::crates` should allocate canonical record DefIds once in sorted external
identity order. Only exported public names enter import scopes; private reachable
layout declarations still retain their own external identity. Source-free records
must not invent syntax or diagnostic NodeKeys. Diagnostics belong at the actual
importing use when no original source location exists.

`sema::crates::type_transport` should seed all canonical nominal TypeIds before
resolving field roots. Publish named record groups through the existing
`Members::insert_external`: external owner, named field identity, completed type,
visibility, variant ordinal zero and stable field ordinals. Reuse retained typed
aggregate/layout preparation once per nominal. Do not reconstruct annotations,
parse source, allocate fake constructor functions or create initializer bodies.

Export planning consumes completed typed member/layout facts after the existing
original/template proof boundary. It must check every reachable nominal's
semantic and representation contracts before producing metadata. Function import
continues through the canonical foreign Registry with its exact signature,
safety, Rust naming/import policy and ordinary shared lowering. No separate
symbol selection or ABI classification engine is needed.

This task does not touch SourceComputation, Target/Requirement, constant-machine
memory or conditional closure proof schemas. It grants no imported const-body
execution permission. Constant metadata remains the existing scalar family;
Image values and relocation fragments are explicitly unsupported for export.
Never serialize raw Image bytes as if symbolic pointers or vtables were scalars.
Checked generic bodies, allocation graphs, generic nominal declarations and
closure contract export remain later PA29 work.

## Declared qualification family

Independently compile provider/client Rust objects before declaring frontend
outcomes. Use two separately compiled libraries with the same record/function
path spellings and duplicate private helper names, then one native client. Cover
record return/pass/field access and moves, private field construction/access and
pattern rejection, provider-created values with private fields, nested records,
static reference fields, reordered dependency input and repeated deterministic
exports. Add non-Copy use-after-move and conflicting nominal identity negatives.
Do not classify unsupported semantic contracts as invalid Rust.

Add native tests as explicitly ignored registered maintainer tests; retain exact
backend pins. Keep scalar/borrowed profile byte fixtures unchanged. Check
8/32/128 records, shared record roots across many functions, and unrelated
ordinary512/scalar-library controls. Charge remapping and layout visits once per
owned declaration/root; no per-function rescans of all imported records. Root
owns activation, oracle review and combined performance/release qualification.
