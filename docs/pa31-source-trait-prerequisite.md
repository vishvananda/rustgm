# Source Copy, Clone and ordinary callable prerequisite

This checkpoint uses real declarations in an explicitly selected `no_core`
provider. It does not replace ordinary upstream library bodies with factories.
It is a prerequisite for the remaining source const-call and destruction work,
not acceptance of the original core crate.

## Identity and proof boundary

`support::bindings::LangItem` appends Copy, Clone, clone_fn, LegacyReceiver,
Tuple, FnOnce, FnMut, Fn and FnOnce::Output after the five sizing/layout roles.
The resolver reserves their actual source DefIds. Semantic publication validates
complete actual headers and member owners transactionally, then registers the
actual callable contracts. Missing, reserved/pending and validated declarations
remain distinct. No bootstrap `BuiltinTrait` identity is installed in source
mode. `Tuple` structural evidence is available only for the validated actual
Tuple application and a real tuple type; it conveys no callable permission.

Const trait/impl syntax and `const`/`[const]` bound effects remain distinct through
canonical/inferred predicates, substitutions, shape keys, clause inheritance,
assumptions and selection. Rust-call is a distinct calling convention. An
ordinary consequence may be shared by identical qualified premises only in the
same exact scope; this never pays a const obligation. Full const implementation
selection and reached conditional const-call execution remain follow-up work.
A const qualification or const impl of a nonconst trait rejects at its actual
owner. Runtime callee-use obligations weaken only Conditional to Ordinary,
using the current body owner; const functions, initializers and inline consts
retain their obligations. Explicit Const and stored defining contracts are
never weakened.

Source Copy decisions are neutral unmet semantic facts until body preparation
selects each demanded distinct type under the actual immutable parameter
Environment. The body-owned prepared carrier retains exact BodyKey, source,
complete Instance, public signature, requirement Arc and Environment Arc. All
selected lifetime/formation debts enter the ordinary contract proof. Region,
move, initialization and loan checks still run normally. Specialization clears
prepared decisions and substitutes neutral requirements before rebuilding them.

`borrow::Environment::source_copy` is the explicit actual-provider policy, read
from `Declarations::uses_source_copy()`. In this mode CopyView rejects a missing
carrier instead of falling back to physical Copies. An active source policy
with no Copy role is different from ordinary mode: literal/no-read bodies can
proceed, but a queried read diagnoses the missing role. Completed absence of an
implementation classifies a move; pending selection is an error, not absence.
Original bodies and contracts must have an exact bijection; unmatched rows in
either direction reject. The public `body::prepare_source_copy` facade exposes
these same checks to the CLI inspector, without granting executable permission. Direct receiver
adapters use the same preparation and exact typed execution signature.

Original proofs are prepared in lower::instances and in demanded original/body
components before region/loan verification. Both ordinary body dispatch and
specialized dispatch retain these duties. Actual Copy implementations also
check field obligations and reject unique-reference fields and destructor
conflicts; physical shape alone never grants source Copy.

A concurred on the body-owned BuiltinRequirements seam and original component
consumers. Its Environment and neutral source facts are preserved exactly;
future source-free site conversion must not manufacture source NodeKeys or
reuse a prepared decision in another environment. C's Drop/Destruct validation
and needs_drop consumer remain separate; their proposed append slots 14/15 are
not part of this fourteen-role checkpoint.

## Frozen source and independent observations

The independent sources and every failed/superseded hypothesis are retained in
`obj/parallel/pa31/source-traits-v1`. `declared.json` and `declared-v2.json` bind
the original twenty and corrected nine source hypotheses. The correction uses
pinned Rust 1.98's actual `tuple_trait` identity, `const impl` syntax, actual
LegacyReceiver provider, and `unboxed_closures` gate. Earlier incorrect
hypotheses were not overwritten. `observations` and `observations-v2` retain
commands, statuses, diagnostics and native observations.

