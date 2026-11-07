# Census batch 6: generic Copy waits in declaration-time constants, generic non-null pattern types and the seventh census

Census 6 ([batch 5](census-batch5.md), pin `1c0cac9f`) checked 42,226
declarations of original core: 32 failed and 52 blocked. Lane B's cause
accounted for 28 failures. This batch:

- repairs "canonical trait selection requires pending declarations" (3 failed,
  26 blocked) with the chosen option (a), in section 1;
- accepts core's generic `!null` pattern types (`pat.rs`, 1 failed), in section 2;
- records three conformance gaps it found and did not change (section 3);
- names the new frontier, `intrinsics::assume`, without repairing it (section 4);
- runs census 7: 42,280 checked, 5 failed and 25 blocked.

Out of scope:

- Lane B owns "associated type requires an unambiguous implementation" and
  `sema/traits/conformance/mapping.rs::contract`; neither is touched.
- Lane C owns cross-crate transport.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend is frozen.

The branch is rebased onto main `0311a54ea`, which joins Lane B's
specialization and no_std batch and Lane C's lifetime nominals.

## Diagnosis method

Each diagnostic was an analysis-only pin of the working tree with one census of
unchanged original core inputs. They are in
`obj/parallel/census-batch6/diagnostics/dN/`; their binaries are retired.

| Run | Base | Checked | Failed | Blocked | Causes gone (newly exposed) |
|---|---|---|---|---|---|
| Census 6 (`1c0cac9f`) | `0681610fc` | 42,226 | 32 | 52 | |
| d1, section 1 without its outlives demand | `933e401c1` | 42,226 | 31 | 54 | pending selection (projected nominal outlives, source intrinsic) |
| d2, section 1 | `933e401c1` | 42,231 | 30 | 49 | projected nominal outlives |
| d3, sections 1 and 2 | `0311a54ea` | 42,280 | 5 | 25 | non-null pattern; Lane B falls from 28 to 4 |

Census 7 has the same records as d3.

## 1. "Canonical trait selection requires pending declarations"

This cause had 3 failures and 26 blocked records. The roots were two `Buf`
aliases and the function `prepare`. Each array length runs `ilog10` during
declaration checking. That const fn copies `Option<T>` and `NonZero<T>` values,
so constant execution needs a Copy decision. That decision selected the derived
`impl<T: Copy> Copy for Option<T>` or `NonZero`'s `Copy`, whose validation was
still queued. Early scalar validation admitted only a type-parameter subject,
so nothing validated those impls first, and the pending selection became a
failure.

The repair follows the decision recorded in the
[census plan](../core-census-plan.md). Copy decisions are never answered from
declaration-level availability.

### 1.1 Early validation of own-parameter nominal impls

`eligibility_profile.rs` admits an impl whose subject is either:

- one of its own type parameters, or
- a nominal that applies exactly its own type parameters, in order.

In both cases every impl parameter must be an indexed own type parameter that
has abilities in the impl's environment. Lifetime and const parameters stay
outside the profile. The impl's requirement on its own subject, such as Copy's
`Self: Clone` supertrait, is admitted beside primitive and own-parameter
requirements.

For a nominal subject, `scalar_readiness.rs` does two things:

- At `Begin`, it queues the subject's field template before its signatures.
- At completion, it demands only that subject's reached nominal outlives
  contracts, through the existing `Bounds::append_reached`, and charges source
  payments. The complete inference later republishes them.

d1 showed that `NonZero`'s field `T::NonZeroInner` then failed with
"demanded projected nominal outlives contracts require completion". A
region-free projected field records no projection pair, so the demanded
inference (`types/outlives/contracts.rs`) now refuses a projected field only
when it may contain regions. A pair that a nominal use instantiates is still
refused when it is enqueued.

### 1.2 A pending Copy decision becomes a wait

1. **Selection.** `select_requirement_waiting` in `sema/nominals/traits.rs`
   returns the `selection::Dependency` that a pending selection names, not its
   message. An impl that is early-unavailable still fails as "implementation
   validation is outside the early scalar profile".
2. **Recording.** `copy_decisions_waiting` serves the non-service path of
   `body/copy_requirements.rs`. It records the dependency in
   `Declarations::pending_copy_selection` and returns the same message as
   before. Every consumer that cannot wait therefore reports the same failure
   as before.
3. **Engine.** The constant execution engine (`pipeline/execution.rs`) clears
   the record before each requirement preparation and each root check. A step
   whose error is exactly that decision's pending message becomes
   `Requirement::Selection(SelectionWait)`, and the session is retained with
   its own requirement. Any other error keeps its cause. The eligibility check
   (`Proxy::check`) treats a waiting root as eligible, and its execution
   request checks the root again.
4. **Worklist.** `constant_dependency` pushes `Work::selection_dependency`,
   the same work item that a pending header selection uses. Cycle detection for
   these edges therefore comes from the existing header dependencies, and the
   `validation-cycle` opponent reports "cycle in implementation header
   dependencies". If the awaited fact is already published and the constant
   still does not resume, it fails with "missing selected declaration fact", so
   a wait never repeats.

