# Census batch 12: `[const]` bounds in `min_specialization`, `[const]` supertraits and the thirteenth census

Census 12 ([batch 11](census-batch11.md), pin `a108ae93`) checked 42,310
declarations of original core with none failed or blocked. Its crate record,
"minimal specialization introduces a non-marker predicate", is core's
`SpecNextChunk` pair (`array/mod.rs:1044` and `1062`). This batch, on main
`ad05f4d61`:

- compares `min_specialization` clauses as rustc's `predicates_of` holds them:
  a `[const] Tr` bound is the ordinary `Tr` clause, and a `const Tr` bound is
  `Tr` plus a host-effect clause. This repairs census 12's record (section 1);
- proves a const impl's `[const]` supertraits and where clauses as `[const]`
  goals, as rustc's wfcheck does. This closes batch 10's `[const]` supertrait
  gap (section 2);
- runs census 13 and reduces its record (section 3).

Both repairs follow the same model: constness lives in const conditions, not
in ordinary clauses.

Census 13 (`1c0219e9`, from `9fb0e2a7d`) checks 42,310 declarations, with
none failed or blocked. Its crate record is "source Copy target is outside the
supported implementation profile" at core's `impl Copy for ! {}`
(`marker.rs:484`), then 7 lookahead records. The `SpecNextChunk` record is
gone. The new record is reduced, with a proposed repair, and not repaired
(section 3).

Out of scope:

- Lane B owns `sema/bodies/field_storage.rs`, `finalization.rs`,
  `types/variance/*` and `types/aggregate/instances.rs`. Lane C owns transport
  and `lifetimes.rs`. This batch touches none of them.
- The static `Sync` check in `sema/declarations/validation.rs` and
  `constrained_scalars.rs` did not change.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend is frozen.
- Census 13's record is located and reduced, not repaired (section 3).

## Method

Each PA31 control was declared before its frozen rustc 1.98.0 oracle
(`obj/parallel/census-batch12/oracle/`). Only then were the controls run with
the census 12 pin, and then with the candidate. Exploration drafts in
`obj/parallel/census-batch12/repro/` were run with rustc 1.98.0 and a census
pin; they are not declared controls.

## 1. `[const]` bounds in `min_specialization` (census 12's record)

The parent is

```rust
const impl<I: [const] Iterator<Item = T>, T, const N: usize> SpecNextChunk<T, N> for I
```

The child adds `TrustedLen`, which is `const unsafe trait TrustedLen: [const]
Iterator` with `#[rustc_unsafe_specialization_marker]`. The child's ordinary
`I: TrustedLen` bound elaborates to the ordinary `I: Iterator`. The parent's
basis held only `I: [const] Iterator`, so minimal admission found a
non-marker clause.

How rustc 1.98 handles it:

- Bound lowering (`rustc_hir_analysis/src/hir_ty_lowering/mod.rs`) puts a
  `[const] Tr` bound in `predicates_of` as the ordinary `Tr` clause. Its
  constness goes only to `const_conditions`. A `const Tr` bound gives `Tr`
  plus a `HostEffect(Const)` clause.
- `check_predicates` (`impl_wf_check/min_specialization.rs`) compares the
  elaborated `predicates_of`. `check_specialization_on` allows only a trait
  clause on a marker trait. Any other clause, a host-effect clause included,
  is "cannot specialize on predicate" unless it is global.
- `specializes` (`rustc_trait_selection/src/traits/specialize/mod.rs`) checks
  constness. A conditionally const parent needs a conditionally const child.
  The parent's const conditions must hold as `[const]` goals under the child's
  param-env.

The repair (`6776a31b9`):

- `sema/specialization/predicate_basis.rs`: `Basis::insert` stores a
  `[const] Tr` clause as `Tr`. It stores a `const Tr` clause as `Tr` and also
  as itself, which stands for rustc's host-effect clause, so it still
  elaborates its `const` supertraits.
- `sema/specialization/minimal_admission.rs`: in `additional`, only an
  ordinary trait clause gets the marker allowance. A `const` marker bound's own
  clause is a host-effect clause and is never a marker.
- `callee_conditions.rs` and the implication step are unchanged. The ordering
  already proves constness as `specializes` does.

Controls: `pa31/tests/spec/const-specialization-bounds/` (7 cases, 7/7
agree), checked by `dev/tests/invariants/host_effect_specialization.rs`.
`rejected_overlap_pairs.rs` now audits this packet too, the thirteenth: no
rejected pair unifies.

| Case | rustc 1.98.0 | Census 12 pin | Candidate |
|---|---|---|---|
| `core-next-chunk` | accept | census message | accept |
| `plain-marker-supertrait` (`TrustedLen: Iterator`) | accept | census message | accept |
| `child-only-const` (ordinary parent, `[const]` child) | accept | census message | accept |
| `ordinary-pair` | accept | accept | accept |
| `child-not-const` (opponent) | E0119 | no specialization ordering | no specialization ordering |
| `child-weaker-const` (opponent) | E0119 | no specialization ordering | no specialization ordering |
| `const-marker-bound` (opponent) | rejected | accept | non-marker predicate |

