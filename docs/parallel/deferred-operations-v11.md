# Deferred async cleanup operation checkpoint

This checkpoint publishes an authenticated symbolic `AsyncDropInPlace` operation.
It does not publish an executable future or close PA42/43. The public entry is
`SourcePublication::CompilerOperations`. The CLI now exposes the same profile as
`--source-profile module-compiler-operations-v11`; combined ordinary-body/core
publication and actual future execution remain integration tasks. The owning packet and frozen source fingerprints are
`pa43/tests/spec/deferred-compiler-op/predictions.json`.

The new profile uses envelope **13**, version **11**. Source envelope10,
Provided11 and native-link wrapper12 retain their encodings. Version11 alone
adds an explicit optional const-body trailer presence before its new columns.
The failed tag12 draft and ambiguous optional-tail decoder are retained in local
qualification receipts; neither existing format was reassigned.

## Actual producers and consumers

Only the real checked source helper can produce the opcode: completed original
body typing, actual language binding/header, unsafe Rust signature, unsized target
formal, late borrow, complete captures, actual opaque owner/ordinal, unit Output
and declaration/registration seals are required before pre-CFG classification.
`FunctionBody` is exclusively Absent, Original or CompilerOperation. The operation
keeps its actual function BodyRef/origin and one opcode-root diagnostic site,
with no ordinary Original, lexical scopes, locals or CFG. Its written source body
is checked and supplies no executable implementation.

Associated-type headers, projection schemas and direct bounds use the existing
canonical trait/member ledger. `Header.results` now borrows the existing
declaration Input representation with real source/imported proof sites. Opaque
declarations use the existing DeclarationOrigin union. Import reserves exactly
one OpaqueId per actual remapped declaration/ordinal, reused for complete
applications; no AST, closure identity, hidden representation or auto-trait proof
is invented. Future, Output, present poll and other real members authenticate or
fence; optional absence never grants a demanded poll.

The source provider is serialized and decoded before an independent SourceClient
types two genuine calls and proves `Future<Output=()>`. The calls retain distinct
type/lifetime applications of one imported identity. Wrong Output and unsafe
calls reject for their semantic causes. Both cold and cached result-rule reads
authenticate current store/catalog/owner, canonical application, schema and the
complete declared Future/Output relation. The canonical comparison kernel is
shared with source export.

One paid shared `Arc<Lookup>` maps body owners to ordinary rows for sparse
operation profiles. Legacy profiles retain None and dense indexing. The fixture
places the helper before an ordinary function: BodyRef1 maps to ordinary row0.
That ordinary body is reconstructed and proved through the fresh consumer;
the helper cannot acquire an ordinary defining environment or Original.

The first profile permits exactly two opaque captures, a Self-only associated
parent and zero projection own arguments. Those authenticated bounds keep the
new dependency enumeration finite. Arbitrary capture/argument profiles require
the indexed dependency cursor before admission. Wider own/GAT families and
ordinary application/member shapes remain required work.

## Qualification

Worktree baseline: `c509736bb23b7e7918c6c2e04f588c1c7bd130af`.
The first complete fresh-consumer gate was
`obj/qualification/fast/1793863888125691855`: three exact tests, strict Clippy,
format and architecture, 84.93 seconds. The final public/legacy gate
`obj/qualification/fast/1793865195901812583` passes **49 exact tests**: five new
recipe tests, 43 metadata codec controls and the actual v10 source nominal
provider/fresh ownership consumer. It completed in 34.72 seconds with strict
checks. No native future was executed.

`obj/parallel/future-recipe-v1/positive.json` and its stdout pin the actual binary
and diagnostics: incompatible types for wrong Output, unsafe-context requirement
for the unsafe call. Public controls cover native12 wrapping inner13, historical
source/Provided packet byte roundtrips, legacy versions/tags, changed operation
body/owner/future/Output/target, unknown opcode and missing coverage. Source body,
unsafe signature, Output and unmarked authored-async controls retain their real
causes; the authored-async construction fence is not Rust rejection parity.

Actual measured layouts in `layout-baseline.json` and `layout-post.json` under
that evidence directory agree: Kind24, BodyOwner20, FunctionBody8, source Type20,
Body456 and IR Owner20. The cold Header/result Input/element measurement is an
explicit pending private test, not inferred from lookalike types.