`SelectionWait` wraps the crate-private `Dependency` so that the public
`Requirement` exposes no private type.

### 1.3 Layout

- `Declarations` grows by 8 bytes, from 7,136 to 7,144. It is once per crate,
  and `prototype_headers.rs` records the size.
- The execution `Requirement` keeps its size, because `AssociatedBody` stays
  its largest variant.
- `selection::Dependency` gains a `Hash` derive. This is the batch's only
  `sema::traits` change.

## 2. Generic non-null pattern types

Census 6's ordinary first error was `pat.rs:2692`:

```rust
impl<T: DispatchFromDyn<U>, U> DispatchFromDyn<pattern_type!(U is !null)> for pattern_type!(T is !null) {}
```

Its type-parameter base was rejected with "non-pointer non-null patterns
require their wider profile". Three changes repair it:

- **Annotation.** `refined_annotations.rs` accepts a `!null` base that is a
  raw pointer or a type parameter.
- **Canonical validity.** `types/refinements.rs` accepts a parameter base.
- **Substitution.** In `types/mod.rs`, an instance whose base is anything other
  than a raw pointer fails as "non-null pattern instance requires a raw pointer
  base".

Every other written base fails as "non-null pattern base must be a raw pointer
or type parameter". rustc 1.98.0 accepts an integer base, a reference base and
an integer instance. These three are intentionally stricter here, because only
a raw pointer has a defined non-null representation in this analysis.

Pattern-type `DispatchFromDyn` coherence already existed in `dispatches.rs`.
Two pattern types relate only under the same constraint, and a changed field
proves its relation through the generic pattern impl. Once the base was
admitted, nothing more was needed there.

## Controls

These cases were declared (`obj/parallel/census-batch6/oracle/*.declared.json`)
before the frozen rustc 1.98.0 oracle ran on the exact bytes, and all 17 agree.
Both are recorded in each directory's `outcomes.json`. The cases are no_core
source-core libraries, built from the census batch 2 lang-item prefix.

| Control | rustc | Census 6 pin | Census 7 pin |
|---|---|---|---|
| `option-copy-wait`, `root-copy-wait`, `nonzero-copy-wait` | accept | pending declarations | accept |
| `copy-length-mismatch` | E0308 | pending declarations | incompatible types |
| `not-copy-move` (non-Copy opponent) | E0382 | pending declarations | moved read |
| `field-not-copy` | E0204 | pending declarations | every field must implement Copy |
| `validation-cycle` (cycle opponent) | E0391 | pending declarations | header dependency cycle |
| `later-projected-copy-wait` (gap) | accept | missing associated type definition | missing associated type definition |
| `core-pattern-dispatch`, `unbounded-pattern-dispatch`, `generic-pattern-field`, `non-null-pointer-dispatch` | accept | wider profile | accept |
| `missing-pattern-dispatch` | E0277 | changed field relation | changed field relation |
| `pattern-to-pointer-dispatch` | E0377 | wider profile | must preserve pointer or owner |
| `integer-base`, `reference-base` (stricter) | accept | wider profile | base must be a raw pointer or type parameter |
| `integer-instance` (stricter) | accept | wider profile | instance requires a raw pointer base |

`root-copy-wait` puts the waiting copy in the anonymous length block itself.
That exercises the root check, the eligibility check and dispatch preparation,
not only a called const fn. `missing-pattern-dispatch` already failed for its
own reason on the census 6 pin and remains a regression control.

Private invariants check every case:

- `dev/tests/invariants/waited_copy_selections.rs`;
- `dev/tests/invariants/nonnull_pattern_bases.rs`.

## 3. Found and not changed

- **`later-projected-copy-wait`.** The const fn destructures the projected
  field of a `NonZero`-shaped wrapper that is declared after it. rustc accepts
  this. Both pins reject it with "missing completed associated type
  definition": that projection completion path in body checking cannot
  suspend.
- **`proj-c`.** `size_of::<Wrap<u8>>()` with `struct Wrap<T: Zeroable>(T::Inner)`
  (`obj/parallel/census-batch5/repro/controls/proj-c.rs`) is still rejected
  with "constant storage layout requires completed fields". A repair needs two
  things:
  - publishing normalized stored fields for a closed projected nominal at a
    declaration-time layout query;
  - a validation wait for the projecting impl.

  Two attempts were reverted. Preparing storage for intrinsic arguments in
  `body/instantiate/projections.rs` changed nothing. Normalizing storage in
  `layout_kernel` reached "associated type requires an unambiguous validated
  implementation".
- **`residual-try-type-mismatch`.** rustc reports E0308. RustGM still rejects
  it, but with "associated type projection requires normalization", not as
  incompatible types. That is not a cheap change.
- `pa31/tests/spec/static-constant-modules/README.md` now names
  `binderless_constant_modules.rs`.

## 4. The new frontier: `intrinsics::assume`

Section 1 lets the `NumBufferTrait` lengths in `fmt/num_buffer.rs`
(`$signed::MAX.ilog10() as usize + 2`) run further. `ilog10` now reaches
`hint::assert_unchecked` (`hint.rs:202`), which calls `crate::intrinsics::assume(cond)`.