The coordinator's stop condition held: `child-not-const` and
`child-weaker-const` stay rejected, so the comparison does not lose
constness. The census 12 pin wrongly accepted `const-marker-bound`; the
candidate rejects it, as rustc does. The exploration drafts of batch 11
(`census-batch11/repro/maybe-const/`) had other bytes.

## 2. `[const]` supertraits of const impls (batch 10's gap)

Batch 10 (section 3) recorded that core's const callable reference impls,
such as `const impl<A: Tuple, F: ?Sized> Fn<A> for &F where F: [const] Fn<A>`,
did not prove `&F: [const] FnMut<A>` from their `[const]` premise. Three
callable sugar controls were kept as gaps.

It is the same split as section 1. `impl_trait_reference`
(`sema/declarations/traits/implementations.rs`) makes a `const impl` header
`Bound::ConstTrait`, and `impl_contract_obligations`
instantiated the trait's requirements under that effect. A `[const] FnMut`
supertrait therefore became a `const FnMut` goal, which the impl's `[const]`
premises cannot prove. rustc's `check_impl` (`check/wfcheck.rs`, lines 1416
to 1437) proves the trait's `const_conditions` as `[const]`
(`BoundConstness::Maybe`) host-effect goals. `param_env`
(`rustc_ty_utils/src/ty.rs`, line 192) adds the impl's own const conditions as
`[const]` assumptions, which elaborate to `[const]` supertraits.

The repair (`9fb0e2a7d`): `impl_contract_obligations`
(`sema/declarations/traits/implementations/validation.rs`) computes a const
impl's requirements under `[const]` instead of `const`. Plain supertraits stay
ordinary. `validate_impl_requirements` still receives the impl's own
predicate.

Controls: `pa31/tests/spec/const-supertrait-requirements/` (7 cases, 7/7
agree), checked by `dev/tests/invariants/supertrait_effects.rs`.

| Case | rustc 1.98.0 | Census 12 pin | Candidate |
|---|---|---|---|
| `shared-premise` (`Once` impl over `F: [const] Shared`) | accept | requirements not satisfied | accept |
| `once-premise` (over `F: [const] Once`, by elaboration) | accept | requirements not satisfied | accept |
| `plain-supertrait` | accept | accept | accept |
| `ordinary-impls` | accept | accept | accept |
| `ordinary-supertrait-impl` (opponent) | E0277 | requirements not satisfied | requirements not satisfied |
| `ordinary-premise` (opponent) | E0277 | requirements not satisfied | requirements not satisfied |
| `const-premise-required` (opponent) | E0277 | requirements not satisfied | requirements not satisfied |

The coordinator's condition held: a non-const supertrait impl under a const
impl stays rejected (`ordinary-supertrait-impl`). A `[const]` goal still needs
a const impl or a `[const]` assumption (`ordinary-premise`), and a `[const]`
assumption does not prove a `const` premise (`const-premise-required`).

Batch 10's three gaps in `pa31/tests/spec/callable-sugar-coherence/`
(`core-pattern`, `array-reference` and `slice-reference`) now pass.
`dev/tests/invariants/vacuous_callable_coherence.rs` moves them from its gap
list into its accepted cases, with no other change.

## 3. Census 13

The pin `1c0219e9` was built from `9fb0e2a7d` on main `ad05f4d61` with an
empty uncommitted patch. Its ordinary observation stops at "file 81: 17247:
source Copy target is outside the supported implementation profile", with
375 sources opened. The census
(`obj/parallel/census-batch12/qualified-1/census/`) checks 42,310
declarations of original core with none failed or blocked, in 61.6 seconds.
It has 8 records: one crate record, then 7 lookahead records.

| # | Pass | Site | Message |
|---|---|---|---|
| 1 | `validate_trait_implementations` | `marker.rs:484` | source Copy target is outside the supported implementation profile |
| 2 | `validate_trait_impl_owners` | `mem/maybe_uninit.rs:1659` | trait implementation requires conformance and selection semantics |
| 3 | `local_annotations` | unlocated | implementation validation is outside the early scalar profile |
| 4 | `complete_alias_normalizations` | unlocated | implementation validation is outside the early scalar profile |
| 5 | `validate_objects` | `any.rs:152` | source receiver compatibility requires the completed immutable impl epoch |
| 6 | `complete_discriminants` | `mem/alignment.rs:440` | associated type requires an unambiguous validated implementation |
| 7 | `complete_layouts` | `ascii.rs:29` | enum layout requires completed discriminants |
| 8 | `require_specialization_execution` | unlocated | specialization verification is unfinished |

