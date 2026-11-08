# Census batch 13: Copy impl targets, rustc witnesses for six PA31 fixtures and the fourteenth census

Census 13 ([batch 12](census-batch12.md), pin `1c0219e9`) checked 42,310
declarations of original core with none failed or blocked. Its crate record,
"source Copy target is outside the supported implementation profile", is
core's `impl Copy for ! {}` (`marker.rs:484`). This batch, on main
`825f443a8`:

- accepts a source Copy impl for exactly the targets rustc 1.98 allows without
  fields, the never type and arrays included, and rejects `str`. This repairs
  census 13's record (section 1);
- records rustc 1.98.0's witnesses for six superseded PA31 fixtures whose
  written tuple and fn pointer Copy impls rustc rejects. The records are
  additive; every declaration and fixture keeps its bytes (section 2);
- shows that census 13's record 2, core's `SpecFill` child at
  `maybe_uninit.rs:1659`, is derived from the failed impl validation, not a
  blocker of its own (section 3);
- runs census 14 and reduces its record (section 4).

Census 14 (`188a5350`, from `7ca25fb64`) checks 42,310 declarations, with
none failed or blocked. Its crate record is "trait implementation does not
satisfy the trait's requirements" at core's `const impl<T> Try for
NeverShortCircuit<T>` (`ops/try_trait.rs:433`), then 7 lookahead records. The
never type record is gone. The new record is reduced, with a proposed repair,
and not repaired (section 4).

Out of scope:

- Lane B owns `sema/bodies/field_storage.rs`, `finalization.rs`,
  `types/variance/*`, `types/aggregate/instances.rs` and `lifetimes.rs`, and
  Lane C owns transport. This batch touches none of them.
- The static `Sync` check in `sema/declarations/validation.rs` and
  `constrained_scalars.rs` did not change.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend is frozen.
- Main's red native test `source_needs_drop_scaling_native_answers` is Lane
  B's bisection; this batch did not run or chase it.
- Census 14's record is located and reduced, not repaired (section 4).

## Method

Each PA31 control was declared before its frozen rustc 1.98.0 oracle
(`obj/parallel/census-batch13/oracle/`). Only then were the controls run with
the census 13 pin, and then with the candidate. Exploration drafts in
`obj/parallel/census-batch13/repro/` and `rustc-check/drafts/` were run with
rustc 1.98.0 and a pin; they are not declared controls.

## 1. Copy impl targets (census 13's record)

rustc 1.98's `type_allowed_to_implement_copy`
(`rustc_trait_selection/src/traits/misc.rs`, lines 53 to 67) accepts these
targets without fields: integers, floats, `bool`, `char`, raw pointers, `!`,
shared references and arrays. It does not check an array's element. An ADT
target must have Copy fields and no destructor. Every other target is "not an
ADT", which `visit_implementation_of_copy`
(`rustc_hir_analysis/src/coherence/builtin.rs`) reports as E0206. Tuples, fn
pointers, fn items and closures have builtin Copy and Clone
(`copy_clone_conditions`, `traits/select/mod.rs`), so a written Clone impl
for one is also ambiguous (E0283).

`source_copy_fields` (`sema/declarations/traits/implementations/copying.rs`)
had no arm for the never type, which reached the last arm with the census
message. It also took an array's element as a field to prove, and accepted
`str` with the other primitives.

The repair (`f6ec93d84`):

- the never type and arrays (`Array` and `ArrayTerm`) join scalars, raw
  pointers, shared references and fn pointers as targets without fields;
- `str` gets its own arm, "str cannot implement source Copy";
- unique references and every remaining kind stay rejected, and ADTs and
  tuples still prove their fields.

`str`, a slice and `!` without a Clone impl are rejected earlier, by the
trait's requirements (`Copy: Clone`, `Clone: Sized`), before the target
check. The `str` arm keeps the target rule matching rustc's anyway.

Controls: `pa31/tests/spec/copy-impl-targets/` (13 cases, 13/13 agree),
checked by `dev/tests/invariants/allowed_copy_targets.rs` (stem `allowed`;
`copy` already has 7 files). The prefix is `#![feature(never_type)]` and the
`function-pointers` foundation.

