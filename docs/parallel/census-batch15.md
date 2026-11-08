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

Then, on main `078d3f238`, it:

- gives a trait path's omitted defaults the path's subject for `Self`;
- keeps a `Self` written among a trait path's arguments as the enclosing
  trait's own (section 5);
- runs census 17 (section 6).

Census 16 (`ed88ef69`, from `90ba44c16`) checks 42,310 declarations, with
none failed or blocked. Its crate record is "source CoerceUnsized
implementation does not retain a permitted pointer coercion" at core's
`CoerceUnsized` impl for pattern types over raw pointers (`pat.rs:80`). The
Flatten record is gone. Census 17 (`c020dad2`, from `d97f986eb`) has census
16's records byte for byte, because neither effect of section 5 reaches core.

Out of scope:

- Lane B owns `sema/bodies/field_storage.rs`, `finalization.rs`,
  `types/variance/*`, `types/aggregate/instances.rs` and `lifetimes.rs`, and
  Lane C owns transport. This batch touches none of them.
- The static `Sync` check in `sema/declarations/validation.rs` and
  `constrained_scalars.rs` did not change.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend is frozen.
- The census fold's label is not changed here (open items).
- The trait-header phase's skip of a default's contract is recorded, not
  changed (section 5).

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

## 5. A trait path's subject in its defaults

Census batch 14 recorded `default-bound-satisfied` as a gap. rustc accepts
`impl Fill for Short` when its argument is left to `R = <Self as
Source>::Item`, and RustGM rejected it with "associated type requires an
unambiguous implementation". This section locates that gap and finds a second
effect of the same mechanism. One commit repairs both.

### Where the divergence was

- `application_arguments` (`generics/arguments.rs`) filled an omitted default
  through `Bound::prefix(owner)`, which maps only the owner's own parameters.
  A trait's `Self` is a separate parameter, owned by the trait's lexical Self
  definition (`self_schema.rs`). So `<Self as Source>::Item` kept Fill's
  `Self`. The unreduced original was folded the same way
  (`unreduced/arguments.rs`).
- `applied_trait` (`trait_paths.rs`) then replaced Fill's `Self` with the
  subject in every argument, through `Bound::parameter_only`. The trait
  application itself therefore came out right.
- But `alias_use` ran first, with the unreplaced arguments, and `Use` had no
  subject. `alias_requirements::expand` built `Bound::joint(Fill,
  arguments)`, which does not map `Self` either. The default's retained
  normalization `<FillSelf as Source>::Item` reached the use site unchanged
  and was rejected by `finish_projection_selection`. That normalization is
  recorded at `Position::Default` by `defer_projection_requirement`.
- A nested default lost its `Self` the same way. In `Fill<R = <Self as
  Other>::X>` over `Other<Q = <Self as Source>::Item>`, the inner default is a
  dependency `Use` of the outer default's contract.

The analysis-only diagnostic `defaults-2` (`2df93ed7`) traced the impl case.
For `impl Fill for Short`:

- the contract records `<T36 as Source>::Item` at `default(DefId(84))`, where
  T36 is Fill's `Self`;
- the impl's `alias_use` passes that same argument, so the expansion is
  unchanged and the selection at the impl is ambiguous;
- the impl's own argument root, `<Short as Source>::Item`, selects.

### Why `Source: Fill` and core's `Try: FromResidual` hid it

- **The header phase stores the default raw.** With a `Source: Fill`
  supertrait (or `where Self: Fill`), Source's header is the first to demand
  Fill's default, while trait header names are active.
  `normalize_annotation_root` then stores any projection inside a trait's
  binder as written (`header_projection_context`).
  - The default's projection never reaches selection at `Position::Default`.
    Nothing is deferred, and the default gets no contract.
  - `retain_header_projection_requirements` revisits each trait's bounds, but
    not its parameters' defaults.
- **Without a contract, nothing expands.** `alias_use` returns at once, at the
  supertrait and at every later use, so the unreplaced contract never
  expands. The argument was already right, and its own root normalizes at the
  use.
