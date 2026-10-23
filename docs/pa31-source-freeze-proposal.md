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

## Current source evidence and remaining qualification

The source-freeze-v4 independent plan corrects the retained v3 hypothesis: an
explicit `Freeze for &N` suppresses defaults for the entire shared-reference
family, including &M. Mutable references remain separate. Const and mutable raw
pointer families are likewise separate. All13 corrected object outcomes match;
five native positives return7. Six observations reuse exact v3 source hashes;
seven new controls were independently compiled. A stronger negative header not
required by its nominal is rejected by Rust even unused (E0367); an exact matching
nominal Copy-bound negative and unused reference-negative both execute7.

The first header-strength profile accepts exact instantiated formation clauses
and checks implicit Sized promises without the impl's own environment. Additional
implied, lifetime and quantified strength proofs remain explicit unsupported
requirements. Structural Freeze formation retains the existing well-formedness
check; nonempty returned formation proof is an explicit unsupported boundary.

Per-node debt completion examines each completed immediate child once and prepays
its bounded extraction visits. Pending, cycle and provisional distinctions remain
separate. The current source includes depth/noise work assertions and exact node,
Selection and selected-slot layout assertions, but they have not executed yet.
All-target check passed refutation-check-v4. Expanded strict development-v1/v2/v3
retains mechanical lint failures; source/native tests did not run behind them.
A further test-only return-type correction is included in this WIP.

Thirty-five unchanged owning source copies are under pa31/tests/spec/source-freeze
and are explicitly consumed by maintainer tests. Two Rust-accepted unused header
profiles remain unsupported; thirteen native-positive cases are required. No
stage activation, text reference, oracle or native acceptance is claimed.

The source Drop registration gate is still the temporary fail-closed prerequisite
until clean C475da0ea is integrated. Automatic cleanup633442 is independently Rust
native7 but is not yet a candidate pass. Direct helper, pointer wrapper, reified,
generic and CTFE glue execution remain distinct requirements owned with C.
