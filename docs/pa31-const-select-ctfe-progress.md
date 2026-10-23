# Constant selector source implementation draft

The preserved interface/declaration checkpoint is `3184e65c`. The qualified
shared API merge is `dd415264`, whose tree differs from root `2da35cf0` only by
that proposal and its declared fixtures. The production draft below is
**unqualified**: no C build, audit or candidate execution has run. The first
independent-only guard and its recovery hypotheses are recorded below. Recovery
submission remains held while B takes its next coordinated development lease.

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
