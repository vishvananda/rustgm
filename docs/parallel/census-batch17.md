# Census batch 17: certified normal premises and the twentieth census

Census 19 ([batch 16](census-batch16.md), pin `480f4169`) checked 42,310
declarations of original core, with none failed or blocked. Its crate record,
"trait implementation does not satisfy the trait's requirements", is core's
`impl<'a, P> FusedIterator for RSplit<'a, P> where P: Pattern<Searcher<'a>: ReverseSearcher<'a>>`
(`str/iter.rs:569`). Record 2 is a lookahead record that names the same impl.
This batch, Lane A, was declared on main `952781b14` and then rebased three
times: to `e0bc74592`, to `dcb956cf8`, where it was qualified, and to
`28e2b19fb`, where census 20 ran. No file of its diff changed on main in
between, and the diff (sha `0d5ae034`) stayed the same. This batch:

- locates the record (section 1);
- declares the broad effect and 19 controls before the rustc 1.98.0 oracle
  (section 2);
- relates a certified premise's rigid projection inputs invariantly, the
  approved option B (section 3);
- certifies an owner's own normalized bounds and carries each certificate
  only through lifetime-only constructions (section 4), with bounded-work
  invariants (section 5);
- qualifies against the declared set (section 6) and runs census 20
  (section 7).

Census 20 (pin `6a699887`, built from `28e2b19fb` and this batch) checks
42,310 declarations, with none failed or blocked. `str/iter.rs:569` passes.
The crate record moves to the next impl that the same macro expansion
generates: `DoubleEndedIterator for RSplit` (`str/iter.rs:595`), which
needs an elaborated supertrait premise certified. That is batch 18's lead.
Record 6 changes text. It is a lookahead over a partial validated impl set,
not a core blocker, and not a leak from this batch (sections 3 and 7).
Records 3, 4, 5, 7 and 8 are unchanged byte for byte.

Out of scope:

- Lane B owns `sema::traits::selection` (`sizing_fallback`),
  `overlap/inputs.rs`, `field_storage`, `finalization`, `types/variance`,
  `aggregate/instances`, `lifetimes.rs` and the closure entry in
  `sema/bodies/closures.rs`. C3b owns `traits/implementations*`, `overlap*`,
  `selection/search/coherence.rs` and `resolve/body_origin/declarations.rs`,
  and Lane C owns transport. This batch touches none of them. In particular
  `Prepared::begin` and `typed_headers/assumed_originals.rs` are unchanged.
- The static `Sync` check in `validation.rs` and `constrained_scalars.rs`
  did not change. LIB-061 stays deferred.
- The CPPGM backend and its configuration did not change.
- The general body relation of rigid projections (option A) is an open item.

## 1. The record

`FusedIterator: Iterator`. The impl's supertrait condition
`RSplit<'a, P>: Iterator` selects `RSplit`'s `Iterator` impl
(`str/iter.rs:543`), which has the same where-clause. Selecting it requires
`<P as Pattern>::Searcher<'r>: ReverseSearcher<'r>`, where `'r` is the
`Iterator` impl's lifetime and must come out equal to `'a`. Nothing
normalizes `<P as Pattern>::Searcher<'r>`: `P` is a parameter and no premise
gives an equation. So the goal's subject is a rigid projection.

