# Joined scalar polling and original callable consumers

Worker c869bbb55 is joined with the current actual AsyncFn invocation wrappers,
FusedIterator source consumers and checked-batch preparation. Real source async
construction now executes the checked scalar body through actual required
Future::poll, creates Ready and drops the completed state. The actual execution
signature uses the normal indirect aggregate-result ABI. The same source,
region and loan seal is consumed before transformation; generated body proof and
publication remain separate. See the [worker protocol](scalar-poll-checkpoint.md).
No Kind/Owner/Body/Work or ordinary Materialized record grows; cold coroutine
Permit48, Completed32 and boxed Seal40 are measured, not assumed.

## Qualification and CI

[Joined evidence](scalar-poll-join-evidence.json) binds the actual source/fixtures
and receipts. The public gate passes34 exact controls across4 targets with
formatting, architecture and strict Clippy. A persistent ignored native test now
executes Ready3, the same source without user u8 Copy, and completed-poll rejection
through the locked CPPGM backend. Its POSIX C child checks actual SIGILL and
disables core dumps before the intentional trap. Root also reruns all9 real
AsyncFn and2 FusedIterator native observers:14 programs pass in the joined
candidate. No exact panic signal/unwind equivalence with Rust is claimed.

Root harness and upstream checks pass. The worker's older harness attempt set
TMPDIR inside its worktree; mock fixture roots inherited that ancestor's real
controller and two storage tests rejected them. Root's normal temporary-root
run passes all280 tests without changing the harness or resource policy.

Immutable CLI15ac6c1c keeps original core at LIB-171 AsyncIterator, file108:636,
in18.8339s/375opens. All722 original input hashes, six pins, fifteen configuration
values, options/capAllow and finite limits remain unchanged. It still exits1 and
produces no original core/alloc/std artifact. This concurrent wall observation
is not an A/B performance claim. Current/predecessor pins are15ac6c1c/8d4a4ff3;
5fc8c666 is retired after retaining its earlier measurements and receipts.

Full42131050558 is terminal failure on the preceding AsyncFn checkpoint:
2725 private controls pass, one unsupported-vocabulary expectation fails, and
one public destruction-header assertion fails. Both actual lending graph/header
opponents, both completed-pair cache invariants and the old wrong-factory
assertion execute successfully. The vocabulary successor is already466df9660.
Pinned Rust independently accepts the bare const Destruct declaration without
rustc_deny_explicit_impl, so the public assertion now requires that actual legal
header. It does not fabricate attribute policy. Source explicit-impl denial
remains separately checked when the declaration really carries that attribute.
Two additional unused Drop signatures also pass Rust; their mistaken initial
predictions/diagnostics remain retained. They are required Rust-positive gaps in
LIB-172 and required-cases.tsv, while existing demand-profile fences stay named.
No generated reference or expected language result hides a compiler failure.

Private scalar-poll graph/current/foreign-store controls were typechecked by the
worker but await execution in successor full CI. Full release/native/debug/course
and independent qualification remains distinct from these focused results.

## Remaining objective

Original AsyncIterator and later raw source roles remain Root/worker admission
work. Genuine broader async bodies/captures, parameter effects, polling/await,
borrowing, cancellation, unwind, recursive destruction and executable source
metadata remain required. The const cleanup consumer stays isolated: frozen
history tests expose four false acceptances and three positive gaps; its current
scoped endpoint is not safe to integrate before the root-local qualifier repair.
CTFE and shared builtin-query storage accounting remain open. No whole assignment
after24 or original standard-library artifact is claimed complete.
