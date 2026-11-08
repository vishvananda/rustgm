# Linear success rows and lifetime contract waits

Lane B batch of 2026-11-08, branch `implement/linear-waits-20261108` on main
`016b1f54d`. It removes the two quadratic hazards the
[nightly private repair](nightly-private-repair.md) found, and a third that
the second's demand exposed. The core census runs field storage and early
conformance at whole-crate width, so a quadratic walk there would become a
budget record.

Every count below was measured on `016b1f54d`, where consumers of
type-generic-only providers also prepare body storage. They are the same as
on the previous base, `e07b0c511`, except that section 2's source contract
work includes section 3's index: one more unit, for its box.

## 1. Success rows

### Cause

`prepare_storage_fields` and `prepare_storage_fields_pending` publish a walk
with `publish_prepared(charged = true)`, which prepays the dense success rows
up to the walk's largest type identity with `prepare_success_rows`. The rows
were reserved exactly: a demand whose largest identity was past the capacity
copied every row, was charged that copy, and left the capacity at exactly its
own row count. The next demand that published a newer type copied again.

Its callers are the layout kernel (`prepare_layout`), source transmute sizes,
`DispatchFromDyn` unchanged fields and the caller `Location` constant. The
crate-wide `prepare` and the body walk publish through `Prepared::insert`,
which already grows geometrically, so the hazard shows after semantic
analysis. Every instantiation that execution interns gets a new identity, and
each layout demand of one then copied the whole row vector.

### Exact growth

`types::aggregate::instances::tests::ascending_storage_completions_copy_success_rows_geometrically`
completes `n` distinct `S{index} { v: u8 }`. The store holds 21 types before
them, so the rows end at `n + 21`.

| n | identity order, before | identity order, after | reverse order (control) |
| --- | --- | --- | --- |
| 64 | 3,809 | 536 | 470 |
| 256 | 39,809 | 2,144 | 1,814 |
| 512 | 145,153 | 4,288 | 3,606 |

Before: `n(n + 1)/2 + 27n + 1`, that is `6n + 1` walk steps, `n + 21` new rows
and a copy of every earlier row at every later completion
(`(n - 1)(n/2 + 21)`). The
reverse order allocates once: `7n + 22`. After: the reverse order plus the
copies at 22, 44, 88, 176 and 352 rows (66, 330 and 682 in all).

### Repair

When the rows outgrow their capacity, the new capacity is at least double the
old one. The charge is unchanged, each copy is still charged when it happens,
and the retained bytes are the capacity actually added. All copies together
are at most twice the final rows.

The invariant requires, at 64, 256 and 512 completions, that the identity
order costs at most the reverse order plus twice the final rows, and that the
reverse order's increments are equal per completion. Before the repair it
fails at 64 (3,809 against 470 + 170).

## 2. Lifetime contract waits

`lifetime_contract_dependency` (`sema/bodies/field_storage.rs`) runs in a
declaration-time body of a source library. It relates the body's generic
lifetime nominals by their variance and outlives contracts before the
crate-wide inference. Every nominal whose field template its declaration has
not published yet costs the body one wait, and the body resumes from
`prepare_publication` after each.

### Causes

1. Every resumption ran `unreached_nominals` again over all of the body's
   solved types: `(waits + 1) × (terms + structures)`.
2. A nominal whose closure met an unpublished template dropped its whole
   variance graph (`append_reached_or_pending` returned with nothing
   retained). After the wait the closure was walked again from its root, so a
   chain of nominals behind references, which storage preparation does not
   expand, cost `1 + 2 + … + n` owner walks.
3. The second scan omitted every nominal the first pass had already completed,
   so the final pass demanded outlives contracts only for the nominals that
   were still unreached. A body that relates a completed nominal before
   another waits lost that nominal's contract. Its region proof then failed
   with "missing nominal outlives facts", where rustc accepts. This was a
   correctness bug, not only cost.

The outlives demand that the body then makes for each collected nominal had
its own product cost. Section 3 repairs it.

### Repair

- `FieldWaits` keeps the body's unreached nominals, collected once, and the
  index of the nominal being completed. A resumed demand continues there, and
  the outlives demand covers every collected nominal, as a body without a
  wait already did.
