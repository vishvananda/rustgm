# Census batch 5: declaration-phase constants, rigid projection equalities, abandoned census work, signature constant privacy and the sixth census

Census 5 ([batch 4](census-batch4.md), pin `5cb7e00c`) checked 42,194
declarations of original core: 62 failed and 55 blocked. This batch:

- repairs six of census 5's causes, and two more that those repairs exposed;
- diagnoses the remaining two without repairing them (sections 2 and 5.1);
- adopts rustc 1.98's type-privacy rule for signature anonymous constants;
- runs census 6: 42,226 checked, 32 failed and 52 blocked.

Out of scope:

- Lane B owns "associated type requires an unambiguous implementation", the
  import-lineage ledger and the expansion `Definition` layout; none is touched.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend is frozen.

The branch is rebased onto main `e421547fa`.

## Diagnosis method

The diagnostics were analysis-only patches and were never committed:
`obj/parallel/census-batch5/apply_diag3.py`, `apply_diag4.py` and
`diag4.patch`. They kept a bounded ring of backtraces and details at each
cause's error site, plus the last 24 dispatched declaration work items. When
census isolation recorded a failure, they printed the ring with type and item
identities expanded. Each pin and census is in
`obj/parallel/census-batch5/diag/diagN/`, and the inputs were unchanged
original core.

| Run | Checked | Failed | Blocked | Causes gone (newly exposed) |
|---|---|---|---|---|
| Census 5 (`5cb7e00c`) | 42,194 | 62 | 55 | |
| diag3, diag4 (instrumentation only) | 42,194 | 62 | 55 | none |
| diag5 | 42,223 | 34 | 53 | invalid canonical and equality selection, nominal requirements (missing aggregate field template) |
| diag6, diag7 | 42,226 | 34 | 50 | source intrinsic, enum discriminants (const body cycle) |
| diag8 | 42,226 | 33 | 51 | later semantic assignments |
| diag9 | 42,226 | 32 | 52 | const body cycle |
| diag10 | 42,226 | 32 | 52 | missing aggregate field template |

From diag5 on, the base includes Lane B's callable-sugar repair. That is why
Lane B's cause also falls, from 35 to 28.

## 1. "nominal declaration requirements are not complete"

This cause had 1 failure and 12 blocked records.

