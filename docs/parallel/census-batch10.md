# Census batch 10: function-pointer coherence, specialization lifetimes, callable sugar and the eleventh census

Census 10 ([batch 9](census-batch9.md), pin `1375fd2b`) checked 42,310
declarations of original core with none failed or blocked. Its crate record
was "overlapping implementation headers have no specialization ordering" at
`ptr/mod.rs:2592` (`impl<F: FnPtr> PartialEq for F`), followed by eight
lookahead records. This batch, on main `f98c5d7c8`:

- decides `FnPtr` evidence in coherence from the outer Self type, as rustc
  1.98 does, including rustc's builtin `impl !FnPtr for T` in negative
  coherence. This repairs `ptr/mod.rs:2592` and explains why core's reference
  impls do not conflict (section 1);
- names each specialization parent header lifetime by the child's
  corresponding one. This repairs the next record, core's `&mut I` iterator
  specializations (section 2);
- reads a callable sugar clause whose binder binds no region as an ordinary
  overlap condition. This repairs the record after that, core's `MultiCharEq`
  (section 3);
- reduces lookahead records 2, 3 and 5 (section 5);
- runs census 11 (section 6).

Census 11 checks 42,310 declarations, with none failed or blocked. Its one
record is "implementation prerequisite work budget exceeded" at
`portable-simd/crates/core_simd/src/ops/shift_scalar.rs:802`, in category
`budget`. That is the crate-wide trait implementation validation reaching a
fixed one-million-unit work cap (section 4). A budget record is sticky, so no
lookahead follows it. Bodies are not entered, so Milestone 2 ("original core
passes declaration analysis") is not reached.

Out of scope:

- Lane B owns re-exports, crate loading, the nightly repairs and off-frontier
  selection such as batch 9's `nested-item.rs`; Lane B is repairing
  `sema/bodies/field_storage.rs`, which this batch does not touch.
- Lane C owns body and item transport, the static `Sync` check in
  `sema/declarations/validation.rs` and `constrained_scalars.rs`; neither file
  changed here.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend is frozen.
- `!null` pattern types over integer or reference bases stay rejected by name.
- Ordinary crates keep batch 7's declaration-time lifetime contract gap.

## Diagnosis method

Each diagnostic was an analysis-only pin of the working tree with one census of
unchanged original core inputs, in `obj/parallel/census-batch10/diagnostics/dN/`.
Their binaries are retired.

| Run | Tree | Checked | Failed | Blocked | End |
|---|---|---|---|---|---|
| Census 10 (`1375fd2b`) | `b8e58f062` | 42,310 | 0 | 0 | crate record: overlapping implementation headers have no specialization ordering (`ptr/mod.rs:2592`), then 8 lookahead records |
| d1 (`ccd83897`), section 1 without its formation change | `f98c5d7c8` + patch | 42,310 | 0 | 0 | crate record: specialization parent capture has no unique child anchor (`iter/traits/double_ended.rs:498`), then 8 lookahead records |
| d2 (`429b7f9d`), adds section 2 | `f98c5d7c8` + patch | 42,310 | 0 | 0 | crate record: conditional nominal formation cannot publish an absence without paying its debts (`sync/atomic.rs:4470`), then 8 lookahead records |
| d3 (`2df68bed`), adds section 1's formation change | `f98c5d7c8` + patch | 42,310 | 0 | 0 | crate record: higher-ranked specialization implication is not implemented (`str/pattern.rs:645`), then 8 lookahead records |
| d4 (`5f53ecc1`), adds section 3 | `f98c5d7c8` + patch | 42,310 | 0 | 0 | budget record: implementation prerequisite work budget exceeded (`shift_scalar.rs:802`) |
| d5 to d8, instrumented | d4 + temporary caps | 42,310 | 0 | 0 | see section 4 |

Small drafts reduced each record. They are in
`obj/parallel/census-batch10/repro/` and were run with analysis-only builds
(`obj/parallel/census-batch10/dev/`, binaries retired). Drafts named
`record2-*`, `const-super-*` and `callable-user-const` were also run with
rustc 1.98.0 for exploration; they are not declared controls.

## 1. `FnPtr` in coherence (census 10's record)

rustc 1.98's builtin `FnPtr` goal (`rustc_next_trait_solver`
`trait_goals.rs`, `consider_builtin_fn_ptr_trait_candidate`) holds only when
the Self type is a function pointer. Its negative goal, `impl !FnPtr for T`,
holds when the Self type is not a function pointer and is
`TyKind::is_known_rigid` (`ty_kind.rs`): every type constructor except
variables, parameters, projections, opaque types, placeholders and bound
types.

