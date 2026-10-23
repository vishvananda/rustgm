# Executable generic original metadata, first family

This is an implementation checkpoint, not PA29 activation or upstream-library
completion. The five independently declared cases in
`pa29/tests/generic-id-outcomes.json` exercise actual source-built
`pub const fn id<T: Copy>(value: T) -> T`, artifact encode/decode, runtime
instances and imported constant calls. Four additional independently declared
formal-bound cases are in `pa29/tests/generic-formal-outcomes.json`: unused
default Sized rejects `[u8]`, while the paired `?Sized` declaration accepts it. The complete 18-scenario contract in
`pa29/tests/generic-contract-outcomes.json` remains required.

The subsequent safe direct-call family uses a distinct row version, described
with its current implementation and evidence in
[direct-call transport](pa29-direct-call-transport.md). It does not reinterpret
the frozen version-zero bytes below.

## Frozen envelope 9, row version 0

Envelope 9 wraps the existing canonical payload. Its descriptor is exactly
`[0, 0, 0]`: row version zero, defining Rust 2024, legacy language-role policy.
The existing language header must agree. Unknown versions/editions/policies,
empty/duplicate/reordered wrappers and unsupported operations reject. Existing
profiles 0–8, stored arrays and variadic paths retain their encodings. Future
contract-bearing fields require an explicit new version/profile; they cannot
reinterpret version zero.

All table counts and references are explicit little-endian u32 values, tags and
booleans are u8, and options use a checked u8 tag followed by a present u32.
Existing bounded string/path encodings remain unchanged. Fixed table order:

1. Declarations and their parallel site counts: functions with stable external
   paths/visibility, owned type parameters with ordinal/name/Sized flag, and
   actual value-parameter/local declarations with BodyRef/ordinal/name.
2. Binders: declaration owner, optional parent, original/public/clause kind,
   complete slot count. Empty original/public binders remain distinct records.
3. Types: scalar, owned parameter, or function with binder, input-root range,
   result, safety, Rust convention and nonvariadic flag; then root and complete
   formal-parameter tables, including unused slots.
4. Clauses: declaration/site/subject, optional exact quantified frame, and Sized
   or legacy Copy; then functions with declaration/body, original/public type
   roots and frames, parameter/clause ranges, declared-prefix count, const flag.
5. Originals: body/declaration/result/site count, locals, explicit incoming
   parameter-to-local associations, values, statements, blocks, scopes, exits
   and temporary lifetimes. Value operations are Unit, ReadLocal and Constrain;
   terminators are Return, Jump and Unreachable. Storage/evaluation/local-store
   statements use the explicit tags in `codec/originals/bodies.rs`.

There are no serialized frontend IDs, source nodes, epochs, Contracts, Checked,
Proved, CallerId, SourceComputation or execution tokens. Nonzero lifetime binders,
source-defined language roles, user/qualified traits, projections, references in
original signatures, non-Rust callable conventions, calls and additional typed
operations reject in this version. Source Copy/Sized identities cannot become a
name or a legacy builtin bit: an active source-role compiler policy rejects this
legacy envelope. These are required later families, not language errors.

## Import and proof transaction

The resolver reserves genuine declaration kinds and lexical owner scopes, checks
the complete external ordinal-to-definition map against the actual imported
metadata, and installs/finally freezes its catalog. Header ingestion interns one
shared canonical graph, retains the complete formal order and immutable declared
Environment, and installs canonical type cells. Only those authenticated cells
can be header-ready; this never marks an imported body/proof complete. Source
AST maps and function-node tables stay source-only.

One cold `ImportedOriginals` allocation owns the converted Template/Contract
pairs. Optional Rc fields in Engine, Prepared and Templates share it. Local
source templates retain their existing Vec without per-template wrappers.
Operation-derived well-formedness, sizedness, Constrain and Return coercions are
reconstructed independently. Every imported original, including unused rows,
passes normal promotion, eligibility, storage, region and loan verification
before the immutable set is retained. Context success is not call permission.

Every formal retains a mandatory well-formedness goal, and its actual default
Sized flag retains a Sized goal even when that formal is absent from all physical
ports and explicit clauses. Declared Copy/Sized clauses also remain pending builtin goals. Ordinary instance
substitution clears the original assumption Environment and proves the actual
substituted goals. A manually selected non-Copy argument fails before Code
publication; a paid cached instance cannot authorize that argument or another
catalog's coincident IDs. Runtime demand prefers an actual Checked original to
the bodyless foreign-function shortcut. CTFE requires the actual full Instance,
const eligibility, paired original/instance proof and current catalog lineage.

