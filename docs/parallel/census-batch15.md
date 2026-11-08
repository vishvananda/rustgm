# Census batch 15: proved rigid projection results and the sixteenth census

Census 15 ([batch 14](census-batch14.md), pin `6df12612`) checked 42,310
declarations of original core, with none failed or blocked. Its crate record,
"invalid canonical trait selection", is core's
`unsafe impl<I> TrustedLen for Flatten<I>` (`iter/adapters/flatten.rs:313`).
This batch, on main `6cd9f9e82`:

- names the cause behind that label and reduces it to one level (section 1);
- installs a proved projection equality's normal form the way a
  normalization installs one, so an impl parameter fixed only by an equality's
  result binds to a rigid projection (section 2);
- runs census 16 (section 3).

Census 16 (`ed88ef69`, from `90ba44c16`) checks 42,310 declarations, with
none failed or blocked. Its crate record is "source CoerceUnsized
implementation does not retain a permitted pointer coercion" at core's
`CoerceUnsized` impl for pattern types over raw pointers (`pat.rs:80`). The
Flatten record is gone.

Out of scope:

- Lane B owns `sema/bodies/field_storage.rs`, `finalization.rs`,
  `types/variance/*`, `types/aggregate/instances.rs` and `lifetimes.rs`, and
  Lane C owns transport. This batch touches none of them.
- The static `Sync` check in `sema/declarations/validation.rs` and
  `constrained_scalars.rs` did not change.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend is frozen.
- The census fold's label is not changed here (open items).
- (a), the default's `Self`, is batch 15's next item, after its locate report.

## Method

Exploration drafts are no_core libraries with core's `const_precise_live_drops`
prefix and `pa31/tests/spec/function-pointers/foundation.rs`, in
`obj/parallel/census-batch15/repro/flatten*/`. They ran with rustc 1.98.0, the
census 15 pin and the analysis-only pins below; they are not declared
controls. The 13 PA20 controls were declared
(`controls/declared-outcomes.json`, sha `617502f7`) before their frozen
rustc 1.98.0 oracle (`controls/rustc-oracle/`, 13/13 agree) and before any
RustGM run of their bytes.

Three analysis-only diagnostics named the cause before any repair. Each was
built from an uncommitted, instrumented working tree, keeps its
`working-tree.patch`, and was never qualified:

| Diagnostic | Instrumentation | Pin |
|---|---|---|
| `fold` | `census::fold_cause` prints its caller and the original cause | `60fec1b3` |
| `unresolved` | `selection::inference` prints a backtrace for `Unresolved` | `ef0a5468` |
| `blocked` | `equate_canonical_input`'s blocked loop prints both sides | `2b325d37` |

## 1. The cause behind census 15's label

"invalid canonical trait selection" is a fold: `census::fold_cause` maps any
cause that is not exhaustion to one fixed text. On full core the `fold`
diagnostic prints one fold, at `nominals/traits.rs:666`
(`select_requirement_waiting`), and its cause is "type annotations needed",
`infer::Error::Unresolved`. The census records are the same as census 15's.

The `unresolved` backtrace runs from `validate_impl_requirements_prepared`
through `prove_clause`, `select_requirement_waiting`, `select_declaration`,
`select_from_checked`, `prove_checked`, `Search::run_checked`, `begin` and
`poll` to `Search::accept`. Its only `map_err(inference)` is the Proven
branch's `equate_proved_predicate`.

TrustedLen's supertrait goal `Flatten<I>: Iterator` selects
`impl<I, U> Iterator for Flatten<I>`, whose `U` is fixed only by
`IntoIter = U`. The equality is proved, and its normal form is the rigid
`<<I as Iterator>::Item as IntoIterator>::IntoIter`. The projection branch of
`equate_proved_predicate` then equates the result hole `?U` with that normal
form through `equate_canonical_input`. Its fresh known term is a projection
with no opaque mark, so `unify_pending` blocks the pair without binding it,
and the blocked loop's `resolved(?U)` is `Unresolved`. The `blocked`
diagnostic prints exactly that pair on full core and on every reduction: the
left side unresolved, the right side the canonical projection.

`apply_normalization` installs a proved normal form by `known` followed by
`retain_opaque_results`, which marks its rigid projection roots. The
projection-result path received a caller-proved normal form too, but never
marked it. A concrete normal form (`u8`, `Pair<V>`) never blocks, so only
rigid projection results failed. This is an incompleteness, not a violated
canonicalization invariant and not a legitimate rejection: rustc accepts the
reduction.

