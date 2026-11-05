# Deferred async cleanup operation checkpoint

This checkpoint publishes an authenticated symbolic `AsyncDropInPlace` operation.
It does not publish an executable future or close PA42/43. The public entry is
`SourcePublication::CompilerOperations`; CLI/core routing remains an integration
task. The owning packet and frozen source fingerprints are
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
