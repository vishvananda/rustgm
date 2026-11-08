# Census batch 7: assume in constant execution, declaration-time lifetime contracts, intrinsic fallback exposure, quantified Drop bounds, pattern-type Destruct and the eighth census

Census 7 ([batch 6](census-batch6.md), pin `a650f86d`) checked 42,280
declarations of original core: 5 failed and 25 blocked. Lane B's cause
accounted for 4 failures and `intrinsics::assume` for 1 failure and 23 blocked
records. This batch:

- gives `intrinsics::assume` a validated constant-evaluation handler
  (section 1);
- rebases onto main `519f8eda3`, where Lane B's last four core sites have
  landed. The per-definition declaration loop then has no failure, and the
  later crate-wide declaration passes run for the first time. Each pass that
  fails ends the census with one crate record;
- repairs the first four crate records in turn (sections 2 to 5);
- names the fifth without repairing it (section 6) and records three gaps it
  did not change (section 7);
- runs census 8 (section 8): 42,310 declarations checked, none failed or
  blocked, and one crate record at `pin.rs:1105`.

Milestone 2 ("original core passes declaration analysis") is **not** reached:
the per-definition loop has no failure, but a later crate-wide declaration pass
still fails, so bodies are not entered.

Out of scope:

- Lane B owns the trait conformance mapping (`sema/traits/conformance/mapping.rs`);
  it is not touched.
