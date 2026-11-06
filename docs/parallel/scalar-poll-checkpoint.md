# Genuine scalar future poll checkpoint

An actual authored async function now constructs its future, executes its real
checked scalar body through the required Future::poll member, builds the real
Ready variant, and drops the completed future through the locked backend. The
C observer returns `3`, matching pinned Rust 1.98. This is a finite first runtime
stage, not full async support or a core/alloc/std artifact.

## Actual ownership and protocol

The public constructor retains the actual function/signature and existing
checked construction provenance. The deferred source remains AsyncBody; one
selected AsyncApplication is consumed by the poll transform. Poll has its
distinct FutureEntry key, real required poll Instance, public signature and
separate execution signature. Drop has its own genuine entry and no invented
FunctionItem, Instance, DefId, closure identity or tuple identity.

The callback returns the existing private verifier completion and actual owned
region result. The SAME cleanup seal moves out of prepare/finish, validates the
current arenas, exact borrowed body, Copy inputs and environment, and is
consumed before the body moves. Const completion contains no async seal and
cannot satisfy this boundary. Remaining formation, scoped input and actual
cleanup requirements are fenced. The generated candidate is re-proved before
publication, with its preserved source graph compared to the checked original.

Protocol queries authenticate actual Pin field, Poll/Ready/Pending members,
Context, Future/Output schema and all three public binder slots. New physical
Pin applications use the existing canonical aggregate field producer. Public
poll keeps two logical inputs; the normal RustGM aggregate ABI uses a result
address followed by Pin and Context, with a void native result. No ABI rule or
generic field validation was weakened.

The source Body/Block/Tail scopes and three ordered exit ranges are preserved.
Only their block/instruction locations change, using checked offsets. The
generated state order is initial check, Poisoned, original scalar computation,
actual Ready construction, Complete, then scope exit. A completed second poll
traps before the source segment. The admitted body cannot suspend or unwind.
Actual registered Pin/Poll destructors are rejected through sealed sync/async
registries: wider generated cleanup must run before completion.

The initial profile requires zero input/capture applications and the actual
single scalar u8 return. It does not fabricate an empty body or unit result.
Nonempty/deferred promotions remain fenced; the original may retain its genuine
empty completed promotion analysis, while the selected/generated body has none.

## Qualification

[Exact fingerprints and receipts](scalar-poll-evidence.json) bind the source,
fixtures, gate, actual rlib and locked backend artifacts.

- Gate `1793925179922592390`: 28 exact tests across four targets pass in
  114.09 seconds, including all stateframe, compiler recipe, selected scalar
  dispatch and three atomic registry controls.
- Final qualifier: all-target/all-feature strict Clippy passes, including
  private-test metadata compilation; 29 pinned fixtures and 126 Rust Reference
  files verify. The bounded batch takes 134.58 seconds.
- Actual native Ready observer returns 3. The source with no u8 Clone/Copy
  implementation also returns 3. Independent ordinary owned-byte code passes;
  borrowed-byte code rejects with E0507. The candidate ordinary borrower also
  rejects, so compiler state access grants no user Copy permission.
- Completed second poll rejects in both implementations with no observer
  output: Rust aborts after its completion panic; the candidate traps. Their
  exact signals and unwind machinery are not claimed equivalent.
- Earlier qualified unpolled divergent-body constructor/Drop controls remain
  passing. The new private control obtains the actual moved Assembled permit
  and tests warm/current/foreign-store and forged field/state/Ready/edge cases.
  It is compiled; execution remains pending full CI. No cold private link ran.

Real measured assertions remain Kind 24, Owner 20, Body 456, Builder 1776, Work 40
and ordinary Materialized 744. Cold coroutine permit grows 40 to 48 bytes;
completion grows 24 to 32 and retains the existing 40-byte seal box only for the
async path. Outer factory permit 16 and Bridge 16 remain unchanged. Actual cold
collection/box costs are prepaid. No general compile-time speed claim is made.

Failed attempts and read-only typed observations remain under
`obj/parallel/future-runtime-v1/`. They explain corrected source scopes, missing
physical Pin field preparation, empty original promotion metadata and the
initial incorrect physical ABI assertion. The initial invalid extern literal
spacing was corrected and independently refrozen before candidate observation.
No source outcome was changed to accommodate a compiler rejection.

## Integration and required work

Own changes follow 785061ff3. Its shared-only ancestors mirror Root 31bd9f767 and
5043b4919/c3e9be5ef; Root already has those. Reconcile the narrow native callable
matches, factory permit union and cstage final-function union with Root's
AsyncFn and Source's unfinished initializer split. The same Seal must continue
through that split. No registry or Body/Owner field is added by this checkpoint.

Private invariant execution is pending full CI. The worker harness rerun stays
pending with Root: mocked temporary roots otherwise inherit this worktree's
actual ancestor controller. No harness mutation or storage-policy bypass was
used. Planned PA43 stays planned; existing public roots remain registered and
no student suite is activated.

Still required are wider authored bodies and CFGs, nonempty raw inputs and
captures, input evaluation and deferred parameter-pattern effects, all capture
and output cleanup/drop orders, borrowed capture loans, await/suspension,
cancellation, unwind/panic behavior, recursive async destruction, actual
AsyncDropInPlace helper execution and executable authored-future metadata.
The compiler-owned helper recipe remains symbolic and nonexecutable. None of
these requirements is waived by the scalar stage.
