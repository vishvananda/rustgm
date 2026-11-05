# Async signature checkpoint, 2026-11-05

This finite component implements genuine source Future/Output declaration
production, async opaque result identity, Output equality and lifetime/type/const
capture formation. It does not release PA31/42/43 or complete original core,
alloc or std. The lane began at plan63e06168c over published63f9177dc.

Source roles complete through the existing catalog ledger. Actual async use
requires Future's genuine Output owner and projection schema. Rust-valid unused
missing/alternate Output owners and arbitrary ordinary poll declarations remain
valid. Declaration completion supplies no implicit polling ABI or execution.

Each async signature publishes the existing opaque identity with real Future and
Output equality clauses. Captures reuse the existing declaration-order machinery,
including Self. The existing body_results map retains the authored output;
consumers authenticate the original async flag, function and opaque owner.
Required trait methods retain absent bodies. Ordinary hidden-result inference
excludes only the authenticated final async future, preserving a nested authored
RPIT's real inferred type. No future representation or helper body is fabricated.
Async CFG demand explicitly fails before publishing an eager ordinary body; its
Prepared construction/reuse counters remain unchanged.

Two separately owned shared predecessors are required:62d61a38d's identity-mapped
associated-definition substitution for captured owners, and Root's normalized
input proof short-circuit before opaque shape lookup. The latter's source SHA-256
is d8786df65f0a35ad34361b9e6e20baa00b6e652de7401df6cfc32f584a6b1288.
It preserves input proof and trait-result flags. The own frontend handoff is
integrated after those shared changes.

## Contracts and independent evidence

The registered async_signatures and future_source suites cover39 contracts:
14 exact no_core Future/source-RPIT cases,22 async cases with explicit standard
core companions, and3 exact no_core required unsized declarations. All22 accepted
and17 rejected outcomes match the pinned independent acceptance observations.
The async companions preserve each authored test body byte-for-byte; nested RPIT
uses real source Copy/Clone declarations in the source prelude. Required trait
methods, actual impl conformance, Output mismatch, generic/const identity,
borrowed captures, invalid unused bodies and const/unsafe/foreign controls are
included. A public integrity test binds the paired bodies.

The initial exact no_core free-body attempt encountered Rust's real ResumeTy
prerequisite. That failed oracle receipt remains under
obj/parallel/async-signatures/oracle/receipt.json. Its missing-role rejections are
not semantic parity. Bodyless async trait consumers also independently accept in
the exact no_core profile. The completed-profile, required-rpit, by-value and
nested-output subdirectories retain the independent commands, hashes and stderr.

A second frozen prediction was corrected: Rust accepts required async fn()->str
even with Sized Output, emitting opaque_hidden_inferred_bound as an advisory
warning. Ordinary fn()->str and genuine Output:?Sized alternatives also accept.
The original1/0/0 prediction and actual0/0/0 receipt remain in sized-predictions.md,
sized-cases.tsv and oracle/sized-output. reviewed-sized-cases.tsv records the
reviewed acceptance contract. No stricter rejection was added; the public test
also checks that the actual Sized schema remains intact. That advisory warning
producer remains required PA40 work; acceptance is not stderr parity.

## Final focused qualification

The command was:

```sh
make fastcheck FAST_ARGS='async_signatures future_source pinning_contracts::actual_pin_roles_keep_source_body_and_ordinary_ownership_contracts'
```

Final gate1793839491003219718 passes11 exact public tests in30.41s: metadata,
formatting, the complete3400-file architecture audit with zero findings, strict
selected-target/CLI all/pedantic Clippy, public execution and the existing25-case
Pin control. Its predecessor1793839104245927490 passed10 tests in79.55s before
the three accepted unsized alternatives were added. No private harness or release
build was performed. The private unsupported-role control's exact Future,
FutureOutput and FuturePoll exceptions await ordinary integrated/private CI.

Final receipt SHA-256:
6adb35b371d7c5ac12d8ab818d69f1eb175f56ba1000ab5b9742674545e28a60.
The receipt's complete compiler-source map has aggregate SHA-256
51d36341c1ec287d0901ab0ebb10cb92dfe47f980f48b9dbd374ca69cebc729a.
Aggregation sorts UTF-8 paths and hashes path, NUL, lowercase content SHA-256,
newline. Fixture maps use the same recipe and are retained in
obj/parallel/async-signatures/fixture-fingerprints.json:

- PA31 future-source packet: fd51cdef8606b7eb32fe8f292c510e9833e65a3c25e91e81665ec6caf7b1f1af.
- PA42 async-signatures packet:4ae49caf91ad92e8fc51fb9ed84d05eb814542ddd732b4440f1a0498b9bffa0a.

Guarded make test-harness test-upstream also passes:279 Python checks,406 roots
in13 binaries,44-stage layout,28 pinned upstream fixtures and126 Reference files.

Earlier failures remain under obj/qualification/fast:139.43s exposed a101-line
function,138.10s exposed an invalid public-test access to a private field,
221.15s exposed an unconstrained outer-future hidden inference term, and96.48s,
29.85s and86.99s isolated the shared required-RPIT normalization predecessor.
The corresponding fixes use a cohesive result preparation helper, public typed
body facts, precise final-future exclusion, and the separately owned shared
proof repair. Guarded read-only gdb traces retain the actual erased inference
error without production diagnostic changes. Initial audit-venv setup, zero-wait
admission refusals and one premature protected-format overlap also remain
recorded. No resource limit or protection was bypassed.

