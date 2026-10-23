> Local unqualified WIP snapshot. Grammar/header phase passed; current expanded
> semantics are compiled but source/native/invariant qualification is pending.
> This is not the final accepted Freeze handoff.

# Actual source Freeze and negative implementation plan

The starting checkpoint is `1b8f8a15`, followed by root signature fix `9546ef11`
(cherry-picked locally as `fb317532`), with actual source Drop header and explicit
call restrictions. Root owns that public/owned Copy-signature pairing
repair. No changes to those Copy/materialization APIs belong to this work.

## Independent contract

`obj/parallel/pa31/source-freeze-v1` preserves the initial 22 declared sources.
Ten explicit implementation contexts omitted `freeze_impls`; their diagnostics
remain intact, including three falsified acceptance hypotheses. Core explicitly
requests `auto_traits`, `freeze_impls` and `negative_impls` at pinned lib.rs lines
112, 131 and 146. A separate `source-freeze-v2` context adds the missing gate only
to those ten sources and reuses twelve unchanged observations. All 22 corrected
object outcomes match, and eight accepted controls execute with answer 7.

Default auto-trait edges require the referent for shared references, mutable
references and raw pointers. Actual upstream-shaped unsafe implementations with
`T: PointeeSized` override that requirement, including when T has a negative
Freeze implementation. A zero-length array still requires its element's Freeze
bound. Negative fields propagate, generic T needs its actual Freeze premise,
positive/negative overlap rejects, and unsafe implementation syntax is required.
An identifier named `auto` remains legal. Macro-produced and cfg-disabled claims
retain their actual configured owners. Unused safe or non-auto Freeze headers
are Rust-accepted but outside the pinned provider profile.

C independently retained the real cleanup source SHA-256
`633442327cf88bd6f30745d737706695860e5ea96cdbfd2662a81cdb25e1d110`.
Its current candidate first failure is parser byte 126, `expected item after
qualifiers`, at unsafe auto trait. Nothing after that point is claimed reached.

## Grammar, identity and record changes

- Append weak Word::Auto without reserving it as an identifier.
- Retain auto_ on syntax ItemKind::Trait and semantic OwnerKind::Trait. Ordinary
  rendering omits a false flag, preserving the existing text contract.
- Source capabilities AutoTraits, NegativeImpls and FreezeImpls use cold u16
  Features. Requests and actual semantic use remain separate checks.
- Freeze is internal role16; DropGlue17 is reserved for the following helper
  checkpoint. Source drop_glue claims are admitted only with the exact completed header;
  the reached runtime consumer explicitly rejects until C installs real cleanup.
- Validate actual Freeze as the unsafe, nonconst, auto, empty, nongeneric source
  declaration. Other custom source auto traits either receive genuine semantics
  later or explicitly reject as unsupported; auto is never erased into ordinary.
- Every source goal remains canonical Bound::Trait(actual declaration). An
  internal Freeze query rule stays out of bootstrap AutoTrait::ALL and confers
  no synthetic declaration identity.

The immutable baseline DWARF receipt is under source-freeze-v2/baseline-layout.
Valid measured sizes are Node36, ItemKind28, Item64, OwnerKind20, Owner32,
SourceCore72, Package240, Resolution2032, Declarations7048, Evidence136,
search Node160, Frame1312, Trial656, Query656, Implementation96, Entry136 and
structural Obligation8. DWARF reports Selection as zero, which is **not valid size
evidence**. The immutable baseline-api-rust pin's exact Rust control subsequently
measured Selection136 (Evidence136), with OldSelected/ProposedSelected16. AST/owner boolean and cold Features changes are expected to fit padding;
that remains a measured check, not an assumption.

## Canonical negative headers and results

Canonical header.owner must resolve to the retained associated impl owner with
that exact self_definition. Its existing negative flag supplies source polarity.
Entry's former validated bool is replaced by a four-state byte status separating
Positive/Negative from Header/Validated. Source publication compares the actual
Inventory row; the explicit positive factory entry requires a registered
source-free schema. Extraction independently checks source polarity again.
No Entry field or separate Freeze-only implementation registry is added. Header
formation, feature/safety checks, overlap and normal prerequisite selection stay
mandatory. Pending matching headers remain pending. Negative candidates cannot
infer an unknown subject into their own Self type, justify themselves through a
coinductive cycle, or become positive implementation evidence.

