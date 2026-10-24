# Constant selector source implementation draft

The preserved interface/declaration checkpoint is `3184e65c`. The qualified
shared API merge is `dd415264`, whose tree differs from root `2da35cf0` only by
that proposal and its declared fixtures. The production draft below is
**qualified for its initial isolated source/native profile** at `736cb69a`.
Merged release and performance qualification remain integrator-owned. Earlier
failures and their exact recoveries are retained below.

The later [integrated qualification](source-ctfe-selector-checkpoint.md) passes
3,488 fresh release/native cases, 8,382 text cases and all 51 performance inputs.
It includes this isolated production checkpoint and the owned-snapshot repair;
the chronological development and handoff records below remain unchanged.

The original [proposal](pa31-const-select-ctfe-proposal.md) and initial fixture
contract remain unchanged. Two additive, pre-probe source contexts in
`contract-additional.json` cover distinct complete instances of one callback and
an existing conditional enclosing owner with two ordinary selected callbacks.

## Implemented source path

- `Inputs::prepare` completes every actual selector relocation after body proof
  and before Code publication, accumulating retention instead of returning after
  the first qualifying owner/call. No-catalog/non-const paths do not inspect
  values. Publication prepares metadata and retains the actual caller contract;
  it demands no selector or callback body.
- `selector_sites::Call` borrows the actual Code/Frame/Call and retains logical Select
  data plus evaluated actual F. Operand IDs and tuple width are derived instead
  of retained redundantly. The proposed `reached_select` filename was avoided
  because the existing `reached_*` filename stem was already at its audit limit.
- `executed_calls` classifies the typed callee's registered intrinsic before
  ordinary lookup. A wrong evaluated selector cannot fall through as another
  direct call. The checked raw selector crosses its Code mapping once; the
  evaluated F Constant is already mapped and is never remapped.
- The ordinary F call uses the existing FunctionBody(F) request, parent
  suspension, common function-entry gates, actual full-instance code index and
  original Call return destination. Closed owned tuple fields spread only after
  the actual callback entry is ready, with charged work and payload preflight.
- Cold Site identity adds Direct/Selected form to the existing one-use
  continuation. Selected inspection binds the exact current F value. Conditional
  F reuses its own Requirements and caller Environment; cached bodies never
  substitute for fresh payment. Existing direct-payment tests add a forged
  Selected-form replay opponent, without fabricating positive selector payment.
- General function/root demand still rejects selector placeholders through
  `check_const_select`. Only the reached adapter enters F. No Body, Code, Frame,
  State, Machine or Engine field was added.

Caller authentication accepts closed actual function or named-initializer snapshots,
checking source/owner, canonical index, initializer kind and signatures. An explicit
suspended/executing phase derives the one current statement from Frame.next;
there is no optional position fallback. Other caller owner forms or open caller signatures
remain explicit unsupported. Callback nonempty binders or lifetime-bearing
entry, selected intrinsic F and nonempty Conditional payment shapes also remain
explicit unsupported. B's callable Unpaid/default-member authorities are
unchanged; its unqualified default defining-context successor is excluded.

## Draft qualification controls

The frontend draft covers nine new ordinary-F source/native contexts plus the
unchanged original empty-ARG CTFE positive. Raw provenance failure and moved-argument
reuse bind specific diagnostics. Reference binders and Conditional callbacks
retain explicit valid-but-unsupported classifications until their owners close
the required boundary. The distinct recovery raw-read hypotheses have not been
observed; the original bounds hypotheses retain their E0608 disagreements below.

Five private controls use actual bodies captured by the existing materialized
body observer and actual cold interpreter suspension. They cover evaluated
selector/F/tuple and frame/site corruption; a wrong same-item complete-instance
request; caller owner/public/internal/Call-signature opponents; cold/warm F entry
counts without selector/G frames; attempted work with cold/warm body caches; and
all-call canonical preparation after earlier/owner retention with zero-work fast
paths. The existing one-use direct payment control gains one additional form
substitution opponent. Positive permission is never constructed from an altered
body snapshot.

Future observations must distinguish actual completed work, retained payload,
new versus retained qualification and unsupported profiles. No assignment
activation, generated reference update or whole-library acceptance is claimed.

## First independent observations and distinct recovery