The only premise that can prove it is the `FusedIterator` impl's own
`<P as Pattern>::Searcher<'a>: ReverseSearcher<'a>`. The exact premise
compare (`equate_trait_predicate`) defers every pair of projection terms: a
blocked pair whose two sides differ is `Error::Normalization`, because a
projection that might still normalize cannot be related by its inputs. Here
the goal's `Searcher<'r>` and the premise's `Searcher<'a>` differ until `'r`
is equated, so the premise never matches and the requirement fails with
census 19's text.

rustc 1.98.0 relates a rigid alias structurally: its arguments are related
invariantly, so `'r == 'a` is a region constraint, and it holds.

## 2. Declaration and controls

`obj/parallel/census-batch17/broad-effect.json` (sha `cbecf91a`) was
declared at `952781b14` with a clean tree, before any candidate build or
run. It claims that ordinary-mode outputs are unchanged across:

| Set | Size | Expected |
|---|---|---|
| Every non-ignored private lib test at base (`lib-baseline-cases.txt`, sha `9a39f3c1`) | 2,926 | all pass |
| `suite_traits` roots | 54 (53 after amendment 1) | all pass |
| `suite_projections`, `suite_regions`, `suite_inference` roots | 28, 23, 29 | all pass |
| PA19 to PA24 course text | every existing reference | unchanged |
| Batch 13 to 16 controls (six lib tests and their fixtures) | all | each status equals its declared outcome |
| The asm-free, projection-free scaling controls | 3 | equal work counts at base and candidate (A/A) |

Its only allowed status changes are 8 control flips to accept. Any other flip
stops the batch for a report with a rustc verdict, and nothing is blessed.
The baseline ran every set at `952781b14` (`baseline/qualify.log`), all
passing.

The 19 controls are no_core-free ordinary programs in
`pa21/tests/spec/assoc-bounds/certified-premises/`. PA21 is the earliest
assignment that owns generic associated types with associated-type bounds.
Their outcomes were declared in three files before rustc 1.98.0 or any pin
ran on them (`controls/outcomes.json` sha `04a536eb`, `outcomes2.json`
`75ebdbc3`, `outcomes3.json` `c3dc6dd2`). All 19 rustc statuses agree with
the declaration.

| Control | Shape | rustc 1.98.0 | Census 19 pin | This batch |
|---|---|---|---|---|
| `bound-positive` | core's shape: the bound form on both impls | accept | requirements text | accept |
| `where-positive` | the same bound as a where-clause on `P::Searcher<'a>` | accept | requirements text | accept |
| `mixed-positive` | bound form on `Walk`, where form on `FusedWalk` | accept | requirements text | accept |
| `hrtb-positive` | `for<'x> P::Searcher<'x>` on both impls | accept | requirements text | accept |
| `hrtb-premise` | a quantified premise instantiated at `Walk`'s `'a` | accept | requirements text | accept |
| `outlives-both-ways` | `'a: 'b, 'b: 'a` and a `'b` premise | accept | requirements text | accept |
| `lifetime-trait-positive` | a non-GAT member of `Pattern<'a>` | accept | requirements text | accept |
| `body-impl-positive` | a body goal selects `Walk` under the caller's premise | accept | unambiguous-assumption text | accept |
| `plain-positive` | a bound without lifetimes | accept | accept | accept |
| `static-positive` | the GAT bound at `'static` | accept | accept | accept |
| `normalizable-region` | a normalizable projection differing only in region | accept | accept | accept |
| `call-positive` | a body call with the same bound | accept | accept | accept |
| `substituted-normalizable` | rigid in its own environment, `u32` after `T := Known` | accept | accept | accept |
| `bound-missing` | no premise for `Walk`'s bound | E0277 | requirements text | requirements text |
| `hrtb-required` | `Walk` needs every `'x`, the premise has only `'a` | E0277 | requirements text | region text |
| `other-lifetime` | the premise names another lifetime `'b` | E0803 | requirements text | region text |
| `outlives-one-way` | only `'b: 'a` | E0803 | requirements text | region text |
| `gate-different-item` | the premise is `P::Other<'a>` | E0277 | requirements text | requirements text |
| `raw-environment-premise` | a raw-environment quantified premise | accept | unambiguous-implementation text | unambiguous-implementation text |

The requirements text is "trait implementation does not satisfy the trait's
requirements", and the region text is "trait argument lifetime requirement is
not satisfied: required lifetime does not outlive its target", which is the
lifetime category of rustc's E0803.

The first 8 rows are the declared flips. `raw-environment-premise` was
declared stricter for now: its premise comes from the raw environment, which
is never certified (section 4).

**Amendment 2** (amendment 1, in `broad-effect-amendment-1.txt`, drops the
ignored `negative_coherence` root):

- `other-lifetime` and `outlives-one-way` were declared in `broad-effect.json`
  as "rustc-matching region text" from the start. The private test's `CASES`
  table carried census 19's requirements text for every reject as a
  placeholder until the candidate's first complete run. It now carries the
  declared region text. No declared outcome changed.
- `hrtb-required` was declared "rejected", with no text. Declared: rejected.
  Observed: the region text. rustc 1.98.0: E0277. The statuses match; the
  texts differ. RustGM relates `P::Searcher<'x>` with the premise's
  `P::Searcher<'a>` and then fails the region proof `'x == 'a` for a
  universal `'x`, where rustc reports the missing higher-ranked bound.

**Amendment 3** (`broad-effect-amendment-3.txt`, sha `856cb2bc`, written
before the qualification on `dcb956cf8`) carries the declared lib set to
the rebased base, with no expectation changed:

- the two baseline cases that `f32b8d168` renamed run under their new names;
- main's 11 non-ignored lib tests added since `952781b14` and this batch's
  13 are included.