Proposed result boundary, before shared selection edits:

```rust
Selection::Refuted(Refutation)
Refutation::DebtFree(Box<ClosedRefutation>)
Refutation::Conditional(Box<ConditionalRefutation>)
```

Both opaque payloads retain the exact goal, canonical negative owner/arguments
and immutable caller binding. A cold snapshot of the assumption index and actual
Environment Arc binds the result to its real caller; empty neutral callers remain
neutral. Constructors are private to completed search. ClosedRefutation requires
fully resolved actual inputs and conditions, no pending/cycle/provisional facts,
no allocated private region interval, no quantified scope and no remaining
lifetime/type/equality debt. ConditionalRefutation owns the complete ordinary
scoped proof and sized prerequisites instead of discarding them. It has no
conversion to absence or a paid false decision. The first consumers explicitly
report that nonempty negative debt payment is unsupported; those valid source
forms remain required continuation work.

Internal Decision and the selected-candidate tag distinguish refutation roots.
Implementation provenance may remain in the existing graph, but the positive
Evidence extractor must validate positive polarity. Refutation extraction is a
separate constructor. Positive memo/reuse exports never admit refutations.
Debt-free refutations may retain their own query-local tag; conditional results
are never cached as absence. A refuted child with any remaining debt propagates
that pending/conditional status rather than making a parent candidate absent.
Ambiguous positive/negative alternatives remain unresolved until ordinary
coherence/selection proves a unique applicable result.

Selection's existing positive Evidence payload should dominate the new small
closed tag; only conditional proof data gets a new box. Replacing the current
Option<usize> selected slot with an equally sized tagged choice is a measured
layout requirement. No graph-node polarity field or global TypeId permission is
planned. Charge negative graph/extraction work through the existing query budget.

## Consumers and proof limits

- State positive-goal checking validates the exact refutation goal/caller. Only
  debt-free refutation becomes an unsatisfied goal; conditional data produces an
  explicit unsupported boundary until owned payment exists.
- C's source Copy decision path may not convert conditional refutation, pending
  selection or an unresolved header into non-Copy/move permission.
- Method applicability reports unsupported scoped payment in this first profile,
  preserving those valid cases as required work. Dispatch,
  normalization and projections never use a negative owner as a selected method
  or associated-type witness.
- Existential coherence queries may not export their assumptions/debts as caller
  evidence; uncertain conditional results mean possible overlap.
- A's later trait metadata must retain actual auto/negative/constness declaration
  data and full clauses, not export refutation as execution authority.
- Structural Freeze reuses the existing iterative auto graph. Its source-identity
  adapter maps child requests back to the actual canonical application. Real
  positive/negative candidates and matching header waits precede structural
  fallback, including actual reference/pointer overrides.

The next independent controls must include incomplete headers, negative cycles,
forged positive extraction, cross-environment replay, nonempty debt, unrelated
positive cache reuse and source order reversal. Generic/reference application
identity and ordinary formation proofs remain intact.

## Following drop_glue checkpoint

The actual pinned lang helper has the exact const unsafe Rust function contract
`drop_glue<T: PointeeSized>(&mut T) -> () where T: [const] Destruct`. Its unused
placeholder body may undergo ordinary original proof, but no reached runtime,
CTFE or reified use can execute it as an ordinary function. Publication requires
the checked actual owner/signature/clauses and a real cleanup consumer, or an
explicit unsupported boundary. Conditional const debt stays at its actual reached
execution boundary. Existing implicit cleanup remains owned by C's validated
canonical destructor path, not by a textual helper alias.

## Qualified local closing checkpoint

The local close is qualified against `bb56fb92` (the explicitly unqualified
`aa8cccd9` source-role snapshot, clean C475da0ea NeedsDrop registration/kernel,
and ff9c6869 layout-only corrections). It is not a combined release or whole-core
acceptance. The earlier WIP claim is superseded only by the receipts below.
Root owns combined qualification and publication.