- `Variances::append_reached_or_pending` takes a `PendingReach` for its
  nominal. At a wait, the paid graph (pending owners, slots, variables,
  solver nodes, edges and products, visited set) is parked without its
  borrowed rows, declarations or payment. An owner without a template stays
  on top of the pending stack. The next append resumes the same graph under
  the current rows and payment, at that owner. A row another body completed
  during the wait is a constant for owners the graph has not reached. When
  the graph completes, a row already published stays as it is; its values
  are the same.
- `append_reached` (no waits) and every crate-wide inference are unchanged.

### Exact growth

Two invariants measure the repair, and the controls record the bug.

`types::variance::reachable::tests::waited_reached_closures_resume_at_the_owner_they_waited_for`
appends the reached variance of `L0<'a> { next: &'a L1<'a> }` through
`L{n-1}<'a> { r: &'a u8 }` and publishes each field template only after the
append waits for it. Each nominal is waited for once, and every row equals the
crate-wide inference's. The paid work is:

| n | resumed | restarted at each wait (old behaviour, control) |
| --- | --- | --- |
| 64 | 3,465 | 84,533 |
| 256 | 13,897 | 1,338,253 |
| 512 | 27,793 | 5,337,425 |

Resumed work is 54 per nominal, and the restarted work is about `20.4 n²`. The
invariant requires near-linear increments: from 256 to 512 at most 1.5 times
the increment per nominal from 64 to 256. It also requires the restarted
control to exceed 32 times the resumed work at 512.

`sema::bodies::field_storage::tests::waited_lifetime_contracts_resume_at_the_waited_nominal`
analyses a source library whose array length calls a const fn over `n`
nominals declared after the length's owner. Its source contract work is:

| shape | n = 6 | n = 24 | n = 96 | before the repair |
| --- | --- | --- | --- | --- |
| `wide`: `n` parameters `M<&L{i}<'_>>` | 1,071 | 2,635 | 8,843 | every `n` rejected with "missing nominal outlives facts"; from 256, "source contract work or retention exceeds its input-graph bound" |
| chain: `L{i}<'a> { next: M<&'a L{i+1}<'a>> }` | 1,335 | 3,966 | 14,346 | 827, 1,129 and 3,534 at 2, 3 and 8; from 64, the input-graph bound |

The increments are 87 and 146 per nominal, and the invariant requires them to
stay near-linear in the same way. It uses 6, 24 and 96 nominals because of the
byte limit recorded in section 4.

### Controls

Three no_core controls were added to
`pa31/tests/spec/declaration-lifetime-contracts/`. Their outcomes were
declared before any rustc or RustGM run of their bytes. The declaration copy
is `obj/lane-b3/declarations/lifetime-contract-waits.declared.json` (sha256
`8b5021878cad…`), and the rustc 1.98.0 oracle receipt is
`obj/lane-b3/oracle-contract-waits/receipt.json`. All three agree.

| control | declared | rustc 1.98.0 | before the repair | after |
| --- | --- | --- | --- | --- |
| `earlier-nominal-length.rs` | accept | accepts | "missing nominal outlives facts" | accepts |
| `covariant-chain-length.rs` | accept | accepts | accepts | accepts |
| `invariant-chain-length.rs` | reject | "lifetime may not live long enough" (`Link0` is invariant) | "required lifetime does not outlive its target" | the same |

`variance_contract_constants.rs`'s `CASES` grows from 8 to 11. The original
eight cases and their expectations are unchanged.

## 3. Explicit nominal bounds

### Cause

`Lifetimes::demand_reached_outlives` (`sema/lifetimes.rs`) grouped every
explicit nominal bound of the crate by owner before each newly demanded
nominal's outlives contract. It was charged, but the work was (newly demanded
nominals × crate bounds). Both of its callers run at whole-crate width:
declaration-time bodies (section 2) and early scalar conformance, which
demands each implementation's nominal subject.

### Repair

`Lifetimes` keeps a boxed index of the bounds by owner and the number of
bounds already indexed. Declarations only append bounds, so a demand indexes
only the bounds published since the previous demand, and each bound is
indexed and charged once. The map growth and up to four slots per new bound
are charged before they are allocated. If the bound list were ever shorter
than the index, the index would be rebuilt. The crate-wide inference, which
groups the bounds once per pass, is unchanged.

### Exact growth

`sema::lifetimes::tests::demanded_outlives_contracts_index_each_explicit_bound_once`
declares `b` structs `B{j}<'a, 'b: 'a> { a: &'a u8, b: &'b u8 }`, whose bounds
no field implies, and `n` structs `L{i}<'a> { value: &'a u8 }`. It demands
every `L{i}` and then `B0`, and checks each contract against the crate-wide
inference. The paid work is:

| | b = 64 | b = 256 | b = 512 |
| --- | --- | --- | --- |
| n = 64 | 1,945 | 2,905 | 4,185 |
| n = 256 | 6,705 | | |
| n = 512 | 13,045 | 14,005 | 15,285 |

Each bound costs 5 and each demanded nominal about 24.8. The bounds' share
is the same at 64 and 512 nominals: the invariant requires those increments
to be equal and both series to grow near-linearly. The restarted control
forgets the index before every demand, as each demand regrouped every bound
before the repair. It costs 1,326,517 at `n = b = 512`, and the invariant
requires at least `512 × 512` more than the repair. Before the repair a
demand was charged `b + 1`, so at `n = b = 512` the grouping alone was
513 × 513 = 263,169.

## 4. Files

- `types/aggregate/instances.rs`: geometric success rows.
- `types/variance/solver.rs`, `types/variance/payments.rs`: park and resume a
  paid graph.
- `types/variance/reachable.rs`: `PendingReach`. The pending owner stays on
  top. A row published during a wait is kept.
- `types/variance.rs`: re-exports `PendingReach`.
- `sema/bodies/field_storage.rs`, `sema/bodies/finalization.rs`: the body's
  nominals, its index and the waiting reach live in `FieldWaits`.
- `sema/lifetimes.rs`: the explicit nominal bound index.
- Tests: the new `tests/invariants/success_rows.rs` and
  `tests/invariants/explicit_bounds.rs`, plus `reachable_variance.rs` and
  `variance_contract_constants.rs`. `Declarations` grows by 8 bytes, to
  7,152, for the boxed index: `prototype_headers.rs` pins it as a literal
  twice, and seven layout tests add `size_of::<Option<Box<()>>>()` to their
  sums.
- No `sema::traits` file or wire byte changes. The boxed per-body
  `Publication` grows by `FieldWaits`' three new fields.

## 5. Remaining limit

**Chained nominals attempt more bytes than their input allows.** A chain
nominal costs about 3.77 KB of attempted source-contract bytes. About 2.16 KB
is the variance graph's conservative `Growth` estimates and about 1.25 KB is
the outlives closure. The input-graph bound allows about 2.94 KB per such
nominal. The cost is linear, but a chain of about 200 or more referenced
nominals that all wait in one declaration-time body exceeds the bound.

The repair is queued after task 3: charge the parked graph's actual retained
bytes instead of an estimate, as the `path_compatibility` repair made the
accounting match real retention. The bound itself stays. The repair moves
ahead of the queue if census 12 or any control reaches this limit.

## 6. Checks

All on this branch's third commit over `016b1f54d`, with receipts under
`obj/qualification/fast/`. The same selections, without the 34 private tests
section 3 added, also passed on the second commit (`1794130804456421647`,
`1794131170068085480`, `1794131274063180692`, `1794131328252468694` and
`1794131407599894826`).

| check | receipt | result |
| --- | --- | --- |
| 64 private tests: the four new invariants, the 11 lifetime contract controls, the five other `reachable_variance` tests, the layout kernel's two, the resumable field walk, batch 5 and batch 7's declaration-time controls, the 11 PhantomData role, callable template and member transaction tests of `016b1f54d`, the nine early conformance tests, the five demanded outlives tests, and all 19 tests that measure `size_of::<Declarations>()` | `1794133823204162398` | pass |
| 96 integration tests: 91 in 13 roots (`shape_constructors`, `nominal_instances`, `nominal_variance`, `nominal_outlives`, `field_outlives`, `finite_storage`, `size_conditions`, `derive_contracts`, `tail_rules`, `nominal_declarations`, `destructor_conformance`, `borrowed_receivers`, `census_mode`) and five `crate_linking::phantom_roles` cases | `1794132844873204870` | pass |
| 32 outlives integration tests: `outlives_types`, `outlives_verification`, `caller_outlives` and `region_constraints::nominal_bounds_are_enforced_and_implied_while_alias_bounds_remain_unenforced` | `1794133227237829766` | pass |
| PA1–PA12 | `1794133274325843176` | pass |
| PA13–PA19, PA21–PA24 | `1794133323458885736` | pass |
| PA20 | `1794133546656439564` | 1451/1451 |
| `make test-upstream` (second commit; the third changes no fixture) | `obj/lane-b3/t1/upstream-2.log` | 128 fixtures, 126 Reference files |