Imported operation diagnostics use validated body sites. Metadata-only failures
without a valid operation location are explicitly Unlocated; no source file zero
or fabricated NodeKey supplies a fallback.

## Bounded work and storage

Codec validation combines old/new table and scratch storage with the 16 MB
metadata bound and one-million logical work bound. Resolver reservation,
canonical header ingestion, original conversion and ordinary proof retain their
separate existing owner budgets. Error work/reservations are charged, and no
partially constructed header/original set is returned.

Header preflight and actual retained payload observations are distinct. Each
validated type node/edge contributes once to an indexed declaration-owner graph
summary during the existing topological import. Scratch rounds deduplicate the
actual graph owners reached by each header's clauses; repeated DAG children do
not expand into tree-size bounds. No flattened per-root closure is retained.
Fresh Environment payload is observed once with the existing retained facade.

At 8/32/128 actual imported declarations, header preflight bytes are
29,928/118,728/473,928; actual header payload bytes are 5,024/19,280/76,304;
header steps are 368/1,472/5,888. Original steps are 176/704/2,816. A shared
25-layer diamond remains 28 nodes/79 edges at 1/32/128 unrelated declarations.
At parameter widths 8/32/128, original preparation is 36/84/276 steps and
4,456/5,896/11,656 bytes, unchanged at 1/8/32 actual uses. Templates borrow the
same body addresses from the Rc owner. These are logical accounting/counter
observations, not whole-compiler performance or heap claims.

Measured layouts on this worker base: Value24, ProofSite12, Body448, Contract184,
Code304; ImportedOriginals72, Engine1688, Prepared368, Templates32. The three
optional imported-owner pointers add 8 bytes each; source-only construction has
None and no imported allocation. No catalog pointer is stored on each Body or
Value. The root's later public/owned Function signature pair must preserve
`execution: None` for imported originals and must not be serialized.

## Focused evidence and remaining requirements

Evidence is under `obj/parallel/pa29-executable-originals/` in the worker:
`generic-codec-fifth` retains the 43 old metadata tests plus five private controls;
`generic-ingest-sixth` establishes real imported runtime/constant execution and
11 private controls; `generic-ingest-eighth/native` passes actual provider
emission and two native clients. `formal-goals-second` passes three public
cases, 16 private controls, strict all-target/all-feature Clippy, audit,
registration and both native clients. `generic-qualified` retains final
diagnostic, layout/work, upstream and 8,362 exact text comparisons.

The independent Rust receipt is
`generic-rows-first/rust-receipts.json`: all five declared outcomes agree, with
both positive clients executed natively.
`unused-formal-before/rust-receipts.json` independently qualifies all four
formal-bound cases. Its pre-fix private failure preserves the demonstrated
manual-instance acceptance of an unused unsized argument; the final tests reject
it on cold and warm requests while accepting the actual relaxed declaration. Failed spelling, masked-readiness,
style and harness receipts remain beside successful runs. The forged unused
Unit-to-T coercion now reaches its actual Imported proof diagnostic, rather than
passing only because declaration readiness failed.

The final debug CLI pin is `generic-qualified/pins/rustgm`, SHA-256
`d0315bf1bfb3cca2e069653ba473160962faa564d5b4d7891216305dc3a6cb19`.
`generic-qualified/evidence-hashes.json` binds the exact runtime/native/Rust and
source-map receipts. The earlier `generic-close` launcher failed containment
before any course test; the corrected run reused the same immutable fixture
archive and passed without oracle changes.
Source hashes and exact commands/binary identities are retained in the receipts.
Worker text qualification uses the immutable `11199b039586240fcbfa7b46d2431cfa593928cb`
fixture snapshot (8,362 comparisons); later public fixtures and combined
release/native/performance qualification remain integrator-owned.

Private helpers/calls, borrowed/early-region results, const generics/arrays,
actual user and source-language traits/impls/effects, transitive generic bodies,
identity collisions, anonymous/conditional outputs, general operations and the
full remaining metadata graph contract remain required. The PA25 original gap
inventory remains in `pa25-completion-gaps.md`. No assignment status, oracle or
wire compatibility claim is inferred from this first executable family.
