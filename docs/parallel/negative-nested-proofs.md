# Negative premises in nested trait proofs

This is Lane B's item B of 2026-11-09, on origin/main `c2132742c`. It follows
the [placeholder sizedness item](nested-sizedness.md). That item fences a
projected-size nominal under placeholders, because the nested trait proof
that answers it does not carry the other header's premises. Item B carries
them into that proof. It lands in four checkpoints:

1. it moves the `MetaSized` premises onto the placeholder handle;
2. it propagates the handle through the nominal trait `State` and the
   selection `Caller`, with the fences kept;
3. it retires the fences under declared controls;
4. it records the evidence and this notice.

## 1. The MetaSized move (3330eac46)

`Placeholders` is one shared handle, `Arc<Sets>`, over the other header's
Sized and `MetaSized` sets. Each set is sorted and deduplicated.
`Placeholders::new(sized, meta_sized)` checks or sorts each set. Its work is
`|S|.max(1) + |M|.max(1)` for ordered sets, plus `n (ilog2 n + 1)` for each
set that must be sorted. `sized(ty)` and `meta_sized(ty)` are binary
searches at `len.max(1).ilog2() + 1`.

The overlap inputs build one handle per side, in `premises()`, and charge its
construction there. The negative query (`may_overlap_with`) and its payment
(`pay_negative`) clone the handle and construct nothing. `NegativePremises`
is removed.

Work accounting was declared before any code or run, and the measurements
agree with it:

| Scaled test, c = 8, 16, 32 | Base `c2132742c` | Step 1 | Declared change |
| --- | --- | --- | --- |
| `MetaSized` premise pair steps | 1946, 3775, 7492 | 1965, 3810, 7559 | 2c + 3 |
| `PointeeSized` control | 1681, 3236, 6391 | 1693, 3256, 6427 | c + 4 |

Each harness was run twice, and both runs agreed. No pinned work assertion
changed.

- **Observed, not declared:** `Shapes` and `sizes::Query` each shrink by one
  word. `Option<Arc<[TypeId]>>` is 16 bytes and `Option<Arc<Sets>>` is 8.
  - The only consumer of either size is the typed-headers byte reservation
    in `contract_completion.rs`, which reserves `size_of::<Proof>()` plus
    `size_of::<Shapes>()` against the retained-storage budget. That counts
    bytes, not work units, and nothing pins it.
- **Unchanged outputs:** the sixty negative-premise twins (status lines and
  every stderr) and the nested reach survey (79 files) are byte-identical to
  `c2132742c`.

## 2. Propagation with the fences kept

The set now travels with every proof that a negative coherence query starts
for its nested obligations.

- **Callers:**
  - `selection::Caller` has a `placeholders: Option<&Placeholders>` field,
    and `OwnedCaller` has the owned handle;
  - every construction site chooses: the State-derived sites pass their
    State's set, and the others write `None`;
  - a closed requirement (in `select_opened`, `prove_quantified` and a closed
    normal form) passes `None`, because it mentions no parameter and its
    reuse slot stays premise-independent.
- **Searches:**
  - `Search::assuming` is the one production constructor. With a set, it
    installs `NegativeSizing`, with fresh debts, and the builtin proof's
    placeholder walk, and it rejects an environment together with a set;
  - the tests' `Search::new` and `assuming` share one infallible body, and
    `new` passes no set;
  - `may_overlap_with` builds its Caller with the set, and no longer installs
    it after construction.
- **Rigid parameter sizing:** a search with a set decides a rigid
  parameter's Sized and `MetaSized` by the other header's premises, at any
  depth. Only the root, which retains Sized debts, records a `MetaSized`
  debt for `pay_negative`; a nested search has decided it.
- **Nominal trait State:** it holds the set of the placeholder proof that
  owns it. Its normalization `Session`, its clause-term and shape cursors,
  and the nested State of `normal_form::begin_reusing` (through
  `normal_form::Sizing`) carry it. Each of these rejects an environment
  together with a set.
- **Fences:** `PLACEHOLDER_SIZE_FENCE` and `sizes::Query::fence` stay, so
  the shape walk still refuses a projected-size nominal under placeholders.

Layout:

| Type | Before | After | Pin |
| --- | --- | --- | --- |
| `nominals::traits::State` | 800 | 808 | `early_conformance.rs`, `closed_callers.rs` |
| `selection::Caller` | 24 | 32 | `placeholder_sizing.rs` (new) |
| `selection::OwnedCaller` | 24 | 32 | `placeholder_sizing.rs` (new) |

