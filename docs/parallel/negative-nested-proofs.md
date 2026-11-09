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
| Census lookahead | `census.tsv` byte-identical, 23 records; `result.json` differs only in CLI, producer and timing identity |
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
| Census lookahead | `census.tsv` byte-identical, 23 records; `result.json` differs only in CLI, output path, process and timing identity |
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