```rust
pub trait Super {}
pub trait Sub: Super {}
pub trait Source { type Out; }
pub struct Wrap<I>(I);
impl<I, U> Super for Wrap<I> where I: Source<Out = U> {}
impl<I: Source> Sub for Wrap<I> {}
```

Every exploration draft that rustc accepts and that keeps `U` as a result
fails with the same cause, `dyn` shapes or not: one-level, item-bound and
nested equalities, core's Flatten shape with and without `FlattenCompat`, the
blanket `IntoIterator` impl or constness, an `Iterator` subtrait, and core's
context with `OneShot` specialization or the array `TrustedLen` impls. A body
fails the same way (`flatten-call`), reported directly as "type annotations
needed". Naming `U` in the subtrait impl passes.

## 2. Proved rigid results

The coordinator approved the repair with six amendments. In
`equate_proved_predicate`'s projection branch, after the projection's inputs
are equated, the proved normal form is installed as `apply_normalization`
installs one: `known(expected)`, then `retain_opaque_results` on that term.
The routine is reused, not copied; its visibility widens from private to
`pub(in crate::types::infer)`, the narrowest that compiles. The recorded
tradeoff: one helper loses encapsulation, and in return both installs of a
proved normal form stay identical.
`equate_canonical_input` itself does not change. Its `known` call returns
the same cached term, now marked, so the blocked unify binds an unresolved
result hole, or a hole under a constructor, to the rigid normal form.

- **Only proved rigid roots are marked (amendment 1).**
  `retain_opaque_results` calls `resolved` on each projection root of its
  term, and `resolved` refuses any term that still holds an inference
  variable. The marked term is `known` of a store `TypeId`, and the type store
  holds no inference variables. Its roots are exactly the roots
  `apply_normalization` marks for the same term.
  - A result that is itself a projection with an unresolved input,
    `<?V as Source>::Out`, still blocks. The blocked loop reports
    `Unresolved` and `?V` stays unresolved.
  - No surface program reaches that path: the `blocked` diagnostic prints
    nothing for `pick::<_, I>()` or `pick()`, which selection rejects earlier
    as ambiguous. So the control is a private equations regression,
    `a_result_projection_with_an_unresolved_input_still_blocks_without_inferring_it`
    in `dev/tests/equality_predicates.rs`. The commit says so.
- **Two results for one hole (amendment 2).** A later result for the same
  hole proves equality and never rebinds it. Two opaque known terms that
  differ meet in `structural`, which compares their store inputs and reports
  `Mismatch`. The private test fixes `?U` by one proof and then checks that a
  second proof with the same normal form passes and one with a different
  normal form fails, leaving `?U` unchanged. The surface controls are
  `two-results-differ` (E0271), `two-results-same` and `two-results-assumed`.
- **Ledger (amendment 3).** The marking pays the inference context's `step`
  and `account_work` counters, through `normalization_roots` and `resolved`,
  in proportion to the normal form's size. These are the counters
  `apply_normalization`'s marking pays. There is no new free path.
- **Other callers (amendment 4).** The trait-input and builtin-subject
  callers of `equate_canonical_input` are byte-for-byte unchanged, and so is
  the result-to-input direction. `body-input` keeps its exact message.
- **The fold (amendment 5)** is not changed in this commit (open items).
- **Controls (amendment 6).** The 13 controls are in
  `pa20/tests/spec/contracts/proved-results/`, PA20 being the earliest
  assignment owning associated-type equality bounds. Every manifest entry
  declares its exit status and rustc status. The references were generated
  with `--bless` for only these 13 cases, by the analysis-only pin `final-1`
  (`80d0c461`), built from the repair's source on `977c328df` before the
  rebase onto `6cd9f9e82`. The rebased tree's PA20 run passes them without
  blessing. The reference batch is
  `pa20/reference-batches/1794156625752752813-66be35a887df.json`.