Census 12's record 1 is gone, and so are its records 3, 4 and 8, which
followed from the failed specialization graph. Records 3 and 4 now follow from
the failed impl validation, and record 8 is "specialization verification is
unfinished", as in batch 11's d7. Record 2 has moved to `impl<T: TrivialClone> SpecFill<T> for
[MaybeUninit<T>]`. Records 5, 6 and 7 are unchanged from census 12. Records 2
to 8 each name the failed impl validation or a pass that depends on it. They
should be rechecked once record 1 is repaired; none was investigated further.

### Record 1: `impl Copy for !` (genuine)

Core's `marker.rs:484` is `impl Copy for ! {}`, under the `never_type`
feature. `validate_source_copy_impl` takes the Copy target's field types from
`source_copy_fields` (`sema/declarations/traits/implementations/copying.rs`,
lines 54 to 107), a `match` on the target's kind. It handles ADTs, primitives, raw pointers, shared references, functions,
arrays and tuples, and rejects unique references. Any other kind, the never
type included, reaches the last arm with the census message. rustc 1.98's
`type_allowed_to_implement_copy` (`rustc_trait_selection/src/traits/misc.rs`,
line 61) accepts `ty::Never` with the other scalars, without fields.

Exploration drafts in `repro/never-copy/` (not declared controls), with the
`function-pointers` foundation prefix and `never_type`:

- `never-copy.rs`: `impl Clone for !` and `impl Copy for !`. rustc 1.98.0
  accepts it; the census 13 pin rejects it with record 1's message.
- `never-clone-match.rs`: the `Clone` impl alone, whose body matches on
  `*self`. Both accept it, so the rejection is the `Copy` impl.
- `never-clone-only.rs`: the `Clone` impl alone with core's body `*self`.
  rustc rejects it (E0507), because without the `Copy` impl `*self` moves out
  of a reference; the pin accepts it under `--emit-types`. It is kept only as
  the first draft.

Proposed repair (owner guess: Lane A, `copying.rs`): the never type joins the
arm with no field types, as in rustc. That needs declared controls, including
an opponent without a `Clone` impl, and another census. Not repaired here.

## Checks

Every check below ran on the tree committed as `9fb0e2a7d`, docs aside.

| Check | Result | Receipt |
|---|---|---|
| Private: 165 lib cases, strict private Clippy, format, architecture | pass | `obj/qualification/fast/1794134667557486267` |
| `census_mode` (10) and PA1 to PA12: 1,885 of 1,885 | pass | `obj/qualification/fast/1794134711743197103` |
| PA13 to PA19 and PA21 to PA24: 5,434 of 5,434 | pass | `obj/qualification/fast/1794134824832550439` |
| PA20: 1,451 of 1,451 | pass | `obj/qualification/fast/1794134900141871134` |
| `make test-upstream` | pass, 128 fixtures, 126 Reference files | `obj/parallel/census-batch12/tmp/test-upstream.log` |
| rustc 1.98.0 oracle | 14/14 agree | `obj/parallel/census-batch12/oracle/` |
| Pin build and ordinary observation | passed observation, `1c0219e9` | `obj/parallel/census-batch12/qualified-1/receipt-build.json` |
| Census 13 | completed: 42,310 checked, 0 failed, 0 blocked, 8 records | `obj/parallel/census-batch12/qualified-1/receipt-census.json` |

The 165 private lib cases are:

- every test in an invariant file that uses `const trait`, `[const]` or
  `const impl`: 129 tests, including the two new tests
  (`host_effect_specialization.rs` and `supertrait_effects.rs`) and the
  callable sugar coherence test;
- batch 11's 39 cases, 36 of them not already selected. They repeat the
  specialization, coherence, selection and const parameter invariants.

Earlier focused runs during development (`1794134218899865115` for section 1
and `1794134473809098186` for section 2) do not count toward the pin. They
passed after admission retries. The first PA20 qualification attempt was
refused a lease while a guarded rustc oracle job of mine held one; it was
rerun. `make test-harness` was not needed, because no integration root was
added.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness and course text.

## Storage

- Kept: the census 13 pin `obj/parallel/census-batch12/qualified-1/pin/`
  (`1c0219e9`, with `.keep-artifacts`).
- Retired by removing `.keep-artifacts` and pruning its directory: census 12's
  pin (`census-batch11/qualified-3`, `a108ae93`). Its hash is in
  `obj/parallel/census-batch12/retired-pins.sha256`. Census 12's records,
  manifest and receipts remain.
- `make tidy` ran after the census; obj stayed under its 16 GiB cap.

## Shared files touched

- Specialization: `sema/specialization/predicate_basis.rs` and
  `minimal_admission.rs` (section 1).
- Declarations: `sema/declarations/traits/implementations/validation.rs`
  (section 2).
- Invariants: the new `host_effect_specialization.rs` (stem `host`, 5 files)
  and `supertrait_effects.rs` (stem `supertrait`, 3 files);
  `rejected_overlap_pairs.rs` audits the new section 1 packet, and
  `vacuous_callable_coherence.rs` has no gap list.
- PA31: the two control directories, `pa31/tests/README.md` and the
  callable sugar coherence README. `spec.md` has a paragraph for each section,
  and its callable sugar sentence no longer names a gap.

## Decisions needed

- Record 1: the proposed repair in section 3, the never type as a Copy target
  without fields, is next for Lane A unless it is assigned elsewhere.

## Commits

On main `ad05f4d61`, in order: `6776a31b9` (section 1), `9fb0e2a7d` (section
2), then this document and the tracker entry.
