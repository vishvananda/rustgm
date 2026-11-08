# Census batch 14: defaulted impl trait arguments and the fifteenth census

Census 14 ([batch 13](census-batch13.md), pin `188a5350`) checked 42,310
declarations of original core, with none failed or blocked. Its crate record,
"trait implementation does not satisfy the trait's requirements", is core's
`const impl<T> Try for NeverShortCircuit<T>` (`ops/try_trait.rs:433`). Its
`const impl<T> FromResidual for NeverShortCircuit<T>` (line 448) leaves
`FromResidual`'s argument to its default, `<Self as Try>::Residual`. This
batch, on main `4a1ac00c4`:

- normalizes an impl trait reference argument left to its default by the
  routine a written argument uses, before the header is published. This
  repairs census 14's record (section 1);
- records one control that rustc accepts as a gap: a default naming another
  trait's associated type is still rejected by name, because the default's
  alias requirement is instantiated without the trait's `Self` (section 2);
- proves that census 15's new crate record, core's `TrustedLen for
  Flatten<I>` (`iter/adapters/flatten.rs:313`), was hidden behind census 14's
  record rather than caused by the repair (section 3);
- runs census 15 (section 4).

Census 15 (`6df12612`, from `d1756b87f`) checks 42,310 declarations, with none
failed or blocked. Its crate record is "invalid canonical trait selection" at
`flatten.rs:313`, then the same 7 lookahead records as census 14. The
`FromResidual` record is gone. The new record is not reduced here; it is
batch 15's first item.

Out of scope:

- Lane B owns `sema/bodies/field_storage.rs`, `finalization.rs`,
  `types/variance/*`, `types/aggregate/instances.rs` and `lifetimes.rs`, and
  Lane C owns transport. This batch touches none of them.
- The static `Sync` check in `sema/declarations/validation.rs` and
  `constrained_scalars.rs` did not change.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend is frozen.
- The gap in section 2 is located and recorded, not repaired. Its repair, (a)
  below, is approved for batch 15 item 2.

## Method

Each PA31 control was declared before its frozen rustc 1.98.0 oracle
(`obj/parallel/census-batch14/oracle/`, declaration sha `6980afc4`). Only then
were the controls run with the census 14 pin, and then with the candidate.
Exploration drafts in `obj/parallel/census-batch14/repro/fill-alias/` were run
with rustc 1.98.0 and both pins; they are not declared controls.

The candidate runs used the analysis-only diagnostic `d1-defaults`
(`b3add77a`), built from the uncommitted repair. Its `dev/src` patch is the
same as `d1756b87f`'s source diff, apart from blob index lines. The committed
tree's private invariant checks every control (section 5).

## 1. Defaulted impl trait arguments (census 14's record)

A written trait argument in an impl header has its own source node.
`begin_trait_reference` (`impl_headers/continuation.rs`) queues it as a path
argument, and `complete_projection_path` normalizes it under the impl's
context, storing the normal form in the node table and the written form in
`facts.unreduced`. An omitted argument is filled in `application_arguments`
(`generics/arguments.rs`) with the parameter's default, and `applied_trait`
(`trait_paths.rs`) replaces `Self` with the impl's type. It has no node, so it
was never normalized. The header was published with `<NeverShortCircuit<T> as
Try>::Residual` where `Try`'s supertrait goal has `NeverShortCircuitResidual`.

rustc fills an omitted type argument with its default instantiated by the
preceding arguments, `Self` included (`hir_ty_lowering/mod.rs`, line 780).
`check_impl` (`check/wfcheck.rs`, line 1387) deeply normalizes the impl's
trait reference and proves it well-formed, including the parameter's bounds,
and `match_impl` (`traits/select/mod.rs`, line 2545) normalizes the
instantiated reference before equating it with a goal.

The repair (`d1756b87f`) follows the approved design:

- `projections::RootKey` is `Node(NodeKey)` or `Argument { implementation,
  index, site }`. An argument root is identified by its impl and argument
  index; `site`, the impl's trait path, only locates diagnostics. The
  projection queries (`roots`), size rechecks, kept headers and the
  `ProjectionSelection` and `ProjectionResult` work items are keyed by it.