`assume` is a declaration intrinsic with an authored fallback body. Two things
stop it:

- `source_selector_call` (`const_eval/executed_calls.rs`) rejects every
  declaration intrinsic that has no dedicated handler, with "source intrinsic
  constant execution is not implemented".
- `reached_intrinsics_cannot_execute_their_authored_fallback` forbids running
  the fallback.

One alias `Buf` fails, and it blocks the other 23 `DEFAULT` and `Buf` items of
that file.

A repair would follow the source bit counts of batch 5:

- a validated selection of the exact `unsafe const fn(bool)` declaration;
- a CTFE handler that returns unit for `true` and faults for `false`, as
  rustc's "`assume` called with `false`" (E0080) does;
- its exclusion in `executed_calls.rs`;
- declared controls.

It is not implemented here, because the alternative is to let selected
intrinsics run their fallback bodies, which needs a decision.

## 5. Census 7

The pin `a650f86d` (`obj/parallel/census-batch6/qualified-1/pin/rustgm`) was
built from `38edd1bf3` with batch 4's flags. No sources were uncommitted.

- Its manifest is census 6's, with only `expected_cli_sha256` and
  `predecessor_manifest_sha256` changed.
- No configuration source pin changed, and nothing was rebound.
- The ordinary first error moved from `pat.rs:2692` to `ops/control_flow.rs:14309`,
  which is Lane B's cause.

The census is analysis-only and ran over unchanged original core inputs: 375
opens in 48 s. Its records are in
`rustgm-pa29-work/obj/parallel/core-census-7/`.

| Census | Checked | Failed | Blocked | End |
|---|---|---|---|---|
| Census 6 (`1c0cac9f`) | 42,226 | 32 | 52 | declaration loop complete |
| Census 7 (`a650f86d`) | 42,280 | 5 | 25 | declaration loop complete |

Declarations do not reach zero, so bodies are still not entered and there are
no body causes. Checked declarations grew by 54.

### Failures by cause

"Blocked" counts every record whose blocking chain ends at a failure with that
message.

| Cause | Census 6 failed / blocked | Census 7 failed / blocked | Sites |
|---|---|---|---|
| associated type requires an unambiguous implementation (Lane B) | 28 / 26 | 4 / 2 | `ops/control_flow.rs`, `iter/adapters/map_windows.rs`, `iter/adapters/scan.rs`, `iter/mod.rs` |
| source intrinsic constant execution is not implemented | 0 / 0 | 1 / 23 | `hint.rs` (section 4) |
| canonical trait selection requires pending declarations | 3 / 26 | 0 / 0 | repaired (section 1) |
| non-pointer non-null patterns require their wider profile | 1 / 0 | 0 / 0 | repaired (section 2) |

Lane B's own fixes, which are in this base, take its cause from 28 to 4.

### Largest blocking roots

| Blocked | Root | Cause |
|---|---|---|
| 23 | alias `Buf`, reported at `hint.rs:7986` | source intrinsic constant execution is not implemented |
| 2 | self-type `Self`, `ops/control_flow.rs:14309` | associated type requires an unambiguous implementation |

Census 6's two `Buf` roots in `int_log10.rs` and `uint_macros.rs`, and its
`prepare` root, no longer fail there.

## Checks

Every check used the sources of `38edd1bf3`, docs aside, which are the pin's
source fingerprint.

| Check | Result | Receipt |
|---|---|---|
| Private: 23 lib cases, strict private Clippy, format, architecture | pass | `obj/qualification/fast/1794091563742874115` |
| `census_mode` and PA1 to PA12 | pass, 9 tests, every PA full | `obj/qualification/fast/1794091876528451045` |
| PA13 to PA19 and PA21 to PA24 | pass, every PA full | `obj/qualification/fast/1794092105066292056` |
| PA20 alone | pass, 1451/1451 | `obj/qualification/fast/1794092183311705669` |
| `make test-upstream` | pass, 127 fixtures, 126 Reference files | `obj/parallel/census-batch6/tmp/test-upstream.log` |
| rustc 1.98.0 oracle | 17/17 agree | `obj/parallel/census-batch6/oracle/` |
| Pin build and ordinary observation | pass | `obj/parallel/census-batch6/qualified-1/receipt-build.json` |
| Census 7 | completed, analysis-only | `obj/parallel/census-batch6/qualified-1/receipt-census.json` |

The 23 private lib cases are:

- the two new invariants;
- the stability transport and derive-owner layout invariants;
- the `Declarations`, `Request` and source Copy record layouts;
- checked and waited queries;
- declaration execution;
- early conformance and unavailable blanket controls;
- constrained scalars;
- binderless modules;
- the Copy carrier controls.

`make test-harness` was not needed, because no integration root was added.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness and course text.

## Decisions needed

- Section 4: whether `intrinsics::assume` gets a dedicated validated CTFE
  handler, as bit counts do, and whether it belongs to this lane. It is now the
  only declaration cause outside Lane B's, and it blocks 23 records.
- Whether this analysis should keep rejecting integer and reference `!null`
  bases, which rustc 1.98 accepts but core does not use.