| Control | rustc 1.98.0 | Census 15 pin | This branch |
|---|---|---|---|
| `rigid-result` | accept | invalid canonical trait selection | accept |
| `flatten-shape` (core's shape, no `unsafe`) | accept | invalid canonical trait selection | accept |
| `result-bound-holds` | accept | invalid canonical trait selection | accept |
| `body-selection` | accept | type annotations needed | accept |
| `nested-rigid-result` | accept | invalid canonical trait selection | accept |
| `two-results-same` | accept | invalid canonical trait selection | accept |
| `two-results-assumed` | accept | invalid canonical trait selection | accept |
| `named-result` (ordinary) | accept | accept | accept |
| `result-bound-fails` | E0277 | invalid canonical trait selection | requirements |
| `concrete-mismatch` | E0271 | requirements | requirements |
| `two-results-differ` | E0271 | invalid canonical trait selection | requirements |
| `body-input` | E0283 | ambiguous | ambiguous |
| `result-input-unresolved` | E0283 | ambiguous | ambiguous |

"requirements" is "trait implementation does not satisfy the trait's
requirements". "ambiguous" is "trait obligation requires an unambiguous
matching assumption or implementation". The private invariant
`proved_rigid_results.rs` checks all 13 controls and their exact first errors.

The two private regressions are discriminating. With the repair's five lines
removed, `a_proved_rigid_result_binds_an_unresolved_result_like_a_normalization`
fails with `Unresolved`, and the unresolved-input regression still passes
(`obj/qualification/fast/1794156188789476238`, a development run; it does not
count).

On analysis-only `candidate-1` (`45828f28`), every exploration draft rustc
accepts passes, and every draft rustc rejects stays rejected. The two rustc E0038 drafts are the exception.
They are not dyn-compatible under the no_core foundation and now reach
RustGM's object check, which accepts them. That is a foundation gap, present
on the census 15 pin with a minimal `&self` trait object
(`repro/dyn-receiver/`) and not caused by the repair. rustc refuses to
dispatch even `&self` there, because the foundation has no `DispatchFromDyn`
impls for references. It is not a core blocker, since core has them.

## 3. Census 16

The pin `ed88ef69` was built from `90ba44c16` on main `6cd9f9e82` with an
empty uncommitted patch. Its ordinary observation stops at "file 189: 2448:
source CoerceUnsized implementation does not retain a permitted pointer
coercion", with 375 sources opened. The census
(`obj/parallel/census-batch15/qualified-1/census/`) checks 42,310
declarations of original core, with none failed or blocked, in 58.6 seconds.
It has 8 records: one crate record, then 7 lookahead records. They are
byte-identical to the analysis-only `final-1` census.

| # | Pass | Site | Message |
|---|---|---|---|
| 1 | `validate_trait_implementations` | `pat.rs:80` | source CoerceUnsized implementation does not retain a permitted pointer coercion |
| 2 | `validate_trait_impl_owners` | `iter/adapters/zip.rs:248` | trait implementation requires conformance and selection semantics |
| 3 | `local_annotations` | unlocated | implementation validation is outside the early scalar profile |
| 4 | `complete_alias_normalizations` | unlocated | implementation validation is outside the early scalar profile |
| 5 | `validate_objects` | `any.rs:152` | source receiver compatibility requires the completed immutable impl epoch |
| 6 | `complete_discriminants` | `mem/alignment.rs:440` | associated type requires an unambiguous validated implementation |
| 7 | `complete_layouts` | `ascii.rs:29` | enum layout requires completed discriminants |
| 8 | `require_specialization_execution` | unlocated | specialization verification is unfinished |

Census 15's record 1 is gone. The new record 1 is core's

```rust
impl<T: PointeeSized, U: PointeeSized> CoerceUnsized<pattern_type!(*const U is !null)> for pattern_type!(*const T is !null) where
    T: Unsize<U>
{
}
```

`pointer_pair` (`traits/implementations/coercions.rs`) accepts reference
and raw-pointer pairs only, so a pattern type over a raw pointer is refused
by name. It is not reduced here. Record 2 moved from `SpecFill`
(`mem/maybe_uninit.rs:1659`) to `ZipImpl` (`zip.rs:248`). It is derived
from the failed impl validation, as [batch 13](census-batch13.md) found for
`SpecFill`, and moves with it. Records 3 to 8 keep their passes, sites and
messages.

## 4. Checks

Every qualifying check below ran on the tree committed as `90ba44c16`, docs
aside.

| Check | Result | Receipt |
|---|---|---|
| Private: 517 lib cases, strict private Clippy, format, architecture | pass | `obj/qualification/fast/1794157260907507740` |
| `census_mode` (10), `equality_predicates` (18), PA1 to PA12 | pass | `obj/qualification/fast/1794157508290346749` |
| PA13 to PA19 and PA21 to PA24 | pass | `obj/qualification/fast/1794157741315951328` |
| PA20: 1,464 of 1,464 | pass | `obj/qualification/fast/1794157854947423080` |
| `make test-upstream` | pass, 128 fixtures, 126 Reference files | `obj/parallel/census-batch15/tmp/test-upstream.log` |
| rustc 1.98.0 oracle | 13/13 agree | `obj/parallel/census-batch15/controls/rustc-oracle/` |
| Pin build and ordinary observation | passed observation, `ed88ef69` | `obj/parallel/census-batch15/qualified-1/receipt-build.json` |
| Census 16 | completed: 42,310 checked, 0 failed, 0 blocked, 8 records | `obj/parallel/census-batch15/qualified-1/receipt-census.json` |

The 517 private lib cases are batch 14's 348 joined cases plus every test
under `types::infer` and `sema::traits::selection`, the new invariant among
them (`obj/parallel/census-batch15/joined-cases.txt`). The same four
fastchecks first passed on the repair's tree on `977c328df`, before the
rebase (`pre-rebase/`). They do not count here.

`make test-harness` was not needed, because no integration root was added.
`make test-backend` was not needed, because the backend contract and bridge
did not change. No struct changed size.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness and course text.

## Open items

- **The census fold keeps one text for distinct causes.** A cause that is not
  exhaustion still folds to "invalid canonical trait selection". Census 15
  hid "type annotations needed" (`infer::Error::Unresolved`) behind it. Lane
  B's task 2 found a second distinct cause behind the same text: "selected
  type retains an unmapped lifetime", raised from `State::normalize` through
  `Search::complete_builtin` and `Proof::prove`. Decided: this is Lane A's
  small follow-up commit, after (a)'s locate report and repair. The fold will
  keep each cause's category: ambiguity (`Unresolved`), an internal region
  invariant, and a genuinely invalid canonical selection. It is not part of
  the Flatten commit. The tradeoff: one fewer moving part in that commit, at
  the cost of a known diagnostic-quality debt meanwhile.
- **The no_core foundation lacks `DispatchFromDyn` for references**, so rustc
  rejects its trait objects (E0038) where RustGM accepts them. This is a
  foundation-profile gap, not a core blocker.
- **(a)**, the default's `Self`, is next. Its round-2 probes (read-only, on
  the existing pins) narrow the divergence. With the impl writing its
  argument, an omitted default at a bound (`T: Source + Fill`) or a qualified
  prefix (`<T as Fill>::Out`, `<Short as Fill>::Out`) is still rejected,
  while written arguments pass. Under a `Source: Fill` supertrait every caller
  passes. The locate report comes before any code.

