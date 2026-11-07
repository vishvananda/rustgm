# Census batch 4: the opaque storage cascade, constant modules, the own-bound scan and the fifth census

Census 4 ([batch 3](census-batch3.md)) completed the declaration loop of
original core: 42,109 declarations checked, 74 failed and 127 blocked. Its
largest blocking cause was "missing storage opaque declaration" (4 failed, 101
blocked). This batch confirms and repairs that cascade and its root cause,
repairs a third small cause, makes the declaration own-bound ambiguity check
linear and records the fifth census. Lane B owns "associated type requires an
unambiguous implementation" and the expansion `Definition` record; neither is
touched here.

## 1. "missing storage opaque declaration"

### Diagnosis (analysis only)

A temporary patch printed every opaque interning with its owner, every opaque
declaration insertion and the storage miss. It was never committed and is
preserved as `obj/parallel/census-batch4/diag1.patch`, with its pin and census
in `obj/parallel/census-batch4/diag/diag1/` and its stderr in `diag1-stderr.txt`.
The inputs were unchanged original core at main `56f4ff908`.

The batch 3 hypothesis holds. `opaque_result` interns `Type::Opaque(id, ..)`
before `opaque_clauses` runs and inserts the opaque's `Declaration` only after.
The census interned 66 opaques and declared 58. The eight undeclared ones are
exactly the eight "outlives bound requires a named lifetime" failures, all
`impl Trait + '_` results: `filter_map` `find`, `flatten` (two), `step_by`
`nth` and `nth_back` (two each) and `iterator` `is_true`. Each failed after
interning. The next generic const fn evaluation (`size_of` behind
`to_be_bytes`, `Buf`, `prepare`) prepared storage from `storage_cursor`, met
`find`'s undeclared `OpaqueId(17)` and failed that unrelated item, which
blocked 101 more.

The same window exists in ordinary checking. An opaque result that waits for a
GAT const argument, `impl Tr<Out<{ size::<u8>() }> = u8>`, is interned and
undeclared while its argument evaluates. That evaluation's storage preparation
rejected a valid crate with the same message.

### Repair

`opaque/reveal.rs::collect_opaque_applications` treats an application with no
declaration, or whose declaration has no hidden type yet, as waiting on its own
identity. That is the existing path for a declared opaque whose defining body is
unchecked (`Preparation::wait`; `close_opaque_results` wakes it). It never
fails the caller, and the message no longer exists. A waiting application
whose declaration later fails is never woken, so it is never revealed.

## 2. `'_` in opaque result outlives bounds

rustc reads `'_` in a result `impl Trait + '_` as the elided output lifetime:
the receiver's, otherwise the only input lifetime, and E0106 without one. A
where-clause `T: '_` stays E0637. `opaque_clauses` resolved every written
outlives bound through `bound_values`, which rejects `'_`. It now resolves an
opaque result's outlives bound through `Checker::region` in the opaque's output
context, the elision path of a written `&'_ T`. Other bound positions keep
`bound_values`. This repairs the eight failures above, the roots of the
cascade.

## 3. Constant modules of binderless const and static annotations

"semantic type or associated item is private" (4) in `io/error/repr_bitpacked.rs`
came from `static_assert!(@usize_eq: size_of::<Repr>(), ..)` assertions,
`const _: [(); $lhs] = [()]`, inside a private module naming its `pub(super)`
`Repr`. A const or static item has no generic binder, so
`schedule_typed_constant` and `schedule_array_length` fell back to `DefId(0)`
and typed the length in the crate root. `regions::Position::Static` now carries
its item (`Context::item_static`), and both schedulers use that item's module
when there is no binder. `Position` already carries a `DefId` in `Default`
and `Alias`, so `Context` keeps its size.

rustc 1.98.0 does not check type privacy in anonymous constants of item
signatures at all. A fn result length, a struct field length and
`const _: [(); { leak(); 1 }]` naming a private type are accepted, while a const
initializer and a fn body naming it are rejected (`tmp/explore/` in the batch
folder). RustGM checks those lengths in the item's own module. That is
stricter only for a private type reached from outside its module, here through
a public alias in a crate-root length. This pre-existing difference is not
changed.

## 4. Controls for sections 1 to 3

