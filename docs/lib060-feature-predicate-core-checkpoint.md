# LIB-060 source-free feature predicate core

This finite typed checkpoint represents `Clause::UnstableFeature(FeatureId)`
without a type subject. It does not publish source attribute admission, a new
metadata profile, original-core compilation or an assignment release. The source
producer, independent source matrix and transport remain separate integration work.

Feature IDs contain an 8-byte private arena nonce and canonical name slot. A
checked monotonic nonce is identity only; exhaustion fails and identities never
wrap or enter metadata/text. Debug output is opaque. The lazy paid name arena
lives inside existing cold clause storage. Imported spellings must re-intern in
the consuming Store. Caller feature premises, their shared index snapshots and
immutable policy retain actual Store witnesses; foreign IDs and foreign snapshots
reject even when their numerical slots coincide.

`Store::intern_feature_charged` and borrowed `Store::feature_name` provide names.
`Environment::with_feature_assumptions_charged` prepays new declared premises,
including a shared-prefix clone when needed. Feature-only environments remain
non-neutral. Ordered clauses retain the requirements; substitutions, binder
composition and const-effect mapping preserve their exact non-type identity.
Opened and extended indexes share the immutable feature snapshot. Warm membership
is borrowed and retains no additional storage; failed payment attempts remain
charged without publishing proof.

`Declarations::publish_feature_policy(enabled, pay)` publishes once into the
existing cold trait-contract owner. The authenticated source publisher supplies
enabled IDs; an empty set means caller-only. The typed core hardcodes no staged
versus ordinary enabling rule. Missing policy is a requirement error.
`Declarations::prove_feature(id, caller, pay)` checks genuine caller membership
or that immutable enabled set. The API has no lint-cap input and grants neither
regular API permission nor a type/trait/unsafe capability.

Nominal proofs and actual implementation conditions dispatch feature obligations
directly. They create no inference type goal, type subject or trait proof frame.
Trait requirements retain unconditional formation debt separately from Self
implications. Direct source calls/reification use their actual caller environment;
feature requirements cannot disappear at a lifetime-only skip. Conservative
overlap never treats a disabled feature as disjointness.

`callee_premises::{requirements,specialize}` now accept an actual optional caller
environment and discharge feature conditions before returning lifetime obligations.
The constant caller path supplies its authenticated declaration environment.
`body::instantiate` still has no actual caller input at its Declared-premises
transfer port and passes None explicitly. That port requires a real caller/witness
adapter; the callee's own marker is never substituted as proof. Contextual callable
feature conditions, specialization continuation proof and specialized compiler
intrinsic contracts retain named requirements where no appropriate non-type port
exists yet.

Old metadata exports and imported body reconstruction explicitly reject the new
clause. No wire ID/tag is added and no condition is dropped to fit old rows.
Transport must carry authenticated canonical spellings and remap them once into
the destination Store. Source-free shape/type-only collectors do not turn a
feature clause into a type property or associated result.

## Qualification

Private gate `1793215416920269825` passes five exact controls in 132.37s, including
strict Clippy, formatting and architecture checks. Actual implementation candidate
selection has both a matching caller success and missing-caller failure; direct
nominal proof, immutable global/caller separation, foreign ownership, clause laws,
paid cold failure and warm index sharing also pass. This is typed-core evidence;
source/native matrix qualification remains independent.

Production/CLI+lexer gate `1793215735989182086` fails strict Clippy in 78.55s
because the deliberately crate-private policy publisher and its constructor have
no production caller on this isolated base. No lexer execution occurred. The
authenticated source publisher still had to make this API live in the joined
compiler; production qualification at that snapshot remained pending. Visibility
was not widened, no default policy was installed and no dead-code waiver hid
this boundary.

Root subsequently integrated the real source publisher at `270390dcb`. It reads
only the completed checked library request inventory, interns ordinary requests
once and publishes an empty enabling set for staged libraries. Its cold storage
and temporary IDs use the existing attribute producer's payer. Marker-free
ordinary crates without a library inventory allocate no policy. This source port
resolves the isolated production dead-code failure without widening visibility.

Final joined private13 passes in 122.71 seconds (`1793217226510599843`);
production/CLI, lexer10 and PA6 pass in 97.45 seconds
(`1793217370219235780`). The joined source handler also executes natively as
[recorded separately](pa31-source-panic-handler-checkpoint.md). Trait/Fn source
capture, preservation through bounds completion, charged source environments,
the sealed native caller adapter and feature metadata remain required. These
results do not close the independent source matrix or the original-core build.

Measured x86-64 widths are FeatureId 8, Clause 12, Store 472, Environment 64 and
NodeFacts 80 bytes. The original two-variant sum, using unchanged canonical
payloads, also measures 12 bytes. Dense/hot records do not grow. Cold owner/index
growth is included in paid allocation before publication.

| Canonical feature names | Charged steps | Charged bytes |
| --- | ---: | ---: |
| 1 | 50 | 772 |
| 8 | 342 | 1,579 |
| 32 | 1,841 | 7,367 |
| 128 | 7,642 | 28,451 |

These are bounded changed-path operation/storage observations, not wall-time or
whole-library claims. Raw measurements were retained in receipt
`1793214914204344146`, whose complete run failed two test predictions: the guessed
Clause width was 16, and its failure payer stopped before any positive-byte
attempt. The corrected invariant compares old/new width directly and rejects at
the first allocation attempt. Earlier static errors and the cold build timeout
remain recorded. The final complete five-control gate passes; no failure was
blessed. Evidence and source-linked handoff are under
`obj/parallel/trait-feature-bound-design-v1/`.