- `normalize_root` is the one entry point: `normalize_annotation` passes
  `RootKey::Node`, and the impl header passes `RootKey::Argument`. Both run the
  same scoped normalization, with no defaulted-only branch. A finished root is
  stored by `store_root`: a node in the node table with its written form in
  `facts.unreduced`, an argument in the pending impl header beside its default.
- A new `Phase::Defaults` runs after `Phase::Reference` and before
  `publish_trait_header`. Each type argument at an index past the path's
  supplied arguments, other than in call-syntax paths and auto traits, starts
  an argument root under the impl's context. The pending header keeps
  `(impl, index) -> (default, normal form)`. When every root has finished, the
  reference is rebuilt from the normal forms, interned with the same
  constness and published. A missing or doubled normal form is an error.
  The reference stays active until then, as it does while written arguments
  normalize.
- The rule that keeps a header projection on its own `Self` as written
  applies unchanged. It is never true for an impl trait reference, because
  the impl schema is declared before its arguments are.
- Rejected, as the coordinator decided: a synthetic source node for the
  default.

Cache audit for the new key:

- Tables keyed by `RootKey` (queries, rechecks, kept headers, work items)
  separate the variants, so an argument root cannot collide with a node root.
- Node-keyed tables (node types, `facts.unreduced` nodes, `raw_bound_nodes`)
  are reached only by `RootKey::Node`. `store_root` dispatches by variant, and
  an argument root is never raw-bound.
- Proofs retained for an argument root run under the impl's context, so they
  join the per-binder `projections.scopes`; `site` is only their error source.
  Body reservations, quantified proofs, body regions and
  `constant_annotation_bodies` need a local or constant position, which an
  argument root never has.

Controls: `pa31/tests/spec/defaulted-impl-arguments/` (8 cases, 8/8 agree),
checked by `dev/tests/invariants/defaulted_impl_arguments.rs` (stem
`defaulted`, new). The prefix is core's `#![feature(const_precise_live_drops)]`
and the `function-pointers` foundation.