Its root is core's `unsafe impl ZeroablePrimitive for u8 { type NonZeroInner =
NonZeroU8Inner; }`, from the `impl_zeroable_primitive!` rule in
`num/nonzero.rs`. A `NonZero` constant evaluated during declarations demands
early scalar validation of that impl. Completion (`complete_scalar_impl` to
`validate_impl_owner`) forms the header and every associated type value. The
diagnostic showed that it met `struct NonZeroU8Inner` before that struct's
nominal contract was complete. The struct is in `num/niche_types.rs`, which
`num/mod.rs` declares after `nonzero`.

The repair is in `prerequisite_queries.rs`. The first time an early attempt
reaches `Step::Complete`, it collects the types completion will form: the
predicate type, the trait's type arguments and each associated `type` value.
It charges one unit of the owner's budget per type, re-queues the owner's
`Requirements` phase and pushes `Work::ValueShape` for each type. The ordinary
nominal contract and shape work therefore finish first. A second `Complete`
validates as before. The admitted early profile is unchanged.

Controls (section 7): `later-associated-value` (accepted) and
`later-associated-value-not-copy` (E0277, rejected for its bound).

## 2. "canonical trait selection requires pending declarations": diagnosed, not repaired

In census 5 this cause had 1 failure and 11 blocked records. Once sections 1,
3 and 4 removed the failures that had hidden it, it had 3 failures and 26
blocked records: `Buf` in `num/imp/int_log10.rs`, `Buf` in
`num/uint_macros.rs` and `prepare` in `num/uint_macros.rs`.

Each is a declaration-time constant that executes a const fn copying
`Option<u32>` or `Option<NonZero<uN>>` values:

- `Engine::prove_body` calls `copy_requirements::prepare`.
- That calls `copy_decision`, which selects `Copy` at `Executable`
  availability, so it waits for validated impls.
- The selected impls are the derived `impl<T: Copy> Copy for Option<T>`
  (pending `ImplementationValidated(#34146)`) and `NonZero<T>`'s `Copy` impl
  (pending `#866`).
- Both subjects are generic ADTs. Early scalar validation admits only a
  primitive subject without parameters or predicates, or an own parameter, so
  neither impl can be validated while declarations are still running.

The failure is honest. It is not a misattribution, and no small repair keeps
the current availability contract. Two options need an integrator decision
(see the end of this record):

- **(a)** Widen the early profile to ADT subjects whose arguments are exactly
  the impl's own parameters. This adds their environment assumptions and lets
  the subject be a requirement type. It also needs a pending copy decision to
  surface as a dependency instead of a failure.
- **(b)** Change the policy so a CTFE copy decision during declarations uses
  `Declarations` availability: headers only, as constant requests already do.

## 3. Constant execution during declarations

### 3.1 Source bit counts

"source intrinsic constant execution is not implemented" (census 5: 1 failed, 4
blocked) was `ctpop`, reached from `prepare` in `num/uint_macros.rs`.

Constant execution now runs core's source `ctpop`, `ctlz`, `cttz`,
`ctlz_nonzero` and `cttz_nonzero` declarations:

- `sema/compiler_items/bit_counts.rs` binds the intrinsic by its real source
  declaration and current header. It accepts only a completed const function
  with the exact `(T) -> u32` Rust signature, `unsafe` exactly for the
  `_nonzero` forms. A closed integer `T` is required, and imported or
  wider-transport provenance is rejected by name.
- `const_eval/bit_count_calls.rs` executes it at that integer's width.
  `_nonzero` of zero is an execution fault ("a nonzero bit count was executed
  on zero"), like rustc's E0080.
- `const_eval/executed_calls.rs` routes the selector through it.

No unsafe or const permission is granted beyond the declaration's own.

### 3.2 Implicit enum discriminants

"constant storage layout requires completed enum discriminants" (census 5: 1
failed, 2 blocked) was a `const _` in `lib.rs` that reads the layout of
`Option<Repr>` before the crate-wide discriminant pass. An enum with no explicit
discriminant has ordinals as its values, and they are final once its shape is.
`publish_implicit_discriminants` (`declarations/aggregates.rs`) publishes them
right after the shape. The later crate-wide pass republishes the same values.
An enum whose variant count overflows its tag stays pending, so that pass still
reports the overflow.

### 3.3 Field templates declared later

Once section 1 let `NonZero` constants evaluate, "missing aggregate field
template" appeared (diag5 to diag9). `prepare` and `Buf` stored
`Option<NonZero<uN>>` in a const fn body typed during declarations, before
`NonZero`'s field template was published. Storage preparation failed
outright.

- `Definitions::prepare_storage_fields_pending` names the unpublished nominal
  instead of failing, and such a miss is never cached as a failed root.
- Body finalization (`field_template_dependency`, called from
  `prepare_publication`) returns `Requirement::Nominal` for that item and
  resumes when it is published. It is skipped at `Validated` availability or
  when projected templates or pointer casts are present, because ordinary
  storage already handles those. A second wait on the same item is still an
  error.
- Constant execution's layout kernel reports the same miss as
  `Requirement::NominalFields`. `require_nominal_fields` schedules that
  nominal's `Fields` work only while its declaration can still publish them.

The cause was gone in diag10. The census then showed the `Copy` selection of
section 2 behind it.

## 4. "invalid canonical trait selection" and "invalid associated equality selection"

Census 5 had 18 and 3 failures. Batch 4 traced both to "associated type
projection requires normalization" escaping `fold_cause`. The diagnostic
(`apply_diag4.py`) traced it further, to its origin at `infer.rs`'s `head`:

- `sema/traits/results.rs::equalities` asked for the subject's constructor
  head, to send trait objects to their object bounds, whenever the crate had
  any dynamic shapes.
- For a rigid projection subject, `head` returns `Error::Normalization`. An
  example is core's `ChangeOutputType` result,
  `<<R as Try>::Residual as Residual<V>>::TryType`.
- The `?` returned that error, which failed every equality selection on such a
  subject.

Core forms trait objects, so it always has dynamic shapes. That is why ordinary
reductions without an object passed (batch 4's `repro/flatten{1,2,3}.rs`).

The repair treats `Error::Normalization` from `head` as "not an object". A
rigid projection has no constructor and no object bounds, and its equations
come from its declaration identity, which is the existing path. Every other
error is still returned.

This is the only change in `sema::traits`: one match in
`dev/src/sema/traits/results.rs::equalities`.

All 21 failures are gone. Five `flatten.rs` methods (`try_fold`, `fold`,
`last`, `try_rfold`, `rfold`) now get further and fail with Lane B's cause
instead. That accounts for Lane B's 28 failures here against 23 in Lane B's own
census.

## 5. Census isolation of abandoned work

### 5.1 pat.rs: diagnosed, not repaired

"non-pointer non-null patterns require their wider profile" (1 failure) comes
from core's `impl<T: DispatchFromDyn<U>, U> DispatchFromDyn<pattern_type!(U is
!null)> for pattern_type!(T is !null)`. `refined_annotations::pattern_type`
admits `!null` only over a pointer base. A generic base needs:

- `!null` pattern types over a type parameter, with a deferred
  pointer-likeness obligation;
- `DispatchFromDyn` coherence for pattern types.

That is medium-sized work in pattern types and dyn dispatch, and it was not
attempted. It is the census's ordinary first error, and it blocks nothing.

### 5.2 control_flow.rs "later semantic assignments"

`from_try` in `ops/control_flow.rs` failed with "nominal or associated types
require later semantic assignments". The diagnostic showed `kind=SelfType
schema=false` for the impl's self type. The impl header itself had failed in
census, with Lane B's cause, so its self type never received a schema. The
later use was a misattributed second failure.

`census_isolated_self` records the failed `ImplHeader` as the failing work.
Such a use is now blocked by the header's record. In ordinary mode the first
error is unchanged. Census integration test:
`a_failed_header_blocks_later_uses_of_its_self_type`. The pin recorded `make`
there as a second failure with the same message
(`repro/census/selfuse.pin.tsv`).

### 5.3 Const bodies a failed root left active

diag6 to diag8 showed "cycle in const function body declaration dependencies"
(1 failed, 3 blocked) on a second `prepare`, for `checked_ilog2`. An earlier
root failed while typing that const body, and isolation discarded the root's work
but left the body in `execution_bodies.active`. The next demand saw the body as
active and reported a cycle.

- Isolation now poisons every active execution body as `Unit::Execution`.
- Both cycle sites ("cycle in const function body declaration dependencies" and
  "cycle in required initializer bodies") call `census_abandoned_body` first,
  which blocks the demand by the record that abandoned the body.
- A genuine cycle in ordinary mode, or in census mode without a poisoned body,
  is unchanged.

Census integration test:
`a_const_body_left_active_by_a_failed_root_blocks_its_later_demands`. The pin
recorded `b` there as a second failure, a cycle
(`repro/census/abandoned.pin.tsv`).

## 6. Signature anonymous-constant privacy

rustc 1.98.0's type-privacy pass visits bodies, including every anonymous
constant nested in one. It never visits an anonymous constant of an item
signature. A first exploration (`repro/privacy/`, 27 shapes) found the
accepted shapes reaching a `pub(super)` type from the crate root:

- fn result and struct field lengths;
- signature const arguments;
- enum discriminants;
- type alias, impl header and trait default lengths;
- const, static and associated const annotations;
- a local annotation nested inside a signature length;
- a length evaluating a public const fn that names the type.

The rejected shapes were body local annotations, repeat counts, body const
arguments, inline consts, const initializers and leaks through a body.
Associated item privacy (E0624) and field privacy (E0616) still apply inside
signature lengths, because they are typeck rules. (`where-clause.rs` failed for an unrelated unsatisfied bound, E0277.)

Batch 4 had kept RustGM's stricter check (its section 3). The integrator
decided to match rustc. The changes:

- `bodies::TypePrivacy { Checked, Signature }` travels with each anonymous
  constant from its scheduler, through `arguments`, `constants::generics::Input`
  and `begin_constant`, into the body `State`.
- `declarations/evaluation.rs::constant_privacy` decides it:
  - a `Position::Local` constant is `Checked`;
  - a `Position::Constant` constant inherits from the constant it is nested
    in, which `Pending.in_bodies` tracks;
  - every other position is `Signature`.
- Enum discriminants are `Signature`. Fn bodies and initializers are
  `Checked`, and a const argument selected inside a body inherits that body's
  setting.
- For a `Signature` body, `Privacy::node` skips the semantic type walk: node,
  adjustment, call and constant-argument types. It keeps the `item()` checks.
- `spec.md` records the rule beside the semantic type visibility rule.

Remaining difference: an inline const or closure inside a signature length is
treated as a body, which is stricter than rustc. Core does not need it.

## 7. Controls

Twenty-seven no_core source-core controls were declared before the frozen rustc
1.98.0 oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024
--crate-type=lib --emit=metadata`). Each directory under `pa31/tests/spec/`
has `outcomes.json`:

- `signature-constant-privacy/`: 11 controls;
- `projection-object-equalities/`: 4 controls;
- `declaration-constant-execution/`: 12 controls.

One amendment was made before a re-run. Oracle v1 of
`declaration-constant-execution` agreed 9/9. The pin already accepted
`implicit-discriminant-length`, so that case did not exercise core's failure,
which is a layout read. v2 added `implicit-discriminant-layout`, its mismatch
opponent and `projected-choice-length`, declared before oracle v2. No v1 case
or outcome changed. All 27 agree with rustc (oracle records in
`obj/parallel/census-batch5/oracle/`).

The census 5 pin (`5cb7e00c`) and the census 6 pin (`1c0cac9f`) were run over
all 27 controls with `--source-core`. The results are in
`obj/parallel/census-batch5/oracle/pin{5,6}-fixtures.txt`.

| Control | rustc | Census 5 pin | Census 6 pin |
|---|---|---|---|
| `bit-count-lengths` | accept | not implemented | accept |
| `bit-count-mismatch` | E0308 | not implemented | incompatible types |
| `bit-count-nonzero-zero` | E0080 | not implemented | nonzero bit count on zero |
| `implicit-discriminant-layout`, `projected-choice-length` | accept | discriminants incomplete | accept |
| `implicit-discriminant-layout-mismatch` | E0308 | discriminants incomplete | incompatible types |
| `implicit-discriminant-length`, `later-field-template`, `later-associated-value` | accept | accept | accept |
| `implicit-discriminant-mismatch`, `later-field-template-mismatch` | E0308 | incompatible types | incompatible types |
| `later-associated-value-not-copy` | E0277 | bound not satisfied | bound not satisfied |
| `residual-with-object`, `residual-output-equation` | accept | invalid equality selection | accept |
| `residual-output-mismatch` | E0308 | invalid equality selection | incompatible types |
| `residual-try-type-mismatch` | E0308 | invalid equality selection | requires normalization |
| seven `signature-*` positives | accept | private | accept |
| `body-local-length`, `body-repeat-count`, `body-const-argument` | reject | private | private |
| `signature-private-method-length` | E0624 | inherent method is private | inherent method is private |

Three positives already pass on the census 5 pin and serve only as regression
controls: `implicit-discriminant-length`, `later-field-template` and
`later-associated-value`. No reduction reproduced section 1's failure outside
core, so census 6 is its only evidence. `residual-try-type-mismatch` is
rejected for a different reason than rustc's: "associated type projection
requires normalization", not as incompatible types.

Private invariants check every case:

- `dev/tests/invariants/privacy_of_signature_constants.rs`;
- `residual_object_equalities.rs`;
- `early_constant_execution.rs`.

`physical_privacy.rs` and `valtree_inputs.rs` were updated for the new
argument.

Found and not changed:

- `size_of::<Wrap<u8>>()` with `struct Wrap<T: Zeroable>(T::Inner)` is
  accepted by rustc. The pin and the candidate both reject it with "constant
  storage layout requires completed fields" (`repro/controls/proj-c.rs`).
- Lane B's override-conformance gap also rejects this batch's source-core
  `Flatten` reductions (`repro/flatten-*.rs`).
- `pa31/tests/spec/static-constant-modules/README.md` names a test file that
  does not exist.

## 8. Census 6

The pin `1c0cac9f` (`obj/parallel/census-batch5/qualified-1/pin/rustgm`) was
built from `0681610fc` with batch 4's flags. Its manifest is census 5's with
`expected_cli_sha256` and `predecessor_manifest_sha256` changed, and one
configuration pin rebound: `dev/src/support/edition.rs`. Its only change since
the census 5 pin is Lane B's doc comment on `LegacyRule::PanicMacros`
(`36c24b144`). No variant, rule, mapping, default or limit changed, and the
diff and reason are in the build receipt.

The census is analysis-only, with original core inputs unchanged: 375 opens in
49 s. Its records are in `rustgm-pa29-work/obj/parallel/core-census-6/`. The
ordinary first error is unchanged: `core/src/pat.rs:2692` ("non-pointer
non-null patterns require their wider profile"). The census records are
identical to diag10's, so section 6 changes nothing in core's declarations.

| Census | Checked | Failed | Blocked | End |
|---|---|---|---|---|
| Census 5 (`5cb7e00c`) | 42,194 | 62 | 55 | declaration loop complete |
| Census 6 (`1c0cac9f`) | 42,226 | 32 | 52 | declaration loop complete |

Bodies are still not entered (batch 4, section 6), so there are no body
causes. Checked declarations grew by 32.

### Failures by cause

"Blocked" counts every record whose blocking chain ends at a failure with that
message.

| Cause | Census 5 failed / blocked | Census 6 failed / blocked | Sites |
|---|---|---|---|
| associated type requires an unambiguous implementation (Lane B) | 35 / 26 | 28 / 26 | `iter/adapters/step_by.rs` (7), `flatten.rs` (5), `fuse.rs` (4) |
| canonical trait selection requires pending declarations | 1 / 11 | 3 / 26 | `num/uint_macros.rs` (2), `num/imp/int_log10.rs` (section 2) |
| non-pointer non-null patterns require their wider profile | 1 / 0 | 1 / 0 | `pat.rs` (section 5.1) |
| invalid canonical trait selection | 18 / 0 | 0 / 0 | repaired (section 4) |
| invalid associated equality selection | 3 / 0 | 0 / 0 | repaired (section 4) |
| nominal declaration requirements are not complete | 1 / 12 | 0 / 0 | repaired (section 1) |
| source intrinsic constant execution is not implemented | 1 / 4 | 0 / 0 | repaired (section 3.1) |
| constant storage layout requires completed enum discriminants | 1 / 2 | 0 / 0 | repaired (section 3.2) |
| nominal or associated types require later semantic assignments | 1 / 0 | 0 / 0 | now blocked by its header (section 5.2) |

Census 5's Lane B row predates Lane B's sugar repair (`0d0cd8c60`), which is in
this base. Lane B's own census on that repair recorded 23 / 25. The five
`flatten.rs` methods of section 4 make the 28.

### Largest blocking roots

| Blocked | Root | Cause |
|---|---|---|
| 11 | alias `Buf`, `num/imp/int_log10.rs:4111` | canonical trait selection requires pending declarations |
| 11 | alias `Buf`, `num/uint_macros.rs:79392` | canonical trait selection requires pending declarations |
| 4 | function `prepare`, `num/uint_macros.rs:78624` | canonical trait selection requires pending declarations |
| 4 each | `spec_nth`, `spec_try_fold`, `spec_fold` in `step_by.rs` | associated type requires an unambiguous implementation |
| 3 each | `spec_next_back`, `spec_nth_back`, `spec_try_rfold`, `spec_rfold` in `step_by.rs` | associated type requires an unambiguous implementation |
| 2 | self-type `Self`, `ops/control_flow.rs:14309` | associated type requires an unambiguous implementation |

All remaining failures belong to Lane B (28), to the decision in section 2 (3)
or to section 5.1 (1).

## Checks

Every final check used the committed sources of `0681610fc`, docs aside,
which are the pin's source fingerprint. The first two commits also passed their
own selections at their own sources.

| Check | Result | Receipt |
|---|---|---|
| `49d39ea65` private (28 lib cases, strict private Clippy, format, architecture) | pass | `obj/qualification/fast/1794084895106767707` |
| `01bac6690` `census_mode` (CLI Clippy and build) | pass, 9 tests | `obj/qualification/fast/1794085132593337759` |
| Final private (29 lib cases: the three batch invariants, early-validation queries, declaration execution, privacy, binderless modules, undeclared opaques, elided outlives, sugar parameters, components, stability transport) | pass | `obj/qualification/fast/1794084092687165798` |
| `census_mode` and PA1 to PA12 | pass, 9 tests, every PA full | `obj/qualification/fast/1794084359033091422` |
| PA13 to PA19 and PA21 to PA24 | pass, every PA full | `obj/qualification/fast/1794084630351039666` |
| PA20 alone | pass, 1451/1451 | `obj/qualification/fast/1794084760146926866` |
| `make test-upstream` | pass, 127 fixtures, 126 Reference files | `obj/parallel/census-batch5/tmp/test-upstream.log` |
| rustc 1.98.0 oracles (v1 for two directories, v2 for the third) | 27/27 agree | `obj/parallel/census-batch5/oracle/` |
| Pin build and ordinary observation | pass | `obj/parallel/census-batch5/qualified-1/receipt-build.json` |
| Census 6 | completed, analysis-only | `obj/parallel/census-batch5/qualified-1/receipt-census.json` |

No hot record, column or wire layout changed:

- The privacy flag fits in the padding of the declaration `Request`.
- `Unit::Execution` carries a `DefId` like its siblings.
- The finalization wait lives in the boxed `Publication`.

So no layout invariant was selected. `make test-harness` was not needed,
because no integration root was added.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness and course text.

## Decisions needed

- Section 2: option (a), widening the early scalar profile to own-parameter ADT
  subjects, or option (b), using `Declarations` availability for copy
  decisions in declaration-time CTFE. Census 6's largest blocking roots
  depend on it.
- Section 5.1: whether generic-base `!null` pattern types and pattern-type
  `DispatchFromDyn` coherence belong to this lane.
