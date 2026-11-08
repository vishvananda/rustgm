# Census batch 8: rigid projection sizedness, the executed-intrinsic test and the ninth census

Census 8 ([batch 7](census-batch7.md), pin `f99b369d`) checked 42,310
declarations of original core with none failed or blocked. Its one crate
record was "trait argument does not satisfy its declaration requirements" at
`pin.rs:1105`. This batch, on main `4a9a3441f`:

- proves sizing goals of rigid projections from their associated type's own
  item bounds, as rustc 1.98's sizedness hierarchy does. This repairs
  `pin.rs:1105` and the next record it exposed, `iter/adapters/flatten.rs:226`
  (section 1);
- names the next record, `option.rs:1386`, without repairing it (section 2);
- updates `reached_intrinsics_cannot_execute_their_authored_fallback` to the
  executed `ctpop` handler and adds an intrinsic with no handler that keeps the
  original rejection (section 3);
- does not add keep-going across the crate-wide declaration passes, because
  it does not fit one checkpoint (section 4);
- runs census 9 (section 5).

Census 9 checks 42,310 declarations, with none failed or blocked. Its one
crate record is "invalid canonical trait selection" at `option.rs:1386`, which
is Lane B's. Bodies are not entered, so Milestone 2 ("original core passes
declaration analysis") is not reached.

Out of scope:

- Lane B owns selection and normalization outside the sizing rule, re-exports
  and crate loading. `option.rs:1386` is in its selection and region mapping
  work.
- Lane C owns body and item transport.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend is frozen.
- `!null` pattern types over integer or reference bases stay rejected by name.
- Ordinary crates keep batch 7's declaration-time lifetime contract gap, and
  PA20 references are not regenerated.

## Diagnosis method

Each diagnostic was an analysis-only pin of the working tree with one census of
unchanged original core inputs, in `obj/parallel/census-batch8/diagnostics/dN/`.
Their binaries are retired.

| Run | Base | Checked | Failed | Blocked | End |
|---|---|---|---|---|---|
| Census 8 (`f99b369d`) | `9f04f8694` | 42,310 | 0 | 0 | crate record: trait argument does not satisfy its declaration requirements (`pin.rs:1105`) |
| d1 (`c702a547`), first sizing repair | `4a9a3441f` | 42,310 | 0 | 0 | crate record: nominal type argument must implement its declared trait (`iter/adapters/flatten.rs:226`) |
| d2 (`998c1cae`), section 1 | `4a9a3441f` | 42,310 | 0 | 0 | crate record: invalid canonical trait selection (`option.rs:1386`) |

Small drafts reduced each record. They are in
`obj/parallel/census-batch8/repro/` and were run with analysis-only builds of
the working tree (`obj/parallel/census-batch8/dev/`, binaries retired) and,
for exploration only, with rustc 1.98.0.

## 1. Rigid projections are sized from their item bounds

rustc 1.98 orders the sizing markers `Sized: MetaSized: PointeeSized`.
`?Sized` relaxes a bound to `MetaSized`, and `PointeeSized` holds for every
type. A rigid projection is `MetaSized` or `Sized` from its associated type's
own item bounds.

Census 8's record is core's

```rust
impl<Ptr: Deref, Q: Deref> PartialEq<Pin<Q>> for Pin<Ptr>
where
    Ptr::Target: PartialEq<Q::Target>,
```

`PartialEq<Rhs: PointeeSized = Self>` requires `<Q as Deref>::Target:
PointeeSized`. Batch 7 supposed that selection decided this goal from item
bounds lacking the implicit `MetaSized`. That was not the cause. Selection
normalizes the goal's inputs (`sema/traits/selection/search/inputs.rs`), then
consults the environment's assumptions. The environment holds a sizing
assumption over another projection: `<Ptr as Deref>::Target: MetaSized`,
elaborated from `PartialEq`'s implicit Self bound. Comparing it with
`<Q as Deref>::Target` returns `infer::Error::Normalization`, and that answer
deferred the goal. Formation then failed it.

The repair is `Search::rigid_sizing`
(`sema/traits/selection/search/sizing_fallback.rs`). It is called only on that
assumption answer:

- it applies only to a `PointeeSized`, `MetaSized` or `Sized` goal whose
  resolved subject is a projection;
- the projection's derived item-bound results decide first: its written
  bounds and the implicit `MetaSized` of a `?Sized` associated type;
- otherwise the source sizing fallback decides: `PointeeSized` always holds,
  and the projection's declared shape proves `MetaSized` and `Sized` from a
  declared (or default) `Sized` bound;
- an absent answer stays deferred, as before.

The source sizing fallback also changed in two arms:

- `PointeeSized` holds for every subject kind. Before, it held for foreign
  types, did not hold for parameters and arrays without a sized shape, and was
  an error for projections and opaques.
- A projection reached after input normalization is rigid, so a `MetaSized` or
  `Sized` goal its item bounds do not prove is absent. Before, every projection
  was an error ("source sizing requires a proved normalized or declared
  subject"), and a projection not reached that way still is.

d1's `rigid_sizing` had the derived results and the `PointeeSized` fallback
only. It moved the record to core's `Flatten` Iterator impl
(`iter/adapters/flatten.rs:226`):

```rust
impl<I, U> Iterator for Flatten<I>
where
    I: Iterator<Item: IntoIterator<IntoIter = U, Item = U::Item>>,
    U: Iterator,
{
    type Item = U::Item;
    fn next(&mut self) -> Option<U::Item> {
```

`Option<U::Item>` needs `<U as Iterator>::Item: Sized`. The environment
assumes `<I as Iterator>::Item: MetaSized` through `IntoIterator`'s implicit
Self bound, so the goal hit the same deferral. Here `Iterator::Item` carries
its default `Sized` bound. Builtin `Sized` is not an indexed item-bound result
(`sema/traits/results/templates.rs` indexes trait, outlives and auto bounds),
so the derived results do not prove it. The fallback to the declared shape
does, and d2 has no record from either cause.

`sema::traits` changes are these two files:

- `inputs.rs`: the assumption answer calls `rigid_sizing` before deferring
  (one changed line, four added);
- `sizing_fallback.rs`: `rigid_sizing`, `normalized_inputs` and the two arms.

No record changed layout.

## 2. The next record: `option.rs:1386` (not changed)

d2 ends at core's `Option::as_deref`:

```rust
pub const fn as_deref(&self) -> Option<&T::Target>
where
    T: [const] Deref,
```

It fails with "invalid canonical trait selection". The cause is "selected type
retains an unmapped lifetime" (`types/substitute/region_mapping.rs:147`). It is
raised on the nominal conjunction path, `sema/nominals/traits/conjunction.rs`
calling `selection::prove_conjunction`, while the well-formedness of
`Option<&'a <T as Deref>::Target>` is proved.

- Reduction (`obj/parallel/census-batch8/repro/dc-named.rs`): the section 1
  prefix with `pub const trait Deref`, `enum Option<T>` and
  `pub const fn as_deref<'a, T>(x: &'a Option<T>) -> Option<&'a T::Target>
  where T: [const] Deref`. It fails here and on base `4a9a3441f`, and rustc
  1.98.0 accepts it.
- `dc-arg.rs`, `deref-const.rs` and `deref-const-free.rs` fail the same way.
- Each of these passes: `&'static T::Target`, a raw pointer, a plain
  `T: Deref` bound, or returning `&'a T::Target` without `Option`.

So the failure needs a `[const]` bound, a reference lifetime and a nominal
argument together. This is shared selection and region mapping work, Lane B's,
and is not changed here. Owner guess: Lane B. It is the next crate-wide
declaration record.

## 3. The executed-intrinsic test

Batch 7 (section 7) recorded that
`sema::compiler_items::prototype_validation::tests::reached_intrinsics_cannot_execute_their_authored_fallback`
fails on its `const` case. Batch 5's source bit-count handler (`65a9eae6c`)
executes `ctpop` in constant evaluation, so
`pa28/tests/spec/intrinsic-declarations/execution/const.rs` is accepted, and
the test still expected "not implemented".

The test's purpose is that no reached intrinsic executes its authored fallback.
It now checks that directly (`dev/tests/invariants/prototype_headers.rs`):

- the const case appends a function whose array length is the constant.
  `ctpop(7)` is 3 and the authored fallback returns 7, so `[u8; 3]` is accepted
  and `[u8; 7]` is rejected with "incompatible types";
- an unhandled case replaces `ctpop` in the same source with `bswap`, which has
  no constant handler. It keeps the original rejection: "source intrinsic" and
  "not implemented". An assertion checks that the replacement changed the
  source;
- the `runtime` and `reified` cases are unchanged.

The fixture bytes are not changed. This amends a control's expectation: the
reason is batch 5's handler, and the independent observation is the length
check, which tells the handler's 3 from the fallback's 7.

## 4. Keep-going across crate-wide declaration passes (not done)

Each crate-wide declaration pass after the per-definition loop in
`Checker::run` (`sema/declarations.rs`) ends the census at its first error
with one crate record. There are about 29 of them, from
`complete_deferred_declarations` to `specialization::require_execution`.
Continuing past a failed pass is not one checkpoint:

- The passes mutate shared facts and publish results that later passes
  consume. For example, `validate_owner_bounds` and `validate_signatures`
  retain trait proofs that `validate_trait_regions` reads, and
  `complete_nominals` publishes fields and variances that layouts, objects and
  discriminants need.
- A pass that fails part-way can leave a partial publication. A later pass
  would then report errors that only follow from the first one, or read
  half-published facts.

Suggested design: record each pass's consumed and published facts. After a
failed pass, run only the later passes that consume nothing it publishes,
record their first errors as further crate records, and block the rest on the
failed pass. That needs an audit of every pass's dependencies.

## 5. Census 9

The pin `10c45c62` (`obj/parallel/census-batch8/qualified-1/pin/rustgm`) was
built from `97110f230`, which is base `4a9a3441f` plus this batch's two code
commits. No dev or scripts source was uncommitted: the build's retained patch
is empty.

- Its manifest is census 8's, with only `expected_cli_sha256` and
  `predecessor_manifest_sha256` changed.
- No configuration source pin changed, and nothing was rebound.
- The ordinary first error moved from `pin.rs:1105` to `option.rs:1386`
  (section 2).

The census is analysis-only and ran over unchanged original core inputs: 375
opens in 54 s. Its records are in
`rustgm-pa29-work/obj/parallel/core-census-9/` (`census.tsv` SHA-256
`8850630191f4a592`).

| Census | Checked | Failed | Blocked | End |
|---|---|---|---|---|
| Census 8 (`f99b369d`) | 42,310 | 0 | 0 | crate record: trait argument does not satisfy its declaration requirements (`pin.rs:1105`) |
| Census 9 (`10c45c62`) | 42,310 | 0 | 0 | crate record: invalid canonical trait selection (`option.rs:1386`) |

A crate-wide declaration pass still fails, so bodies are not entered: 0
bodies are checked, and there are no body causes and no checked-of-total body
count.

### Failures by cause

| Cause | Census 8 | Census 9 | Sites |
|---|---|---|---|
| trait argument does not satisfy its declaration requirements | crate record | none | `pin.rs:1105`, repaired (section 1) |
| nominal type argument must implement its declared trait | not reached | none | `iter/adapters/flatten.rs:226`, repaired (section 1, d1) |
| invalid canonical trait selection | not reached | crate record | `option.rs:1386` (section 2, Lane B) |

Neither census has a per-definition failure or blocked record, so no cause has
a failed or blocked count.

### Largest blocking roots

None. The one crate record ends the census. It blocks every later crate-wide
declaration pass and all bodies, which the census does not count.

## Controls

These cases were declared
(`obj/parallel/census-batch8/oracle/*.declared.json`) before the frozen rustc
1.98.0 oracle ran on the exact bytes and before any candidate run, and all
agree. Both are recorded in each directory's `outcomes.json`.

| Directory | Cases | Accept | Reject | rustc |
|---|---|---|---|---|
| `projection-sizedness` | 8 | 6 | 2 (E0277) | 8/8 agree |
| `normalized-projection-sizing` | 5 | 4 | 1 (E0277) | 5/5 agree |

- The prefix is the first 54 lines of batch 7's
  `repro/p/meta-arg.rs` draft: the exact pinned core `PointeeSized`,
  `MetaSized` and `Sized` fragment and minimal `LegacyReceiver`, `Clone` and
  `Copy`, then a minimal `Deref` with `type Target: ?Sized`.
- Batch 7 explored the first three `projection-sizedness` shapes with rustc,
  and this batch explored the first three `normalized-projection-sizing` shapes
  before their declaration. The READMEs say so; those are regression records of
  that exploration.

Every new positive control except two fails on base `4a9a3441f`
(analysis-only build `obj/parallel/census-batch8/dev/base0`, `f120c641`), and
passes here:

- `pointee-rhs`, `meta-rhs`, `pin-partial-eq`, `struct-where` and
  `relaxed-meta` fail with "trait argument does not satisfy its declaration
  requirements";
- `flatten-next`, `inherent-item` and `free-item` fail with "nominal type
  argument must implement its declared trait";
- `pointee-item` and `sized-item-meta` already pass. They guard the item-bound
  and declared `Sized` paths that do not need the repair.

The opponents are rejected on the base too. `pointee-item-meta` was rejected
there with "invalid canonical trait selection", and is now rejected with the
formation error.

`dev/tests/invariants/sizedness_hierarchy.rs` checks every case, attached to
`sema/nominals/formation.rs`. Its filename stem `sizedness` now has 3 files.

## Checks

Every check used the sources of `97110f230`, docs aside, which are the pin's
source fingerprint. The two code commits were checked as this final tree only,
not one by one.

| Check | Result | Receipt |
|---|---|---|
| Private: 17 lib cases, strict private Clippy, format, architecture | pass | `obj/qualification/fast/1794103333665476729` |
| `census_mode` and PA1 to PA12 | pass, 9 tests, every PA full | `obj/qualification/fast/1794103750846698891` |
| PA13 to PA19 and PA21 to PA24 | pass, every PA full | `obj/qualification/fast/1794103928803786635` |
| PA20 alone | pass, 1451/1451 | `obj/qualification/fast/1794104005018980429` |
| `make test-upstream` | pass, 127 fixtures, 126 Reference files | `obj/parallel/census-batch8/tmp/test-upstream.log` |
| rustc 1.98.0 oracle | 13/13 agree | `obj/parallel/census-batch8/oracle/` |
| Pin build and ordinary observation | pass | `obj/parallel/census-batch8/qualified-1/receipt-build.json` |
| Census 9 | completed, analysis-only | `obj/parallel/census-batch8/qualified-1/receipt-census.json` |

The 17 private lib cases are:

- the new invariant and the updated
  `reached_intrinsics_cannot_execute_their_authored_fallback`;
- the stability transport hot-record and derive-owner layout invariants;
- the source sizing invariants: the hidden sized projection's lifetime debt,
  the two size wait cases, the three completed sizing proof reuse cases and
  the two sizing default cases;
- batch 7's five invariants: `assume`, declaration lifetime contracts,
  fallback exposure, quantified Drop coverage and const Drop pattern fields.

No record changed layout. An earlier private run failed formatting only
(`obj/qualification/fast/1794103231555012287`); the layout of one assertion
was fixed. `make test-harness` was not needed, because no integration root was
added.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness and course text.

## Storage

Only the census 9 pin is kept. The census 8 pin
(`obj/parallel/census-batch7/qualified-1/pin`), the d1 and d2 diagnostic pins
and the analysis-only builds in `obj/parallel/census-batch8/dev/` were retired
and pruned; their hashes stay in their receipts. `obj` reached 14 GiB during
the checks, so `make tidy-build` ran once, before the pin build. `obj` is
7.3 GiB after `make tidy`.

## Shared files touched

- `sema/traits/selection/search/inputs.rs` and
  `sema/traits/selection/search/sizing_fallback.rs` (section 1). These are the
  batch's only `sema::traits` changes.
- `sema/nominals/formation.rs`: the test attachment only.
- `dev/tests/invariants/prototype_headers.rs` (section 3).

## Decisions needed

- Section 2: Lane B's ownership of `option.rs:1386`, a `[const]` bound with a
  reference lifetime in a nominal argument. It is the next crate-wide
  declaration record.
- Section 4: whether to fund keep-going across the crate-wide declaration
  passes, which needs a dependency audit of every pass.