- `State` grows by one word, its `Option<Placeholders>`, so both existing
  pins move from 800 to 808.
- `selection::Pending` and `normal_form::Query` hold an `OwnedCaller` and a
  `State` inline, so each grows by 8. The projections `Cursor` grows by one
  word as well.
- Two retained-byte charges that use `size_of::<selection::Pending>()` rise
  by 8 per charge: the source-retention charge in
  `normal_form/data_result.rs` and the projection-requirement charge in
  `specialization/obligation_chain.rs`.

The step's outcomes were declared before any code or run, and the evidence
agrees with them. The base is the step-1 CLI; the candidate CLI was built from
this step's tree before its two test-only edits, the `closed_callers.rs` pin
and the test-only State accessor.

| Gate | Result |
| --- | --- |
| Sixty negative-premise twins | status lines and every stderr byte-identical |
| Flip survey, 336 files | byte-identical |
| Census lookahead | `census.tsv` byte-identical, 23 lines with 8 records; `result.json` differs only in CLI, producer and timing identity |
| Golden table, premise-less pass | byte-identical |
| Golden table, placeholder pass (empty set, every type parameter) | byte-identical, so no placeholder row moved |
| Scaled `MetaSized` pair and `PointeeSized` control | unchanged: 1965, 3810, 7559 and 1693, 3256, 6427, in both runs |

- **Coverage:** every placeholder row that normalizes still fails at its first
  trait step, as at the base. In this step, only the new unit test runs a
  nested search under a set.
- **Checks:** focused fastchecks passed:
  - the private bodies, methods and dispatch chunks (459, 325 and 213 cases);
  - the trait integration roots (116 and 240);
  - crate linking's absence and modifier transport (5) and their native
    clients (2);
  - native negative coherence (1);
  - `pa20`.

## 3. Cache audit before the fences go

This audit was read on `7acc24208`, the step-2 commit rebased onto
`645df0a92`. Between `b920d5043` and `645df0a92`, T and S changed nothing
under `dev/src/types`, `sema/nominals`, `sema/generics`, `sema/traits`,
`sema/projections` or `sema/lifetimes.rs`.

A placeholder proof starts in `may_overlap_with`. Its searches, its builtin
proof and every State, Session and Cursor they start carry the set. The audit
covers every memo that outlives one of those owners, and marks each memo that
the path cannot reach as unreached.

### Facts-level memos

| Memo | Key | Set or environment in key | Placeholder path | Evidence |
| --- | --- | --- | --- | --- |
| `types` interning and interned properties | type structure | no | reached: every substitution and normalization interns | Properties are computed at intern time from structure and declarations. `is_sized` includes a parameter's declared requirement, never a set. |
| `generics.nominal_requirements` | nominal `TypeId` | no | reached: `Proof::check_inside` | `nominals.rs:387`, then `requirements.rs:427-447`. The value is the item's declared where clauses substituted by `instance`. No proof runs. |
| `lifetimes.nominal_requirements` | `TypeId` | no | reached | `nominals.rs:340,356,385` call `.prepare(types, ty)`. These are declared outlives bounds. |
| `aggregates` instances | `TypeId` | no | reached | `terms.rs:116`. `preparation.rs:19-24`: it "proves neither sizedness nor cycles". |
| `aggregates.metadata_ready` | `TypeId` | no | reached in physical mode: `sizing_fallback.rs:196` | `metadata.rs:93` returns `Unresolved` for a parameter, projection or opaque tail before the write loop at `:107`, so only concrete tails are recorded. |
| `size_rules.rules.projections` | item and arguments | no | not now: `Query::fence` precedes `apply` (`sizes/query.rs:227-229`); reached after step 3 | `types/sizedness.rs:63-73` stores the substituted rule template. It is premise-free. |
| `copies` | `TypeId` | no | closed types only: `terms.rs:121` in `closed_builtin` | It takes no environment or set, and a closed type has no rigid parameter. |
| `generics.implementations` candidate index | trait item and goal shape | no | reached: `search.rs:564` | It is built from declared headers. |
| Specialization pair attempts | pair of impl entries | no | reached only when two impls apply to one goal (`frames.rs:416`) | `candidate_order.rs:100` settles the pair from its own headers. No caller and no set is passed. |
| `generics.selection_memo` | trait item and closed input identities | no | unreached: its only users are body goals (`bodies/trait_goals/inference.rs:407-449`) | It will be pinned by a test. |
| `size_rules` rule preparation | item | no | unreached | `nominals::sizes::prepare` is called only from declaration phases: `crates.rs:163`, `declarations/aggregates/sizedness.rs:27` and `crates/imported_records.rs:54`. |
| `unreduced` | node or definition | no | unreached | It is written only in `declarations.rs` and `declarations/{unreduced,projections,aggregates}.rs`. |
| `alias_requirements` | owner | no | unreached | No proof module references it. |
| `layouts` | `TypeId` | no | unreached | Its users are compiler items called from bodies (`bodies/effect_facts.rs`). |
| `equality` | `TypeId` | no | unreached | No proof module references it. |
| `opaques`, `associated`, `destructors`, `source_bindings` | none | none | read only | They are lookups and substitution only. |
| `body_work.canonical_steps` and the source-header and retention ledgers | none | none | reached | These are work and byte counters, not answers. |