- **Core never instantiates a default contract.** In `defaults-2`'s
  whole-core census:
  - none of the 1,448 default uses has a contract;
  - no selection happens at any default position;
  - the only three contract normalizations are in type aliases.

  FromResidual's binding is first used at `pub const trait Try: [const]
  FromResidual` (`ops/try_trait.rs:133`). It is next used at `impl<T>
  FromResidual for NeverShortCircuit<T>` (line 448). Neither use has a
  contract, so the census could not see the gap.
- **Recorded, not changed: the header-phase skip.** It loses no requirement in
  this batch's controls.
  - Each rustc-rejected opponent of the header-phase and plain shapes is
    rejected: a missing `Source` impl at an impl, a bound, a qualified path, a
    generic impl and a nested default.
  - They are rejected because each use's argument is normalized at the use.
  - The trait-header phase is unchanged.

### The second effect: a written `Self`

`applied_trait`'s blanket replacement also rewrote a `Self` written among a
trait path's arguments inside the trait's own items:

- `fn take<T: Fill<Self>>` became `T: Fill<T>`;
- `type Out: Fill<Self>` became `Out: Fill<Out>`.

Against rustc 1.98.0, census 16's pin diverged on five programs:

- it rejected three that rustc accepts: an impl method writing the bound, a
  call through it, and the associated bound;
- it accepted two that rustc rejects with E0277: `method-call-opp` and
  `associated-bound-opp`.

The replacement stood in for default substitution. Once defaults are
instantiated at their source, the replacement is only wrong. Either half of
the repair alone leaves the tree wrong in a new way. Decided with the
coordinator: one commit, giving up two bisectable ones, with both effects
named in its subject.

### Caller audit

Every caller of `applied_trait` lowers a written trait path, whether it calls
directly or through `trait_path` and `bound_values`. The path's arguments come
from `application_arguments` of the same path.

A written argument is lowered in the path's own scope. The applied trait's
`Self` parameter is in scope there only inside that trait's own declaration,
and there it means that trait's `Self`. So the only arguments that needed
`Self` mapped to the subject were defaults. `application_arguments` now
instantiates those at their source. No caller needed the mapping for a
non-default argument.

| Caller | Subject | Written `Self` of the applied trait | Needed beyond defaults |
|---|---|---|---|
| `trait_path` ← `impl_trait_reference` (`traits/implementations.rs`) | the impl's self type | never; an impl's `Self` is its own type | no |
| `bound_values` ← `complete_written_bound` (`generics/bounds/finish.rs`): parameter bounds, where clauses, supertraits and associated-type bounds | the bounded type | only inside the trait itself; identity when the subject is that `Self` (`where Self: Fill<Self>`), otherwise rustc keeps it (`T: Fill<Self>`) | no |
| `bound_values` ← `abstract_types.rs` (`impl Trait` bounds) | the opaque type | as above | no |
| `bound_values` ← `objects.rs` (a `dyn` principal) | the principal's own `Self` | identity | no |
| `complete_projection_path` (`projections.rs`), normal and unreduced | the qualified self type | as above | no |
| `projection_seed_inputs` (`projections/lookup.rs`), a shorthand bound seed | the seed's parameter | as above | no |

Supertrait elaboration does not call `applied_trait`.

- `trait Sub: Super<Self>` is lowered with Sub's `Self` as both its subject
  and its argument. The replacement never touched it, because it maps
  Super's `Self`.
- `T: Sub` elaborates to `T: Super<T>` in `Generics::expanded_trait_bounds`,
  which substitutes stored predicates itself.

`canonical_alias_edge` (trait aliases) lowers its target's arguments without
a subject, and it maps the alias's `Self` in its own expansion. It is
unchanged.

Three audit positives cover the identity cases, and rustc accepts all three:

- `supertrait-self`: `Sub: Super<Self>`, used through `Sub`;
- `where-self`: `where Self: Fill<Self>` inside the trait;
- `impl-parameter`: `impl<T> Fill<T> for Wrap<T>`.

### The repair

- **One substitution.** `Bound`'s all-arguments mapping has an optional
  subject slot, set only by `Bound::with_subject`.
  - The slot holds a type parameter that the owner does not own, and it is
    set before any substitution is computed.
  - The default's normal fold (`Bound::prefix`), its unreduced fold and the
    contract expansion (`Bound::joint` in `alias_requirements::expand`) are
    all built through it.
- **Callers pass the subject.** `application_arguments` takes an optional
  `TraitSubject`, in both its normalized and its written form. These callers
  pass one:
  - impl trait references (`trait_path`);
  - bound trait paths (`bounds.rs`), with the normal and the original
    subject;
  - qualified projection prefixes (`projections.rs`), with the subject and the
    original subject;
  - shorthand seeds (`lookup.rs`).

  Type aliases, nominal paths, generic associated members and trait-alias
  targets pass none.
- **`alias_requirements::Use` carries the subject.**
  - A dependency `Use` maps its subject through the parent's parameters
    exactly as it maps its arguments.
  - The body-annotation expansion (`sema/annotations/requirements.rs`) passes
    none. Its partial projections already instantiate defaults in inference.
    In exploration, `<Wrap<_> as Fill>::Out` was accepted and its missing-impl
    twin rejected, both before and after the repair. A full body annotation
    `<Short as Fill>::Out` goes through `complete_projection_path`. It was
    rejected before the repair and is accepted after it, as rustc accepts it.
    These exploration drafts are in `repro/body-annotation/` and are not
    declared controls.
- **`applied_trait` only interns the application.**
- **Cache and ledger.**
  - `generics.substitutions` is keyed by owner and arguments, so a
    subject-bearing fold is not retained there.
  - Each use instead pays its fold's steps to `Generics::charge_unretained`.
    Those steps grow with the arguments the fold pushed and substituted.
  - The account is part of the substitution total, `substitution_steps`.
    Folds on the cached path keep their steps in that same total, and both
    sit under one shared limit.
  - The account is a `u32` that fits in `Generics`' existing padding, so
    `Declarations` keeps its pinned size (7,152 bytes).
  - In a contract expansion, a dependency with a subject pays one more step,
    as one more argument would. A dependency without one pays what it paid
    before, so `diamond_dependencies_are_linear_and_argument_sensitive` keeps
    its bound.

### Controls

**Fixture home.** PA19 owns trait type-parameter defaults (its `dispatch-*`
defaults, including `U = Self`), and PA20 owns projections and associated
types.

- The defaults group and the associated-bound pair are PA20 cases.
- The six written-`Self` method, where-clause, supertrait and impl controls
  use only traits, bounds and methods, so they are PA19 cases.
- PA31 is not used. Its batch 14 packet is no_core and needs its lang items,
  and these are std programs.

The 24 controls were declared before their frozen rustc 1.98.0 oracle and
before any RustGM run of their bytes. The declaration is
`controls-a/declared-outcomes.json` (sha `5bba6aa4`), in two groups:
"defaults" (16) and "written Self" (8). The oracle is in
`controls-a/rustc-oracle/`, and all 24 agree.

In the table, "unambiguous" is "associated type requires an unambiguous
implementation".

| Group | Control | Path | rustc | Census 16's pin | Repaired |
|---|---|---|---|---|---|
| defaults | `impl-omitted` | impl reference | accept | rejects: unambiguous | accept |
| defaults | `impl-generic` | impl reference | accept | rejects: unambiguous | accept |
| defaults | `impl-header-phase` | impl reference, `Source: Fill` | accept | accept | accept |
| defaults | `impl-missing` | impl reference | E0277 | rejects: unambiguous | rejects: unambiguous |
| defaults | `impl-generic-missing` | impl reference | E0277 | rejects: unambiguous | rejects: unambiguous |
| defaults | `bound-omitted` | bound | accept | rejects: unambiguous | accept |
| defaults | `where-omitted` | where clause | accept | rejects: unambiguous | accept |
| defaults | `bound-header-phase` | bound, `Source: Fill` | accept | accept | accept |
| defaults | `bound-missing` | bound | E0277 | rejects: "…unambiguous validated implementation" | same |
| defaults | `qualified-concrete` | qualified prefix | accept | rejects: unambiguous | accept |
| defaults | `qualified-generic` | qualified prefix | accept | rejects: unambiguous | accept |
| defaults | `qualified-missing` | qualified prefix | E0277 | rejects: unambiguous | rejects: unambiguous |
| defaults | `shorthand` | shorthand seed | accept | rejects: unambiguous | accept |
| defaults | `nested-default` | nested dependency | accept | rejects: unambiguous | accept |
| defaults | `nested-default-missing` | nested dependency | E0277 | rejects: unambiguous | rejects: unambiguous |
| defaults | `object` | `dyn` principal | E0393 | rejects: "object arguments cannot depend on an erased Self default" | same |
| written Self | `method-bound` | method bound | accept | rejects: "implementation method imposes a stronger generic requirement" | accept |
| written Self | `method-call` | method bound | accept | rejects: "trait obligation requires an unambiguous matching assumption or implementation" | accept |
| written Self | `method-call-opp` | method bound | E0277 | accepts | rejects: "trait obligation requires…" |
| written Self | `associated-bound` | associated-type bound | accept | rejects: "associated type definition does not satisfy its declared bounds" | accept |
| written Self | `associated-bound-opp` | associated-type bound | E0277 | accepts | rejects: "associated type definition does not satisfy…" |
| written Self | `supertrait-self` | supertrait (audit) | accept | accept | accept |
| written Self | `where-self` | where clause in a trait (audit) | accept | accept | accept |
| written Self | `impl-parameter` | impl reference (audit) | accept | accept | accept |

- Census 16's pin disagreed with rustc on 13 of the 24 controls. Eleven were
  false rejections and two were false acceptances. The repaired tree agrees
  on all 24.
- The six rustc-rejected defaults controls (five missing implementations and
  `object`) were already rejected, at the same byte and with the same
  message, before the repair.
- PA19's controls are in `trait-bounds/written-self/` (6), and PA20's are in
  `projections/self-defaults/` (16) and `projections/written-self/` (2).
  Their references were generated by `course.py --bless` from the qualified
  pin only after the declaration and the oracle. Each one was then checked
  against the declared outcome.

`default-bound-satisfied` moves from `GAPS` to `CASES` in
`dev/tests/invariants/defaulted_impl_arguments.rs`. The packet's README and
`spec.md` no longer call it a gap.

`dev/tests/invariants/subject_defaults.rs` checks every control's outcome and
first error. Its scaling test builds 8 and 32 copies of an impl, a bound and a
qualified path that each omit the default, plus a body that calls through the
bound and the path. The test checks three things:

- the unretained account grows with the uses;
- the calls select, so the selection measure is live;
- quadrupling the uses at most quintuples each measure.

| Uses | Unretained | Substitution | Trait selections | Inference steps | Canonical steps |
|---|---|---|---|---|---|
| 8 | 120 | 120 | 16 | 1,774 | 0 |
| 32 | 480 | 480 | 64 | 7,078 | 0 |

Every live measure grows four times (inference by 3.99). The substitution
account equals the unretained account, because no fold in this program is
cached. Canonical steps stay 0 in this program, so that column shows nothing
about scaling. The rustc 1.98.0
shape check of the generated program is in `repro/subject-scaling/`.

## 6. Census 17

The pin `c020dad2` was built from section 5's tree on main `078d3f238`. That
tree was then committed, docs aside, as `d97f986eb`. Its ordinary observation
stops where census 16's did, at "file 189: 2448: source CoerceUnsized
implementation does not retain a permitted pointer coercion", with 375
sources opened. The census
(`obj/parallel/census-batch15/qualified-2/census/`) checks 42,310
declarations of original core, with none failed or blocked, in 63.6 seconds.
Its `census.tsv` is byte-identical to census 16's (sha `09b48304`): the same
8 records, with the same passes, sites and messages.

This was expected. Section 5 found that core instantiates no default
contract, so neither of its effects reaches core's declarations.

## 7. Checks for section 5

Every qualifying check below ran on the tree committed as `d97f986eb` on main
`078d3f238`, docs aside.

| Check | Result | Receipt |
|---|---|---|
| Private: 621 lib cases, strict private Clippy, format, architecture | pass | `obj/qualification/fast/1794164823440212944` |
| Trait suites: `suite_traits` (168) and `suite_frontend` (72) | pass | `obj/qualification/fast/1794165325441209879` |
| PA1 to PA12 | pass | `obj/qualification/fast/1794165495263119255` |
| Projection suites (38), PA13 to PA18 and PA21 to PA24 | pass | `obj/qualification/fast/1794165547891198472` |
| PA19: 1,199 of 1,199; PA20: 1,482 of 1,482 | pass | `obj/qualification/fast/1794165634622155923` |
| `make test-upstream` | pass, 128 fixtures, 126 Reference files | `obj/parallel/census-batch15/tmp/test-upstream-a.log` |
| rustc 1.98.0 oracle | 24/24 agree | `obj/parallel/census-batch15/controls-a/rustc-oracle/` |
| Pin build and ordinary observation | passed observation, `c020dad2` | `obj/parallel/census-batch15/qualified-2/receipt-build.json` |
| Census 17 | completed: 42,310 checked, 0 failed, 0 blocked, 8 records | `obj/parallel/census-batch15/qualified-2/receipt-census.json` |

- **The 621 private lib cases** are listed in
  `obj/parallel/census-batch15/a-cases.txt`. They are section 4's 517 cases,
  which include the defaulted-argument invariant, plus 104 more:
  - every test under `types::substitute`, `sema::generics`,
    `sema::alias_requirements` and `sema::declarations::aliases`;
  - every trait associated-type parameter test and impl prerequisite-query
    test;
  - the new subject invariant and Lane B's `normalized_sizing_lifetimes`.

  `sema::declarations::generics` and `impl_headers` are fully selected.
  `trait_paths` and `unreduced` have no tests of their own.
- **The trait suites and PA1 to PA12 ran as two fastchecks.** Together, with
  a cold integration build, they exceeded the five-minute budget.
- **Pre-rebase passes.** The same checks first passed on `f49a62675`, before
  the rebase onto Lane B's commits. They do not count here.
  - The pre-rebase pin `aadb14db` generated references byte-identical to
    `c020dad2`'s. They are kept in `pre-rebase/refs-a/`.
- **Three failures were fixed before these checks passed.**
  - **Contract expansion.** It charged every dependency one step more, whether
    or not it had a subject, which exceeded
    `diamond_dependencies_are_linear_and_argument_sensitive`'s bound. It now
    charges that step only when there is a subject.
  - **Scaling bodies.** The scaling program first had no bodies, so its
    selection measures were 0. Calls were added: the program shape is checked
    with rustc in `repro/subject-scaling/`.
  - **Canonical liveness.** Canonical steps stay 0 even with the calls, so
    the test asserts liveness only for selections.
- **Not needed.**
  - `make test-harness`, because no integration root was added.
  - `make test-backend`, because the backend contract and bridge did not
    change.
  - A size pin edit, because no struct changed size. `Generics`' new `u32`
    fills existing padding, so `DECLARATIONS_SIZE` is unchanged.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness, course text, and PA25 to PA31.

## Open items

- **The census fold keeps one text for distinct causes.** A cause that is not
  exhaustion still folds to "invalid canonical trait selection". Census 15
  hid "type annotations needed" (`infer::Error::Unresolved`) behind it. Lane
  B's task 2 found a second distinct cause behind the same text: "selected
  type retains an unmapped lifetime", raised from `State::normalize` through
  `Search::complete_builtin` and `Proof::prove`. Decided: this is Lane A's
  small follow-up commit, and it is next, now that (a) is repaired. The fold
  will keep each cause's category: ambiguity (`Unresolved`), an internal region
  invariant, and a genuinely invalid canonical selection. It is not part of
  the Flatten commit. The tradeoff: one fewer moving part in that commit, at
  the cost of a known diagnostic-quality debt meanwhile.
- **The no_core foundation lacks `DispatchFromDyn` for references**, so rustc
  rejects its trait objects (E0038) where RustGM accepts them. This is a
  foundation-profile gap, not a core blocker.
- **(a), the default's `Self`, is repaired** in section 5. Two things are
  recorded there and not changed:
  - the trait-header phase gives a default no contract when a header demands
    it first, and loses no requirement in this batch's controls;
  - the body-annotation expansion passes no subject, because its partial
    projections already instantiate defaults in inference.

## Storage

- Kept: the census 17 pin `obj/parallel/census-batch15/qualified-2/pin/`
  (`c020dad2`, with `.keep-artifacts`).
- Kept until section 5's commit is integrated: the analysis-only pin
  `defaults-2` (`2df93ed7`) under `obj/parallel/census-batch15/diagnostics/`.
- Retired by removing `.keep-artifacts` and pruning the directory. Each
  hash is in `obj/parallel/census-batch15/retired-pins.sha256`, and each
  directory keeps its receipt, census and `working-tree.patch`.
  - Census 15's pin (`census-batch14/qualified-1/pin`, `6df12612`), after
    census 16's pin was built.
  - The analysis-only pins `fold`, `unresolved`, `blocked`, `candidate-1`
    and `final-1`, after sections 1 to 4 were integrated.
  - The analysis-only `candidate-a1` (`bf12a6b2`).
  - The pre-rebase section 5 pin `qualified-a` (`aadb14db`), which the
    rebase onto `078d3f238` superseded.
  - Census 16's pin (`qualified-1`, `ed88ef69`), after census 17's pin was
    built.
- `defaults-1` was removed differently. That analysis-only pin is
  `34e651b1`, as recorded in `diag-defaults-1.log`. Its diagnostic directory
  was removed with `rm -r` before inspection, which is not the retirement
  routine. Its census log and its probes under `probes/defaults-1/` remain.
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

Section 5:

- Substitution: `types/substitute/parameters.rs` (the subject slot and
  `with_subject`) and `types/substitute.rs` (one export).
- Alias contracts: `sema/alias_requirements.rs` (the `Use` subject and its
  dependency mapping), its tests, and `sema/annotations/requirements.rs`,
  which passes no subject.
- Declarations:
  - `generics/arguments.rs`, `generics.rs` and `generics/bounds.rs`;
  - `generics/bindings/selectors.rs`;
  - `trait_paths.rs` and `aliases.rs`;
  - `projections.rs`, `projections/aliases.rs`, `projections/applications.rs`
    and `projections/lookup.rs`;
  - `unreduced/arguments.rs`.
- Ledger: `sema/generics.rs` (`charge_unretained`).
- Tests: the new `dev/tests/invariants/subject_defaults.rs`, and
  `dev/tests/invariants/defaulted_impl_arguments.rs` (one case moved from
  `GAPS` to `CASES`).
- PA19 has 6 controls and PA20 has 18, each with references, manifest
  entries, a reference batch and a README sentence. No existing case
  changed.
- `spec.md`: the recorded-gap sentence is replaced by a paragraph on trait
  path subjects. The defaulted-argument packet's README is updated.

## Decisions needed

- None blocking. Next, in the coordinator's order: the fold follow-up.

## Commits

- Section 2 is `08c17d735`. It was followed by this document's first record
  and the batch's tracker and plan entries (`f49a62675`), on main
  `c9fb67cc3`.
- Section 5 is `d97f986eb`, one commit on main `078d3f238`. It is followed
  by this document's sections 5 to 7 and the tracker and plan entries.