The result is `rebased/lib-run.txt` (sha `e3c0cd4b`): every non-ignored lib
test of the candidate on `dcb956cf8`, 2,950 in all. The four suite root
lists match the current registrations.

`dev/tests/invariants/certified_premises.rs` checks every control's first
error against `CASES`. The manifest declares each exit status, rustc
status and, for the 13 accepted, a native pass. The references were
generated only after the candidate was complete. All 19 exit statuses equal
the manifest and the declarations, and every rejection's first error equals
its `CASES` text.

- They were first generated on `dcb956cf8` with the analysis pin
  `bbeedccb`. The 13 accepted controls passed natively there
  (`check_native.py pa21`: 13 of 13 executable comparisons, evidence
  `obj/native/pa21/selected/1794200655404611326`).
- They were regenerated on `28e2b19fb` with census 20's pin `6a699887` as
  reference batch
  `pa21/reference-batches/1794204005456968510-9cdd6f673c51.json`. All 57
  sidecars are byte-identical to the `dcb956cf8` batch, which is kept in
  `obj/parallel/census-batch17/refs-dcb956cf8/` and was removed from the
  tree. The two batch receipts differ only in their compiler source hashes,
  for the 59 source files main changed.
- The integrator's join reruns the native run on the joined tree.

## 3. The relation (option B)

`Context::equate_normal_premise` (`types/infer/predicates/equations.rs`)
replaces `equate_trait_predicate` only for a premise its environment
certified normal. It shares the exact compare's unification and differs only
in the blocked pairs, which `relate_normal_premise`
(`types/infer/predicates/normal_premises.rs`) handles in their original
order:

- A pair whose resolved types are equal needs nothing.
- Every other pair is related only when all of these hold:
  1. its goal side is a recorded rigid normal form;
  2. its premise side's canonical type is one of the premise's outermost
     projection subterms;
  3. both are `Projection` applications of one associated item, with the same
     `trait_arguments` count;
  4. their argument slot kinds and input edge counts are equal.
- A related pair relates its complete input list invariantly: lifetimes
  become region equalities for the caller's region proof
  (`equate_arguments(.., true)`), constants are equated, and each type input
  is unified again in blocked mode, its own blocked pairs going back on the
  list. `trait_arguments` is only the layout gate.
- A pair that fails a gate, and a related pair whose relation fails,
  returns `Error::Normalization`: the exact compare's result. `Budget` passes
  through, so an exhausted relation never accepts.

**Rigid marks.** The goal side is marked only by a complete input rewrite:
when the selection search has finished its fold of a goal's inputs, every
projection left in a replacement was proved opaque in that frame, and
`retain_rigid_results` records each outermost projection root of the
replacement in the context's `rigid` map. A partial rewrite marks nothing.
Marks key the context's one term per canonical type (`Context::known`), so
expansion cannot hide a mark. A trial import carries a root's mark only when
the source term is the source's own term for that type, and a known type's
marks only when every one of its outermost projection subterms is marked.
Only gate 1 and the import read the map.

**Why a separate map.** The first candidate stored these marks in the
context's existing `opaque` map, which `apply_normalization` fills with
projections it proved opaque. That map's readers (`head`, `structural`,
`needs_normalization`, `freshen` and `apply_normalization` itself) treat an
entry as settled: a marked projection is never normalized again. A fold
output is rigid only in the frame that folded it, and only gate 1 needs to
know that. So rigid marks live in their own `rigid` map, which gate 1 and the
import carry read and which no other operation sees.

**A wrong explanation, disproved.** The preview census on the first
candidate changed record 6 (`mem/alignment.rs:440`) from census 19's text to
"incompatible types". A clean `e0bc74592` pin reproduced census 19, so the
change came from this tree. Lane A attributed it to the marks leaking through
the opaque map. That explanation was wrong. With the separate map (pin
`eb5c4b74`), every record was byte-identical to the first candidate's,
record 6 included. Experiment E0 (`diag/e0-declared.txt`, sha `45aeb5c8`,
declared before its build) then disproved any leak. Its tree was clean
`e0bc74592` plus a diagnostic-only `continue` (`diag/e0-skip569.patch`) that
skips exactly census 19's record 1 header (`str/iter.rs:17456`, expanded from
24562). The skip fired once. The pin (`analysis/e0-skip569`, sha
`6af24d24`, receipt sha `b1e187cd`, census sha `d3ce57cd`) reproduced the
candidate's record 1 and record 6 byte for byte. Record 2 was at 17456,
not 18420. The declaration had predicted that departure in its known
difference: the skipped impl stays unvalidated, and
`validate_trait_impl_owners` reaches it. Record 6 follows record 1's move,
and section 7 analyzes it. The pin was removed after these hashes were
recorded.