No facts-level memo that the path reaches has the set or the environment in
its key, or computes its value under the set. Each holds a declared or
structural fact, or an answer for a closed type.

### Memos owned with one set

| Memo | Owner | Why it holds one set |
| --- | --- | --- |
| `Query.completed`, `families`, `auto_provisional`, `active` | one `Search` | The caller is fixed at `Search::assuming`. |
| `selection::Reuse`, State slots | nominal trait `State` | `traits.rs:547`: the open slot serves the State's set and the closed slot serves `None`. |
| `selection::Reuse`, Cursor slot | projections `Cursor` | Its searches take the Cursor's set. Body sessions (`projections.rs:122`) have none. |
| `State.normalized`, `quantified`, `terms`, `shapes` | `State` | `sizes::Query::with_placeholders` receives the State's set. |
| `Proof.checked`, `copies`, `shapes`, `term_shapes` | `nominals::Proof` | `Proof::assume_placeholders` requires a fresh proof (`nominals.rs:143-148`). |
| `selection::Pending`, `normal_form::Query` | resumed query | `OwnedCaller` clones the set (`resumed_queries.rs`). |

- **Gap:** `State::assume_placeholders` and `Cursor::assume_placeholders`
  used to overwrite the set, whereas `Proof`'s rejects a used proof. Every
  caller installs the set right after construction.
- **Hardening (checkpoint 3a):** both now take their set once, before they
  cache any work. Afterwards any install fails with
  `PLACEHOLDER_REINSTALL`, whether it passes a set, an empty set or none:
  - A State has cached work once either reuse slot, `normalized`,
    `quantified`, `terms` or `shapes` holds anything.
  - A Cursor has cached work once it has started a query or holds reused
    proofs.
- **Second gap:** `State::prove_equality` normalizes through
  `projections::normalize`, which has no set. It needs a declaration scope,
  which a placeholder proof never has (`Proof::new` leaves `scope` empty), so
  it would fail before normalizing anything. Since 3a it fails first under a
  set, with `PLACEHOLDER_EQUALITY_FENCE`. No gate input reaches that text.

### Interned answers read under a set

These are reads, not memos, but they are the same class of leak: a
premise-less answer used under a set.

| Read | Reached | Status |
| --- | --- | --- |
| `sizes/query.rs:82`: `is_sized` of a projection that normalizes to itself | after step 3 | Step 3 must decide a rigid projection by the set and its declared bounds. |
| `sizes/query.rs:257`: `is_sized` of a parameter | premise-less `begin` only | `placeholder_begin` (`:189`) decides a parameter by the set. |
| `terms.rs:113`: `is_sized` | `closed_builtin`, closed types only | safe |
| `metadata.rs` ordinary shortcut | abi callers only | unreached |

### Checkpoint 3a evidence

The checkpoint lands the audit, its two tests and the hardening with both
fences kept. Its outcomes were declared before any 3a code or run, with four
amendments, each written before the run it governs.

- **Base:** `base3`, the dev-fast CLI built at `7acc24208`. `cand1` predates
  the T and S transport changes merged into `645df0a92`, so it is not the
  comparison.
- **Candidate:** `cand3`, built from the 3a tree.