### Why core's reference impls do not conflict

Batch 9 found that rustc rejects (E0119) a reduction with core's `&A`
`PartialEq` impl beside `impl<F: FnPtr> PartialEq for F`, yet accepts core.
The difference is core's `#![feature(with_negative_coherence)]`
(`lib.rs:166`), which the reduction lacked:

- Without it, coherence asks only whether the intersection's where clauses
  are impossible. `&A: FnPtr` with `A` unknown is not knowable
  (`rustc_next_trait_solver/src/coherence.rs`, `trait_ref_is_knowable`):
  `&` is fundamental, so a downstream crate could own `&Local`. The impls
  overlap (E0119).
- With it, `overlap` (`traits/coherence.rs`) also tries
  `impl_intersection_has_negative_obligation`: the intersection's unknowns
  become placeholders, and a negated where clause that holds proves
  disjointness. `&A: !FnPtr` holds by the builtin negative impl, since `&A`
  is rigid and not a function pointer. The same holds for core's `&mut A`
  impls.

### Repair

`sema/traits/selection/search/pointers.rs` adds `rigid_non_fn_pointer`, the
`is_known_rigid` classification less function pointers, over the caller's
inference head:

- A source `FnPtr` goal whose Self type is rigid and not a function pointer
  is absent even when its arguments are unresolved. Coherence still turns that
  absence into a deferral when a downstream crate could own the Self type
  (`coherence_absence`), which keeps the stable-mode rejections.
- `may_overlap_with` (`search/coherence.rs`) answers such a goal as disjoint
  in negative and strict mode, as the builtin negative impl does, without
  starting a search.
- `prepare_nominal_formation` (`search/construction_conditions.rs`) no longer
  forms the Self type of such a goal: the absence never depends on it. Without
  this, d2 stopped at core's `impl fmt::Debug for AtomicBool`
  (`sync/atomic.rs:4470`) with "conditional nominal formation cannot publish
  an absence without paying its debts", and so did the `fundamental` control.

### Controls

`pa31/tests/spec/fn-pointer-coherence/` has fourteen controls, declared before
the rustc 1.98.0 oracle and before any candidate run; all fourteen agree:

- accepted with negative coherence: core's exact const `PartialEq` shapes
  (`core-references`), a reference, a fundamental nominal, a trait object and,
  under `#[rustc_strict_coherence]`, a tuple;
- rejected with negative coherence (E0119): a parameter and a function
  pointer;
- without the feature: core's shapes, a reference and a fundamental nominal
  are rejected (E0119); a tuple, a local nominal, a raw pointer and an array
  are accepted.

On the census 10 pin, `core-references`, `reference`, `strict-tuple` and
`stable-tuple` failed with overlap messages and `fundamental` with the
formation message; the four other accepted controls passed and every
rejection was as now. `dev/tests/invariants/refuted_fn_pointers.rs` (attached
to `pointers.rs`) checks every case.

## 2. Specialization header lifetimes (d1's record)

With section 1's first part, d1 stopped at core's

```rust
impl<I: DoubleEndedIterator> DoubleEndedIteratorRefSpec for &mut I
```

(`iter/traits/double_ended.rs:498`), which specializes the same impl with
`I: ?Sized`, with "specialization parent capture has no unique child anchor".
The specialization inputs (`sema/specialization/directional_inputs.rs`)
equated the two headers' shapes, without regions, so the parent's `&mut`
lifetime had no child region to name it.

rustc's `specializes` (`specialize/mod.rs`) equates the child's trait
reference with the parent's fresh instantiation, lifetimes included. The
repair, `header_regions`, does the same after the shape equation: it equates
the headers' type arguments with regions and records each differing header
lifetime pair as an equality. This only names each parent header lifetime by
the child's corresponding one, which restores the parent's arguments; as in
rustc it resolves no region constraint. `min_specialization` still rejects a
`'static` argument where the parent is generic, and a repeated lifetime now
reaches its own rejection.