Guard83057 completed all fourteen independent rows and stopped before static or
candidate checks. Eleven metadata/object/native triples matched, including owned
tuples, mapped returns, reference binders, operand order, repeated/unreached
selection and both Conditional contexts. Owned argument reuse rejected with
E0382 as declared. The two index-fault sources instead rejected with E0608:
this no_core provider does not declare the actual Index contract. Their intended
CTFE-fault/runtime-positive hypotheses were invalid for that incomplete provider;
the observations supply no RustGM Index qualification or selection result.

The original sources, declarations and all three disagreements remain unchanged.
`contract-recovery.json` binds the twelve unchanged successful oracle rows by
source, result-row, binding and receipt hashes. Only two distinct inputs are
declared for new probes: selected-null-raw-read-fault (E0080 hypothesis) and
null-raw-read-runtime-opposite (native nine hypothesis). They use the existing
PA27 raw pointer cast/read profile, with the null address supplied through a
parameter. Candidate fault checking requires the existing provenance diagnostic.
The unselected runtime F body still must not be demanded or executed.

The initial guard's cleanup receipt is
`obj/cleanup/1792795525468227666.jsonl`: zero disposable files remained because
successful reference object/native teardown had already removed them. Inputs,
diagnostics, all oracle rows and pinned evidence are retained. No recovery
compiler/probe result is claimed yet.


## Reached-source recovery and solved callable publication

Frozen `230f046e` guard73506 authenticated twelve prior oracle rows and observed
both new raw-read hypotheses exactly: selected constant F rejects with E0080;
the runtime opposite emits metadata/object code and executes with output nine.
Format and the 2,149-file audit passed. Strict Clippy stopped on one test-only
semicolon. Commit `a4b05d27` changes only that semicolon; an exact inverse restores
every audited source byte. Its guard36168 retained the authenticated audit/oracles,
then passed fresh formatting, all-target/all-feature strict Clippy, harness,
upstream and backend checks. All 37 private controls passed. The frontend batch
passed eight of ten test functions and stopped before native execution.

The two frontend failures are retained separately. The owned-argument negative
reaches the intended uninitialized/moved read rejection; its expected diagnostic
now names the actual `local may be uninitialized at this read` gate. The original
mapped-through-return source is valid Rust and was rejected as unsupported by
the lifetime-bearing entry check. Its callback ABI is closed, but its generic
reference argument retains a caller region whose static equality was already
proved. No native acceptance is inferred from the other frontend results.

The reviewed correction reuses `borrow::check::static_targets` after actual
region and loan proof. Original-purpose checks create no Code. Code-purpose
checks create one owned cold Prepared snapshot while the exact Body, Contract
and solved region identities are borrowed. The solved sparse callable map moves
into the existing Code.callables field; the Body remains unchanged. Empty static
solutions allocate no map and do not scan values. Existing canonicalization
caches bound graph work, and attempted fold/value work is charged to the CTFE
owner even on failure. The cold snapshot preflights payload, and Catalog checks
final mode, scope and index storage before registering the owned code.

CheckedInput owns the prepared snapshot, so expression preparation can safely
return it beside Templates. Catalog consumes that snapshot for ordinary,
anonymous, selected source and paid-child publication. Later Body metadata or
matching cardinalities cannot retarget its instructions. Actual source witness
and live parent-scope authority remain independent prerequisites. The opaque
defining-head branch uses the same completion helper rather than returning a
missing executable snapshot. No Body, Code, Frame, Machine, State or Engine
field is added; cold CheckedInput and Prepared sizes are reported separately.

The original mapped source bytes are preserved. A separate predeclared
`mapped-without-static-proof.rs` removes the static premise and retains a valid
Rust/native hypothesis with an explicit candidate lifetime-context boundary.
`contract-static-opponent.json` binds that new input before its independent
probe. The `needs_region_context` and callback-binder gates remain unchanged.
New private controls inspect actual raw/static F and G identities and the owned
snapshot's independence from a later changed source Body. The completed isolated
qualification and remaining limits are recorded next.


## Isolated qualification and integration handoff

Guard94779 (scope token `6177878b3c87418285142aa21d194c76`) finished with
status zero against frozen `736cb69a2c267ed8c4d9b0ac93dffc03a6363ec4`.
The evidence directory is
`obj/parallel/pa28/const-eval-select/ctfe-source-v4/`. Its source/runner binding
is `prepared-snapshot.json`; no source changed while the guard ran.