Actual Freeze16 and DropGlue17 retain real configured declarations. Freeze is an
unsafe, nonconst, empty, nongeneric auto trait in this provider profile. Other
source auto traits remain explicitly unsupported. The active `auto_traits`,
`negative_impls` and `freeze_impls` capabilities do not grant arbitrary trait
permission. Positive/negative impl headers keep actual polarity through canonical
header preparation, coherence, selection, completion and extraction. The four-state
header byte preserves Entry136; registered source-free factories are positive-only.
A forged/missing source owner cannot select that path.

Negative results remain separate from positive Evidence. Debt-free refutation
retains the actual goal/owner/arguments and immutable caller authority. Conditional
refutation keeps its graph and is explicitly unsupported at consumers which cannot
pay it. No negative proof enters positive extraction, callable dispatch or memo
exports. The first header-strength profile accepts exact instantiated nominal
formation clauses and checks implicit Sized promises without borrowing the impl's
own environment. Additional valid implied/lifetime/quantified strength proofs
remain required unsupported work. Lifetime-only Self arguments now come from the
actual canonical nominal template, not an absent type-parameter table.

The 35 original source cases retain two Rust-accepted unused header profiles
(safe/non-auto Freeze) outside this provider profile. They are unsupported Rust
positives, not language-negative claims. All 35 declared candidate outcomes pass;
13 original native positives return7. Initial falsified hypotheses and missing
feature contexts remain in source-freeze-v1/v3 evidence, with corrected independent
v2/v4 observations kept separately.

## Retained formation and measured work

`Proof::new` has no scoped trait-selection fold. Calling its full nominal check
from structural Freeze therefore failed the unchanged W<N> source at a real source
sizing obligation. The repair keeps those obligations in ordinary iterative
selection before physical fallback. It does not delete formation, infer proof
from layout, invoke recursive fresh State queries, or weaken equality.

The first continuation prototype collected flattened requirements per frame.
Its measured depth8/32 work was4,837/71,461 and edges442/6,370; depth128 exhausted
the existing work budget. This prototype is retained in development-v11 and is not
the final algorithm.

The final query uses immediate nominal contracts and explicit formation-only
child frames. The existing family index is a typed map: Member-active semantics
remain unchanged; Nominal entries are Active or Completed(private WF node), keyed
by exact TypeId, surrounding callable depth and ScopeId in one immutable caller
query. The existing `lift` rules still defer binder-local obligations. Every
immediate dependency remains a graph edge; shared completed WF nodes replace
flattened prefixes. A formation-only node has no selected source and cannot enter
ordinary trait caches, positive extraction, dispatch or export. Actual source
selection resumes after its complete prefix, even when it is empty. Active keys
remain live and auto coinduction cannot cross formation prerequisites.

Proven/Refuted answers retain the prefix and all child/region/Sized obligations.
Deferred stays retryable. An Absent answer cannot discard an unpaid prefix: this
first profile explicitly diagnoses that case rather than publishing/caching false.
Source Freeze, MetaSized/PointeeSized and ordinary Destruct use the shared physical
shape accessor only after this completion gate.

Final charged work includes actual collector visits, including duplicate edges:

| Axis | 8 | 32 | 128 |
| --- | ---: | ---: | ---: |
| Nested depth, work | 1,428 | 5,772 | 23,148 |
| Nested depth, nodes | 91 | 355 | 1,411 |
| Nested depth, edges | 142 | 574 | 2,302 |
| Repeated width, work | 248 | 296 | 488 |
| Repeated width, nodes/edges | 17/23 | 17/23 | 17/23 |

Each observation is identical with0 or128 unrelated proof nodes. Private tests
assert linear bounds and cross-extent growth. Active/completed/binder-depth cache
states, no-early-publication, warmed reuse, source=None extraction denial and
unpaid-absence attacks pass. The exact pointer-override opponent confirms that a
nominal containing N need not demand Freeze for N when the real source pointer
impl deliberately ignores its negative Freeze implementation.

Ordinary non-source queries take a constant-time path and allocate no formation
state. Formation is cold176→200bytes (+24 for immediate dependency storage).
Its shared-map key/entry are24/16bytes; member demand consequently uses a larger
cold map row than its former20-byte set key. Query656, Frame1312, Node160,
Selection/Evidence136, Trial656, ASTNode36, ItemKind28, Item64, OwnerKind20,
Owner32, SourceCore72, Package240, Resolution2032 and Declarations7048 are unchanged.