Thirteen no_core source-core controls were declared before the frozen rustc
1.98.0 oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024
--crate-type=lib --emit=metadata`). They are in `pa31/tests/spec/`
`undeclared-opaque-storage/` (4), `elided-opaque-outlives/` (6) and
`static-constant-modules/` (3), each with `outcomes.json`. The drafts were
probed with the pinned and candidate compilers before the declaration.

Two amendments were made before re-runs, and both are recorded in
`outcomes.json`:

- Oracle v1 stopped four accepted cases at rustc's `requires copy lang_item`.
  The v2 `Clone::clone(&self)` then needed `legacy_receiver`. v3 inserts batch
  2's frozen prefix lines 47 to 54 (`legacy_receiver`, `clone`, `copy`). No
  declared outcome or case body changed.
- v1 declared `private-through-alias` (the alias in a crate-root length)
  rejected, and rustc accepted it, as section 3 explains. It was retired, and
  `private-initializer-through-alias` (the alias in a crate-root const
  initializer) was declared rejected, without an error code, before oracle v2.

At v3 all thirteen agree with rustc, including codes E0106 (three cases),
E0271 and E0637. The candidate agrees with all thirteen. The pinned census 4
compiler (`6e53028c`) disagrees with the six accepted positives and rejects
`waiting-gat-mismatch` for the wrong reason ("missing storage opaque
declaration" instead of "incompatible types").

| Control | rustc | Pin | Candidate |
|---|---|---|---|
| `declared-before-length` | accept | accept | accept |
| `failed-before-length` | E0106 | reject | reject (elision) |
| `waiting-gat-argument` | accept | storage miss | accept |
| `waiting-gat-mismatch` | E0271 | storage miss | incompatible types |
| `reference-parameter`, `argument-position-parameter`, `receiver` | accept | named-lifetime error | accept |
| `no-input-lifetime`, `two-input-lifetimes` | E0106 | reject | reject (elision) |
| `where-clause` | E0637 | reject | reject |
| `restricted-const-length`, `restricted-macro-length` | accept | private | accept |
| `private-initializer-through-alias` | reject | private | private |

Private invariants check every case
(`dev/tests/invariants/{undeclared_opaque_storage,elided_opaque_outlives,binderless_constant_modules}.rs`).
The census integration test
`an_opaque_failing_after_interning_leaves_later_storage_to_its_items` checks
that `failed-before-length`'s shape in census mode records only the failed
opaque. The pinned compiler also failed the later `bytes`.

## 5. The quadratic own-bound scan

### The 160 to 192 "regime" was a probe artifact

Batch 3's attribution table showed four inference steps per bound and no
`alternatives::choose` probes up to 160 bounds, then over a million steps at
192. The probe (`census-batch3/attribution_probe.rs`) wrote its ledger file only
at `mark()` calls and each time its global ledger crossed another
1,000,000-step dump threshold. It wrote nothing at exit. Own-bound validation
runs in `validate_signatures`, after the last mark (`declaration-loop-end`).
Up to 160 bounds the ledger never reached its next dump, so the file still held
the state at that mark, which was 4 steps per bound of declaration-loop work.
At 192 the ledger crossed a dump and recorded the scan. The probe also counted
each trial twice: once inside the trial `Context` and again when `choose`
charged that context's steps to the declaration (`alternatives.rs:39`).

The pinned compiler shows no regime change. The scan is quadratic at every
size. Each own bound is trial-unified against every assumption with the same
trait and head, at 16 steps per trial in the declaration's one
million-step `Context`. That puts the cap at `16 n^2 ≈ 1,000,000`, or about 250
bounds. The pin accepts 248 bounds and rejects 250 bounds with "type
constraint work budget exceeded" (`tmp/threshold-pin.txt`). Its time grows from
0.23 s at 64 bounds to 1.03 s at 192.

### Repair

`sema/traits/assumptions/discriminants.rs` groups one declaration's
assumptions once, charged one step per indexed assumption, by trait, subject
head and first non-lifetime argument when that argument is a closed value: an
evaluated const or a primitive type. Two distinct closed values never unify.
`Assumptions::unambiguous_declaration` takes the caller's `Discriminants`
(held beside the assumptions in validation's `TraitEvidence`). For an own
bound with a closed first value it trial-unifies only against assumptions with
that value or an open argument there, through the same `choose` and
equivalence rule as `select`. An own bound with an open first argument keeps
the full scan. The result is unchanged: every excluded assumption would have
been a mismatch, and the goal itself is always among the candidates, so a
skipped normalization outcome cannot change the answer. The cap is not
raised.

Touched in `sema::traits`: `assumptions.rs` (the `Discriminants` argument and
the `choose_flat` helper shared with `select`), the new
`assumptions/discriminants.rs` and the call in `assumptions/tests.rs`.
`alternatives.rs` and `selection` are unchanged, apart from one test call in
`dev/tests/invariants/universal_premises.rs`.

### Controls

In `dev/tests/invariants/own_bound_discriminants.rs`:

- `distinct_closed_own_bounds_pay_flat_work_per_bound`: 64, 256 and 1,024
  distinct own bounds pay exactly 2 steps per bound (128, 512 and 2,048
  steps). The test allows 5/4 of the 64-bound rate.
- `own_bounds_with_one_value_and_unifying_lifetimes_remain_ambiguous`: the
  opponent. `B<'a, 0>` and `B<'b, 0>` share their value and unify, so the own
  bound is still ambiguous. Its 255 distinct companions are not.
- `declaration_bound_families_are_checked_and_ambiguity_is_still_rejected`:
  the batch 3 source family at 64, 256 and 1,024 bounds is accepted, and the
  256-bound lifetime opponent is rejected with "ambiguous trait assumptions in
  declaration".

The existing `unrelated_rigid_subjects_do_not_make_proof_quadratic`, the
declaration-assumption and universal-premise tests and the PA20
`projected_bound_staging_keeps_linear_headroom_in_ordinary_crates` invariant
are unchanged and pass.

## 6. Census into bodies: not implemented

The declaration census already isolates the declaration loop. After it,
`Checker::run` runs about 30 later passes (`complete_deferred_declarations`,
`complete_nominals`, compiler value headers, opaque outlives, trait defaults,
destructor, owner, associated-type and family bound validation,
`validate_signatures`, compiler-item completion, trait implementation, region,
overlap, annotation, alias, object and discriminant passes, layouts, API
exposure, alias execution and specialization). Each consumes the completed
facts of every declaration and propagates its first error. None is isolated per
item. `census_with_executor` returns facts only when they all finish, and
bodies need those facts.

An analysis-only diagnostic (`obj/parallel/census-batch4/diag2.patch`, census in
`diag/diag2/`, never committed) removed the early return and marked each pass.
`complete_deferred_declarations` passed. `complete_nominals`, the second pass,
stopped the whole census with a crate-level "invalid lifetime constraint
identity" (`panic.rs:2318`), not attributable to any item. Continuing into
bodies therefore needs census isolation in every later pass. It also needs a
body-to-declaration dependency closure, which does not exist before checking:
method, impl and layout dependencies are discovered while a body is checked,
so a body reaching a poisoned declaration would surface as a misattributed
failure rather than a blocked record naming its root. That is an invasive
change across the declaration pass pipeline and was not attempted. Census 5
covers declarations only. The existing `census_mode` contracts are unchanged.

## 7. Census 5

The pin `5cb7e00c` (`obj/parallel/census-batch4/qualified-1/pin/rustgm`) was
built from `2b52871ed` with the same flags as batch 3's pin. Its manifest is
batch 3's with only `expected_cli_sha256` and `predecessor_manifest_sha256`
changed: no configuration source pin changed since `8c6da24a9`, so nothing is
rebound. The census is analysis-only, with original core inputs unchanged:
375 opens in 53 s. Its records are in `rustgm-pa29-work/obj/parallel/core-census-5/`.
The ordinary first error is unchanged: `core/src/pat.rs:2692`
("non-pointer non-null patterns require their wider profile").

| Census | Checked | Failed | Blocked | End |
|---|---|---|---|---|
| Census 4 (`6e53028c`) | 42,109 | 74 | 127 | declaration loop complete |
| Census 5 (`5cb7e00c`) | 42,194 | 62 | 55 | declaration loop complete |

Bodies are still not entered (section 6), so there are no body causes.
Checked declarations grew by 85. Four causes the cascade hid now appear: all 29
of their blocked records were blocked by "missing storage opaque declaration"
in census 4, and their four root failures have no census 4 record.

### Failures by cause

"Blocked" counts every record whose blocking chain ends at a failure with that
message.

| Cause | Census 4 failed / blocked | Census 5 failed / blocked | Sites |
|---|---|---|---|
| associated type requires an unambiguous implementation (Lane B) | 35 / 26 | 35 / 26 | `iter/adapters/step_by.rs`, `filter_map.rs`, `fuse.rs` |
| invalid canonical trait selection | 18 / 0 | 18 / 0 | `iter/adapters/flatten.rs` (15), `iter/adapters/mod.rs` (2), `iter/mod.rs` |
| nominal declaration requirements are not complete | 0 / 0 (hidden) | 1 / 12 | `num/nonzero.rs` (`impl ZeroablePrimitive`) |
| canonical trait selection requires pending declarations | 0 / 0 (hidden) | 1 / 11 | `Buf`, `num/imp/int_log10.rs` |
| source intrinsic constant execution is not implemented | 0 / 0 (hidden) | 1 / 4 | `prepare`, `num/uint_macros.rs` (`ctpop`) |
| invalid associated equality selection | 3 / 0 | 3 / 0 | `array/mod.rs` (2), `iter/traits/iterator.rs` |
| constant storage layout requires completed enum discriminants | 0 / 0 (hidden) | 1 / 2 | `const _`, `lib.rs` |
| non-pointer non-null patterns require their wider profile | 1 / 0 | 1 / 0 | `pat.rs` |
| nominal or associated types require later semantic assignments | 1 / 0 | 1 / 0 | `ops/control_flow.rs` |
| missing storage opaque declaration | 4 / 101 | 0 / 0 | repaired (section 1) |
| outlives bound requires a named lifetime | 8 / 0 | 0 / 0 | repaired (section 2) |
| semantic type or associated item is private | 4 / 0 | 0 / 0 | repaired (section 3) |

### Largest blocking roots

| Blocked | Root | Cause |
|---|---|---|
| 12 | self-type `Self`, `num/nonzero.rs:1829` | nominal declaration requirements are not complete |
| 11 | `Buf`, `num/imp/int_log10.rs:4111` | canonical trait selection requires pending declarations |
| 4 | `prepare`, `num/uint_macros.rs:3278` | source intrinsic constant execution is not implemented |
| 4 each | `spec_nth`, `spec_try_fold`, `spec_fold` in `step_by.rs` | associated type requires an unambiguous implementation |
| 3 each | `spec_next_back`, `spec_nth_back`, `spec_try_rfold`, `spec_rfold` in `step_by.rs` | associated type requires an unambiguous implementation |
| 2 | `const _`, `lib.rs` | constant storage layout requires completed enum discriminants |

The four newly exposed causes have not been diagnosed. The `nonzero.rs` and
`int_log10.rs` locations are inside macro definitions, so the census site is
the macro's rule.

## 8. Causes diagnosed but not repaired

"invalid canonical trait selection" (18) and "invalid associated equality
selection" (3) are `fold_cause`'s wording for the same underlying error. The
diagnostic patch printed it at both sites: "associated type projection requires
normalization", from `nominals/traits/normal_form.rs` (18) and the associated
equality selection (3). The failing `flatten.rs` methods are exactly those
whose signature or where-clauses mention `Self::Item` or `Self: Sized`
(`try_fold`, `fold`, `last`, `try_rfold`, `rfold` in three impls). `next`,
`size_hint`, `advance_by` and `count` of the same impls pass. The impls'
second parameter is determined only through a nested projection equality,
`I: Iterator<Item: IntoIterator<IntoIter = U, Item = U::Item>>`, as is
`GenericShunt`'s `R` in `I: Iterator<Item: Try<Residual = R>>` (the
`iter/adapters/mod.rs` sites). Ordinary-mode reductions with these headers,
the trait method `where Self: Sized`, and core's blanket
`impl<I: Iterator> IntoIterator for I` all pass (`repro/flatten{1,2,3}.rs`,
`repro/shunt1.rs`), so the trigger needs more of core's surroundings. The
repair belongs in canonical selection's normalization and was not attempted
in this batch.

## Checks

Every check used the final sources (`2b52871ed`, docs aside), the pin's source
fingerprint. The first two commits passed their private and integration
selections at their own sources before the third was added.

| Check | Result | Receipt |
|---|---|---|
| Tasks 1/4 private (9 lib cases, strict private Clippy, format, architecture) at `802a311a9` | pass | `obj/qualification/fast/1794075294714816361` |
| Tasks 1/4 integration (`census_mode` 7, `opaque_outlives` 7) at `802a311a9` | pass | `obj/qualification/fast/1794075336866642806` |
| Final private (31 lib cases: tasks 1, 3 and 4 invariants, `assumptions`, `declaration_tests`, universal premises, PA20 bound-staging headroom, privacy, binop) | pass | `obj/qualification/fast/1794076569089412194` |
| `census_mode`, `opaque_outlives` and PA1 to PA12 | pass, 14 tests, every PA full | `obj/qualification/fast/1794076794932936661` |
| PA13 to PA19 and PA21 to PA24 | pass, every PA full | `obj/qualification/fast/1794077181251227599` |
| PA20 alone | pass, 1451/1451 | `obj/qualification/fast/1794077247010968207` |
| `make test-upstream` | pass, 127 fixtures, 126 Reference files | `obj/parallel/census-batch4/tmp/test-upstream.log` |
| rustc 1.98.0 oracle v3 | 13/13 agree | `obj/parallel/census-batch4/controls/rustc-oracle.json` |
| Pin build and ordinary observation | pass | `obj/parallel/census-batch4/qualified-1/receipt-build.json` |
| Census 5 | completed, analysis-only | `obj/parallel/census-batch4/qualified-1/receipt-census.json` |

No hot record, column or wire layout changed (`regions::Context` keeps its size;
`Discriminants` lives in validation's per-owner `TraitEvidence`), so no layout
invariant was selected. `make test-harness` was not needed: no integration
root was added.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness and course text.