| Gate | Result |
| --- | --- |
| Sixty negative-premise twins | status lines and every stderr byte-identical |
| Flip survey, 336 files | byte-identical |
| Census lookahead | `census.tsv` byte-identical, 23 lines with 8 records; `result.json` differs only in CLI, output path, process and timing identity |
| Golden table, both passes | byte-identical, 5674 lines |
| New texts in gate outputs | neither `PLACEHOLDER_EQUALITY_FENCE` nor `PLACEHOLDER_REINSTALL` appears |
| Scaled `MetaSized` pair and `PointeeSized` control | unchanged: 1965, 3810, 7559 and 1693, 3256, 6427, in both runs |

- **Test 1:** `placeholder_answers_leave_no_trace_in_facts_memos`.
  - q1 is `(u8, Wrap<B>)` Sized. On fresh facts it answers false without a
    set, true under {B} and false under {A}, so both the set and which set
    it is change the answer.
  - The orders [none, {B}], [{B}, none], [{B}, {A}] and [{A}, {B}] on shared
    facts each answer as the fresh proof does.
  - The declared State route was dropped before any run: a State without a
    declaration scope answers `prove` from canonical assumptions and runs no
    search.
- **Test 2:** `negative_queries_leave_the_body_selection_memo_unchanged`.
  - On `nested-client-side`, both `possible()` orders are disjoint.
  - They run negative queries, including at least one with a non-empty set.
  - `selection_memo` has the same number of entries before and after.
- **Hardening test:** `placeholder_owners_take_their_premises_once_before_any_work`.
  - A fresh State, a State already used by a premise-less proof of
    `P<u8>` Sized, a fresh Cursor and a started Cursor all reject a late
    install.
  - An equality under a set fails with its fence. Without a set it fails, as
    before, for want of a scope.
- **Recorded failed prediction:** fastcheck `1794231888971333396`.
  - The used-State input was first declared as `P<A>` Sized, where A is
    Carrier's parameter, with the label "as at base". No base run had
    checked it.
  - It answered `Err("shape parameter is outside its environment")`: a
    premise-less proof's shapes use an empty environment, so
    `Shapes::composite` fails on A before any size request reaches the
    State.
  - Amendment 4 replaced the input with the closed `P<u8>`, whose
    `Projection` rule sends its size through `State.shapes` and a Cursor.
- **Layout:**
  - `State` stays 808 bytes, and `Caller` and `OwnedCaller` stay 32.
  - The projections `Cursor` measures 584 bytes. `used` sits at offset 576,
    beside the one-byte `Availability` at 577, inside the struct's tail
    padding. The base size is inferred, not measured: without `used` the
    small fields end at 577, which also rounds to 584. Nothing pins the
    Cursor's size.
- **Checks:** focused fastchecks passed:
  - the 15-case post-rebase set plus the four new or affected tests;
  - the private bodies, methods and dispatch chunks (459, 325 and 213 cases);
  - the trait integration roots (116 and 240);
  - crate linking's absence and modifier transport (5) and their native
    clients (2);
  - native negative coherence (1);
  - `pa20` (10).

## 4. The fences retire (checkpoint 3b)

The checkpoint's outcomes were declared in five parts, each written before the
runs it governs: part 1 before any code, part 2 after the rustc 1.98.0 oracle
of the fourteen new twins, part 3 after their base runs, part 4 during coding
before any build, and part 5 after the first runs, before any change they
suggested. The base is `base3`, the dev-fast CLI of `7acc24208`, and the
candidate is `cand5`, built from the fixed 3b tree.

### The change

- **Shape walk:** under placeholders, the walk's size request is answered by
  the proof's trait `State`, through `State::shape_size`, as the premise-less
  arm already was. If the size proof or the answer fails, `refuse_size`
  cancels the walk, the proof keeps its shapes and the error is returned, so
  the interned fallback stays unreachable.
- **The State a walk starts** takes the walk's own set
  (`Proof::walk_trait_state`), never the proof's shapes slot, which the walk
  has emptied. Starting a State is one function, `start_trait_state`, which
  takes its premises explicitly. See the plumbing gap below.
- **Size query:** `sizes::Query::fence` is gone. Under placeholders the query
  applies a projected rule and descends into its tail, which the State's
  Cursor normalizes under the set.
- **Rigid aliases (`sizes/query.rs:82`):** a projection or opaque that
  normalizes to itself is decided by `rigid_sized`. Without placeholders it
  reads `is_sized`, as before. Under them it reads the alias's declared bound.