- Lane C owns cross-crate transport.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend is frozen.
- `!null` pattern types over integer or reference bases stay rejected by name
  (the integrator's recorded deviation).

## Diagnosis method

Each diagnostic was an analysis-only pin of the working tree with one census of
unchanged original core inputs. They are in
`obj/parallel/census-batch7/diagnostics/dN/`; their binaries are retired.

| Run | Base | Checked | Failed | Blocked | End |
|---|---|---|---|---|---|
| Census 7 (`a650f86d`) | `0311a54ea` | 42,280 | 5 | 25 | declaration loop stopped |
| d1, section 1 | `efa273f4f` | 42,304 | 4 | 2 | declaration loop stopped; only Lane B's cause |
| d2, section 1 | `519f8eda3` | 42,310 | 0 | 0 | crate record: invalid lifetime constraint identity (`panic.rs`, from `pat.rs`) |
| d3, d2 instrumented | `519f8eda3` | 42,310 | 0 | 0 | the same, at the variance length check |
| d4, sections 1 and 2 | `519f8eda3` | 42,310 | 0 | 0 | crate record: const-stable body cannot expose this unstable intrinsic (`intrinsics/mod.rs:460`) |
| d5, sections 1 to 3 | `519f8eda3` | 42,310 | 0 | 0 | crate record: quantified clause requires a scoped consumer (`mem/drop_guard.rs:136`) |
| d6, sections 1 to 4 | `519f8eda3` | 42,310 | 0 | 0 | crate record: const Drop declaration field requires const Destruct (`array/drain.rs:103`) |
| d7, sections 1 to 5 | `519f8eda3` | 42,310 | 0 | 0 | crate record: trait argument does not satisfy its declaration requirements (`pin.rs:1105`) |

d1 drops `assume`'s 1 failure and 23 blocked records and exposes nothing new.
d5's receipt records a fingerprint assertion after the census: two test
attachments were added while it ran. Its census itself ran on the built pin.

## 1. `intrinsics::assume` in constant execution

`hint::assert_unchecked` calls `crate::intrinsics::assume(cond)`. The `ilog10`
lengths of `fmt/num_buffer.rs` reach it during declaration checking. The repair
follows the source bit counts of batch 5, as the plan decided:

- **Selection.** `sema/compiler_items/assume_contract.rs` selects only the
  current source provider's attributed `assume` declaration. The completed
  function must be the declaration, with no generic arguments and the exact
  `unsafe fn(bool)` Rust signature. The provider report, the intrinsic
  identity, the current header origin and its const declaration are checked
  again before use.
- **Execution.** `const_eval/assume_calls.rs` validates the reached caller,
  the actual call arguments against the selected signature and the const call.
  A true condition returns unit. A false one is
  `Failure::Fault("`assume` was executed with a false condition")`, the analog
  of rustc's "`assume` called with `false`" (E0080).
- `executed_calls.rs` excludes `assume` from the generic source-selector
  rejection and dispatches to the handler after the bit counts.

The fallback body is never entered, and
`reached_intrinsics_cannot_execute_their_authored_fallback` still passes.

## 2. Lifetime contracts in declaration-time bodies

On the rebased tree, `complete_nominals` failed with "invalid lifetime
constraint identity". The pattern macros of `pat.rs` reach `panic!`, whose
`const_format_args!` relates `fmt::Arguments<'a>` instances in a const fn body
that a declaration-time constant executes. Crate-wide variances are computed
only after the Fields pass, so that body found no variance row for
`Arguments`, and `relate_arguments` failed its length check (d3). The same body
then lacked the nominal's outlives contract.

Body finalization under declaration availability now demands both, through
`lifetime_contract_dependency` in `sema/bodies/field_storage.rs`:

- `Variances::unreached_nominals` (`types/variance/reachable.rs`) walks the
  solved body types once each and collects generic nominals that have no row
  and whose instance carries a free lifetime. A relation skips identical
  binder-free types, so region-free instances, such as batch 6's `NonZero`
  fields, are never demanded.
- Each is passed to the existing reached variance inference. A new
  `append_reached_or_pending` returns the closure nominal whose field template
  is not published yet, instead of failing, and retains nothing of that
  closure. The body then waits with `Requirement::Nominal`, the same field wait
  batch 5 introduced, guarded by the same once-per-nominal record. A second
  wait on the same nominal is an error. `append_reached` keeps its message.
- Each nominal's reached outlives contract is then demanded with
  `Lifetimes::demand_reached_outlives` (`sema/lifetimes.rs`). This is batch 6's
  scalar-readiness demand, moved so that both callers share one function.

Complete inference later republishes the same declared rows. No record changes
layout.

The demand runs only in a source library, as batch 6's scalar-readiness demand
and the site records do. An ordinary crate has the same failure:
`obj/parallel/census-batch7/repro/o/ordinary-length.rs` fails with "invalid
lifetime constraint identity" on base `519f8eda3`, and the unscoped demand
accepts it. But the outlives demand interns field types earlier, which
renumbers the type tables of 11 PA20 `--emit-trait-semantics` references
(`member-lifetimes`, `definition-readiness` and `lookup-readiness`). Each
differs only in numbering, with the same line count and structure. References
are not regenerated here, so ordinary crates keep their existing order and the
gap is recorded (section 7).

## 3. Intrinsic fallback exposure

Core's `intrinsics::unlikely` is an unmarked `const fn` with only regular
`#[unstable]`, so it is implicitly const-stable-indirect and its body is
checked recursively. It calls `cold_path()`, an intrinsic with an empty
fallback and no `rustc_intrinsic_const_stable_indirect`. RustGM rejected the
call. rustc 1.98 also asks whether the intrinsic has a fallback body that is
itself safe to expose, which this one is.

`sema/bodies/api_stability.rs` now accepts that case:

- `safe_on_stable` is the existing "recursively checked" query (const-stable,
  implicitly inherited, or `rustc_const_stable_indirect`), now shared by its
  three callers.
- `exposed_fallback` accepts an intrinsic whose completed function has a source
  body that passes `safe_on_stable`.

A bodiless intrinsic and an explicitly `rustc_const_unstable` one are still
rejected, as rustc rejects them.

## 4. Quantified Drop bounds

Core's `const impl<T, F> Drop for DropGuard<T, F> where F: [const] FnOnce(T)`
failed with "quantified clause requires a scoped consumer". Paren-sugar bounds
are kept in a binder scope, and two Drop consumers read only flat predicates:

- **Coverage** (`sema/declarations/destructors/coverage.rs`). A quantified
  requirement is substituted and proved inside its scope with the existing
  `Proof::prove_clause`. A const impl's `[const]` bound becomes the ordinary
  bound inside the same binder. Outlives bodies stay excluded, and other
  non-predicate clauses keep their message.
- **Region coverage** (`sema/destructors.rs`). A quantified trait bound adds
  no lifetime requirement and is skipped. A quantified outlives body is still
  rejected by name.

Added, stronger and added higher-ranked bounds are still rejected, as rustc's
E0367 does.

## 5. Pattern types are `[const] Destruct` as their base

Core's `const impl<T: [const] Destruct, F> Drop for array::Drain` has a
`NonNull<T>` field, which holds `pattern_type!(*const T is !null)`. Const
Destruct admitted no pattern type. A pattern type now destroys exactly its base
value:

- `destruct_predicate` (`sema/compiler_items/roles/destructibility.rs`) admits
  `Kind::Pattern` for an effectful Destruct goal;
- `const_destruct_components` (`sema/traits/selection/search/cparts.rs`) proves
  the base, as it does an array's element.

rustc 1.98 also requires `T: [const] Destruct` for a pattern over a type
parameter base; the opponent confirms it.

## 6. The new frontier: projection sizedness in trait arguments

d7 ends at `pin.rs:1105`:

```rust
impl<Ptr: Deref, Q: Deref> PartialEq<Pin<Q>> for Pin<Ptr>
where
    Ptr::Target: PartialEq<Q::Target>,
```

`PartialEq<Rhs: PointeeSized = Self>` requires `<Q as Deref>::Target:
PointeeSized`, and `Deref::Target` is declared `?Sized`, which is implicitly
`MetaSized`. Formation (`sema/nominals/formation.rs`) cannot prove either
sizing bound for a rigid projection, so it fails with "trait argument does not
satisfy its declaration requirements". The drafts in
`obj/parallel/census-batch7/repro/p/` reduce it: `P::Target:
PartialEq<Q::Target>` and `P::Target: Takes<Q::Target>` with `Rhs: MetaSized`
both fail here and pass in rustc. With `Rhs: Sized`, rustc and this analysis
both reject, as they should.

The source sizing fallback is never reached for that goal. Selection decides
it earlier from the alias's item bounds, which do not carry the implicit
`MetaSized` of a `?Sized` associated type. This is shared trait selection and
projection work, so it is not changed here.

## 7. Found and not changed

- **`later-projected-copy-wait`.** It still fails with "missing completed
  associated type definition". The failing call is
  `normalize_known_projection` in `sema/bodies/projections/known.rs`. It
  normalizes the pattern field type synchronously and returns a type, so it
  has no channel for a dependency. A wait needs it to go through the
  suspendable projection continuation that unknown projections use.
- **`proj-c`.** It still fails with "constant storage layout requires
  completed fields"; batch 6's analysis stands.
- **`residual-try-type-mismatch`.** It still reports "associated type
  projection requires normalization". It is not cheap.
- **Declaration-time lifetime contracts in ordinary crates** (section 2). The
  repair is scoped to source libraries; extending it renumbers 11 PA20 type
  tables.
- **`reached_intrinsics_cannot_execute_their_authored_fallback` fails on its
  `const` case, on base `519f8eda3` as well.** Batch 5's source bit-count
  handler (`65a9eae6c`) executes `ctpop` in constant evaluation, so
  `pa28/tests/spec/intrinsic-declarations/execution/const.rs` is now accepted.
  The test still expects "not implemented". Its fallback is still not
  executed: with a length of `ctpop(7u32)`, `[u8; 3]` is accepted and
  `[u8; 7]` is rejected, on the base and on this batch
  (`obj/parallel/census-batch7/repro/f/`). The `runtime` and `reified` cases
  still fail as expected. The test is not changed here.

## 8. Census 8

The pin `f99b369d` (`obj/parallel/census-batch7/qualified-1/pin/rustgm`) was
built from `9f04f8694`, which is base `519f8eda3` plus this batch's four code
commits. No dev or scripts source was uncommitted: the build's retained patch
is empty.

- Its manifest is census 7's, with only `expected_cli_sha256` and
  `predecessor_manifest_sha256` changed.
- No configuration source pin changed, and nothing was rebound.
- The ordinary first error moved from `ops/control_flow.rs:14309` (Lane B's
  cause) to `pin.rs:1105` (section 6).

The census is analysis-only and ran over unchanged original core inputs: 375
opens in 50 s. Its records are in
`rustgm-pa29-work/obj/parallel/core-census-8/` (`census.tsv` SHA-256
`085dbeed9416ec4e`).

| Census | Checked | Failed | Blocked | End |
|---|---|---|---|---|
| Census 7 (`a650f86d`) | 42,280 | 5 | 25 | declaration loop stopped |
| Census 8 (`f99b369d`) | 42,310 | 0 | 0 | crate record: trait argument does not satisfy its declaration requirements (`pin.rs:1105`) |

The per-definition declaration loop has no failure for the first time. Checked
declarations grew by 30. A later crate-wide declaration pass still fails, so
bodies are not entered: 0 bodies are checked, and there are no body causes and
no checked-of-total body count.

### Failures by cause

"Blocked" counts every record whose blocking chain ends at a failure with that
message.

| Cause | Census 7 failed / blocked | Census 8 failed / blocked | Sites |
|---|---|---|---|
| associated type requires an unambiguous implementation (Lane B) | 4 / 2 | 0 / 0 | repaired by Lane B (`519f8eda3`) |
| source intrinsic constant execution is not implemented | 1 / 23 | 0 / 0 | repaired (section 1) |
| trait argument does not satisfy its declaration requirements | 0 / 0 | crate record | `pin.rs:1105` (section 6) |

Sections 2 to 5 repaired crate records that census 7 never reached, so they
have no census 7 counts. d2, d4, d5 and d6 show each of them.

### Largest blocking roots

None. The census has no per-definition failure, so no record is blocked. The
one crate record ends the census; it blocks every later crate-wide declaration
pass and all bodies, which the census does not count.

## Controls

These cases were declared (`obj/parallel/census-batch7/oracle/*.declared.json`)
before the frozen rustc 1.98.0 oracle ran on the exact bytes, and all agree.
Both are recorded in each directory's `outcomes.json`.

| Directory | Cases | Accept | Reject | rustc |
|---|---|---|---|---|
| `assume-constant-execution` | 7 | 4 | 3 (E0308, E0080, E0080) | 7/7 agree |
| `declaration-lifetime-contracts` | 8 | 5 | 3 (lifetime errors) | 8/8 agree, after one amendment |
| `intrinsic-fallback-exposure` | 5 | 3 | 2 (exposure errors) | 5/5 agree |
| `quantified-drop-bounds` | 7 | 3 | 4 (E0367, E0271) | 7/7 agree |
| `pattern-const-destruct` | 6 | 5 | 1 (E0367) | 6/6 agree |

- **Amendment.** The first `declaration-lifetime-contracts` bytes returned
  `Note<'a>` and `Slot<'a>` by value from const fns. rustc rejected all three
  such shapes for the missing `freeze` lang item of the no_core prefix, not for
  the stated rule. The shapes now bind the related nominal to an annotated
  local. The first declaration and its oracle are retained as `*.v1.*`.
- `quantified-drop-bounds` and `pattern-const-destruct` were explored with
  rustc before their declaration, which says so. They are regression records
  of that exploration.

Every new positive control fails on base `519f8eda3` (analysis-only build
`obj/parallel/census-batch7/dev/base1`, `f5fa9068`), and passes here:

- `declaration-lifetime-contracts`: every case fails with "invalid lifetime
  constraint identity";
- `intrinsic-fallback-exposure`: `cold-hint-length` and `cold-hint-body` fail
  with "const-stable body cannot expose this unstable intrinsic";
- `quantified-drop-bounds`: `plain-guard`, `core-const-guard` and
  `higher-ranked-guard` fail with "quantified clause requires a scoped
  consumer";
- `pattern-const-destruct`: every case fails with "const Drop declaration
  field requires const Destruct in its actual impl environment";
- `assume-constant-execution`: d1 drops the census cause.

Private invariants check every case:

- `dev/tests/invariants/assume_constants.rs`;
- `dev/tests/invariants/lifetime_contract_constants.rs`;
- `dev/tests/invariants/fallback_exposure.rs`;
- `dev/tests/invariants/binder_drop_bounds.rs`;
- `dev/tests/invariants/destruct_patterns.rs`.

## Checks

Every check used the sources of `9f04f8694`, docs aside, which are the pin's
source fingerprint. The three code commits after `9d38ebb10` were checked as
this final tree only, not one by one.

| Check | Result | Receipt |
|---|---|---|
| Private: 17 lib cases, strict private Clippy, format, architecture | pass | `obj/qualification/fast/1794099861469903615` |
| `census_mode` and PA1 to PA12 | pass, 9 tests, every PA full | `obj/qualification/fast/1794099956457222391` |
| PA13 to PA19 and PA21 to PA24 | pass, every PA full | `obj/qualification/fast/1794099998524243440` |
| PA20 alone | pass, 1451/1451 | `obj/qualification/fast/1794099739405609722` |
| `make test-upstream` | pass, 127 fixtures, 126 Reference files | `obj/parallel/census-batch7/tmp/test-upstream.log` |
| rustc 1.98.0 oracle | 33/33 agree | `obj/parallel/census-batch7/oracle/` |
| Pin build and ordinary observation | pass | `obj/parallel/census-batch7/qualified-1/receipt-build.json` |
| Census 8 | completed, analysis-only | `obj/parallel/census-batch7/qualified-1/receipt-census.json` |

The 17 private lib cases are:

- the five new invariants (section "Controls");
- declaration execution of bit counts, discriminants and later nominals;
- the generic Copy wait;
- generic non-null patterns;
- the five reached variance invariants: cycles and failed payment, shared wide
  dependencies, paid scratch growth, zero scratch budgets and the wide
  closure;
- the const fn allowance and glue const-policy controls;
- the stability transport hot-record and derive-owner layout invariants.

No record changed layout. `reached_intrinsics_cannot_execute_their_authored_fallback`
is not in the selection: it fails on base `519f8eda3` as well (section 7).
`make test-harness` was not needed, because no integration root was added.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness and course text.

## Storage

Only the census 8 pin is kept. The census 6 pin
(`obj/parallel/census-batch5/qualified-1/pin`), the census 7 pin
(`obj/parallel/census-batch6/qualified-1/pin`), the d1 to d7 diagnostic pins
and the analysis-only builds in `obj/parallel/census-batch7/dev/` were retired
and pruned; their hashes stay in their receipts. `obj` is 12 GiB after
`make tidy`.

## Shared files touched

- `sema/traits/selection/search/cparts.rs`: the pattern-type arm (section 5),
  two added lines and one changed. This is the batch's only `sema::traits` change.
- `sema/declarations/traits/implementations/scalar_readiness.rs`: batch 6's
  outlives demand now calls the shared `Lifetimes::demand_reached_outlives`,
  with the same cached early return.
- `sema/lifetimes.rs`: the new shared `Lifetimes::demand_reached_outlives`
  (32 lines), moved from scalar readiness.

## Decisions needed

- Section 6: who owns sizing bounds of rigid projections in trait selection
  (the implicit `MetaSized` of a `?Sized` associated type, and `PointeeSized`
  for every type). It is the next crate-wide declaration record.
- Each crate-wide declaration pass ends the census at its first failure. A
  keep-going mode for those passes would list the remaining crate records at
  once instead of one per census.
- Section 7: whether `reached_intrinsics_cannot_execute_their_authored_fallback`
  should expect its `const` case to be accepted now that batch 5 executes
  `ctpop`, and who changes it.
- Section 2: whether ordinary crates should also demand lifetime contracts
  early, which needs the 11 renumbered PA20 references regenerated and
  reviewed as a new contract, or keep the recorded gap.
