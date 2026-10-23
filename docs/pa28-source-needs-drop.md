# Source-bound destruction queries

This work follows the Rust 1.98.0 source commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`. It is not a new bootstrap
library API. The previous string-word checkpoint is `324d1840`.

The exact intrinsic declaration is in
[`core/src/intrinsics/mod.rs`, lines 859–875](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/intrinsics/mod.rs#L859-L875).
It takes one `T: ?Sized`, has no value arguments, and returns `bool`. Actual
requires-drop types return true; actual Copy types return false. The result
for a type satisfying neither condition is unspecified. A pinned rustc
observation for that last category is not a universal byte-for-byte contract.

The ordinary stable wrapper in
[`core/src/mem/mod.rs`, lines 692–700](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/mem/mod.rs#L692-L700)
uses a const block. An ordinary runtime call to this wrapper must specialize
and evaluate that block to a bool. It must not inherit a blanket runtime ban
from the inner intrinsic. Conversely, the direct intrinsic has no backend
implementation. The pinned source guard is explicit in
[`rustc_codegen_ssa/src/mir/intrinsic.rs`, lines 128–136](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_codegen_ssa/src/mir/intrinsic.rs#L128-L136).

## Independent evidence

Before-probe declarations, exact upstream excerpts, every contextual source
and command output live under
`obj/parallel/pa28/source-needs-drop/`. No reference was generated.

The first 29-case declaration is `declared-v1.json` (SHA-256
`cdfe2c696ed31c24bf5f9b57200aa0f10dd3547872f990be61da857b94b9cabf`).
Its failures include missing lexical Sized imports and invalid unparenthesized
block casts in the test context. Corrected inputs were declared separately in
`declared-v2.json` (SHA-256
`d79ff3955cc24ca253f3140e14bba474e5acc74b5f6a66b486e88315441f88c0`).
The old bytes and failed hypotheses remain intact.

The corrected independent batch matched 18 native observations: concrete,
unsized and generic const-block wrappers; direct constant calls; ordinary
same-spelled functions; real Drop, nested Drop, zero-length Drop arrays,
references to Drop and real Copy-bound calls. The direct runtime intrinsic
accepted metadata and triggered a rustc object-codegen ICE, rather than a
language diagnostic. Intrinsic reification and unsafe signatures rejected.
The original hypothesis that rustc permits an unsafe intrinsic declaration
was falsified. A narrower Sized-only declaration accepted in rustc; the actual
source profile still requires the pinned ?Sized declaration, so its explicit
profile rejection must not be called a Rust-invalid program.

The full attributed wrapper needs a nonduplicating no-core context: declaring
its diagnostic item while linking ordinary core produced a duplicate-item
error. The separate no-core full-excerpt inputs are in `declared-v3.json`;
their first attempted batch was refused before launch by the unchanged disk
reservation policy. The later admitted replay accepted both new inputs in
metadata, object and native modes, including the exact full upstream excerpts. Initial candidate types/constants commands also passed an
inapplicable library artifact flag and returned usage errors. Those outputs
are not frontend rejection evidence; corrected command replay is retained as
a separate prepared batch.

## Typed identity and reached execution

`SourceIntrinsic::{Layout(LayoutIntrinsic), NeedsDrop}` distinguishes actual
attributed DefIds. Only declaration attribute decoding reads an intrinsic
name. The existing layout selection keeps its usize contract and does not
become bool/callback permission. Shared validation checks the actual source,
parent, signature, own generic parameter and full selected Instance.

The planned `NeedsDropSelection` contains only definition, closed element type
and fallback presence. A fresh private reached-call witness must validate the
current checked Code, statement, evaluated callee, complete Instance,
signature, const eligibility, zero arguments and bool destination on every
execution, including cache hits. Existing `Requirement::FunctionBody` handles
suspension; no new Target or ambient const permission is needed.

The Engine kernel must retain ordinary original/specialized body proof and
caller obligations for any source fallback. The mem wrapper follows ordinary
function selection and existing InlineInstance normalization. Only the inner
checked const call obtains a compiler result. The evaluator produces
`Scalar::Boolean`; no usize result reinterpretation is involved.

One cold SourceResults owner may hold layout and needs-drop maps within one
shared retained-storage budget. A cached bool is data, not caller proof.
Only closed Instances with completed actual Copy/Drop roles and completed
destructor preparation may publish query results. Missing or pending roles
must not become false or a negative cache entry. The existing fixed-environment
NeedsDrop query supplies destruction semantics and head readiness; symbolic
conservative cleanup is not automatically an executable intrinsic answer.

## Actual Drop prerequisite

The pinned
[`Drop` declaration](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ops/drop.rs#L206-L268)
is a const trait with default `drop(&mut Self)` and
`pin_drop(Pin<&mut Self>)` methods. Exactly one implementation method must be
overridden. Neither two defaults nor both overrides authorize an implementation.
Its original default body calls ordinary `Pin::new_unchecked` and `pin_drop`.
The actual `Pin` declaration has a lang item and transparent representation;
matching layout or source spelling cannot substitute for that identity.

B owns shared role/qualified-goal wiring, with Drop/Destruct following the
existing fourteen roles. The source-role lane owns typed header validation. This lane owns destructor
registration and intrinsic execution/query selection. Reserved Drop identity may classify potential cleanup before trait
completion. Only sealed exact header/role readiness can authorize executable
selection. The source path must not alias bootstrap `resolution.drop_trait`.
A private DropRole can be derived from the existing checked role catalog;
there is no need for another global permission index. Completed destructor
preparation remains distinct from an empty implementation list.

The pinned
[`Destruct` declaration](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/marker.rs#L1053-L1063)
is a const marker over PointeeSized with explicit-implementation prohibition.
Ordinary evidence is universal and does not pay const or conditional debts.
Actual const destruction needs separate proved eligibility. Pin/default bodies,
full original declarations and pin-only implementations remain required until
independently accepted and executed; a reduced context does not prove complete
`pin.rs` acceptance.

Ten original-Drop/Destruct hypotheses are frozen in `drop-declared-v1.json`
(SHA-256 `14467a7af7a1b61489628075c0b2e2a0416c2abb305525c1c9bf2de2c8f283d3`).
They include full unchanged Drop bytes, drop-only/pin-only implementations,
empty/both overrides, actual cleanup counts and ordinary/const opposition.
The first admitted context probe stopped on missing actual LegacyReceiver.
That evidence remains separate from the new `drop-declared-v3.json` context
(SHA-256 `64bd443c69e683a324fa2d5a338a9df4e49657d145792f658a9cd8bddf73bf0c`),
which retains exact LegacyReceiver/ref/Pin impl excerpts and actual Clone/Copy
headers. The new controls are prepared, not run during the integration performance
hold. Full Pin and method override behavior remain required.

The remaining PA27/PA28 obligations and the two original panic-macro positives
remain in [the lane inventory](pa27-pa28-remaining.md). This preparation does
not close source Drop, source Copy, needs-drop execution or upstream core.

Until those actual roles are available, the registered NeedsDrop kind stops
explicitly before an ordinary source fallback can execute. Bodyless, true and
false placeholder controls must all retain this boundary; this is not a bool
answer, a completed kernel, or a negative cache entry.

## Qualified identity prerequisite

The intrinsic-identity checkpoint passes strict all-target/all-feature Clippy,
1929-file architecture audit with zero findings, and eight focused debug
invariants. Five new invariants cover reservation versus validation, complete
closed instance identity, separation from layout permission, malformed/ordinary
names, and bodyless/true/false placeholder rejection. Three existing layout
controls retain their original const, unpaid-caller and runtime behavior.

Measured sizes are SourceIntrinsic 1, IntrinsicDeclaration 24, validated
descriptor 32, source Catalog 184 and execution Catalog 160 bytes. The descriptor
and reservation sizes equal their previous LayoutIntrinsic shapes. No result
cache, Frame field, global Value/Origin schema or executable Drop permission is
added by this prerequisite.

A distinct minimal actual-role provider is frozen in `minimal-declared-v1.json`
(SHA-256 `a6c66857028ce536dbbf5d67107b6a182e67fa2868c9e9f64622aa9ba400baf0`).
All ten independent metadata/object hypotheses match; seven native observations
pass. Those include actual Drop/recursive/generic true, primitive/reference/
zero-array/Copy-bound false, plus unpaid Copy and malformed Drop negatives.
This valid provider has an ordinary required drop method; it does not rewrite
or qualify the original two-default-method upstream Drop/Pin closure.

The declaration/kernel dependency plan remains source-backed:

- **Deref/Target:** retain actual trait and associated-type DefIds, validate the
  safe shared-reference method and exact target projection. A checked selection
  carries the full method Instance, receiver borrow adjustment, projected target
  and ordinary or const proof debts. No reference capability comes from layout.
- **Pin:** retain the actual nominal identity and transparent field contract.
  `self: Pin<&mut Self>` already parses; semantic receiver classification needs
  the proved actual Deref target and LegacyReceiver chain. The ordinary
  `new_unchecked` body remains checked Rust. No name-based reference treatment
  or synthetic constructor is permitted.
- **Destruct:** validate the exact marker owner, PointeeSized supertrait and
  implementation restriction. Ordinary universal evidence does not prove
  conditional/const destruction. Those consumers wait for the per-execution
  continuation from the source-trait lane.
- **drop_glue:** the pinned helper takes `&mut T`, not a raw pointer, and has
  `T: [const] Destruct`. Its empty placeholder must be replaced only by a checked
  selected cleanup recipe after fresh call/borrow/const proof. Existing raw
  `drop_in_place` Rust source creates that reference through the normal gates.

The first full-Drop context replay after adding LegacyReceiver still rejects
for missing genuine Deref target/receiver support and Destruct; the const
Destruct context additionally requires drop_glue and correct const-stability
attributes. Every original remains a required positive or its independently
settled semantic negative. None is relabeled complete by the minimal provider.
B owns appended role identities and qualified-goal wiring. Any role-mask growth
past the existing sixteen slots must be measured and coordinated before editing
shared records.

## Sparse concrete query prerequisite

The private `NeedsDrop::concrete_sparse` constructor shares the existing iterative
walker and immutable destructor registry, but retains only reached TypeIds. The
cold Scope uses an optional environment without growing either query record.
Measured sizes remain NeedsDrop 72 bytes and Scope 48 bytes.
Its absent environment enables no symbolic assumptions. Unrevealed opaque heads
and unresolved projections still reject, including declarations with Copy flags;
those flags do not establish actual source-role proof. Existing ordinary and
symbolic constructors retain their behavior.

The focused debug qualification in
`obj/parallel/pa28/source-needs-drop/sparse-concrete-v1/` passes five new private
tests and two existing ownership tests, strict all-target/all-feature Clippy,
format check and the architecture audit. The 27-point width/depth/noise grid
varies widths 8/32/128, depths 1/8/32 and unrelated Store entries 0/1024/8192.
Every result retains exactly depth+1 states, unchanged capacity/work across noise,
and work bounded by 4×width×depth+2. Each grid point also performs 128 warm queries
without growing traversal work. Missing-head failures remain sticky in their own
query; readiness requires a fresh query rather than a cached false answer.

This internal prerequisite does not execute source needs_drop. The actual Drop
and Copy roles, completed destructor preparation, fresh reached-call witness and
closed-instance result publication remain mandatory for the forthcoming kernel.


## Bounded concrete query prerequisite

`NeedsDrop::concrete_bounded` preflights the cold query and immutable registry
before consuming entries. `demand_concrete_bounded` uses the same iterative
walker, with sparse state/work capacity checks before growth. Existing-key
updates avoid map insertion, which may otherwise reserve before finding a key
in a full table. `retained_bytes` reports owned logical record and payload
capacity; temporary old/replacement payloads are conservatively charged before
growth. Allocator headers/control bytes and canonical Store allocation remain
outside this accounting boundary. Ordinary and symbolic APIs keep their existing
behavior and record sizes: NeedsDrop 72 bytes and Scope 48 bytes.

Each bounded attempt consumes work, including warm cache reads and over-budget
attempts. Error cleanup charges its actual sparse-state scan; failed states stay
failed. A pending opaque head requires a fresh query after readiness, and never
publishes false. State/work preflight refusal preserves already completed answers
without allocating an error entry for an unseen root.

The focused debug batch under
`obj/parallel/pa28/source-needs-drop/bounded-query-v5/` passes six new budget
controls, five prior sparse controls and two existing ownership tests. Strict
all-target/all-feature Clippy, format and the 1932-file audit pass. With a shared
32-level DAG and root widths 8/32/128, cold traversal visits are 828/99/387,
retained states 34/35/36, and logical retained bytes 6152 throughout. All values
are identical with 0 versus 8192 unrelated Store entries. Each root's 128 warm
queries consumes exactly 128 additional steps and no retained growth. The first
root includes shared-DAG preparation; later roots reuse it.

Earlier attempts remain evidence: an import/lint correction, format correction,
and two private failures exposed full-map existing-key reservation and uncharged
exhausted attempts. Both behavioral fixes are covered by the passing controls.
This remains a private query prerequisite, without source intrinsic execution or
actual Drop/Copy permission. Actual source role validation, exhaustive destructor
registration (including completed-empty), fresh caller authority and final
conformance validation remain required.

## Qualified actual-role kernel

This checkpoint supersedes the identity-only execution stop for the independently
qualified minimal provider. Its fifteen original inputs and thirteen separately
declared work controls now live under `pa31/tests/spec/needs-drop/`, with exact
source hashes and original independent contracts. They remain planned-stage
maintainer inputs, without generated references or assignment activation.

Source-mode reservation classifies potential cleanup from the actual Drop DefId.
The exhaustive collector completes ordinary retained ImplHeaders and their
canonical declaration/override mapping before registration. A cold exact-DefId
marker distinguishes absent, reserved, validated-unprepared, completed-empty and
completed-nonempty inventories. The marker publishes only after the complete
successful registration pass; later trait, method, body and Drop-bound checks
remain mandatory. Ordinary bootstrap registration retains its previous route.
Source Copy preparation and cold/warm const-call gates still require this marker.
Explicit actual Drop-member calls/function values retain their separate identity
denial, including matched overrides and aliases.

`NeedsDropSelection` remains data. Both suspended preparation and resumed
execution validate the current checked Code entry, frame block/statement,
evaluated callee, complete Instance/signature, const eligibility, zero arguments
and bool destination. Every warm result repeats these checks and the actual-role
and registration gates. Ordinary original/specialized proof and caller debts are
required before evaluating a source fallback; true/false placeholder bodies do
not execute as an intrinsic implementation.

One cold SourceResults owner holds the existing layout map and an optional
DropResults owner. The latter shares one immutable destructor inventory and one
bounded sparse concrete query across requests. Only successful closed-Instance
bool answers enter its map. Missing roles, incomplete registration, unresolved
physical heads and failed queries never publish false. Query and answer growth
preflight their retained and replacement payload capacities under the existing
shared execution budget. The accounting boundary covers owned logical record
and payload capacities, not allocator overhead or canonical Store allocation.

The unchanged reference-positive initially failed because the specialized
Contract signature had one owned binder while the public callable had none.
The exact original source and GDB evidence are retained. The separate reviewed
signature-pairing prerequisite retains both genuine views; this kernel does not
erase binders, compare only their counts or replace a public signature. See
[the pairing review](pa31-source-copy-signatures.md).

`kernel-v8/` passes four focused frontend tests, ten private invariants and all
eleven original native outputs. The source family contains eleven positives and
four semantic negatives. Exact call/destination/statement, full-Instance identity,
wrong/pending registration, cold/warm authority, tiny-budget and signature/layout
opponents pass. Strict all-target/all-feature Clippy, format and the 1983-file
architecture audit pass. The prior `kernel-scaling-v1/` independently passes all
thirteen native work controls. Their unchanged source snapshots and earlier
failed batches remain retained under `obj/parallel/pa28/source-needs-drop/`.

Actual-pipeline query work for widths 8/32/128 is 29/101/389 steps and
388/772/2308 retained logical bytes, with one cached answer. All counts remain
identical across 8/32/128 source calls and unrelated declaration counts 64/1024.
The unused-intrinsic control allocates no query; 128 primitive calls use four
query steps and 280 retained bytes. These are logical work/storage invariants,
not measured wall-time or RSS neutrality. SourceResults is 40 bytes, DropResults
104, and the execution Catalog remains 160. The cold source Catalog is 424
bytes, eight more than the header-only 416-byte record for the exact marker;
Validated is 28, Reservations 432, Declarations 7048 and Resolution 2032.

## Required source closure remains

The bool kernel/native tests do not instantiate and execute an actual Drop value.
The original cleanup-count hypothesis first lacked the real Freeze declaration;
adding it exposed the genuine drop_glue requirement. A distinct complete context
with exact Freeze/Destruct/drop_glue declarations independently returns seven
under pinned rustc. Its unchanged source hash is
`633442327cf88bd6f30745d737706695860e5ea96cdbfd2662a81cdb25e1d110`.
The candidate still stops parsing its original unsafe auto trait at byte 126.
This remains a required positive owned by the source-role lane, alongside full
Drop/Pin/default-method and const/conditional destructor execution.

The complete CLI matrix is retained in `kernel-views-v1/`. Types/constants and
checked LowIR/native lowering have distinct publication surfaces: library LowIR
artifact export still rejects exposed source traits/generic declarations outside
the scalar metadata profile. A separately declared no_main context independently
accepts in rustc but still reaches the candidate's required-root-main boundary.
Neither is a Rust rejection contract. The original sources remain required;
checked internal lowering/native qualification does not claim a complete source
library artifact or full upstream core acceptance. The two original string
panic-macro positives and the rest of PA27/PA28 remain required as before.
The two exact attributed upstream-wrapper contexts in `declared-v3.json` also
remain required source-closure controls; the distinct minimal provider does not
qualify their complete attribute/role environment.