## Storage

- Kept: the census 16 pin `obj/parallel/census-batch15/qualified-1/pin/`
  (`ed88ef69`, with `.keep-artifacts`).
- Kept until this batch's commits land: the analysis-only pins `fold`,
  `unresolved`, `blocked`, `candidate-1` and `final-1` under
  `obj/parallel/census-batch15/diagnostics/`. Each keeps its receipt, census
  and `working-tree.patch`.
- Retired by removing `.keep-artifacts` and pruning its directory: census 15's
  pin (`census-batch14/qualified-1/pin`, `6df12612`), after census 16's pin was
  built. Its hash is in `obj/parallel/census-batch15/retired-pins.sha256`.
- obj reached its 16 GiB cap during development. Superseded incremental
  sessions were retired by fastcheck's own routine (`storage_incremental.retire`),
  which took obj from 17 GiB to 14 GiB.

## Shared files touched

- Inference: `types/infer/predicates/equations.rs` (the projection branch)
  and `types/infer/deferred/normalization.rs` (one visibility).
- Selection: `sema/traits/selection/search.rs` mounts the new invariant.
- Tests: `dev/tests/equality_predicates.rs` (two regressions) and the new
  `dev/tests/invariants/proved_rigid_results.rs`.
- PA20: 13 controls with references, their manifest entries, the reference
  batch, and a sentence in `pa20/tests/README.md`. No existing PA20 case
  changed.
- `spec.md` has a paragraph on proved equality results.

## Decisions needed

- None blocking. Next, in the coordinator's order: (a)'s locate report before
  any (a) code, then (a), then the fold follow-up.

## Commits

On main `6cd9f9e82`, in order: `90ba44c16` (section 2), then this document,
the tracker entry and the census plan's batch 15 entry.