- **Failed size proofs:** under placeholders, `State::shape_size` drops the
  failed scope's size query and its Cursor before it returns the error, so the
  next size request in that scope starts afresh (part 4).
- `PLACEHOLDER_SIZE_FENCE` is removed. `PLACEHOLDER_EQUALITY_FENCE` and
  `PLACEHOLDER_REINSTALL` stay.
- **Test-only additions:** a thread-local count of the normalizations that
  size proofs start, premise-less and under a set; `State::work`;
  `Proof::trait_state_work`; and `Proof::trait_state_premised`. Part 3
  assumed that `types` would need an accessor for `infer::Context::steps`.
  That field is already public, so none was added (part 4).

### The `:82` argument, for test D

No input can make the declared bound differ from the interned bit, so the
change at `:82` changes how the answer is read, not the answer.

- `projection_properties` (`types/projection.rs:101-104`) sets a projection's
  Sized property to `projection_schema(item).sized`, and `opaque_properties`
  (`types/opaque.rs:105`) sets an opaque's to `declaration.sized`.
  `seal_nominal_sizes` recomputes both from the same declarations.
- The declarations are immutable: `declare_projection` rejects a conflicting
  schema, and nothing rewrites an opaque contract.
- No other bound applies. A placeholder `State`, `Cursor` and `Search` have no
  environment, so no environment projection promise adds one. rustc's other
  input for an alias is a `param_env` premise, membership of the alias
  itself, and `placeholder_begin` decides membership before `:82` is reached.

Test D, `rigid_aliases_decide_their_size_by_premises_and_declared_bounds`,
reaches `:82` under a set with a normalizer that returns its input. The
`?Sized` alias is not Sized under the empty set, the implicitly Sized alias is,
and an alias in the set is Sized without a normalization. For both aliases the
interned bit equals the declared bound, and the premise-less query gives the
same answer as the empty set.

### Work bound of the reset

In production a failed size proof ends the negative query, so a scope pays at
most one failed demand per analysis. Only test C retries on the same proof.
Every caller on the path returns the error with `?` and none retries:

- `pay_negative`;
- the Search's builtin proofs and `builtin_requirement`;
- `overlap::possible`, `iteration` and `inherent`;
- method overlap (`method_overlap.rs:69`) and conformance overlap
  (`conformance.rs:137`);
- `methods/overlap.rs`, which maps the error and returns it.

### The premise plumbing gap (part 5)

The first runs failed five unit predictions and two twins (fastcheck
`1794237160822701743` and the `cand4` CLI). A trace CLI, never evidence, found
the cause.

- `Proof::shape_properties` takes `self.shapes` and `term_shapes` out of the
  proof for the walk. The keep arm then started the trait State lazily, and
  `trait_state` read its premises from `self.shapes`, which was empty. So the
  State, its size query, its Cursor, `normal_form` and their Searches all ran
  without the set.
- A premise-less nested proof decides a rigid parameter by its declaration.
  That is how eight of `cand4`'s rejected → accepted files and `P<A>` under the
  empty set were answered: the leak this item forbids. They agreed with rustc
  only because each representative's declaration happened to match the other
  header's premises. None of `cand4`'s observations is evidence.
- Before 3b the path was unreachable: the keep arm returned the fence before it
  started a State. Step 2's and 3a's tests checked the State's set with the
  shapes in place.
- Section 3's cache audit covered memos and interned reads. It did not ask
  whether every holder of a set reads it from a slot that can be empty, so it
  could not find this gap.
- **Fix:** the State takes its premises explicitly, and the keep arm passes
  `shapes.placeholders()`. Premise-less proofs are unchanged.
- **Guard:** every placeholder-set test that starts a State asserts that the
  State holds a set (`trait_state_premised`): B, C, E and the q2 proofs of
  test 1.
- **Set shape:** production puts every Sized premise into the `MetaSized` set
  as well (`overlap/inputs.rs:143-145` and `:310-312`, with written
  `MetaSized` premises at `:289-292`). The new `premises(sized, meta_only)`
  helper builds that shape. The older `placeholders(set)` helper gives a Sized
  set with an empty `MetaSized` set, which production never builds. The
  accepted step-2 and 3a tests that use it (the walk and size-query tests,
  `placeholder_callers_install_their_premises_in_every_search`,
  `placeholder_owners_take_their_premises_once_before_any_work`, and q1 of
  test 1) read no `MetaSized` premise, and they keep it.