**Bounded work.** The premise's subterm set is computed at most once per
equation, on the first pair whose sides differ, by `projection_subterms` in
a fresh graph, and charged to the context's ordinary work. A related set
keeps each pair related at most once. A projection-free type costs nothing.

**Tradeoffs recorded with the approval:**

- B is a gate that option A will later replace.
- Body equations stay stricter than rustc (open items).
- B is never more permissive than rustc: every accepted pair is one rustc
  relates structurally, and its lifetimes still need the region proof.
- Doing A now would change body texts in the middle of the batch.

## 4. Certification

A premise is certified only when its own environment proved it normal:

- it has no projection, which needs no certificate and is never recorded; or
- it is one of its owner's declared bounds, and its owner's bound
  normalization completed (`complete_bound_environment`). Every context and
  root context must have no written header equations, no deferred Self-naming
  impl header roots and no alias template.

`retain_normal_bounds` keeps those bounds until `complete_source_environment`
publishes the owner's environment, and `certify_environment` then certifies
them only in an environment Arc that nothing else holds yet. Elaborated
supertrait premises are appended at publication, so none is ever certified.

**Identity.** Certificates are keyed by the `Clause` value (the interned
`Predicate` or `QuantifiedId`). That value is the premise's complete
identity: two clauses with one value are the same premise to selection,
which never distinguishes them, and a substitution that changes any input
produces another value. A key by position would not survive the
deduplication that `insert` and `rebuild_observed` perform.

**Storage.** `TraitAssumptions::normal` and `ClauseIndex::normal` are each an
`Option<Box<HashSet<Clause>>>`, `None` unless something is certified, so
the common case allocates nothing. Only a base `ClauseIndex` layer built from
caller assumptions has one; an opened view reads its base's set, and a
private extension layer has none.

### Sites

