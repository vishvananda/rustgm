# Scoped const qualification prepare split (review proposal)

The scoped decision API and typed initializer split are now authored; the type
request/classifier/refinement connection below remains unfinished. The eighteen independent
controls in histo.json all match the pinned Rust compiler (reference-v15).
The parallel per-coordinate history sketch is superseded. One sticky BAD
qualification bit per existing owned root follows the pinned resolver law;
existing coordinate/tag/need facts continue to describe selected cleanup.

## Current owner and proposed split

Symbolic: borrow/loans/templates.rs Verifier::prepare_body currently owns the
physical initialization result, then calls symbolic selection (which refines
once), then constructs templates::Prepared and its existing Seal. Move creation
of that SAME Prepared/Seal immediately after physical initialization. Its
existing optional plan stays empty only during an explicitly unfinished cold
partition stage. Finish/active/eligibility must reject that unfinished stage.
No-enum/no-qualification bodies complete through the existing immediate path.

Concrete: borrow/check/cstage.rs prepare has the analogous initialization,
refinement and select_reusing sequence. Seal immediately after initialization;
reuse its existing inline empty Plan during the unfinished cold stage (do not
change every Prepared to Option<Plan> or grow it). The pending state belongs in
the existing optional paid Partition. Every finish validates that it completed.
Preserve current-main generated CopyView/Permit joins when integrating.

Split initialization::Facts::refine_variants into preparation of its existing
cold Refine/Partition owner and completion of its existing ONE refined fixed
point. This adds no initialization pass. The symbolic/native builders share
existing selection kernels after that completion, instead of calling refinement
again. Existing ordinary public selection retains its old combined wrapper.

Pipeline then does, in order:

1. Obtain physical facts plus the same sealed unfinished Prepared.
2. Resolve full semantic root/coordinate TypeIds using the shared field fold;
   authenticate all retained constructor candidates, including unused ones.
3. Classify needed complete types in the actual source definition scope.
4. Append successful scoped evidence to the real owned Contract; recheck its
   regions before any classifier answer can drive qualifier publication.
5. Revalidate the existing seal (body, store/definitions arenas, owner,
   signature, environment, Copy provenance); finish the one refinement and
   select the one real cleanup plan. No source type, binder, signature, premise
   or place identity may be changed by classification.
6. Resolve/publish selected semantic requests and finish existing region,
   ownership, drop-check and loan validation. Code activity stays physical and
   remains fenced from CTFE execution.

## Exact scoped query law

The classifier belongs in existing compiler_items/cdrop and uses the existing
actual defining Prepared owner (proofs::prepare), not a new role registry.
Requests are full TypeIds owned by the paid cold graph and the same actual
body/environment/effect/source/arena scope. Deduplicate by full type, never by
physical region-erased shape or nominal spelling.

A classification is one of:

* NoDrop: existing scoped NeedsDrop is false. This is value filtering, not a
  Whole const Destruct claim. Retain normal Source Copy provenance separately.
* Destruct: a genuine actual-language Destruct application succeeds in the
  authenticated definition effect/scope. Its full Obligations are retained in
  the existing Contract proofs/defining claims with their actual purpose.
* MayQualify: only a completed unsatisfied application (ordinary Absent or
  debt-free Refuted). It is not an error, assumption, proof, or runtime token.

State::prove currently folds Deferred into false. Do NOT use that bool API for
this classifier. Factor a small typed decision path in the SAME State/select
kernel: Proven retains evidence, Absent yields completed unsatisfied, Deferred
returns a pending dependency error, Refuted must pass existing reject (an unpaid
conditional negative remains an error), and invalid identity/budget/ambiguity
errors propagate. Never catch these errors and return MayQualify.

Isolate each query's inference progress so false/error cannot leak equations or
constraints into a later query. Share the actual caller assumption Arc/index and
owner lifetime fold rather than reconstructing the full environment per type.
A fresh query Context can use the existing select_requirement dispatch and
normal Caller; append only successful evidence/equalities to the owner. Existing
selection reuse must retain its own scope laws; do not clone a Store or invent
an alternate trait solver. Add false-then-true/cold/warm/conditional-negative and
malformed-scope opponents before relying on this join.