| Gate | Result |
| --- | --- |
| Retained oracle authentication | 14 valid rows retained; original two Index/E0608 disagreements preserved |
| New independent static opponent | metadata, object and native output `7 1` matched; valid Rust, required candidate unsupported boundary |
| Formatting / architecture audit | pass; 2,152 Rust files, zero findings |
| All-target/all-feature strict Clippy | pass |
| Harness / upstream / backend | pass |
| Focused private / frontend | 48 / 10 test functions passed |
| Native | four test functions, 52 declared source contexts passed |

The native contexts are ten initial CTFE/runtime-opposite programs, seventeen
runtime selector programs, eight retained const-callable programs and seventeen
reached-payment programs. Unsupported source cases are checked separately and
are not counted as language support. The retained original CTFE-positive source,
original mapped-return source and the original runtime scaling sources are
unchanged. The selected raw-null read reaches the provenance rejection; owned
argument reuse reaches its actual uninitialized-read gate. Both original Index
hypotheses, their declarations and all E0608 diagnostics remain intact.

Actual Code observation established const F raw type132 to target119 and runtime
G raw135 to target123. Both raw items required a caller region; both targets
were closed after the actual static proof. The same ItemId, public signature,
changed full arguments, actual F Code instance and idempotent target lookup were
asserted. The unconstrained opponent still rejects at the lifetime-context gate.

Measured cold records are CheckedInput320 and Prepared312 bytes. Hot sizes remain
Body448, Code304, Frame248, State448, Machine4728 and Value24 bytes. Engine1696,
SelectorCall56, retained input Row80 and payment Continuation80 bytes are recorded
separately. These are record sizes, not whole-heap measurements.

Default storage cleanup completed before lease release. Receipt
`obj/cleanup/1792799435419410196.jsonl` records 55 removals totaling 478,912
logical file bytes; the wrapper rounded unshared blocks to 0.00GiB. Inputs,
diagnostics, source/tool hashes and all failed predecessors are preserved.

### Commit ancestry and exact receipts

The qualified shared base and merge base with published `b8fea59e` is
`2da35cf08362ce978955d31cbfae7c5d055e7b40`. The isolated proposal is
`3184e65c7e291fc8e917a9fced07cb592c462232`. Merge
`dd4152646c2441a1172af57d6a5e6733fbf89b17` imports that qualified shared base;
its tree differs from the base only by the proposal/declared fixtures.
The proposal plus the six subsequent feature commits below form the seven-commit
cherry-pick sequence (the shared-base merge is not
a new implementation delta):

1. `a2a6dd1743a46b4034ce3bd670a1812453c1a6a2` — reached selector/F/tuple and existing continuation adapter.
2. `230f046ec695510762ea823ac3b22245c1be5e22` — preserve invalid Index hypotheses and declare the distinct raw-read pair.
3. `a4b05d27a52130070fe2354b86468a7e34e0e7bb` — test-only semicolon correction.
4. `111c60971e8eadeab0a7180fa916fe7b387fb4d3` — owned proved Code publication and solved static callable targets.
5. `4601c145c85698013552147ad3d2e5b8e023d32c` — retire the obsolete mutable target-map API.
6. `736cb69a2c267ed8c4d9b0ac93dffc03a6363ec4` — consume preflighted expression identity during registration.

The integrator staged proposal plus these six commits cleanly atop main `b8fea59e`
at validation branch `qualify-ctfe-selector-20261023`, HEAD `fc36be0e`; its
production dev/src tree matches this qualified isolated snapshot exactly.
A later documentation-only evidence commit does not change the qualified compiler.

| Receipt under v4 unless stated | SHA-256 |
| --- | --- |
| prepared-snapshot.json | e44e2619506b018e5ec7c71869b57a82a7f316103d97648ac53aca5b19568af4 |
| run.py | b9ef9d9bab005d846582c93f89b4f4fe6674d84e245b9f7775a05acebf9f9fb7 |
| checks.json | ef5b6792a338eac5df3a3b210ddd70ae5484e4441a8c122f17958c7ecae7a07f |
| retained-check.json | 8386f1b5829489e1e92cb1c1b0f52ef31ddc26cbd7dd14b7c88ae535de241501 |
| independent-v1/binding.json | 1f7f5f654c43b8ea266703d20cc2fea7191cebb45e8f826f40290609f6de0961 |
| independent-v1/results.json | 6fba77a3cb1fa3ec5304fc1baeebe5e739fcf4c0add7a406159fb355aa35906b |
| focused/receipt.json | a0fc599bf30d4258025dc881b3e324a329b2880dcd3868e1501d52addfd61729 |
| native/receipt.json | 0673d86354196fe6b1e828f8421620887f0509fab365b60fb05720cc7ebc2427 |
| resource token receipt in obj/parallel/resources | 5c034d67eaad717b2e0a65802f2ba871a881498346d6dc966c24f83c4fbcfdc1 |
| cleanup receipt above | 326b990c73de93890d8fd75ca7ecd4ae25b1bf9fb0f816b4c82dde4dc0619cd7 |