| Case | rustc 1.98.0 | Census 14 pin | Candidate |
|---|---|---|---|
| `default-omitted` (`impl FromResidual for Short`) | accept | requirements not satisfied | accept |
| `default-written` (`FromResidual<<Short as Try>::Residual>`) | accept | accept | accept |
| `default-other-member` (`R = <Self as Try>::Output`) | accept | requirements not satisfied | accept |
| `default-self-bounds` (omitted, `where Self: Sized`) | accept | requirements not satisfied | accept |
| `core-never-short-circuit` (core's const generic shape) | accept | requirements not satisfied | accept |
| `default-bound-satisfied` (twin, gap) | accept | unambiguous implementation | unambiguous implementation |
| `default-bound-unsatisfied` (opponent) | E0277 | requirements not satisfied | requirements not satisfied |
| `default-scaling` (3 impls, each selected through 2 traits) | accept | requirements not satisfied | accept |

The twin and the opponent declare `Fill<R: Marker = <Self as Source>::Item>`.
The twin's `Item` is `Marked: Marker`; the opponent's is `Plain`, which is not.

The scaling invariant builds `default-scaling.rs`'s shape (it checks that
`scaling(3, 2)` is the control's exact bytes) at 4 and 16 impls with 1 and 4
uses each. Every size normalizes exactly one argument root per impl, however
many traits select it, and quadrupling either the impls or the uses at most
sextuples the trait selections and canonical steps.

## 2. The recorded gap: a default naming another trait's member

`default-bound-satisfied` is accepted by rustc and rejected here by
`complete_alias_normalizations`, "associated type requires an unambiguous
implementation". As the coordinator decided, it is committed as a recorded
gap (`GAPS` in the invariant, and the packet's README) rather than blocking
the repair.

The cause is a second divergence, independent of the first. Every default a
path applies is also retained as an alias requirement: `application_arguments`
calls `alias_use` for it, and `alias_requirements::expand` instantiates the
requirement through `Bound::joint` over the owner's listed parameters only. A
trait's `Self` is not listed, so `<Self as Source>::Item` keeps a free `Self`
and has no unambiguous impl.

Exploration drafts (`repro/fill-alias/`), with the same prefix:

| Draft | rustc 1.98.0 | Census 14 pin | Candidate |
|---|---|---|---|
| `written-argument.rs`: `impl Fill<Marked> for Short` | accept | accept | accept |
| `written-default.rs`: `impl Fill<<Short as Source>::Item> for Short` | accept | accept | accept |
| `bounded-default-unused.rs`: the traits, no `Fill` impl | accept | accept | accept |
| `unbounded-default.rs`: `Fill<R = <Self as Source>::Item>`, omitted | accept | unambiguous implementation | unambiguous implementation |
| `try-no-supertrait.rs`: `default-omitted` without `Try: FromResidual` | accept | unambiguous implementation | unambiguous implementation |
| `source-supertrait.rs`: the twin with `Source: Fill` | E0277 | trait argument does not satisfy its declaration requirements | the same |

The divergence needs neither the bound nor this batch's repair: it appears
whenever an impl omits a default that names another trait's member. Core's
`Try: FromResidual` supertrait hides it: `default-omitted` passes and
`try-no-supertrait` does not. Why the supertrait hides it is not explained
yet; it belongs in batch 15's locate report. A supertrait cannot make the twin
pass, since rustc rejects `Source: Fill` (E0277, `Item: Marker` not proved).

Approved for batch 15 item 2, as (a): instantiate trait-path defaults with
`Self` set to the path's subject through one shared substitution in
`alias_requirements::expand` or `Bound::joint`, with all three callers
passing the subject (impl trait references, bound trait paths at
`bounds.rs:658`, and qualified projection prefixes at `projections.rs:188`),
controls for each caller declared before the oracle, and a locate report
before code.

## 3. Census 15's record 1 was hidden, not caused

Census 15's crate record moved to `flatten.rs:313`:

```rust
unsafe impl<I> TrustedLen for Flatten<I>
where
    I: Iterator<Item: IntoIterator>,
    FlattenCompat<I, <I::Item as IntoIterator>::IntoIter>: TrustedLen,
{
}
```

As the coordinator asked, the cause was proved before qualifying.
`try_trait.rs:448` is the only impl in core that leaves a projection default
to its path, so the repair changes nothing else in core. The discriminating
runs used the census 14 pin `188a5350`, without the repair, over two copies of
original core with line 448 written out (`discriminate/`, analysis-only,
census 14's exact argv):

| Run | Pin | Core | Records | Record 1 |
|---|---|---|---|---|
| A/A | `188a5350` | original | 8, the same as census 14 | `try_trait.rs:433` |
| Written projection | `188a5350` | `FromResidual<<NeverShortCircuit<T> as Try>::Residual>` | 8, the same as census 15 | `flatten.rs:313` |
| Written normal form | `188a5350` | `FromResidual<NeverShortCircuitResidual>` | 8, the same as census 15 | `flatten.rs:313` |
| `d1-defaults` | `b3add77a` (repair) | original | 8, the same as census 15 | `flatten.rs:313` |

Each run checks 42,310 declarations, with none failed or blocked. Without the
repair, writing the argument out leads to the same Flatten record, so the
record is uncovered: `validate_trait_implementations` stops at its first
failed impl, and the Flatten impl came after `NeverShortCircuit` in
conformance order. Each patched copy changes only line 448, in its own file
(link count 1); the original core's hashes were verified unchanged.

## 4. Census 15

The pin `6df12612` was built from `d1756b87f` on main `4a1ac00c4` with an
empty uncommitted patch. Its ordinary observation stops at "file 142: 8198:
invalid canonical trait selection", with 375 sources opened. The census
(`obj/parallel/census-batch14/qualified-1/census/`) checks 42,310
declarations of original core, with none failed or blocked, in 62.9 seconds.
It has 8 records: one crate record, then 7 lookahead records, the same as the
`d1-defaults` diagnostic.

| # | Pass | Site | Message |
|---|---|---|---|
| 1 | `validate_trait_implementations` | `iter/adapters/flatten.rs:313` | invalid canonical trait selection |
| 2 | `validate_trait_impl_owners` | `mem/maybe_uninit.rs:1659` | trait implementation requires conformance and selection semantics |
| 3 | `local_annotations` | unlocated | implementation validation is outside the early scalar profile |
| 4 | `complete_alias_normalizations` | unlocated | implementation validation is outside the early scalar profile |
| 5 | `validate_objects` | `any.rs:152` | source receiver compatibility requires the completed immutable impl epoch |
| 6 | `complete_discriminants` | `mem/alignment.rs:440` | associated type requires an unambiguous validated implementation |
| 7 | `complete_layouts` | `ascii.rs:29` | enum layout requires completed discriminants |
| 8 | `require_specialization_execution` | unlocated | specialization verification is unfinished |

Census 14's record 1 is gone. Records 2 to 8 keep their passes, sites and
messages. Record 2, `SpecFill`, is derived from the failed impl validation
([batch 13](census-batch13.md), section 3); it stays while a failure precedes
it in conformance order. Records 3 to 8 name the failed impl validation or a
pass that depends on it. Record 1 is genuine and next.

## 5. Checks

Every check below ran on the tree committed as `d1756b87f`, docs aside.

| Check | Result | Receipt |
|---|---|---|
| Private: 348 lib cases, strict private Clippy, format, architecture | pass | `obj/qualification/fast/1794148972808893920` |
| `census_mode` (10) and PA1 to PA12: 1,885 of 1,885 | pass | `obj/qualification/fast/1794148088743681606` |
| PA13 to PA19 and PA21 to PA24: 5,434 of 5,434 | pass | `obj/qualification/fast/1794147591371428640` |
| PA20: 1,451 of 1,451 | pass | `obj/qualification/fast/1794147674542329385` |
| Runtime size assertions (9 lib cases, below) | pass | `obj/qualification/fast/1794149550494815286` |
| `make test-upstream` | pass, 128 fixtures, 126 Reference files | `obj/parallel/census-batch14/tmp/test-upstream.log` |
| rustc 1.98.0 oracle | 8/8 agree | `obj/parallel/census-batch14/oracle/` |
| Diagnostic `d1-defaults` | completed analysis-only, 8 records | `obj/parallel/census-batch14/qualified-1/d1-defaults/receipt-diagnostic.json` |
| Discriminating runs (section 3) | 3 runs, 8 records each | `obj/parallel/census-batch14/discriminate/` |
| Pin build and ordinary observation | passed observation, `6df12612` | `obj/parallel/census-batch14/qualified-1/receipt-build.json` |
| Census 15 | completed: 42,310 checked, 0 failed, 0 blocked, 8 records | `obj/parallel/census-batch14/qualified-1/receipt-census.json` |

The 348 private lib cases are:

- batch 13's 251 joined cases;
- item 1's 121 cases, 97 of them not already selected: the two tests of the
  new invariant, and the tests of the invariant files on impl headers,
  projections, defaults, associated types, trait implementations and
  selection.

`make tidy-build` ran before qualification, with obj at 15.2 GiB, so the
first stages ran cold. The joined stage exhausted fastcheck's budget twice in
`clippy-private` (`1794147032452480199`, `1794147750975433874`), and PA1 to
PA12 passed its suites but exhausted the wall-clock budget at 282 seconds
(`1794147308945991805`). None of those counts. The private Clippy cache was
then warmed by fastcheck's exact `clippy-private` command and environment,
run once outside the budget under the storage wrapper
(`warm-private-lint.log`), after which `clippy-private` took 0.12 seconds and
both stages passed. An earlier focused run during development
(`1794145865796503685`) failed as expected on `default-bound-satisfied`,
before it was declared a gap, and does not count.

`make test-harness` was not needed, because no integration root was added.
`make test-backend` was not needed, because the backend contract and bridge
did not change.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness and course text.

### Size assertions

As the coordinator asked at join time, every `size_of` assertion over the
declarations `Checker` and over every struct that encloses it by value was
grepped, summed spellings included
(`obj/parallel/census-batch14/size-assertions.txt`). The repair adds two
hash maps to the pending impl headers inside the `Checker`, and changes the
key of three projection tables and two work items.

- The declarations `Checker` (`sema/declarations.rs:209`) is private, and no
  struct holds it by value. `constants.rs:194` holds `bodies::Checker`.
- Every `size_of::<Checker<'_>>()` over it is printed, never asserted:
  `declarations.rs:382`, `execution.rs:253`, and the tests
  `transparent_contracts.rs:441`, `transparent_naming.rs:67`,
  `provider_execution.rs:10`, `sizing_alias_contracts.rs:262` and
  `early_conformance.rs:327`.
- `-Zprint-type-sizes` shows the declarations `Work` staying at 80 bytes,
  with `RootKey` at 16 and `Request` at 32. No assertion names this `Work`;
  `named_owners.rs:283` (`Work == 64`) is `constants::Work`, and
  `waiting_headers.rs:58` and `prefix_arguments.rs:202` are body work items.
- The summed assertion at `collection_declarations.rs:316` (`7048 +
  size_of::<Vec<()>>() + ...`) covers `sema::model::Declarations`, which does
  not change. `sharing_sizing.rs:347` (`656 + size_of::<Vec<usize>>()`) is the
  selection search `Query`, not the projection `Query`.
- `provider_execution.rs:6` (`Phase == bool`) is `execution::Phase`, not the
  impl header phase. `staging_continuations.rs`' compile-time 96-byte checks
  are `generics::bindings` pending states, which this batch does not change.
- The remaining runtime assertions, on `prototype_headers.rs`'
  `Declarations`, `metavalue_witnesses.rs`' `Work` and `endpoint_origins.rs`'
  `gate_sites::Declarations`, name types this batch does not touch.

The 9 lib cases holding these runtime assertions pass on the committed tree:
`collection_layout_preserves_scalar_owners_and_padded_derive_entries`,
`unrelated_builtin_does_not_acquire_sizing_sharing_and_hot_layouts_stay_fixed`,
`named_width_and_depth_charge_linear_stack_work_without_pattern_allocation`,
`preparation_phase_retains_boolean_storage`,
`registry_retains_distinct_identities_and_shared_finite_profiles`,
`suspended_candidate_headers_keep_linear_work_and_existing_hot_widths`,
`prefix_storage_preserves_compact_public_records`,
`attribute_work_is_sparse_and_scales_with_demanded_values` and
`sparse_endpoint_rows_scale_without_growing_ordinary_nodes_or_bodies`.
Every compile-time `const _` size check is compiled by each private run.

## Storage

- Kept: the census 15 pin `obj/parallel/census-batch14/qualified-1/pin/`
  (`6df12612`, with `.keep-artifacts`).
- Retired by removing `.keep-artifacts` and pruning their directories: the
  `d1-defaults` diagnostic pin (`b3add77a`), after section 3's runs, and census
  14's pin (`census-batch13/qualified-1`, `188a5350`), after census 15's pin was
  built. Their hashes are in `obj/parallel/census-batch14/retired-pins.sha256`.
  Census 14's records, manifest and receipts remain.
- `make tidy-build` ran before qualification (15.2 GiB to 6.1 GiB). `make
  tidy` ran after the census; obj is 10.8 GiB, under its 16 GiB cap.

## Shared files touched

- Declarations: `sema/declarations.rs` (the two work items), `projections.rs`
  and its `selection`, `scoped`, `scoped/proofs`, `deferred`, `environments`
  and `bound_environments` modules (the root key), and
  `impl_headers/continuation.rs` (the defaults phase).
- Invariants: the new `defaulted_impl_arguments.rs` (stem `defaulted`, 1 file).
- PA31: the `defaulted-impl-arguments` packet and an entry in
  `pa31/tests/README.md`. No existing PA31 declaration or fixture changed.
- `spec.md` has a paragraph on defaulted impl trait arguments, naming the gap.

## Decisions needed

- None blocking. The batch 15 order is the coordinator's: census 15's record
  1 (`TrustedLen for Flatten<I>`, the frontier) first, then (a), the
  default's `Self`, with a locate report before code that explains why
  `Try: FromResidual` hid it.
- The doc was committed after census 15's pin was built, not before; the
  pin's source is `d1756b87f`, and the doc changes no source.

## Commits

On main `4a1ac00c4`, in order: `d1756b87f` (sections 1 and 2), then this
document, the tracker entry and the census plan's batch 14 decisions.