The final DWARF measurement requires complete fields and the18-slot array bounds:
Catalog464, Reservations472, Declaration20, Validated28. Each language array is
360bytes, so the two additional roles account for40bytes in each cold array.
C's prior destruction-registration pointer remains a separate8-byte change.
These are source-demand owners, not ordinary dense record growth.

## Actual Drop captures and cleanup

The unchanged cleanup source SHA256
`633442327cf88bd6f30745d737706695860e5ea96cdbfd2662a81cdb25e1d110`
exposed source Drop's incorrectly hardcoded zero capture count. Registered
Implementation.parameters was1 while FunctionRegions.captures was0, collapsing
the impl lifetime with the receiver late slot. Actual source Drop now validates
its canonical role/header/schema/target, takes captures from that registered
implementation and validates the ordinary callable template. Separate late
receiver slots, original/specialized proof and conformance remain mandatory.
Bootstrap cleanup keeps its existing flattened argument mapping; it has no
ordinary source Drop header to justify this source-only adapter.

Named/elided/static and two-capture controls preserve seven independent Rust
observations, including the ordinary bootstrap reference. All six source-profile
candidate outcomes match, and the four native positives return7/7/7/11. The
unchanged automatic cleanup program now executes7. This is real scope cleanup
through C's exact registered destructor, not execution of the unused glue stub.

DropGlue17 validates the actual const unsafe Rust helper, one PointeeSized
parameter, one independent late &mut T→unit signature and Conditional Destruct
clause. `drop_glue_member` classifies the exact actual DefId; pending matched
readiness errors. Until C installs its independently proved runtime consumer,
ordinary direct/reified selection explicitly rejects runtime lowering. The empty
placeholder is never ordinary execution authority. Direct helper/ptr wrapper,
reified/generic, slice/dyn/ZST and CTFE cleanup remain separately required; paid
Conditional→Const execution still awaits its scoped continuation. Full upstream
Drop/Pin/Deref and Unsize/CoerceUnsized contracts remain required as recorded by C.

## Receipts and consumer handoff

All paths below are relative to this worker checkout:

- `obj/parallel/pa31/source-freeze-v5/development-v15`: strict plus13 private
  and7 frontend roots; all35 originals,13 additional formation cases, capture
  cases and the existing Copy/Clone/FnOnce original/instance controls pass.
- `obj/parallel/pa31/source-freeze-v5/closing-v16`: fmt, all-target/all-feature
  strict, audit2003/0, registration364roots/13binaries, six invalidated private
  tests plus the moved capture root, and four native roots containing28 answers.
  Final exact work/layout replay is in `final-counts.stdout`.
- `nominal-source-formation-v1` preserves the independently valid but unsupported
  Destruct header context; v2 adds the exact deny-explicit-impl attribute in a
  distinct10-source plan (8 native7,2 Rust rejections). v3 independently qualifies
  the pointer opponent and callable-local positive/native7 and negative.
- `source-drop-captures-v1` preserves all7 original Rust outcomes/native receipts.
- `closing-v16/cold-layout.*` preserves the complete-field DWARF measurements;
  missing/zero/ambiguous layouts are not accepted evidence.

Tests read exact owning fixture bytes at runtime; include macros and a production
fixture dependency were removed. Private helper file names reflect construction,
WF graph work and cache scope ownership. No audit suppressions, source/AST record
relaxation, stage activation, oracle regeneration or active-reference edits occur.
No whole-core acceptance, generic inspector completion or combined release claim
is made. The unchanged source-library/PA26/PA30 remaining requirements survive.

C consumes the existing stable APIs:
`Catalog::validated_language(LangItem::DropGlue)` and
`compiler_items::drop_glue_member(facts, DefId) -> Result<bool, &'static str>`.
Its runtime adapter replaces only the explicit Ordinary::select boundary after
complete actual Instance/materialization/original/specialized proof. NeedsDrop's
exact registration marker and both mandatory pre-proof/pre-cache gates are intact.
