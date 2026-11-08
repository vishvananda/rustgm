# Census batch 11: the overlap budget, specialization fixpoint, feature-bound const parameters and the twelfth census

Census 11 ([batch 10](census-batch10.md), pin `fcd444bc`) checked 42,310
declarations of original core with none failed or blocked. Its one record was
the budget record "implementation prerequisite work budget exceeded" at
core_simd's `ops/shift_scalar.rs:802`. This batch, rebased onto main
`016b1f54d` (Lane C's `PhantomData` role):

- answers coherence pairs whose trait references differ at a rigid
  constructor before instantiating either header, as rustc's deep reject does,
  and corrects how reserved overlap conditions are charged (section 1);
- repairs a selection regression from `f9a2c96d4`: a pattern marker whose base
  lacks `Copy` relisted its impl candidates until selection work ran out
  (section 2);
- solves a specializing child's parent obligations to a fixpoint and omits the
  parent arguments the parent's own equalities determine, as rustc's
  `specializes` and `unconstrained_parent_impl_args` do. This repairs the
  record the budget repair uncovered, core's `SpecExtend` pair (section 3);
- proves a const parameter's type under its owner's environment. This repairs
  lookahead record 2, core's `TransmuteFrom<Src, const ASSUME: Assume>`. A
  follow-up repairs the regression it caused at core's `Arguments::new<const
  N, const M>`, which the first census 12 pin found (section 4);
- runs census 12 and locates its record (section 5).

Census 12 (`a108ae93`, from `13fcdf82d` on main `016b1f54d`) checks 42,310
declarations, with none failed or blocked. Its crate record is "minimal
specialization introduces a non-marker predicate", unlocated, then 7
lookahead records. d9 locates it at core's `SpecNextChunk` pair
(`array/mod.rs:1044` and `1062`): a `[const]` parent bound does not admit the
ordinary bound that the child's marker supertrait elaborates to. It is
reduced, with a proposed repair, and not repaired (section 5).

Out of scope:

- Lane B owns re-exports, crate loading and the nightly repairs. Lane B
  changed `sema/bodies/field_storage.rs`, `finalization.rs`,
  `types/aggregate.rs` and `aggregate/preparation.rs`; this batch touches
  none of them.
- Lane C owns body and item transport, the static `Sync` check in
  `sema/declarations/validation.rs` and `constrained_scalars.rs`; neither file
  changed here.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend is frozen.
- The `[const]` supertrait gap (batch 10, section 3) was not attempted.
- Census 12's record is located and reduced, not repaired (section 5).

## Diagnosis method

Each diagnostic was an analysis-only pin of the working tree with one census of
unchanged original core inputs, in `obj/parallel/census-batch11/diagnostics/`.
Their binaries are retired. d1 to d6 ran on `71f4bc1e9` plus temporary
instrumentation and raised caps; d7 ran on `e4271ffe7` with instrumentation
only. d8 is a census of the pre-rebase pin `91b8ab1c` (from `a7b65202c`, the
four commits on main `e07b0c511`). d9 ran on `7f951ab78` with instrumentation
only. Those three trees are earlier states of this branch; the rebased
commits are listed under Commits.

| Run | What it adds | Checked | Failed | Blocked | End |
|---|---|---|---|---|---|
| Census 11 (`fcd444bc`) | | 42,310 | 0 | 0 | budget record: implementation prerequisite work budget exceeded (`shift_scalar.rs:802`) |
| d1 `d1-pairs` | per-pair work and a rigid-constructor reject count, caps raised | 42,310 | 0 | 0 | instrumented report, then 3 lookahead records |
| d2 `d2-conditions` | condition work by owner | 42,310 | 0 | 0 | as d1 |
| d3 `d3-sizes` | header node counts against work | 42,310 | 0 | 0 | as d1 |
| d4 `d4-deferred` | the record under raised caps | 42,310 | 0 | 0 | crate record: specialization implication still awaits a declaration dependency: Some(None) (`iter/traits/collect.rs:589`), then 3 lookahead records |
| d5 `d5-phases` | condition sub-phases | 42,310 | 0 | 0 | as d1 |
| d6 `d6-reserve` | condition reservation charges | 42,310 | 0 | 0 | as d1 |
| d7 `d7-postfix` | section 1 committed, caps restored | 42,310 | 0 | 0 | instrumented report, then 8 lookahead records |
| d8 `d8-prerebase` | sections 1 to 4, before the rebase | 42,310 | 0 | 0 | crate record: minimal specialization introduces a non-marker predicate (unlocated), then 8 lookahead records |
| Census 12, first pin (`2ad5137b`) | `7f951ab78`, rebased | 42,310 | 0 | 0 | the same 9 records as d8; record 3 is section 4's regression |
| d9 `d9-marker` | the first pin's tree, naming record 1's impls and clause | 42,310 | 0 | 0 | instrumented record 1, then the same 8 lookahead records |

Small drafts reduced each record into the PA31 controls of sections 1, 3 and
4. Every control was declared before its frozen rustc 1.98.0 oracle
(`obj/parallel/census-batch11/oracle/`). Exploration drafts in
`obj/parallel/census-batch11/repro/` were also run with rustc 1.98.0 and the
first census 12 pin; they are not declared controls.

## 1. The overlap budget (census 11's record)

Core forms 22,895 coherence pairs. d1 measured 2,404,914 overlap steps against
the fixed one-million cap, and 2,466,187 units of crate-wide pending work.

- 20,986 pairs have trait references that differ at a rigid constructor, yet
  each instantiated both headers first. They cost 984,440 steps, 846,588 of
  them in header instantiation. rustc's `overlapping_impls` answers such a
  pair with `DeepRejectCtxt::args_may_unify` before instantiating either
  header.
- The 1,456 pairs whose heads equate spend 1,378,629 steps on their
  conditions. `ptr/mod.rs`'s `FnPtr` blanket impls account for most of it:
  one owner alone has 431 pairs and 398,489 steps. d5 and d6 show where the
  work goes. Of 1,289,791 condition steps, 1,240,896 were the byte size of
  reserved condition vectors, charged as steps: `reserve_traits` passed its
  bytes to the step ledger.

The repair (`76ebf2e58`):

- `overlap/deep_reject.rs` compares the two trait references before
  instantiation, Self first, eight constructor levels deep, as rustc's
  `types_may_unify_inner` does. Parameters of either impl, projections, opaque
  types and lifetimes may unify with anything. A rejected pair pays one step
  per compared node pair. A difference below the depth, behind an alias or
  under a parameter's projection still reaches the complete equation.
- A reserved overlap condition is charged one step in the pair's overlap
  ledger and, for a source crate, one step in the source-header retention
  ledger. Its bytes go to the retention byte ledger only when the vector
  regrows, and only for the newly reserved elements. The one-million cap
  stays as runaway protection; no size-based cap was added.

Measured after the repair (d7): 263,268 overlap steps (was 2,404,914) and
324,541 units of pending work (was 2,466,187). The reject compared 22,555
pairs and rejected 20,674. None of the rejected pairs has headers whose
complete equation succeeds. The other 340 pairs are answered before any
comparison, as in d1. The committed reject answers 312 fewer pairs than d1's
temporary instrument counted; that difference was not investigated.

Controls: `pa31/tests/spec/deep-reject/` (19 cases, 19/19 agree):
disjoint rigid differences, guards past the depth and a closed projection,
and opponents behind a parameter, crossed arguments, a lifetime, a const
parameter, an alias and two projections (E0119).
`dev/tests/invariants/rejected_overlap_pairs.rs` checks them, and checks that
the reject's work is constant across 6, 24 and 96 impls at header widths 1 and
8, while the overlapping control's work grows with the header. It also audits
every pair in twelve PA31 coherence and specialization packets: no rejected
pair unifies.

## 2. The `f9a2c96d4` regression

`f9a2c96d4` made the source-nonnull opponent whose pattern base lacks `Copy`
(`abi::storage_relation::tests`, one of main's known CI failures) exhaust
selection work instead of reporting the missing `Copy`. It listed impl
candidates again after every forwarded prerequisite. That is right after a
declined reference referent, which is the user-callable-output case, but a
refinement marker forwards after its impl candidates were already listed. The
repair (`32e0d8259`) records each forwarding's origin. Only a declined
referent relists; a declined operation prerequisite completes with the
frame's own decision; a refinement keeps its decision. This is a different
cause from section 1, so it has its own commit.

`dev/tests/invariants/user_callable_output.rs` keeps the user-callable-output
controls and adds two scaling invariants over 6, 24 and 96 unrelated impls:
each subject without a signature or referent lists its candidates exactly once
(lists of 1, 2, 1 and 2 for `S0`, `&mut S0`, `Plain` and `&mut Plain`), and a
pattern marker without base evidence never relists.

Amended control, at integration on 2026-11-08: those invariants read a
`#[cfg(test)]` `candidate_lists: Vec<usize>` log on the selection `Query`. The
joined check found that this grows the test build's `Query` from 656 to 680
bytes, which fails `sharing_sizing`'s hot-layout assert. The production layout
is unchanged, because the field exists only under `cfg(test)`. The assert now
reads `656 + size_of::<Vec<usize>>()`, naming the test-only field. `Node` and
`Frame` are unchanged.

## 3. Specialization over where-clause-only parent parameters (d4's record)

With the overlap budget raised, d4 stopped at "specialization implication
still awaits a declaration dependency: Some(None)" at core's `SpecExtend` pair
(`iter/traits/collect.rs:574` and `589`). The parent names `T` only in where
clauses. Its `T: Sized` goal stays ambiguous until `ExtendT: Extend<T>` infers
`T`, and an ambiguous goal stopped the comparison with no dependency to wait
for.

rustc's `specializes` registers every parent obligation under the child's
environment and solves them together. Leftover ambiguity means the child does
not specialize; it is never an error, and it never rejects the impl. The
repair keeps each direction's goals in passes: an ambiguous goal is retried
after any other goal is proven. A pass with no progress ends the direction as
"does not specialize". Each pass and its deferred goals are charged.

Once `T` is inferred, `min_specialization` refused the parent's equality
`I: Iter<Item = T>` by name ("projection-constrained parent arguments are not
implemented"). rustc's `check_always_applicable` applies its repeat and
`'static` checks only to `unconstrained_parent_impl_args`, which drops each
parent parameter that a constraining position of one of the parent's own
equalities determines, in one ordered pass. `Constraints::projection_determined`
(`types/parameters/constraints.rs`) is that pass, and `minimal_admission.rs`
filters the parent arguments with it.

Controls, both checked by `dev/tests/invariants/inferred_parent_obligations.rs`:

- `pa31/tests/spec/specialization-fixpoint/` (4 cases, 4/4 agree): core's
  shape and a concrete `u8` specialize; an opponent that infers a different
  `T` and one that leaves `T` ambiguous do not (E0119).
- `pa31/tests/spec/determined-parents/` (6 cases, 5/6 agree): rustc's own
  `check_duplicate_params` example and a determined `'static` lifetime,
  ordered by a specialization marker, are accepted; their guards without the
  equality are rejected. The first declaration predicted that the unmarked
  determined lifetime pair specializes. rustc reports E0119 for it, because
  `specializes` ignores region constraints and so holds in both directions.
  That wrong prediction is kept, and the marked pairs were declared
  afterwards, before their own oracle.

## 4. Feature-bound const parameter types (lookahead record 2)

Record 2, "const parameter type must implement actual ConstParamTy" at
`mem/transmutability.rs:91`, was genuine. `TransmuteFrom<Src, const ASSUME:
Assume>` carries `#[unstable_feature_bound(transmutability)]`, and `impl
ConstParamTy_ for Assume` holds only under the same bound. rustc's
`check_param_wf` proves `Ty: ConstParamTy_` under the owner's param-env, which
contains the owner's feature bound.

The shared closed proof is unchanged. `Proof::prove_closed_trait` in
`sema/nominals.rs` has two callers: `generics/validation.rs:130`, which proves
an item's global where clauses without its environment, and
`Checker::prove_const_parameter_type` in
`sema/declarations/generics/parameter_kinds.rs`. Only the second changed. It
now calls the environment proof `Proof::prove` with the owner's declaration
proof, which it already built. A const parameter type is closed, so the
environment adds only feature bounds and global where clauses. RustGM does not
implement `trivial_bounds`, so a global where clause must already hold without
the environment through the first caller. A closed type therefore still cannot
justify itself.

Controls: `pa31/tests/spec/feature-bound-const-parameters/` (5 cases, 5/5
agree after one recorded fixture amendment). Core's trait shape and a function
with the same bound are accepted. An unbound trait, a trait bound to another
feature and a global `where Assume: ConstParamTy_` are rejected; the last one
fails at its where clause, as in rustc. `dev/tests/invariants/unstable_bound_parameters.rs`
checks every case.

### The second const parameter (the first census 12 pin's record 3)

The first census 12 pin added lookahead record 3, "invalid lifetime constraint
identity" in `validate_trait_regions`, at the second const parameter of core's
`Arguments::new<const N: usize, const M: usize>` (`fmt/mod.rs:729`), a method
of `impl<'a> Arguments<'a>`. d7 does not have it. Drafts in
`repro/two-const/` show that rustc 1.98.0 accepts each shape. The pin rejects
an owner with a lifetime and two const parameters, and accepts one const
parameter or two without a lifetime.

The cause is the repair above. `validate_const_parameter_types` built one
declaration proof per owner and retained its lifetime witnesses after each
parameter. Retaining takes the proof's trait state, so the next parameter
started a fresh state at the same first region, and its witnesses collided
with the retained ones. The closed proof had retained no witnesses, so the
reuse had been harmless. The repair (`13fcdf82d`) builds each parameter's
proof with `declaration_proof`, which numbers it after the owner's last
retained proof.

Controls: `pa31/tests/spec/const-parameter-environments/` (5 cases, 5/5
agree): core's shape, three const parameters under two lifetimes, a function
and a trait method with two are accepted; core's shape whose second parameter
type lacks `ConstParamTy_` is rejected (E0741). They were declared after the
drafts' runs, which the declaration records. The first census 12 pin rejects
all four accepted controls with the census message.
`unstable_bound_parameters.rs` checks every case.

## 5. Census 12

The first pin, `2ad5137b`, was built from `7f951ab78` on main `016b1f54d`
with an empty uncommitted patch. Its census had the same nine records as d8,
so the rebase changed none. Its record 3 was a regression from section 4,
repaired by `13fcdf82d`. Census 12 is the census of the pin built after that
repair.

The pin `a108ae93` was built from `13fcdf82d` with an empty uncommitted patch.
Its ordinary observation stops at "minimal specialization introduces a
non-marker predicate", with 375 sources opened. The census
(`obj/parallel/census-batch11/qualified-3/census/`) checks 42,310 declarations
of original core with none failed or blocked. It has 8 records: one crate
record, then 7 lookahead records.

| # | Pass | Site | Message |
|---|---|---|---|
| 1 | `validate_trait_implementations` | unlocated (d9: `array/mod.rs:1062`) | minimal specialization introduces a non-marker predicate |
| 2 | `validate_trait_impl_owners` | `num/imp/dec2flt/common.rs:20` | trait implementation requires conformance and selection semantics |
| 3 | `local_annotations` | `num/niche_types.rs:122` | specialization graph validation previously failed |
| 4 | `complete_alias_normalizations` | `num/niche_types.rs:122` | missing completed const argument |
| 5 | `validate_objects` | `any.rs:152` | source receiver compatibility requires the completed immutable impl epoch |
| 6 | `complete_discriminants` | `mem/alignment.rs:440` | associated type requires an unambiguous validated implementation |
| 7 | `complete_layouts` | `ascii.rs:29` | enum layout requires completed discriminants |
| 8 | `require_specialization_execution` | unlocated | specialization graph validation previously failed |

Lookahead records 2 to 8 each name a missing validated impl epoch, the failed
specialization graph or the discriminants that depend on them. Records 2, 3
and 4 are batch 10's derived records. 5, 6 and 7 were also in d7, and 8 was
there with "specialization verification is unfinished". They should be
rechecked once record 1 is repaired. None was investigated further.

### Record 1: a `[const]` parent bound and a marker's supertrait (genuine)

d9 named the relation. The parent is

```rust
const impl<I: [const] Iterator<Item = T>, T, const N: usize> SpecNextChunk<T, N> for I
```

(`array/mod.rs:1044`). The child at line 1062 adds `+ TrustedLen`, which is
`pub const unsafe trait TrustedLen: [const] Iterator` with
`#[rustc_unsafe_specialization_marker]`. The child's `I: TrustedLen` bound is
ordinary, so its supertrait elaborates to the ordinary `I: Iterator`. The
parent's basis holds only `I: [const] Iterator`. `minimal_admission.rs`
therefore finds `I: Iterator` absent from the permitted clauses, and `Iterator`
is not a marker.

rustc 1.98's `check_predicates` (`impl_wf_check/min_specialization.rs`)
compares elaborated `predicates_of`. Trait bound lowering
(`hir_ty_lowering/mod.rs`) puts a `[const] Tr` bound in `predicates_of` as
the ordinary `Tr` clause; its constness goes only to `const_conditions`, which
`check_predicates` does not read. The parent's predicates thus contain
`I: Iterator`, and the child adds only the marker.

Exploration drafts in `repro/maybe-const/` (not declared controls):

- `maybe-const-core.rs`, the core pair over a `const trait Iter` with a
  `[const] Iter` marker supertrait, and `maybe-const-plain-super.rs`, the same
  with a plain `Iter` supertrait: rustc 1.98.0 accepts both, and the first
  census 12 pin rejects both with record 1's message.
- `child-only-const.rs`: an ordinary `I: Iter` parent bound and a child
  `I: [const] Iter + Trusted`. rustc accepts it; the pin rejects it.
- `ordinary-control.rs`: the same pair without `const`; both accept.

Proposed repair (owner guess: Lane A, `sema/specialization/predicate_basis.rs`):
a `[const] Tr` clause enters each minimal-admission basis as the ordinary
`Tr` clause, as rustc's `predicates_of` holds it. That covers all three
rejected drafts. Whether a `const Tr` bound, which rustc also gives a
`HostEffect` clause, needs its own rule was not checked. Not repaired here; it
needs declared controls, and its fix needs another census.

## Checks

Every fastcheck below ran on the tree committed as `13fcdf82d`, docs aside.
The first census 12 pin's fastchecks ran on `7f951ab78` (receipts
`1794129777947810728`, `1794129922697300290`, `1794130051246915413` and
`1794130149832788610`, all passed with the same PA counts).

| Check | Result | Receipt |
|---|---|---|
| Private: 39 lib cases, strict private Clippy, format, architecture | pass | `obj/qualification/fast/1794131205166597775` |
| `census_mode` (10) and PA1 to PA12: 1,885 of 1,885 | pass | `obj/qualification/fast/1794131372962137641` |
| PA13 to PA19 and PA21 to PA24: 5,434 of 5,434 | pass | `obj/qualification/fast/1794131577836574089` |
| PA20: 1,451 of 1,451 | pass | `obj/qualification/fast/1794131666215927988` |
| `make test-upstream` | pass, 128 fixtures, 126 Reference files | `obj/parallel/census-batch11/tmp/test-upstream-3.log` |
| rustc 1.98.0 oracle | 38/39 agree; the wrong prediction is recorded | `obj/parallel/census-batch11/oracle/` |
| Pin build and ordinary observation | passed observation, `a108ae93` | `obj/parallel/census-batch11/qualified-3/receipt-build.json` |
| Census 12 | completed: 42,310 checked, 0 failed, 0 blocked, 8 records | `obj/parallel/census-batch11/qualified-3/receipt-census.json` |

The 39 private lib cases are the ten new tests of sections 1 to 4 (four
deep-reject, two relisting, two specialization and two const parameter tests),
the two trait-region formation invariants beside `trait_regions.rs`,
the user callable output control, the nine reservation and coherence invariants
in `future_impls.rs`, the `FnPtr` coherence invariant, the three
`source-nonnull` storage relation tests, the specialization header-lifetime,
selection, marker, sealed dispatch, applicable-trait and verifier
invariants, the impl constraint invariant, and the five const parameter
derive invariants. Earlier focused runs during development do not count
toward the pin. One of them failed the architecture audit, because its new
file was a seventh `feature_*` stem; the file is
`unstable_bound_parameters.rs`. One qualification run failed formatting
before the last commit and was rerun. `make test-harness` was not needed,
because no integration root was added.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness and course text.

## Storage

- Kept: the census 12 pin `obj/parallel/census-batch11/qualified-3/pin/`
  (`a108ae93`, with `.keep-artifacts`).
- Retired by removing `.keep-artifacts` and pruning their directories: the
  first census 12 pin (`qualified-2`, `2ad5137b`), the pre-rebase pin
  (`qualified-1`, `91b8ab1c`), census 11's pin
  (`census-batch10/qualified-1`, `fcd444bc`) and d9's diagnostic pin
  (`5d460621`). Their hashes are in
  `obj/parallel/census-batch11/retired-pins.sha256`. d1 to d8 binaries were
  pruned earlier. Each census's records, manifests and receipts remain.
- `make tidy` ran after the last census; obj stayed under its 16 GiB cap.

## Shared files touched

- Coherence: `sema/traits/implementations/overlap.rs`, `overlap/inputs.rs`
  and the new `overlap/deep_reject.rs` (stem `deep`, 1 file; section 1).
- Selection: `sema/traits/selection/search.rs`, `search/continuation.rs` and
  `search/forwarding.rs` (section 2).
- Specialization: `sema/specialization/implication_steps.rs`,
  `overlap_attempts.rs` and `minimal_admission.rs` (section 3).
- Types: `types/parameters/constraints.rs` (section 3).
- Declarations: `sema/declarations/generics/parameter_kinds.rs` (section 4,
  two commits).
- New invariants: `rejected_overlap_pairs.rs` (stem `rejected`, 1 file),
  `inferred_parent_obligations.rs` (stem `inferred`, 6 files) and
  `unstable_bound_parameters.rs` (stem `unstable`, 4 files), and the five PA31
  control directories.

## Decisions needed

- Record 1: the proposed repair in section 5, a `[const]` bound read as its
  ordinary trait clause in minimal admission, is next for Lane A unless it is
  assigned elsewhere.
- Section 1: the committed deep reject answers 312 fewer pairs than d1's
  temporary instrument counted. It was not investigated; no rejected pair
  unifies in any control.

## Commits

On main `016b1f54d`, in order: `32e0d8259` (section 2), `76ebf2e58`
(section 1), `99af814a3` (section 3), `7f951ab78` and `13fcdf82d` (section
4), then this document and the tracker entry.