#### Appendix: the holder sweep

Every holder of a set on the placeholder path was read for where it gets its
set.

- **Taken slots:** only `Proof.shapes` and `term_shapes` are ever taken, and
  only inside `shape_properties`. The `State`, `Cursor`, Search caller and
  `negative_sizing` fields are never taken. The tests' `Search::new` sets
  none.
- **Explicit holders**, which receive the set from their constructor or
  their owner:
  - Searches: `search.rs:207`, `conjunction.rs:38`, `coherence.rs:90` and
    `suspension.rs:371`.
  - `Caller` and `OwnedCaller`: `traits.rs:226` and `:573`,
    `quantified.rs:57`, `conjunction.rs:68`, `decide.rs:39`,
    `normal_form.rs:458` and `:573`, `cursor.rs:208`, `resumed_queries.rs:26`
    and `suspension.rs:95`. A closed requirement passes none.
  - `normal_form`'s State: `normal_form.rs:206`, through `opening.rs:45`.
  - Size queries: `shape_normalization.rs:118` and `sized_obligations.rs:33`.
    `sized_obligations.rs:89` is never advanced.
  - Cursors: `shape_normalization.rs:116`, `term_normalization.rs:82`, and
    the `Session` at `normalization.rs:34`, through `projections.rs:153`.
  - `Shapes::normalized_query` (`shape.rs:160`) inherits its walk's set.
  - `NegativeSizing`: `continuation.rs:129`.
  - Proofs: `inputs.rs:568` is explicit; `inputs.rs:612` is the premise-less
    `pay_own`; `continuation.rs:145` and `:150` install the set right after
    construction.
- **Three holders without a set, each safe:**
  - The Cursor at `disjointness.rs:302` has none, but only closed tails, with
    `contains_parameters` false, reach it.
  - A `Normalizer` holds no set: a Search's normalizations descend on that
    proof's own stack.
  - `prepare_shapes` (`nominals.rs:220-238`) copies `self.shapes`' set into
    `term_shapes`. It is called only at proof entry (`:200` and `:377`),
    before anything is taken, and nothing inside a walk re-enters its proof.

No other holder reads its set from a slot that can be empty.

### Tests

- **Test 1** gains q2, `P<B>` Sized. Without a set it fails with "shape
  parameter is outside its environment", under {B} it is Ok(true) and under
  {A}, with B `MetaSized` only, it is Ok(false). The four orders on shared
  facts each answer as a fresh proof does. q1 keeps its 3a sets and answers.
- **B**, `placeholder_proofs_decide_projected_sizes_by_premises_and_never_fall_back`:
  `P<A>` is Sized under {A}, though its interned property is not, and not
  Sized when A is `MetaSized` only, though A's declaration is Sized.
- **C**, `failed_placeholder_size_proofs_cancel_their_walk_and_answer_afresh`:
  `Q<A>` has no implementation to normalize its field. The same proof fails
  it twice with the same error and twice the walk work, then answers a closed
  type, a type with parameters and `Q<A>` as a fresh proof does.
- **D:** above.
- **E**, `placeholder_projected_size_work_is_linear_in_depth`, at depths 1,
  2, 4 and 10, under `premises([B])` (Sized) and `premises([A], [B])` (not
  Sized). It asserts:
  - parity: the placeholder proof of `P^d<B>` starts as many normalizations,
    all under its set, as the premise-less proof of `P^d<A>` in an
    environment;
  - its accounted work, the walk's steps plus the State's work, stays on or
    under the line through depths 1 and 2;
  - the State holds a set.

  It does not assert the count: both proofs start 2^d - 1 normalizations. That
  is the pre-existing performance item recorded in the
  [blockers](../library-build-blockers.md): the nested proofs of each level
  share no work, and their work is not charged to the outer proof. The
  observed work is 22, 38, 70 and 166 (Sized) and 23, 39, 71 and 167, exactly
  16 a level. The premise-less twin is over the Sized A, because a hand-built
  environment gives a `?Sized` parameter no `MetaSized` premise.
- **WF test:** it no longer checks that a header failing well-formedness
  avoids the fence, which is gone. It still requires a well-formedness error,
  never the overlap text.
- **`coherence_premise_sides`:** the fence is gone. CASES has 59 entries,
  47 before; the probes stay out. The symmetry pairs go from 17 to 23.