### Integration performance inputs and retained work controls

No worker benchmark or successor build was run. For A/B against the published
compiler, use existing inputs that both sides support and preflight exact output
equality. Ordinary CTFE controls include
`pa25/tests/spec/inline-specialized/const-loop.rs` (SHA
`98e368aad884814529fc205537fa9fd571be0814a343e43522cb7e598cbd5b6a`)
and `pa25/tests/spec/associated-execution/const-machine.rs` (SHA
`2235682caf38b12b5cb03e8ec0a51befb73f7dab68b9197b7f1b24ee3ccdbdcc`),
using the ordinary LowIR CLI profile. The existing source-lowering performance
adapter can retain `pa31/tests/spec/source-const-payment/work/repeated-reached-8/32/128.rs`
and the distinct-callers/distinct-instances families from the integrator's
`library-stability-callable-perf-v1/matrix.json`. Those paths exercise actual
reached constant execution and separate repeated uses from Code/instance growth.
Keep an ordinary scalar/string control as well.

Runtime selector A/B controls against current published main are
`pa31/tests/spec/const-eval-select-runtime/scale-mapped-8/32/128.rs` and
`scale-pointer-128.rs`, using the same existing source-lowering adapter and native
clients. Their exact source/client hashes are already frozen in the published
runtime contract and the integrator's matrix. That older matrix's candidate-only
labels described its older baseline; current main now supports these rows.
New CTFE selector positives remain candidate-only relative to main b8 and must
not turn an old unsupported rejection into a speedup claim.

The v4 private lane retained these algorithmic controls; these are candidate
work/storage observations, not timing or whole-frontend linearity measurements:

| Source row | Completed origin builds | Completed rows | Maximum index payload bytes |
| --- | --- | --- | --- |
| mapped8 | 2 | 142 | 284 |
| mapped32 | 2 | 574 | 1,148 |
| mapped128 | 2 | 2,302 | 4,604 |
| pointer128 | 0 | 0 | 0 |

Ordinary and pointer-two-target controls also build no origin table. Shared
origin chains at depths1/64/2048 retained3/129/4097 rows with zero warm extra
work, and failed work/storage budgets publish no usable rows. Selector relocation
preparation visits every value, including both Calls after prior/owner retention:
its same-item and conditional-owner controls charge exactly V+10, while no-catalog
and non-const paths charge zero. Warm actual F entry executes two callbacks plus
one caller (three calls, peak two frames), without selector/G frames. A step
budget128 rejects both fresh and warm cached F and retains attempted work.

The retained payment replay controls at widths8/32/128 pay every invocation and
bound fourfold growth by fivefold work/storage/execution limits; unrelated128
source definitions leave reached replay observations unchanged. These counters
start after Code materialization and do not measure the new snapshot construction
cost. Changed-path timing, peak RSS, output equality, A/A calibration and paired
ABBA remain required in the integrator's exclusive gate, followed by the merged
full release, affected source/native suites and active text checks.

### Remaining profile

Conditional selected-F eligibility/payment remains pending B's authority; the
empty/nonempty fixtures currently retain the explicit scoped-continuation
frontier, and nonempty selected conditional tuple payment is separately denied.
Open callback binders, unresolved/caller-dependent lifetime entry, open caller
signatures, unsupported caller owners, imported callback headers and selected
intrinsic execution remain explicit frontiers. The next read-only investigation
is the original reference-binder source and actual pinned core reference uses.
Default/abstract const-member execution still belongs to B's common eligibility
and execution gate. Full staged-provider const/default stability remains root-owned.
This checkpoint establishes neither whole-core acceptance nor assignment activation.