## Native evidence and source pins

The exact authored source Future::Output projection consumer emits ordinary
LowIR and returns7 through both rustc and the old locked CPPGM object backend,
using the same independent C caller. The final frontend gate regenerates identical
LowIR bytes, SHA-256
42f061c6f170fa5ea9a673ec4555a43befb405b1c7878bf78bc3231cccd6bd1d.
This is an ordinary source projection observer, not async runtime execution.
Native receipt obj/parallel/async-signatures/native/receipt.json has SHA-256
9e1b3fd45d1e67c4161404fd1f0bb3b14363e52f0dc564b61ef7c1078403dc87;
it records source, caller, commands, object and executable hashes. Binaries and
objects were pruned after successful observation.

The native evidence uses backend905bd7e4412a2d77d88694fe6abc7f3e1c27bce1,
lock SHA-2566d300e6addfcd7050afef7940e8511f946f8113ad91e82901ab3f499c61757aa,
and object tool ded1b6800ce1bdaeaad45cd28c2903d7c96de7cdc29261f6834869a6eaf525fc.
Root owns the subsequently requested backend update; this lane changes no pins,
lock, bridge or LowIR contract and makes no new-backend qualification claim.

Pinned upstream source root is rust-1.98.0-88d9e12ae178fab0fb5cc050a94da85685d449ea
under /home/vishvananda/rustgm-sources. Inspected future/future.rs SHA-256 is
0220631d95ab45f4b769911df75da8d5542eae1eacb771aa9164c48ddb452bcf;
future/async_drop.rs is
7a2ea6e4131f0df0c6bc6be75428b99de7b4e1b2de61b04fa1e3f562fdd0c83c.
No upstream source was changed or skipped.

## Required remaining work

Async CFG ownership, regions/loans, state-machine construction, suspension
liveness, stored locals, readiness, polling, await/async-block transformations,
pinning, cancellation and destruction remain required. Signature/body type
checking is not a completed CFG, lifetime, loan or runtime proof. Wider Future
and Output obligations must be discharged by the real generated implementation;
ordinary declaration acceptance supplies no invented poll kernel or auto-trait
proof. Advisory diagnostics and full integrated performance/release gates remain.

Source/imported Future protocol and async original body/value/API metadata need
authentic transport and fresh consumers. Original AsyncDrop, the real
compiler-owned async_drop_in_place implementation and complete original
core/alloc/std artifacts remain required. This isolated component does not claim
to clear unchanged original core's LIB-154 boundary. The full assignment and
original-library goal is preserved.

## Joined main qualification

The integrator joins this implementation with the published Deref/receiver and
required-RPIT repairs plus the current CPPGM backend. Both central merge
conflicts preserve the union of genuine source roles; neither predecessor
repair is duplicated. Three additional authored PA43 controls compose an actual
`Pin<&mut Self>` required async receiver with source Future/Output: the positive
passes, while missing LegacyReceiver and wrong Self reject. Pinned Rust confirms
all three before candidate observation. The positive retains actual Self and
lifetime captures, required-method/body-output identity and hidden=None.
This is an ordinary authored trait composition, not AsyncDrop role/runtime
admission or an edited original-core source.

The final combined gate1793842781823360234 passes17 exact public tests across
three targets plus1193 PA19 cases in65.81s, with formatting, architecture and
strict selected Clippy. It includes all worker source-signature matrices,
Deref/receiver, full PA21 opaque/region opponents and the exact-carrier control.
The actual ordinary Future/Output observer again matches pinned Rust through
new CPPGM b7c6b8e72, printing7. Ordinary/value LowIR and all15 configuration values
remain byte-identical to the backend integration baseline.

Unchanged original core still stops at AsyncDrop, file246 byte1138, after
26.2730s/375 source opens with all722 inputs and existing options/cap/finite limits.
That concurrent development observation is not an isolated performance estimate.
No state machine, poll/await, async loan/cancellation/destruction, compiler-replaced
helper, wider generated Future proof, authenticated metadata or library artifact
is supplied by this signature checkpoint.

Joined CLI SHA256: `e551c9670df664b82bcd0fc78a1d238a9392040db9292d72d49034b8c482d261`. Combined gate receipt SHA256:
`7051ca831867975d7f4b76af589bbe871765847f8c4ef8b24e94b16dca3f4b2b`. Native/config/original-core receipt
`obj/parallel/async-integration-v1/qualification-v1/receipt.json`:
`5ea62bea2fe19e5e3f11250495788deffa565db57d3f7c0b50ef577d3610e4cd`. Composed receiver reference:
`d9ef54b7696e87760c6416a9192874453a9a7d23060b676e316dab7aa82a563f`. Worker receipts above remain historical
and explicitly bind the old backend; the joined receipt binds the current pin.
Full backend CI41948482651 is running on2185cbd74 and precedes these joined
frontend changes. A full successor remains pending.