- State stays 808 bytes, and `Caller` and `OwnedCaller` stay 32.

### Twins

Twelve files flip, all approved by name. The other 62 of the 74 are
byte-identical to `base3`, status lines and stderr.

| Twin pair, both orders | rustc 1.98.0 | `base3` | 3b |
| --- | --- | --- | --- |
| `nested-projected-size` | 0 | fence | 0 |
| `nested-projected-sized-impl` | 0 | fence | 0 |
| `nested-projected-composite` | 0 | fence | 0 |
| `nested-projected-metasized` | 0 | fence | 0 |
| `nested-projected-premise-side` | E0119 | fence | overlap |
| `nested-projected-unsized` | E0119 | fence | overlap |

- The `nested-projected-written` and `nested-normalized-builtin` pairs are
  accepted by both, as rustc accepts them.
- The `probe-projection-premise` pair is rejected by both with the overlap
  text, and rustc accepts it. It is a known rejects-valid for item C: the
  negative impl `impl<T> !Seal for &T` first retains `P<V>`'s formation goal
  `V: Tr`, which fails because the other header's `U: Tr` is a trait premise,
  so no shape walk reaches `P<V>`.

### Reach

An instrumented `cand5` (CLI `5fd50567…`, never committed; the five
instrumented files were restored and their hashes checked) printed a marker at
each placeholder install, walk, size request, State size proof under a set
(`size-proof`) and `normal_form` selection under a set (`normalize`). Its
status lines and stderr, without the markers, equal `cand5`'s.

| File | size | size-proof | normalize |
| --- | --- | --- | --- |
| `nested-projected-size`, each order | 8 | 2 | 2 |
| `nested-projected-sized-impl`, each order | 8 | 2 | 2 |
| `nested-projected-composite`, each order | 10 | 2 | 2 |
| `nested-projected-metasized`, each order | 3 | 1 | 1 |
| `nested-projected-premise-side`, each order | 5 | 1 | 1 |
| `nested-projected-unsized`, each order | 5 | 1 | 1 |
| `nested-projected-written`, each order | 1 | 0 | 0 |
| `nested-normalized-builtin`, each order | 0 | 0 | 0 |
| `probe-projection-premise`, each order | 0 | 0 | 0 |

- Every declared marker appears, the written pair has no size proof, and the
  probe has neither new marker.
- The 336 survey files print no size-proof and no normalize marker, and no
  walk or size marker; 31 installs in all.

### Recorded failed predictions

1. **Probe before (part 2):** declared as the fence; `base3` gave the
   overlap text in both orders. Part 2's trace began at `P<V>`'s size request
   and missed the formation goal that comes first, so its after and reach
   predictions for the probe were wrong as well. Part 3 amended them before
   any candidate run.
2. **Flip count:** parts 2 and 3 gave 16 and 14 flip files. There are 12,
   the 8 rejected → accepted and 4 rejected → rejected files above. The
   files and outcomes were declared correctly; the totals were miscounted.
3. **Part 5:** the five unit predictions and two twins above, failed by the
   plumbing gap. They pass as declared after the fix, and none of the
   failing run's observations is evidence.
4. **Test E's count:** declared as exactly d normalizations; fastcheck
   `1794238322804605380` counted 3 at d = 2. The diagnosis
   (`obj/lane-b9/step3b/diag/depth-diagnosis.txt`, a diagnostic build) shows
   2^d - 1 for the premise-less and placeholder proofs alike, through d = 12.
   E now asserts parity, not the count.
5. **Test E's twin:** declared over `P^d<B>` in a hand-built environment;
   fastcheck `1794240293859845637` failed it with "associated type requires
   an unambiguous validated implementation", because that environment gives
   the `?Sized` B no `MetaSized` premise. The twin is over the Sized A.

Not evidence: fastcheck `1794236873051504922`, which ran out of budget cold,
and the diagnostic fastchecks.

### Evidence before the rebase

| Gate | `base3` | `cand5` |
| --- | --- | --- |
| 74 twins, status lines | `7a396df0…` (60 files) and `e0eda4ed…` (the 14 new) | `72d756f1…`; only the 12 files above differ, in status and stderr |
| Flip survey, 336 files | `9e1b7e43…` | `9e1b7e43…` |
| Census lookahead, `census.tsv`, 23 lines with 8 records | `9d654755…` | `9d654755…` |
| Golden table | `399d7a0e…`, 5674 lines, 60 fixtures | `6d634d2f…`, 8356 lines, 74 fixtures |