`pa31/tests/spec/lifetime-specialization/` has nine controls, declared before
the oracle and before any candidate run of these bytes; all nine agree. Seven
accept (core's double-ended and iterator shapes, a shared reference, named
lifetimes, a concrete referent, a nominal with a lifetime parameter and a
`'static` guard). `static-specialization` (E0119) and `repeated-lifetime`
("specializing impl repeats parameter `'b`") are rejected. On the census 10 pin
the six non-static positives and `repeated-lifetime` failed with "no unique
child anchor"; `static-reference` passed and `static-specialization` was
rejected as now. `dev/tests/invariants/specialized_header_lifetimes.rs`
(attached to `directional_inputs.rs`) checks every case.

## 3. Callable sugar in overlap conditions (d3's record)

d3 stopped at core's `impl<const N: usize> MultiCharEq for [char; N]`
(`str/pattern.rs:645`) beside

```rust
impl<F> MultiCharEq for F
where
    F: FnMut(char) -> bool,
```

with "higher-ranked specialization implication is not implemented". Callable
sugar always opens a clause binder, so `F: FnMut<(char,)>` is a
`Clause::ForAll` that binds no region. The overlap inputs
(`sema/traits/implementations/overlap/inputs.rs`) skipped every quantified
condition, so the impls overlapped and specialization was asked to order them.

rustc writes that clause under `Binder::dummy`, and its overlap check
evaluates it like any where clause: `[char; N]: FnMut<(char,)>` is knowable and
has no impl or builtin candidate. The repair lowers such a clause out of its
empty binder with selection's existing `vacuous` helper (now
`pub(in crate::sema)`, re-exported from `sema/traits/selection.rs`) and uses it
as an ordinary condition. A clause whose binder binds a region still proves no
disjointness, which is conservative. Specialization's parent goals are
unchanged.