Three private selections in `dev/tests/invariants/opclaims.rs` are **execution
pending for CI** under the no-cold-private-link policy:

* `sema::opaque::symbolic_tests::serialized_recipes_reject_ordinal_capture_schema_projection_and_body_forgery`
* `sema::opaque::symbolic_tests::cold_and_warm_result_queries_reject_changed_actual_owner_and_promises`
* `sema::opaque::symbolic_tests::actual_symbolic_body_cannot_supply_an_executable_original`

They cover typed schema/ordinal/arity/frame/depth/early-capture/ownership opponents,
real cold cache epochs and 32 shared warm queries, foreign catalog/source origin,
changed result/captures/owner, stale store, and the actual Original-demand runtime
fence. Final formatting, architecture, harness, upstream and v10 immutable byte
comparison receipts live in `obj/parallel/future-recipe-v1/final/`.
That final batch passes 280 Python harness tests, suite/layout registration,
architecture (3436 files, zero findings) and all pinned upstream checks. The
actual emitted v10 packet is byte-identical to immutable check26: 7118 bytes,
SHA256 `86b0e0a80de3160fbc18973e3ea9cf781513a6206547f55a35b78e2df636f03b`.

## Required continuation

The descriptor grants no constructor, poll, drop, layout, auto-trait, cancellation,
recursive cleanup or executable-body authority. A real future representation and
distinct typed poll/drop scheduler identities remain required, together with
input evaluation/move once, deferred body and pattern effects, actual Pin/Poll/
Output contracts, Ready/completion, initialization/drop ownership, unpolled and
completed Drop, !Unpin/capture auto traits, await/regions/cancellation and recursive
async destruction. Arbitrary authored async export requires its real checked
typed body/ownership data. Actual full-core members and broader imported profiles
must transport or fence; they are not skipped. Core's lack of an `async_drop`
request remains valid and imposes no blanket helper gate.


## CLI integration and original-core frontier

The CLI selects the existing typed producer with full SourceCore library identity,
LowIR and metadata outputs. Every normal option/dependency/ownership guard remains.
The [PA43 packet](../../pa43/tests/spec/compiler-operation-cli/manifest.json)
references eight unchanged, previously qualified inputs by hash and retains the
complete pinned original future/async_drop.rs byte-for-byte. It supplies no new
executable operation or Rust language semantics.

Focused1794662507576101300 passes eight exact tests across linking/frontend in
45.56s, with strict selected/CLI Clippy, full formatting and architecture checks.
The actual CLI file decodes/re-encodes exactly, retains one CompilerOperation and
one separate ordinary body, and supplies a fresh independent typed consumer's
two genuine Future<Output=()> applications. Wrong Output and unsafe calls reject
for their actual causes. Real authored body/signature checks, forgery opponents,
legacy byte packets, native metadata wrapping and placeholder CFG guards pass.
No native future is executed. First test setup used SourceClient --emit-types,
which the existing CLI contract expressly forbids; its exit2 is retained as
invalid invocation, not semantic rejection. The fresh typed API consumer owns
this analysis contract, and no CLI mode restriction was relaxed.

All3963 compiler/test sources and nine manifest inputs hash-check unchanged;
315 harness/495 roots/13 binaries/44 stages/one component and128 upstream+126
Reference checks pass. No old metadata byte format, runtime guard, backend or
quota changes. Raw evidence: obj/parallel/compiler-operation-cli/.

Clean7db0c611e/release27 now reaches the end of unchanged original-core analysis
with zero records:42310 declarations/25858 bodies/35120 item attempts in22.1444s
under the14/20 load gate. Postcheck722/375/six/3963/empty patch and fixed limits
passes. A census still never builds core. The genuine module-bodies-v13 library
attempt fails at async_drop_in_place after19.8253s with no metadata and empty
LowIR: compiler-owned async destruction requires generated future state/polling.
That full-body original command remains a required positive. Exposing this v11
symbolic profile does not discharge it, nor real constructor/poll/drop/Pin/Output,
cancellation, recursive cleanup, native or ordinary authored-future requirements.
A combined body/operation transport must be explicit, authenticated and compatible
with all existing formats; do not silently assign operation authority to v13.