- The golden table moves only on the four projected twins, which now give
  their overlap or acceptance rows, and gains the 14 new fixtures.
- Focused fastcheck `1794240625255197153` passed the 20 selected private
  cases and the crate-linking integration case.
- The final gates, on the rebased tree against a fresh base pin, are in
  checkpoint 4.

## 5. Final evidence and notice (checkpoint 4)

The four commits were rebased once onto `c8c2a8eda`: step 1 is `b37529879`,
step 2 `6231c4955`, 3a `54e6b984d` and 3b `196fd2667`. The only conflict was
the blockers tracker's top entry, where both entries were kept. The final
gates were declared after the rebase and before any build or run, with one
amendment for the golden hook, also written before any run.

- **Base:** `base6`, the dev-fast CLI of `c8c2a8eda`, sha256 `b612c926…`. It
  has none of item B, so these gates span all four commits.
- **Candidate:** `cand6`, the dev-fast CLI of `196fd2667`, sha256 `75472a60…`.

| Gate | `base6` | `cand6` |
| --- | --- | --- |
| 74 twins, status lines | `4642dfe2…`; its lines and every stderr equal `base3`'s | `72d756f1…`, equal to `cand5`'s |
| Flip survey, 336 files | `9e1b7e43…` | `9e1b7e43…` |
| Census lookahead, `census.tsv` | `9b678716…` | `9b678716…` |
| Golden table | `399d7a0e…`, 60 fixtures | `6d634d2f…`, 74 fixtures |

- **Twins:** exactly the twelve files of section 4 differ, in status and
  stderr. Every other file is byte-identical.
- **Census:** `result.json` differs only in CLI identity, the output path in
  stderr, and the traced process and timing identity. Main's `4df942467`
  changed the census itself since `base3`: it now has 9 lines with one
  record, `local_annotations` at `ptr/metadata.rs`, after 42310 checked
  declarations.
- **Golden:** the tables equal `base3`'s and `cand5`'s. The base uses the hook
  with main's one-set `Placeholders::new`; it differs from the candidate's
  hook only in those two calls. The tables differ only on the four projected
  twins and the fourteen new fixtures.
- **Texts:** neither `PLACEHOLDER_REINSTALL` nor `PLACEHOLDER_EQUALITY_FENCE`
  appears in any gate output, and the retired fence text appears in no
  candidate output.
- **Work:** the scaled `MetaSized` pair and `PointeeSized` control are
  unchanged, 1965, 3810, 7559 and 1693, 3256, 6427, in both runs. Both
  scaling tests print the same lines as at 3a.
- **Fixtures:** `make test-upstream` verified 128 pinned upstream fixtures
  and 126 Reference files.
- **Checks:** focused fastchecks passed on `196fd2667`:
  - the 3b set: 20 private cases and the crate-linking case;
  - the private bodies, methods and dispatch chunks (459, 325 and 213
    cases), with step 2's list carrying 3b's two renamed tests;
  - the trait integration roots (116 and 240);
  - crate linking's absence and modifier transport (5) and their native
    clients (2);
  - native negative coherence (1);
  - `pa20` (10).
- **Not evidence:** a cold fastcheck that ran out of budget in its build, and
  a selection failure before step 2's list was given 3b's two test names.
- **Correction:** sections 2 to 4 first called the census "23 records". It
  is 23 lines, with 8 records.

### Notice

- A negative coherence query's nested trait proofs now assume the other
  header's Sized and `MetaSized` premises: the builtin proof's trait State,
  its size queries and Cursors, `normal_form` and every Search they start.
  A projected-size nominal is decided under the set, and the projected-size
  fence text is gone.
- A holder of a placeholder set takes it from its owner explicitly. None may
  read it from a slot that a walk can take.
- Twelve twins change outcome, as listed in section 4. Every other input of
  the gates is unchanged.
- Open for item C: the `probe-projection-premise` and `probe-trait-premise`
  pairs, which rustc accepts, need the other header's trait premises.
- Open, and Lane B's next item: the pre-existing 2^d - 1 nested
  normalizations of `P^d<T>`, whose work is not charged to the outer proof
  (see the [blockers](../library-build-blockers.md)).