`pa31/tests/spec/callable-sugar-coherence/` has nine controls, declared before
the oracle and before any candidate run; all nine agree with oracle v2. Oracle
v1 rejected four fixtures for the fixture itself (E0405: the callable
reference impls' `use` named no `Sized`). The one amendment adds `Sized` to that
`use`, with no case body or outcome changed, and was declared before oracle v2
and before any candidate run. The accepted controls are core's four impls with
core's callable traits and reference impls (`core-pattern`), an array, an array
reference, a slice reference, `u8`, a local struct and the explicit
`FnMut<(char,), Output = bool>` guard. The opponents are a second blanket over
`FnMut(u8) -> bool` and `impl<T: ?Sized> MultiCharEq for &T` (each E0119). On
d3's candidate only the explicit guard passed and every opponent was rejected
as now.

### Recorded gap: `[const]` supertrait requirements

`core-pattern`, `array-reference` and `slice-reference` include core's const
callable reference impls, such as

```rust
const impl<A: Tuple, F: ?Sized> Fn<A> for &F
where
    F: [const] Fn<A>,
```

Their `MultiCharEq` impls are now disjoint, but these impls then fail with
"trait implementation does not satisfy the trait's requirements": the
`[const] FnMut<A>` supertrait of `&F` is not proven from the `[const]`
premise. The same happens without callable traits
(`repro/const-super-struct.rs`: `const trait Shared: [const] Once` with
`const impl<F> Shared for W<F> where F: [const] Shared` and the matching
`Once` impl); rustc 1.98.0 accepts both, and the ordinary-premise form passes
here. This check runs after coherence, so it was masked before. Whether core's
own `ops/function.rs` impls hit it is not known: census 11 stops first. Owner
guess: const trait requirement proving
(`sema/declarations/traits/implementations/validation.rs`,
`validate_impl_requirements`, through `nominals::Proof::prove_clause`), Lane B
if it is effect selection. `dev/tests/invariants/vacuous_callable_coherence.rs`
(attached to `inputs.rs`) checks every case and lists the three as gaps.

## 4. The next frontier: the crate-wide overlap work caps (not repaired)

d4 and census 11 stop at "implementation prerequisite work budget exceeded",
on a macro-generated impl at `shift_scalar.rs:802` (`impl_splatted_shifts`).
The repairs above let `validate_trait_implementations` check every overlap
pair, where it used to stop at the first failure. Two fixed caps are
crate-wide and cumulative:

- `Pending::charge` (`traits/implementations/scalar_readiness.rs`): one
  million units for all early scalar implementation work and the pair checks
  it runs;
- the overlap step count shared by `validate_impl_pair`,
  `validate_reference_coherence`, `reservation_pair`, collection conformance
  and `specialization::requested`: one million steps.

Instrumented analysis-only builds (never committed) measured the walk:

- d5 raised the `Pending` cap and the pair check's own cap, and d6 also the
  reference comparison's. Both stopped with "trait implementation overlap work
  budget exceeded" at a derived `PartialEq` (`num/imp/flt2dec/mod.rs:308`).
- d7 raised every overlap cap. The crate pass then ended with "specialization
  implication still awaits a declaration dependency: Some(None)" (unlocated),
  followed by lookahead records 2 and 3 and a new budget record, "source range
  identity query exceeds its work budget" (`num/niche_types.rs:116`).
- d8 stopped right after the crate-wide pair check: 2,404,914 overlap steps
  and 2,466,187 `Pending` units, with no pair failing.

So core's pair work is about 2.4 times the cap. The specification forbids
raising a limit to hide repeated payment. The pairs are already prefiltered by
head and closed shape when they are registered. These totals do not show
whether any of the work is repeated payment; d8's pair count after the walk,
22,895, is not established to be the whole set. This needs a decision
before repair: either show the work is proportional to the admitted impls and
grant the caps per pair or impl, as the macro ledger grants per source byte, or
find the repeated payment. After that, d7's dependency record is next.

## 5. Lookahead records 2, 3 and 5

### Record 2: a feature-bound const parameter type (genuine)

`mem/transmutability.rs:91`, "const parameter type must implement actual
ConstParamTy", in `validate_const_parameter_types`. Core's

```rust
#[unstable_feature_bound(transmutability)]
pub unsafe trait TransmuteFrom<Src, const ASSUME: Assume = { Assume::NOTHING }>
```

needs `Assume: ConstParamTy_` from `#[unstable_feature_bound(transmutability)]
impl ConstParamTy_ for Assume {}` (`transmutability.rs:293`). The draft
`repro/record2-gated.rs` reduces it: rustc 1.98.0 accepts it and this compiler
rejects it with the same message. Without the impl's feature bound
(`record2-plain.rs`) both accept.

rustc's well-formedness check proves the const parameter type in the item's
parameter environment, which carries the trait's own `UnstableFeature` clause.
Here `prove_const_parameter_type`
(`sema/declarations/generics/parameter_kinds.rs`) uses
`Proof::prove_closed_trait`, whose selection caller has no assumptions, so the
impl's feature condition cannot use the trait's feature bound. A repair would
let a closed proof keep the environment's feature clauses, and only those.
Owner guess: Lane A, as the census frontier, in `parameter_kinds.rs` plus the
closed caller in `sema/nominals/traits.rs`. Not repaired: it is not on census
11's path, and the closed-proof change is shared.

### Record 3: derived

`num/imp/dec2flt/common.rs:20` (`impl ByteSlice for [u8]`), "trait
implementation requires conformance and selection semantics", in
`validate_trait_impl_owners`. That check requires a validated impl, and none is
validated when `validate_trait_implementations` fails, so it reports the first
impl in owner order. It recurs in d7. No repro is needed; recheck once the
crate pass passes.

### Record 5: derived

`num/niche_types.rs:122`, "missing completed const argument", in
`complete_alias_normalizations`. It follows record 4 at the same site, "specialization
graph validation previously failed" in `local_annotations`, which names its
cause. The site is `define_valid_range_type!`'s `pub struct
NonZeroU8Inner(pattern_type!(u8 is 1..))`. A repro sketch is a no_core
`pattern_type!(u8 is 1..)` field in a crate whose specialization graph fails.
Owner guess: none until the crate pass passes. In d7 the `local_annotations`
record at that macro became the budget record above, which ends the
lookahead.

## 6. Census 11

The pin `fcd444bc` was built from `d7d0cfc97` with an empty uncommitted patch
(`obj/parallel/census-batch10/qualified-1/`). Its ordinary first error is
"file 352: 802: implementation prerequisite work budget exceeded".

| Phase | Checked | Failed | Blocked |
|---|---|---|---|
| frontend | 0 | 0 | 0 |
| declarations | 42,310 | 0 | 0 |
| bodies | 0 | 0 | 0 |
| completion | 0 | 0 | 0 |

- Causes: one record, category `budget`.
- Lookahead records: none, because budget exhaustion is sticky.
- Largest blocking roots: none.
- Ordinary first error: as above, at `shift_scalar.rs:802`.

The census file is byte-identical to d4's. The census stage was given the
relative output `census`, which it wrote at the worktree root; it was moved
unchanged to `qualified-1/census/`, and the receipt's argv keeps the original
path.

## Checks

All four fastchecks ran on the tree committed as `d7d0cfc97`, docs aside.

| Check | Result | Receipt |
|---|---|---|
| Private: 16 lib cases, strict private Clippy, format, architecture | pass | `obj/qualification/fast/1794117046212253811` |
| `census_mode` and PA1 to PA12 | pass, every PA full | `obj/qualification/fast/1794117087913071056` |
| PA13 to PA19 and PA21 to PA24 | pass, every PA full | `obj/qualification/fast/1794117195769915472` |
| PA20 alone | pass, 1451/1451 | `obj/qualification/fast/1794117299095260042` |
| `make test-upstream` | pass, 127 fixtures, 126 Reference files | `obj/parallel/census-batch10/tmp/test-upstream.log` |
| rustc 1.98.0 oracle | 32/32 agree | `obj/parallel/census-batch10/oracle/` |
| Pin build and ordinary observation | pass | `obj/parallel/census-batch10/qualified-1/receipt-build.json` |
| Census 11 | completed, analysis-only | `obj/parallel/census-batch10/qualified-1/receipt-census.json` |

Three earlier private attempts did not count: a cold build timeout (status
124, `1794116502394544705`), a wrong module path
(`1794116789896209947`) and a `negative_coherence` suite selection whose only
test is native and ignored (`1794116996911770783`).

The 16 private lib cases are the three new invariants; the stability transport
hot-record and derive-owner layout invariants; both negative coherence
invariants; the three callable sugar impl-parameter invariants; the two
specialization selection invariants; the fundamental owner and overlap
invariant; the coherence cycle cutoff test; the user callable output
invariant; and the census lookahead test. No record changed layout.
`make test-harness` was not needed, because no integration root was added.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness and course text.

## Storage

Only the census 11 pin (`qualified-1/pin`) is kept. The census 10 pin
(`obj/parallel/census-batch9/qualified-2/pin`), the d1 to d8 diagnostic pins
and the analysis-only builds in `obj/parallel/census-batch10/dev/` were retired
and pruned; their hashes stay in their receipts. `obj` was 13 GiB before the
pin build, so `make tidy-build` ran once, before it; it is 7.8 GiB now.

## Shared files touched

- Selection: `sema/traits/selection/search/pointers.rs`,
  `search/coherence.rs`, `search/construction_conditions.rs` (section 1),
  `sema/traits/selection.rs` and `selection/instantiation.rs` (the `vacuous`
  re-export, section 3).
- Coherence: `sema/traits/implementations/overlap/inputs.rs` (section 3).
- Specialization: `sema/specialization/directional_inputs.rs` (section 2).
- New files: `dev/tests/invariants/refuted_fn_pointers.rs` (stem `refuted`,
  1 file), `specialized_header_lifetimes.rs` (stem `specialized`, 2 files) and
  `vacuous_callable_coherence.rs` (stem `vacuous`, 1 file), and the three PA31
  control directories.

## Decisions needed

- Section 4: how core's crate-wide overlap work is budgeted (2.4 million
  steps against fixed one-million caps), and who owns it.
- Section 3: ownership of the `[const]` supertrait requirement gap.
- Section 5: whether Lane A repairs record 2 now, which shares the closed proof
  caller in `sema/nominals/traits.rs`.