## Payer and answer ownership

The existing paid Partition/Selected is the sole cold owner of full semantic
TypeIds, deduplicated requests, immutable answers, transfer indexes and scratch.
Expected Root20->24 (one qualifier bit identity), Node76->68 (remove partial
Range); actual widths must be measured. No Body/Store/Plan/Proved/Code field or
ordinary Prepared record grows. Type query request/map/scratch growth, full
field substitution, selection work and retained proof/region storage are paid
before append/publication under existing per-body limits. Fold reuse stays by
complete semantic type/application.

An answer set is tied to the borrowed actual body, current arenas, actual
validated source definition/signature/effect, environment Arc, exact request
TypeIds and proof frontier. It is consumed only after the corresponding region
recheck and existing seal validation. No loose bool answer or imported policy
can bypass that join. The same actual Contract owns all successful obligations;
failed/pending queries append no evidence. Ordinary/native destruction and
physical initialized/read/loan facts never consult these classifier answers as
permission.

The private replay-owned foreign Selected swap still needs executable evidence
before adding ownership state. No such witness has been added from inspection.

## Transfer/oracle mapping

* hmov/hmvk/hmvn: capture destination first, clear unexposed whole source only.
* hcop/hcpn: structural constructed value vs unknown whole-type input; Copy
  no-drop filtering does not become const Destruct(T).
* hjoin/hjbad: OR join of actual incoming qualification.
* hmraw/hmshr: sticky raw mutation exposure vs shared Freeze borrow.
* hrnew/hrold/hrpre: qualification never clears merely on whole overwrite;
  actual replacement cleanup order still applies.
* hcont/hcntp: projected-type filtering then root-local qualification; precise
  policy cannot authorize a contaminated projected value's active cleanup.
* hsub/hsubp: partial field reinitialization does not reset root qualification.
* hwrap/harray: known nested/merged values remain value-based, with no array
  length expansion.

Tag tests cannot clear the sticky BAD bit. Current selected-coordinate facts may
exclude only their own impossible payload and remain independent of fresh,
sibling, replacement, read, loan, unsafe and ordinary-call obligations.

Scope still required: exact request/proof/answer API typing before implementation;
remaining seven original positives and ordinary body causes; private
foreign/stale/purpose/full-argument controls; no-enum/noise/shared-declaration and
array scaling; native cleanup observers; actual CTFE scheduling/execution.

## Concrete interface checkpoint

The implementation uses a stronger typed unfinished token rather than an empty
Plan sentinel: loans::templates::Initial owns actual verified physical Facts,
parameters/builtins and the original Seal. Verifier::begin produces it;
select_initial revalidates and consumes it into unchanged Prepared. Facts::body
returns its stored body borrow lifetime. No finish operation accepts Initial.

Native cstage has the analogous Initial without Plan, initialize_using and
select_initial. Existing prepare/prepare_using interfaces remain wrappers;
generated CopyView/Permit checks still precede physical initialization. The SAME
Seal moves into Prepared. Incremental source-const begin/select exposes this
boundary before the recipe Catalog borrows the Store. Final finish is left
available for the async worker's finish_sealed union.

Batch::begin returns InitialRow owning the actual source input, region view,
verifier/cursor borrow and unfinished Initial. Its select consumes that token
into the existing Row. It cannot finish; dropping it leaves cursor/loan coverage
incomplete, so Batch::finish rejects publication. Future arena witness remains
only in successful final Batch::finish.

This is an infrastructure checkpoint, not qualification completion: no BAD bits,
new scoped type request set, classifier answers or historical permissions have
been published. Seven independently positive historical cases and the masked
ordinary-body cause remain required. CTFE cleanup still has its physical fence.

Validation so far: qualify-api2 strict all-target/private metadata PASS;
qualify-api3 focused strict PASS; qualify-proof1 seventeen unchanged public
source tests PASS. Init2 symbolic split strict PASS; first native+Batch strict
attempt stopped at one owner pass-by-value lint; Init4 retries that exact fixed
wrapper. No new private execution or native cleanup claim.