| Site | Effect |
|---|---|
| `Generics::certify_environment`, from `complete_source_environment` | certifies the owner's own normalized declared bounds in its unshared published environment |
| `TraitAssumptions::insert` (every `with_clauses`, `with_clauses_charged` and `with_predicates`) | drops every certificate |
| `TraitAssumptions::substitute_observed`: `substitute_regions`, `substitute_declared_regions_charged`, a materialized opened view | a lifetime-only folder carries each certificate to its premise's mapped value; any type substitution drops them all |
| `Prepared::for_declaration` (a body's defining clauses) | carries each certified source's image at the body's universal aliases, through `Environment::opened_certificates`; nothing when any source is not one of the declaration's own trait premises or a const trait default's Self premise is added |
| `Checker::trait_assumptions` (`sema/bodies/trait_goals.rs`) | `Assumptions::copied` gives each copied or aliased premise its own source's certificate and no other |
| `Environment::new`, imported and staged header environments, raw bound environments | never certified |
| `dispatch.rs:209` | conservative-uncertified: method dispatch builds its own `Assumptions` from substituted clauses; a carry would need each clause's source and a lifetime-only substitution |
| `declarations/method_prefix.rs:59` | conservative-uncertified: a receiver prefix's premises; a carry would need them to be lifetime-only copies of a certified environment's clauses |
| `declarations/generics/validation.rs:216` | conservative-uncertified: validation assumptions inserted one by one; a carry would need the owner's certificates, which are only published after validation |

The two body sites are lifetime-only substitutions: `alias_clause` and the
defining clauses use `Bound::new(universal_regions)`, which replaces bound
regions only. Both were reported to the coordinator before qualification.

### Cache proof

A certified environment and an uncertified one never share a cached result:

- selection reuse compares assumption snapshots by `Arc::ptr_eq`
  (`Assumptions::same_snapshot`) and environments by `Arc::ptr_eq`
  (`refutation.rs`);
- raw and completed environments are distinct objects with separate
  `OnceLock` indices, and certification resets the completed index;
- the `Availability`-keyed cache is used only when there is no caller
  environment, so no certificate can reach it.

**Named open risk:** region substitution can't make a rigid projection
normalizable. Every carrying site relies on it. A lifetime-only substitution
replaces lifetime inputs only, and no impl or equation in RustGM selects on a
lifetime's identity, but this is an argument, not a proof.

**The census case.** In core's `FusedIterator for RSplit` impl the premise
`<P as Pattern>::Searcher<'a>: ReverseSearcher<'a>` is a declared bound that
completed its own normalization, so it is certified. The elaborated
`<P as Pattern>::Searcher<'a>: Searcher<'a>` is not certified, and the
supertrait condition does not need it.

**Tradeoff:** this is more machinery than one bit for each environment. The
bit could not tell the census premise from an elaborated or raw one in the
same environment.

## 5. Invariants and bounded work

`dev/tests/invariants/normal_premises.rs` (types) and
`certified_premises.rs` (sema) hold 13 new tests:

| Test | Checks |
|---|---|
| `certified_premise_relates_rigid_goal_inputs_as_region_equalities` | the relation's region equality |
| `expanded_goal_keeps_its_mark_through_a_trial_import` | marks survive a trial import; an unmarked twin still defers |
| `unmarked_goal_keeps_the_exact_normalization_result` | gate 1 |
| `projection_outside_the_premise_keeps_the_exact_normalization_result` | gate 2 |
| `different_item_keeps_the_exact_normalization_result` | gate 3 |
| `different_layout_keeps_the_exact_normalization_result` | gate 4 |
| `equal_pairs_allocate_no_premise_subterms_or_related_set` | no allocation for equal pairs; a projection-free result costs no work |
| `many_subterms_and_blocked_pairs_cost_linear_work` | linear work |
| `exhausted_work_fails_closed_at_every_cut` | `Budget` at every cut, never acceptance |
| `opened_clauses_carry_only_held_certified_sources` | the defining-clause site |
| `certified_bound_premises_match_declared_outcomes` | all 19 controls |
| `certified_premise_requests_do_not_grow_with_member_lifetimes` | request counts |
| `copied_premises_keep_only_their_own_sources_certificates` | the body assumption site; certifying an unheld premise is an error |

Each gate negative compares against the exact compare's result, declared
before the candidate ran (`outcomes3.json`'s `private` rows).

**Request count.** N GAT lifetime arguments by M premises
(`scaled(width, premises)`):

| M premises | Matches | Retries | Same for N = 4 and 16 |
|---|---|---|---|
| 1 | 8 | 1 | yes |
| 4 | 20 | 4 | yes |
| 16 | 68 | 16 | yes |

Requests do not grow with N. Matches grow by 4 for each added premise, and
each supertrait condition is retried exactly once, after its one complete
rewrite.

**Linear relation work.** A tuple of W rigid pairs against a premise with W
subterms: 248, 968, 3,848 and 15,368 steps for W = 16, 64, 256 and 1,024.

**A/A.** The three projection-free scaling controls on the integration
base. The clean `dcb956cf8` (`scaling/base-dcb956cf8.txt`, fastcheck
`1794199869033226175`) is compared with the candidate on it
(`scaling/candidate-3.txt`, fastcheck `1794200101521999011`). The base's
counts equal `952781b14`'s (`scaling/base-952781b14.txt`).

| Control | Base | Candidate |
|---|---|---|
| pattern coercion steps | [3031, 10543, 2226, 6978] | [3031, 10543, 2226, 6978] |
| self-default scaling | [[120, 120, 16, 1774, 0], [480, 480, 64, 7078, 0]] | the same |
| receiver prefix | steps 84, bytes 3,224, `ClauseIndex` 128 | steps 84, bytes 3,280, `ClauseIndex` 136 |

The candidate's request counts and linear relation steps above are from the
same run.

Work counts are unchanged. The first candidate (`candidate-1.txt`) charged
one step in `projection_subterms` before its projection-free check, so
self-default inference grew by one step a use (1,782 and 7,110). That step
now follows the check, and the counts match the base.

**Size change.** `ClauseIndex` grows from 128 to 136 bytes and
`TraitAssumptions` by 8, one `Option<Box<HashSet<Clause>>>` each.
`Assumptions` stays 8 and `Environment` 64. The receiver prefix's retained
bytes grow by 56, 8 for each of its seven charged index allocations.

## 6. Qualification

The candidate was qualified on `dcb956cf8` (`candidate-1/`,
`tools/qualify-rebased.sh`), one fastcheck at a time, against the declared
set as carried by amendment 3. Every receipt passed:

| Set | Fastcheck | Tests |
|---|---|---|
| lib chunk 00 | `1794201402075086672` | 1,012 |
| lib chunk 01 | `1794201534644177773` | 946 |
| lib chunk 02 | `1794201610535336929` | 992 |
| `suite_traits` roots, part 00 | `1794201686460436178` | 129 |
| `suite_traits` roots, part 01b | `1794201884166826791` | 227 |
| `suite_projections` roots | `1794202035790069932` | 131 |
| `suite_regions` roots | `1794202144764600149` | 142 |
| `suite_inference` roots | `1794202205212574414` | 149 |
| PA19 to PA24 course text and `census_mode` | `1794202284989461491` | every stage status 0 |

No status flipped beyond the 8 declared control flips, and every batch 13
to 16 control is among the passing lib tests and course text. No coherence
or overlap failure occurred, so none needed comparing with `aa6c36f65`.
The 13 new tests are in the lib set.

On `28e2b19fb` the batch ran one focused fastcheck (22 exact tests, the 13
new ones and their neighbors, `1794203371089448037`), which the pin build
requires. As the integrator directed, the A/A and the qualification were
not repeated there. None of the 106 files that changed in
`dcb956cf8..28e2b19fb` is in this batch's diff. Main's changes are on the
export and transport paths, not on the declaration checking the census
exercises. The integrator's join reruns amendment 3's sets, the four suites'
roots, PA19 to PA24 and `census_mode` on the exact joined tree.

## 7. Census 20

Census 20 uses the pin `obj/parallel/census-batch17/qualified-1/pin/`
(`6a699887`), built by `run.py build` from `28e2b19fb` plus this batch and
bound to fastcheck `1794203371089448037`. Its manifest (`531f75d3`) chains
from census 19's. The census (`qualified-1/census/census.tsv`, sha
`6b88904a`) checks 42,310 declarations, with none failed or blocked. It is
byte-identical to the analysis previews on `e0bc74592` and `dcb956cf8`.
Against census 19:

| Record | Pass | Census 19 | Census 20 |
|---|---|---|---|
| 1 (crate) | `validate_trait_implementations` | `str/iter.rs:569`, requirements text | `str/iter.rs:595`, requirements text |
| 2 | `validate_trait_impl_owners` | `:569` | `:595`, the same text |
| 3, 4 | `local_annotations`, `complete_alias_normalizations` | early scalar profile | unchanged |
| 5 | `validate_objects` | `any.rs`, impl epoch | unchanged |
| 6 | `complete_discriminants` | `mem/alignment.rs:440`, "associated type requires an unambiguous validated implementation" | the same site, "incompatible types" |
| 7 | `complete_layouts` | `ascii.rs`, completed discriminants | unchanged |
| 8 | `require_specialization_execution` | unfinished | unchanged |

Records 1 and 2 gain an inner expansion frame at `:574` (byte 17604). That
is the recursive `generate_pattern_iterators!` call into the `double ended`
arm, inside the same outer call at `:786` (byte 24562).

### Record 1: an elaborated supertrait premise (batch 18's lead)

Census 20's crate record is core's
`impl<'a, P> DoubleEndedIterator for $reverse_iterator<'a, P> where P: Pattern<Searcher<'a>: DoubleEndedSearcher<'a>>`
(`str/iter.rs:595`, `RSplit` in the first `generate_pattern_iterators!`
expansion), with "trait implementation does not satisfy the trait's
requirements". `DoubleEndedIterator: Iterator`, so the supertrait condition
`RSplit<'a, P>: Iterator` selects the `Iterator` impl at `:543`. That impl
requires `<P as Pattern>::Searcher<'r>: ReverseSearcher<'r>`, the census 19
goal again.

This impl's declared bound is `Searcher<'a>: DoubleEndedSearcher<'a>`, which
is certified. The premise that proves the goal is its elaborated supertrait
`Searcher<'a>: ReverseSearcher<'a>`, which `DoubleEndedSearcher<'a>:
ReverseSearcher<'a>` implies. Section 4 never certifies an elaborated
premise, so the pair keeps the exact compare's `Normalization` result.
rustc 1.98.0 accepts the impl. Declared probes (`repro/r6/declared.txt`,
round 4):

- `r1-elaborated`, the impl alone: rustc accepts; RustGM gives the
  requirements text.
- `r1-explicit`, the same with `+ ReverseSearcher<'a>` written in the
  bound: rustc accepts; RustGM accepts, since the written bound is certified.

**What batch 18 needs.** An elaborated supertrait premise inherits its
source's certificate when the source is certified. The supertrait's own
arguments are then substituted from the source's arguments, and they must
add no projection that the source's normalization did not cover. Here the
subject is the source's own rigid projection, and the supertrait argument is
the lifetime `'a`, so a regions-only carry suffices. A type-argument
supertrait would need the same check that section 4 applies to declared
bounds. Batch 18 extends the site table and the named open risk to
elaboration, with its own declaration and controls.

### Record 6: a lookahead over a partial Shl inventory

Record 6 is a lookahead (`blocked_by` 1). It is `complete_discriminants`
at `mem/alignment.rs:440`, the discriminant `_Align1Shl0 = 1 << 0` of the
64-bit `#[repr(usize)] enum AlignmentEnum`, with the text "incompatible
types". Census 19 reported "associated type requires an unambiguous
validated implementation" at the same site. E0 (section 3) showed that the
change follows record 1's move and is not a leak from this batch.

- **Phase and comparison.** Analysis pin `r6-diag` (`diag/r6-diag.patch`,
  on `e0bc74592`, sha `c7a54b03`, census byte-identical to census 20's)
  printed the
  backtrace of the Mismatch display. Declarations, `complete_discriminants`
  → `constants::discriminants::advance` → `Queries::enter_variant` → the
  anonymous constant's body → `coerce` → `coerce_equality` →
  `callable_shape` → `shape_equation`. The coercion equates the type of
  `1 << 0` with the repr type `usize`, and the equation fails.
- **Why.** The `<<` operator selects `{integer}: Shl<{integer}>` among the
  validated impls. A lookahead runs on whatever validation finished before
  record 1 stopped `validate_trait_implementations`, in conformance order:
  - census 19 stopped at position 4,108 (`RSplit`'s `FusedIterator`),
    before any `Shl` impl, so there was no source;
  - census 20 stops at position 5,714 (`str/iter.rs:595`);
  - position 5,596 validated exactly one `Shl` impl, the first expansion of
    `shl_impl! { $t, u8 }` (`ops/bit.rs:483`, from :502 inside
    `shl_impl_all!` at :518), which is `Shl<u8> for u8`. The other 143 come
    after the stop.

  Over that one-impl subset the selection is unambiguous, the operands
  become `u8`, and `u8` does not equate with `usize`. In ordinary mode,
  discriminants run only after every impl is validated. Neither text can
  happen there.
- **Repro** (`repro/r6/declared.txt`; rustc 1.98.0 declared before each
  round). Round 1's drafts made rustc ICE in their `self << other` impl
  bodies under `no_core`. They did not meet the declaration and were
  replaced with `loop {}` bodies. In round 2 rustc accepted all six drafts,
  and every RustGM result matched its declaration:

| Draft | Order | Census |
|---|---|---|
| `la-u8-first` | `Shl<u8> for u8`, STOP, the other 143, enum | STOP, then `complete_discriminants` "incompatible types" |
| `la-u8-min` | `u8`, STOP, `Shl<i32> for usize`, enum | the same |
| `la-usize-first` | `usize`, STOP, `u8`, enum | STOP, no discriminant record |
| `la-none-first` | STOP, `u8`, `usize`, enum | STOP, then census 19's "unambiguous validated implementation" |
| `ord-min`, `ord-full` | no STOP | ordinary mode accepts; no records |

  STOP is census 20's record 1 shape: `DoubleEndedIterator for RSplit`
  under `P: Pattern<Searcher<'a>: DoubleEndedSearcher<'a>>`. The smallest
  repro is `la-u8-min-sized.rs` (2,481 bytes). Its prefix keeps only
  `PointeeSized`, `MetaSized` and `Sized`.
- **Ownership.** No premise is involved, so batch 17's relation is not the
  cause, and batch 17 does not change it. It is a candidate for the next
  batch, alongside record 1's elaborated-premise lead. A lookahead that
  treats a selection over a partial inventory as unambiguous can report a
  failure ordinary checking never reaches. The record disappears once
  validation reaches the rest of the `Shl` family.

### Next batch: lookahead selection over a partial validated set

**Classification.** Record 6 is a lookahead over a partial validated set,
not a core blocker. Census 19's record 6 text came from the same mechanism
with zero validated `Shl` impls, so both censuses' discriminant records
were false leads.

**Proposed fix: option (i).** In census mode, a lookahead selection never
treats a partial validated set as complete. When an impl that is not yet
validated could apply to the goal, the selection reports a record
`blocked_by` the crate record and counts it. Option (ii), consulting the
full impl set, was not chosen:

- **Option (ii) needs more than headers.** An impl's header is registered
  before validation, but a projection through it needs that impl's
  completed associated-type definition. `implementation_result` fails with
  "missing completed associated type definition" without it, and
  `validate_impl_owner` computes those definitions through
  `trait_impl_types`. Defining-type resolution also reads the
  specialization graph, whose verification record 8 reports as unfinished.
  So (ii) would
  run member validation out of order, or trust members that never
  validated. The crate record's own impl is one of those.
- **Option (i) follows the census's own convention.** Records 5 and 7
  already report that a later pass requires a completed result. Record 5
  says so for this very kind of dependency ("requires the completed
  immutable impl epoch"), and records 3 and 4 report validation outside the
  early profile. Option (i) gives `complete_discriminants` the same honest
  form.
- **Option (i)'s cost, measured on the repro family.** Under (i),
  `la-u8-first`, `la-u8-min` and `la-none-first` each report one blocked
  discriminant record in place of a false text. `la-usize-first` also
  becomes blocked, because `Shl<u8> for u8` is unvalidated and its header
  could apply. Today it passes by luck: the one validated impl happens to be
  the right one. Its `complete_layouts` would then report the derived
  "requires completed discriminants" record, as the `u8` drafts do today.
  `ord-min` and `ord-full` do not change, because ordinary checking
  validates every impl first. So (i) loses at most the lucky passes and no
  real lead.
- **Controls.** The six-draft family is the fix's control set:
  `la-u8-first` and `la-u8-min` (a `u8` impl before STOP), `la-usize-first`,
  `la-none-first`, `ord-min` and `ord-full`, with the reduced
  `la-u8-min-sized`. All are rustc-accepted, with declarations in
  `repro/r6/declared.txt`.

## Checks

- Fastchecks: as in section 6. The first focused run on `28e2b19fb`
  (`1794202505811412237`) exhausted the five-minute budget in its cold
  private Clippy build. After the warm step outside the budget, the rerun
  passed.
- `make test-upstream`: 128 pinned upstream fixtures and 126 Rust 1.98.0
  Reference files verified.
- PA21 references for the 19 controls: 19 of 19, regenerated with pin
  `6a699887`, byte-identical to the `dcb956cf8` batch (section 2).
- Native: 13 of 13 on `dcb956cf8`; the integrator's join reruns it.
- Census 20: `run.py build` (ordinary observation: first error
  `str/iter.rs:595`), then `run.py census`, both bound to fastcheck
  `1794203371089448037`.
- Pending in scheduled or manual CI: full release regular and native tests,
  uncached audits, all-target strict Clippy and the harness.

## Open items

- **Option A: the general body relation of rigid projections.** rustc relates
  any two rigid aliases structurally. Here only a certified premise's
  equation does, and only by the gates in section 3. In a body,
  `x: T::A<'a>` used as `T::A<'b>` still reports "mismatched types". Option
  A needs its own batch with its own broad-effect declaration, covering
  `equate`, `equate_deferred` and `equate_proved_predicate`.
- **A type-parameter GAT member.** `repro/drafts/gat-type-param.rs`, the
  census shape with `type Searcher<T>`, is accepted by rustc 1.98.0. The
  census 19 pin reports "invalid canonical trait selection" at both impls.
  It is outside the census and is recorded, not changed.
- **`hrtb-required`'s text.** RustGM reports the region text where rustc
  reports E0277; the statuses match (amendment 2).
- **Stricter than rustc:** `raw-environment-premise`. Its premise comes from
  the raw environment, which is never certified.
- **Conservative-uncertified sites:** `dispatch.rs:209`, `method_prefix.rs:59`
  and `generics/validation.rs:216` (section 4).
- **The named open risk** (section 4).
- **Census 20's record 1** (`str/iter.rs:595`): certifying elaborated
  supertrait premises (section 7). Batch 18's lead.
- **Lookahead selection over a partial validated set** (section 7): census
  record 6, with option (i) proposed for batch 18.

## Storage

- Kept: census 20's pin `obj/parallel/census-batch17/qualified-1/pin/`
  (`6a699887`, with `.keep-artifacts`), the new census baseline.
- Retired, by removing `.keep-artifacts` and pruning the directory: census
  19's pin (`census-batch16/qualified-1/pin`, `480f4169`), after census 20's
  pin was built.
- Analysis-only pins built and retired: `final-1` (`da3ba877`), `final-2`
  (`eb5c4b74`), `base-e0bc74592` (`0baea37a`), `e0-skip569` (`6af24d24`),
  `r6-diag` (`c7a54b03`) and `rebased-1` (`bbeedccb`). Each one's receipt,
  working-tree patch and census remain.
- All retired hashes are in `obj/parallel/census-batch17/retired-pins.sha256`.
- obj reached 18.2 GiB after the warm step on `28e2b19fb`. `make tidy-build`
  before the pin build brought it to 7.4 GiB, and it was 7.6 GiB at the end.