The unchanged sources are retained as named raw constants in
`dev/tests/provider_traits/cases.rs`; they are test data, not Rust modules to be
compiled by the host. `retained-inputs/sources.json` and
`embedded-byte-equality.json` verify all seventeen original byte hashes.
There is no assignment activation, fixture manifest or generated oracle.
The executable cutouts include:

| Source | SHA-256 | Independently declared answer |
| --- | --- | --- |
| generic-copy.rs | 6176b426e048dddbe8186d4db4594c37545d2dc48efc56d657f7a37de04d20b1 | 7 |
| source-clone-ufcs.rs | 774cee14c2bf78e8c1fe1d12e81957be49ab247d2330636943c0fd42901ba672 | 11 |
| callable-function-item.rs | dd5a4c7d513bf2d934c16c990187825cc2035cec121dc035102cc824d5ac2476 | 23 |
| const-callable-item.rs | 781192de892cac77522d5e1f70041d18ac34ff30a6103a96c0f45841c0fe519c | 29, required follow-up |
| conditional-runtime-nonconst-target.rs | 662150c1503344d393165a71f6138335bd281faf662ca1d043cd039ac4fc81c0 | 29, required follow-up |

The first three, and the unused-conditional runtime control (answer7, source
SHA-256 `50f5a24b42ee12552d2f9ca05ed4b19fec9aeba040483caf5b682d63473f1c25`),
execute through `pipeline::analyze_source`, ordinary original
and specialized region/loan proofs, and existing `build_roots_prepared` with
library export roots. This deliberately preserves the original source bytes.
It does not establish whole-crate artifact acceptance: CLI library export of
these public source traits still reaches the existing scalar-only PA29 metadata
boundary. Original artifact-mode commands and failures remain in development-v17.
The earlier non-library LowIR attempts stopped at missing main; passing an
artifact flag to `--emit-types` was an invalid command and is retained as such.
Neither observation is evidence of successful compilation.

The original whole-core probe remains blocked at active `bstr_internals`
admission (original lib.rs byte4097, line89); only root admission was reached.
It used the immutable source tree at revision
88d9e12ae178fab0fb5cc050a94da85685d449ea, original target cfg and original sources.
No root/module traversal or parse success is claimed for unvisited code.

## Qualification and storage

`development-v28/results.json` is terminal0: format, all-target/all-feature
strict Clippy, five provider source/header/body test functions, seven private
proof/effect checks, four native clients and architecture audit of 1,943 files with zero findings.
The unchanged call_operators group, all three method_contracts groups and all
five template_ownership groups passed in development-v24 before test-data
packaging and unchanged-function helper extraction; those successes were not
needlessly repeated. development-v26 also retained the first native results.
Audit's earlier include-macro/nonmodule-fixture/function-cap findings and their
correction remain in the failed v26/v27 receipts; no audit rule was relaxed.

`closing-tail-v2` is terminal0. Its three additional effect invariants pass.
All 8,362 active PA1–24 text cases match unchanged committed references from
frozen published 488109dd (the relevant fixture bytes are unchanged at b6).
`text-evidence.json` binds every stage result path/hash. No reference was
regenerated. Source hashes in `sources.json` were rechecked after the run.
The immutable final debug CLI is
`obj/parallel/pa31/source-traits-v1/closing-tail-v2/candidate/rustgm`,
SHA-256 `f14a9283f2f465aa57b6bad95f94f49ff2bb8a75d5604718acf028f0912a1f50`.
`candidate.json` and `candidate-after.json` match; this is no performance claim.

The nongeneric Clone11 source passes ownership, regions and loans inspection.
The absent-Copy literal passes all three views. Its one-use read rejects in
ownership and loans with `file0:273: source Copy language item is missing`;
regions alone proves lifetime facts and does not claim Copy permission.
The generic Copy7 and FnOnce23 originals remain required-positive CLI inspector
gaps at the preexisting `body::build` generic-construction boundary. For example:

```
rustgm --emit-ownership --edition=2024 --source-core minicore@source_traits_v1 \
  --crate-name minicore --crate-disambiguator source_traits_v1 generic-copy.rs
ERROR: file 0: 622: generic body construction requires parameterized ownership and destruction
```

FnOnce23 reports the same boundary at 1123; all three modes preserve the failure.
These are pending PA19/PA24 generic inspector work, not negative coverage or
replacement source hypotheses. Their original/specialized proof and native
successes above remain distinct. The first failed tail is retained unchanged.

The provisional const-call gate indexes only completed actual const-function
predicates, including unused and inherited clauses, once per declaration in a
cold source catalog. Missing/pending never becomes Ordinary. Every reached
Machine Call checks that data before either fresh preparation or cached Code;
Engine FunctionBody retrieval also checks before its cache return. Conditional
calls explicitly report missing scoped const-debt payment. The unused runtime
call executes 7; pinned rustc also object-compiles/executes both that source and
the original paid-const positive as 7. Candidate paid-const remains unsupported.
The actual inherited header is separately checked for its retained Conditional
clause and pending raw-method eligibility. The cold/warm execution test is
explicitly adversarial: it forges a checked runtime snapshot's const-context bit
to reach the inner gate, after the unmodified runtime entry was correctly
rejected. This is not accepted const-source coverage. The earlier proposed
unpaid const defining body was independently falsified by rustc and is retained
as a rejected source hypothesis.

The b6 normal no-feature release rlib and dependencies were copied into the
immutable `baseline-layout-b6` with source/fingerprint/hash provenance before
root cleanup. The independently executed size client reports ItemKind28,
Item64, OwnerKind20, Owner32, Bound8, canonical Predicate12, inferred Predicate24
and Convention2. Focused source-effect tests assert those exact sizes. The paired same-client measurements are:

| Record | Exact b6 baseline | Candidate |
| --- | ---: | ---: |
| borrow Environment | 32 | 40 |
| Body | 448 | 448 |
| Contract | 184 | 184 |
| body BuiltinRequirements | 32 | 48 |

The source-only neutral requirements are 32 bytes, prepared body carrier 96 bytes,
semantic source Catalog 376 bytes and indexed Effect 1 byte. Source catalogs remain absent
in ordinary mode; their two new optional pointers and nine declaration slots
are cold, with no permission cached in those facts. The Body/Contract pointer
fields are retained; no per-body source-mode flag or ordinary dense permission
map is introduced. Read discovery visits each body's
values once and charges those visits in existing canonical work accounting;
selection is per distinct read/repeat element type. Ordinary mode returns
before this source-only discovery/index allocation.

## Required successor contracts

The const29 positive and conditional runtime/const opposing family are not
closed by ordinary callable execution. A cached Code is structure, never an
Instance-keyed const permission. Every reached CTFE use must pay exact effect
requirements in its own materialized caller/contract context, including unused
conditional clauses, and consume a one-use continuation before invocation.
The pre-cut rejection is safe but does not implement valid payment. Runtime-first
then CTFE, warmed result from a different caller, replay, changed
target and nested suspension controls are required. Fn/FnMut/closure/pointer
invocation must not be inferred from this ordinary FnOnce function-item slice.

Actual Drop/Destruct/Pin, complete LegacyReceiver validation, Freeze, Clone's
remaining methods, const-drop eligibility, generic/public trait metadata and
upstream provider expansion remain required. Rust-accepted unused malformed
headers outside this bounded provider profile are unsupported, not
Rust-invalid. Full source feature coverage and all original PA26/PA30 gaps
remain required, including outstanding collection ownership/range variants,
structural/tuple/str/unit comparisons and primitive float/char contracts. The
endpoint is upstream core/alloc/std and real crates, not additional convenience
method factories.