| Case | rustc 1.98.0 | Census 13 pin | Candidate |
|---|---|---|---|
| `core-never` (core's `!` impls) | accept | census message | accept |
| `scalars` (`bool`, `char`, `i32`, `f64`) | accept | accept | accept |
| `raw-pointers` | accept | accept | accept |
| `core-array` (`T: Copy` element) | accept | accept | accept |
| `array-clone-element` (`T: Clone` only) | accept | field not Copy | accept |
| `array-fixed-length` (`[Token; 2]`, `Token` not Copy) | accept | field not Copy | accept |
| `unique-reference` (opponent) | E0206 | unique references | unique references |
| `str-target` (opponent) | E0206 | requirements not satisfied | requirements not satisfied |
| `slice-target` (opponent) | E0206 | requirements not satisfied | requirements not satisfied |
| `never-without-clone` (opponent) | E0277 | requirements not satisfied | requirements not satisfied |
| `tuple-target` (`(u8, bool)`, opponent) | E0206, E0283 | field not Copy | field not Copy |
| `fn-pointer-target` (opponent, gap) | E0206, E0283 | accept | accept |
| `tuple-copy-elements` (`(u8, &'static u8)`, amendment, gap) | E0206, E0283 | accept | accept |

The coordinator's conditions held: `!` is accepted with its opponent
`&mut T` rejected, the `T: Clone`-only array control was declared before the
oracle, and `str` is excluded as rustc does.

Two gaps are recorded, as the coordinator decided: this compiler still
accepts a written Copy impl for a fn pointer or for a tuple whose elements are
Copy. `tuple-target` did not show the tuple gap, because its `bool` element
has no Copy impl in the foundation. `tuple-copy-elements` is a recorded
amendment for that: it was declared after the census 13 pin's run of the
first twelve cases, and before its own oracle and any candidate run. Both
declarations are in `outcomes.json`.

The rustc check of the tuple and fn pointer rule (run during the integrator
hold, by the coordinator's exception) also showed that three other PA31
packets record outcomes that rustc 1.98.0 does not produce. Section 2
records them.

## 2. rustc witnesses for six superseded PA31 fixtures

Six fixtures in three packets write a tuple or fn pointer Copy impl. rustc
1.98.0 rejects every one of them, and each packet's declaration predicted or
recorded otherwise. All six are superseded predecessors: each packet already
has an active successor that rustc accepts, or whose recorded outcome rustc
witnessed.

As the coordinator decided, the superseded declarations stay frozen. Each
packet gets one additive `reference-corrections.json` beside its README,
registered by an entry in `pa31/tests/README.md` (PA31 has no graded
manifest), with a README line in the packet. Each entry gives the superseded
declaration and the outcome it predicted or recorded; the witnessed rustc
1.98.0 result (compiler sha, fixture sha, edition, environment and argv, exit
status, and every error code with its span); and the active successor. The
witness summary is copied into the record; `obj/parallel/census-batch13/witness/`
is named only as provenance. Commit `7ca25fb64`.

| Packet | Fixture | Declaration and its outcome | rustc 1.98.0 | Active successor |
|---|---|---|---|---|
| `pointer-composition` | `source-profile-positive.rs` | `source-profile-predictions.json`: status 0 | E0206, E0283 at line 187 | `source-profile-v2-predictions.json`, `source-profile-valid.rs` |
| `pointer-composition` | `source-profile-wrong-signature.rs` | the same: status 1 | status 1 as predicted, E0308 as intended, plus E0206, E0283 at line 187 and two E0271 | `source-profile-invalid-signature.rs` |
| `source-format-macros/arguments` | `constructor-provider.rs` | `constructor-contract.json`: metadata status 0 in both editions | 12 errors (11 with `RUSTC_BOOTSTRAP`), among them E0206 at 302, E0283 at 301, E0034 at 205, E0508 at 411 | `constructor-complete-contract.json`, then its successors |
| `source-format-macros/arguments` | `constructor-complete.rs` | `constructor-complete-contract.json`: metadata status 0 in both editions | E0206 at 304, E0283 at 303 and 337, E0204 at 332, E0508 at 413 | `constructor-builtin-contract.json`, then the real and statement contracts |
| `unadjusted-request` | `native-unadjusted.rs` | `contract-v2.json`: rustc status 0 | E0206 at 19, E0283 at 18, E0600 at 28 | `contract-v4.json`, `native-unadjusted-v4.rs` |
| `unadjusted-request` | `native-unadjusted-v3.rs` | `contract-v3.json`: rustc status 0 | the same three, E0600 at 28 | the same |

`pointer-composition`'s README cites `docs/pa28-raw-pointer-composition.md`
line 74, which already records that disagreement, and does not repeat it.

The records also hold two cross-references:

- `constructor-complete-contract.json` classifies the observation of
  `constructor-contract.json` by three causes (E0554, missing derive macro
  imports and Drop const stability). They cover 8 of the 12 errors; E0206,
  E0283, E0034 and E0508 are not named.
- `contract-v2.json` and `contract-v3.json` carry a `rustc_stdout` that
  neither fixture can produce, since neither compiles. It is contract-v4's
  witnessed output, copied in when all four were committed together in
  `410394bfd`.

Findings, not fixed (each needs the coordinator's call per packet): besides
the tuple and fn pointer impls, `constructor-complete.rs` has an unrelated
array Copy error in its observer (E0204 at line 332, with E0283 at 337 and
E0508 at 413), and both `native-unadjusted` fixtures apply unary minus to
`f64` without a `Neg` role (E0600 at line 28). The active successors no
longer have these errors.

## 3. Census 13's record 2: `SpecFill` (derived)

Census 13's record 2 is "trait implementation requires conformance and
selection semantics" from `validate_trait_impl_owners`, at core's

```rust
impl<T: TrivialClone> SpecFill<T> for [MaybeUninit<T>]
```

(`mem/maybe_uninit.rs:1659`), the specializing child of
`impl<T: Clone> SpecFill<T> for [MaybeUninit<T>]`.

After section 1, the analysis-only diagnostic `d1-copy` (`98907a92`, from
`7ca25fb64`, empty patch) has the same record 2, although its crate record
moved to `ops/try_trait.rs:433`. The record is derived:

- `validate_trait_implementations` (`implementations/validation.rs`) stops at
  its first failed impl. Every impl after it in conformance order stays
  unvalidated.
- `conformance_order` (`sema/specialization/inherited_paths.rs`) sorts the
  specialization graph's impls by depth into the graph's slots, so every
  depth 1 child moves behind every root.
- `validate_trait_impl_owners` reports the first unvalidated trait impl in
  owner order. A child that moved behind the failed impl is reported, even
  though nothing in it failed.

Exploration drafts (`obj/parallel/census-batch13/repro/spec-fill/`, with the
`function-pointers` foundation and `min_specialization`), run with rustc
1.98.0 and the `d1-copy` pin in census mode:

| Draft | rustc 1.98.0 | `d1-copy` pin |
|---|---|---|
| `spec-fill.rs`: the pair, with a `TrivialClone` marker | accept | no record |
| `spec-fill-const-marker.rs`: core's `const unsafe trait TrivialClone: [const] Clone` | accept | no record |
| `spec-fill-later-pair.rs`: the pair, then another specialization pair | accept | no record |
| `spec-fill-after-failure.rs`: the pair, `impl Copy for NotClone {}`, then another pair | E0277 | record 1 at the Copy impl, record 2 at the `SpecFill` child |
| `failure-without-later-pair.rs`: the pair, then the failing impl | E0277 | records 1 and 2 both at the Copy impl |

With a later pair, the second root takes the child's slot in front of the
failure, and the child moves behind it. Without one, the child stays in front
and is validated. Record 2 needs no repair; it goes away when the failure
before it does. No change was made to the lookahead.

## 4. Census 14

The pin `188a5350` was built from `7ca25fb64` on main `825f443a8` with an
empty uncommitted patch. Its ordinary observation stops at "file 95: 17457:
trait implementation does not satisfy the trait's requirements", with 375
sources opened. The census (`obj/parallel/census-batch13/qualified-1/census/`)
checks 42,310 declarations of original core with none failed or blocked, in
58.7 seconds. It has 8 records: one crate record, then 7 lookahead records.
The `d1-copy` diagnostic of the same tree has the same 8 records.

| # | Pass | Site | Message |
|---|---|---|---|
| 1 | `validate_trait_implementations` | `ops/try_trait.rs:433` | trait implementation does not satisfy the trait's requirements |
| 2 | `validate_trait_impl_owners` | `mem/maybe_uninit.rs:1659` | trait implementation requires conformance and selection semantics |
| 3 | `local_annotations` | unlocated | implementation validation is outside the early scalar profile |
| 4 | `complete_alias_normalizations` | unlocated | implementation validation is outside the early scalar profile |
| 5 | `validate_objects` | `any.rs:152` | source receiver compatibility requires the completed immutable impl epoch |
| 6 | `complete_discriminants` | `mem/alignment.rs:440` | associated type requires an unambiguous validated implementation |
| 7 | `complete_layouts` | `ascii.rs:29` | enum layout requires completed discriminants |
| 8 | `require_specialization_execution` | unlocated | specialization verification is unfinished |

Census 13's record 1 is gone. Records 2 to 8 keep their passes, sites and
messages. Record 2 is derived (section 3). Records 3 to 8 each name the failed
impl validation or a pass that depends on it, as in census 13; they were not
investigated further.

### Record 1: `Try for NeverShortCircuit<T>` (genuine)

Core's `try_trait.rs:433` is `const impl<T> Try for NeverShortCircuit<T>`.
`Try` is `pub const trait Try: [const] FromResidual`, and `FromResidual`'s
parameter defaults to `<Self as Try>::Residual`. Core's `const impl<T>
FromResidual for NeverShortCircuit<T>` (line 448) is the only `FromResidual`
impl in core that leaves that argument to its default. The `Option`, `Result`
and `ControlFlow` impls write it.

Exploration drafts in `repro/never-short-circuit/` (not declared controls),
with the `function-pointers` foundation, `never_type` and core's
`const_precise_live_drops`:

| Draft | rustc 1.98.0 | Census 14 pin |
|---|---|---|
| `core-shape.rs`: core's `Try`, `FromResidual` and `Residual`, and the three `NeverShortCircuit` impls | accept | record 1 at the `Try` impl |
| `residual-unbounded.rs`: no `T: [const] Destruct` on the `Residual` impl | accept | the same |
| `residual-ordinary-destruct.rs`: an ordinary `T: Destruct` | accept | the same |
| `explicit-from-residual.rs`: the impl writes `FromResidual<NeverShortCircuitResidual>` | accept | no record |
| `minimal-const.rs`: a unit type, with only `Try` and `FromResidual` | accept | record 1 at the `Try` impl |
| `minimal-ordinary.rs`: the same, with ordinary traits and impls | accept | record 1 at the `Try` impl |
| `minimal-ordinary-swapped.rs`: the `FromResidual` impl first | accept | record 1 at the `Try` impl |
| `minimal-ordinary-projection.rs`: the impl writes `FromResidual<<Short as Try>::Residual>` | accept | no record |
| `minimal-ordinary-explicit.rs`: the impl writes `FromResidual<ShortResidual>` | accept | no record |

The first run of the four core drafts lacked `const_precise_live_drops`.
rustc rejected each with E0493 in `branch`, and the pin's results were the
same as above. That run is kept in `first-run/`.

The failure needs no constness, no `Destruct` bound and no impl order. It
needs only an impl whose trait argument is a defaulted projection on `Self`.
The same impl with the projection written out passes. In
`application_arguments` (`sema/declarations/generics/arguments.rs`), the
default `<Self as Try>::Residual` is applied, and `applied_trait`
(`sema/declarations/trait_paths.rs`) then replaces `Self` with the impl's
type. The defaulted and written forms should therefore name the same
reference. Something between there and the `Try` impl's supertrait goal
treats them differently; this batch did not locate it. rustc's `match_impl`
(`rustc_trait_selection/src/traits/select/mod.rs`, line 2545) normalizes the
impl's instantiated trait reference before equating it with the goal, so
a defaulted argument selects like a written one.

Proposed repair (owner guess: Lane A, declarations): give a defaulted trait
argument in an impl header the same projection treatment as a written one,
so `impl FromResidual for Short` matches `Short: FromResidual<ShortResidual>`.
`minimal-ordinary.rs` and `minimal-ordinary-projection.rs` are the smallest
pair. That needs declared controls, including an opponent whose default
normalizes to a different type, and another census. Not repaired here.

## Checks

Every check below ran on the tree committed as `7ca25fb64`, docs aside.

| Check | Result | Receipt |
|---|---|---|
| Private: 251 lib cases, strict private Clippy, format, architecture | pass | `obj/qualification/fast/1794140684864855538` |
| `census_mode` (10) and PA1 to PA12: 1,885 of 1,885 | pass | `obj/qualification/fast/1794140736277392065` |
| PA13 to PA19 and PA21 to PA24: 5,434 of 5,434 | pass | `obj/qualification/fast/1794140823688311120` |
| PA20: 1,451 of 1,451 | pass | `obj/qualification/fast/1794140904678718317` |
| `make test-upstream` | pass, 128 fixtures, 126 Reference files | `obj/parallel/census-batch13/tmp/test-upstream.log` |
| rustc 1.98.0 oracle | 13/13 agree | `obj/parallel/census-batch13/oracle/` |
| rustc 1.98.0 witnesses (section 2) | 12 runs, all status 1 | `obj/parallel/census-batch13/witness/witness.json` |
| Diagnostic `d1-copy` | completed analysis-only, 8 records | `obj/parallel/census-batch13/qualified-1/d1-copy/receipt-diagnostic.json` |
| Pin build and ordinary observation | passed observation, `188a5350` | `obj/parallel/census-batch13/qualified-1/receipt-build.json` |
| Census 14 | completed: 42,310 checked, 0 failed, 0 blocked, 8 records | `obj/parallel/census-batch13/qualified-1/receipt-census.json` |

The 251 private lib cases are:

- batch 12's 165 joined cases, which repeat the const trait, specialization,
  coherence, selection and const parameter invariants;
- item 1's 109 cases, 86 of them not already selected: the new
  `allowed_copy_targets.rs` test, the tests of the invariant files on source
  Copy impls, Copy requirements and Copy selection, and the tests named for
  copy or clone.

The 12 witness runs are both PA31 `pointer-composition` fixtures in edition
2024, and the other four fixtures in editions 2021 and 2024, with
`constructor-provider.rs` also run without `RUSTC_BOOTSTRAP`.

An earlier focused run during development (`1794139563761303878`, item 1's
109 cases) passed on the same sources and does not count toward the pin.
Joined admission was retried 7 times while other worktrees held the
heavy-job slot. `make test-harness` was not needed, because no integration
root was added. `make test-backend` was not needed, because the backend
contract and bridge did not change.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness and course text.

## Storage

- Kept: the census 14 pin `obj/parallel/census-batch13/qualified-1/pin/`
  (`188a5350`, with `.keep-artifacts`).
- Retired by removing `.keep-artifacts` and pruning their directories: census
  13's pin (`census-batch12/qualified-1`, `1c0219e9`) and the `d1-copy`
  diagnostic pin (`98907a92`). Their hashes are in
  `obj/parallel/census-batch13/retired-pins.sha256`. Census 13's records,
  manifest and receipts remain.
- `make tidy` ran after the census; obj is 12.2 GiB, under its 16 GiB cap.

## Shared files touched

- Declarations: `sema/declarations/traits/implementations/copying.rs`
  (section 1).
- Invariants: the new `allowed_copy_targets.rs` (stem `allowed`, 2 files).
- PA31: the `copy-impl-targets` packet, three `reference-corrections.json`
  records with a README line in each packet, and two entries in
  `pa31/tests/README.md`. No existing PA31 declaration or fixture changed.
- `spec.md` has a paragraph on Copy targets.

## Decisions needed

- Section 2's findings: the unrelated E0204 and E0508 in
  `constructor-complete.rs` and the E0600 in both `native-unadjusted`
  fixtures are recorded, not fixed. Each needs a per-packet call.
- The tuple and fn pointer gaps of section 1 stay recorded until a batch
  rejects written Copy impls for builtin Copy types.
- Census 14's record 1: the proposed repair in section 4, a defaulted impl
  trait argument treated like a written one, is next for Lane A unless it is
  assigned elsewhere.

## Commits

On main `825f443a8`, in order: `f6ec93d84` (section 1), `7ca25fb64` (section
2), then this document and the tracker entry.
