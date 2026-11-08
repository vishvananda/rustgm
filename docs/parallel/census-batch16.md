# Census batch 16: pattern-type coercions and the nineteenth census

Census 18 ([batch 15](census-batch15.md), pin `b5bcfc0f`) checked 42,310
declarations of original core, with none failed or blocked. Its crate record,
"source CoerceUnsized implementation does not retain a permitted pointer
coercion", is core's
`impl<T: PointeeSized, U: PointeeSized> CoerceUnsized<pattern_type!(*const U is !null)> for pattern_type!(*const T is !null) where T: Unsize<U>`
(`pat.rs:80`). Record 2, `ZipImpl` at `zip.rs:248`, is a lookahead record
that names record 1 as its parent. This batch, item C2c, on main `bfbf43049`:

- locates the record, and finds out whether core uses `DispatchFromDyn` for
  pattern types (section 1);
- freezes 16 declared controls against the rustc 1.98.0 oracle (section 2);
- admits a `CoerceUnsized` impl between two pattern types as rustc does
  (section 3), with a scaling invariant (section 4);
- runs census 19 (section 5).

Census 19 (`480f4169`, from `9430c64e7`) checks 42,310 declarations, with
none failed or blocked. Census 18's record 1 is gone, and so is its derived
record 2, `ZipImpl` at `zip.rs:248`. The new crate record is "trait
implementation does not satisfy the trait's requirements" at core's
`impl<'a, P> FusedIterator for RSplit<'a, P> where P: Pattern<Searcher<'a>: ReverseSearcher<'a>>`
(`str/iter.rs:569`, expanded by `generate_pattern_iterators!` at
`str/iter.rs:786`). Records 3 to 8 are census 18's, byte for byte.

Out of scope:

- Lane B owns `sema/bodies/field_storage.rs`, `finalization.rs`,
  `types/variance/*`, `types/aggregate/instances.rs` and `lifetimes.rs`, and
  Lane C owns transport. This batch touches none of them.
- The static `Sync` check in `sema/declarations/validation.rs` and
  `constrained_scalars.rs` did not change.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend and its configuration did not change.
- General arrays and `CoerceUnsized`, and executing a pattern-type coercion,
  stay fenced: no core declaration needs either (section 3).

## Method

The controls are no_core libraries with the census batch 2 core prefix that
`pa31/tests/spec/generic-non-null-patterns/` uses, plus core's own pointer,
pattern and `NonNull` impls without their stability attributes. Type
formability was settled first, on type-only files with no impl
(`obj/parallel/census-batch16/repro/` and `repro2/`):

| Type | rustc 1.98.0 |
|---|---|
| `pattern_type!(u32 is 1..=4)` | accepted |
| `pattern_type!(u32 is 1..)` | rejected: requires the `RangeMax` lang item |
| `pattern_type!(*mut T is !null)` | accepted |
| `pattern_type!(&'a T is !null)` | accepted |
| `pattern_type!(pattern_type!(*const T is !null) is !null)` | accepted |
| `pattern_type!(*const [u8] is !null)` in a signature | accepted |

So the controls use closed ranges. The 16 outcomes were then declared
(`oracle/pattern-coercions.declared.json`, sha `6af93903`, declaration sha
`b81ee0cb`) before the frozen rustc 1.98.0 oracle ran on the exact bytes
(`oracle/pattern-coercions.rustc-oracle.json`) and before any RustGM run of
them. All 16 statuses agree.

## 1. The record and core's two pattern impls

`validate_source_coercion_impl` (`traits/implementations/coercions.rs`)
sends a pair of nominals to `structure::schema`, and every other pair to
`pointer_pair`. `pointer_pair` admits reference to reference, reference to
raw pointer and raw pointer to raw pointer, and nothing else. A pair of
pattern types reached its fallback, which is census 18's text.

rustc 1.98.0's `coerce_unsized_info` handles a pair of pattern types first:
the two patterns must be identical ("only pattern types with the same pattern
can be coerced between each other"), and then it registers
`base_a: CoerceUnsized<base_b>` as an ordinary obligation in the impl's
environment. The bases are not matched against a built-in pointer rule. The
oracle confirms both halves: two different patterns fail with that text and
no code, and core's exact impl without core's `*const` impl fails with
E0277, "the trait bound `*const T: CoerceUnsized<*const U>` is not
satisfied".

Core uses `DispatchFromDyn` for pattern types:
`impl<T: DispatchFromDyn<U>, U> DispatchFromDyn<pattern_type!(U is !null)> for pattern_type!(T is !null) {}`
(`pat.rs:85`), over a type parameter base. `NonNull`'s nominal impl
(`ptr/non_null.rs:1626`) reaches it through its one field. `dispatches.rs`
has admitted a pair of pattern types with one identical pattern since
[batch 6](census-batch6.md), which is rustc's rule: no base obligation. Batch
6's `generic-non-null-patterns/` controls cover both core shapes. This batch
adds two range controls for the same rule and changes nothing there.

`pat.rs:87` to `:91` are `Unpin` and `Freeze` impls for non-null pointer
patterns; they are not coercion impls.

## 2. Controls

`pa31/tests/spec/pattern-coercions/` owns the 16 controls, because PA31 is
the earliest assignment that owns both pattern types and `CoerceUnsized`.
PA31 has no graded manifest, so, as batch 6 did, the packet is registered in
`pa31/tests/README.md`, its `outcomes.json` records both the declaration and
the oracle, and the private invariant
`dev/tests/invariants/refined_coercions.rs` checks every case.

| Control | Shape | Declared | rustc 1.98.0 | This analysis |
|---|---|---|---|---|
| `core-pattern-coercion` | core's `pat.rs:80` impl over core's raw pointer impls | accept | accept | accept |
| `mutable-to-const` | `*mut T is !null` to `*const U is !null` | accept | accept | accept |
| `generic-base` | `impl<T: CoerceUnsized<U>, U>` over `T is !null` | accept | accept | accept |
| `nominal-pattern-field` | core's `NonNull` shape and its `CoerceUnsized` impl | accept | accept | accept |
| `dispatch-same-range` | `DispatchFromDyn` between two `u32 is 1..=4` | accept | accept | accept |
| `different-patterns` | `u32 is 1..=4` to `u32 is 1..=8` | reject | reject, no code | identical-pattern text |
| `integer-base` | `u32 is 1..=4` to itself | reject | E0277 | field-relation text |
| `pattern-to-pointer` | `*const T is !null` to `*const U` | reject | E0377 | permitted-pointer text |
| `mutability-mismatch` | `*const T is !null` to `*mut U is !null` | reject | E0277 | field-relation text |
| `missing-unsize` | core's impl without `T: Unsize<U>` | reject | E0277 | field-relation text |
| `missing-pointer-impl` | core's impl without the raw pointer impls | reject | E0277 | field-relation text |
| `nominal-without-pattern-impl` | `NonNull` shape without `pat.rs:80` | reject | E0277 | field-relation text |
| `dispatch-different-ranges` | `DispatchFromDyn` between two ranges | reject | reject, no code | dispatch-shape text |
| `reference-base` | `&'b T is !null` to `&'a U is !null` | accept | accept | rejected by name |
| `nested-pattern` | non-null pattern over `*const T is !null` | accept | accept | rejected by name |
| `executed-coercion` | a body returns `*const [u8; 4] is !null` as `*const [u8] is !null` | accept | accept | "incompatible types" |

- The declared reason for `pattern-to-pointer` guessed E0376; rustc reports
  E0377. Its status agrees.
- The field-relation text is "source CoerceUnsized implementation does not
  prove its exact field relation", the text a nominal impl's changed field
  already uses. The base is a pattern type's one storage field.
- The last three are stricter than rustc, like batch 6's three bases. A
  non-null pattern over a reference or over another pattern type is rejected
  by name before impl validation: "non-null pattern base must be a raw
  pointer or type parameter". A body that executes the coercion fails with
  "incompatible types", because body unsizing has no pattern-type rule. No
  core declaration needs any of them.

## 3. The change

`9430c64e7` (on `bfbf43049`):

- `validate_source_coercion_impl` matches a pair of pattern types beside the
  nominal pair. Two different constraints fail with "source CoerceUnsized
  pattern types require one identical pattern". Otherwise the bases are
  proved by `prove_coercion_field` with the `CoerceUnsized` role, the same
  goal a nominal impl's changed field proves. No region pair and no nominal
  schema is retained, so nothing new reaches body unsizing.
- `prove_coercion_field` adds its proof's selection steps to
  `body_work.canonical_steps`, as `automatic_domains.rs` does for its
  proofs. This is an observation, not a language fact, and no budget
  changes.
- `pointer_pair` and `dispatches.rs` are unchanged.

## 4. Scaling

`pattern_coercion_work_is_linear_in_impls` builds core's prefix and
`nominal-pattern-field.rs`, then N distinct pattern impls
(`*mut [FamilyN; 4] is !null` to `*mut [FamilyN] is !null`, apart from core's
`*const` impl), and M `NonNull`-shaped consumers, each with its own
`CoerceUnsized` impl. The rustc 1.98.0 shape check of the generated program
is in `obj/parallel/census-batch16/repro3/`; a `*const` family overlaps
core's impl (E0119), so the families use `*mut`.

| Impls | Consumers | Canonical steps |
|---|---|---|
| 8 | 1 | 3,031 |
| 32 | 1 | 10,543 |
| 1 | 8 | 2,226 |
| 1 | 32 | 6,978 |
| 8 | 8 | 5,397 |
| 16 | 16 | 13,005 |
| 32 | 32 | 35,901 |

The test asserts the first four rows: each count's work grows, and
quadrupling either count at most quintuples it. Each pattern impl costs about
313 steps and each consumer about 198.

The last three rows are an analysis-only observation. The pre-rebase test
binary (`39b735cf`) measured them before the test was trimmed to its
asserted rows; its first four rows are the table's. They fit
`a + 471n + 20n²`: about 20 steps for each pair of an impl and a consumer.
`Head::Pattern` keys a candidate by its constraint only, so a consumer's
field goal visits every non-null pattern impl, as a raw pointer goal visits
every impl of its mutability. Core has one pattern `CoerceUnsized` impl, so
this is recorded, not changed.

## 5. Census 19

The pin is `obj/parallel/census-batch16/qualified-1/pin/rustgm`
(`480f4169`), built from `9430c64e7` with no uncommitted change and bound to
the three fastchecks below. Its manifest chains from census 18's
(`census-batch15/qualified-3/manifest.json`); only `expected_cli_sha256` and
`predecessor_manifest_sha256` change. The ordinary first-error observation
reports the same site as census 19's record 1.

Census 19 (`qualified-1/census/census.tsv`, sha `0e6b96ed`) has 8 records,
1 crate and 7 lookahead, over 42,310 declarations, with none failed or
blocked.

| Record | Census 18 | Census 19 |
|---|---|---|
| 1 (crate) | `pat.rs:80`, "source CoerceUnsized implementation does not retain a permitted pointer coercion" | `str/iter.rs:569`, "trait implementation does not satisfy the trait's requirements" |
| 2 (lookahead, parent 1) | `zip.rs:248` (`ZipImpl`), "trait implementation requires conformance and selection semantics" | `str/iter.rs:569`, the same text |
| 3 to 8 | unchanged | unchanged |

Record 2 is a lookahead of `validate_trait_impl_owners`
(`declarations/traits.rs:156`). It reports the first trait impl owner that
is neither a destructor nor validated, so it depends on where record 1
stopped trait impl validation. In census 18 that owner was `ZipImpl`. In
census 19 it is record 1's own impl, so `ZipImpl` disappears with record 1.

Census 19's lead is `str/iter.rs:569`, the first expansion of
`generate_pattern_iterators!` (`Split` and `RSplit`):

```rust
impl<'a, P> FusedIterator for RSplit<'a, P>
where
    P: Pattern<Searcher<'a>: ReverseSearcher<'a>>,
{}
```

`FusedIterator: Iterator`, and `RSplit`'s `Iterator` impl
(`str/iter.rs:543`) has the same where-clause, an associated-type bound on
the generic associated type `Searcher<'a>`. This batch does not locate the
cause; the forward `FusedIterator` impl at `str/iter.rs:566`, whose only bound
is `P: Pattern`, is not the record.

## Checks

| Check | Result | Evidence |
|---|---|---|
| rustc 1.98.0 oracle | 16/16 statuses agree | `obj/parallel/census-batch16/oracle/` |
| Lib cases (`c-cases.txt`): the two new tests, batch 6's non-null patterns, unsizing, dyn receivers, coercion transfer, pattern and refinement cases | 68/68 | fastcheck `1794175414223226453` |
| Suites `coercion_provider`, `coerce_validation_source`, `census_mode` | 18/18 | fastcheck `1794175593209004168` |
| PA23, PA24 | 10/10 | fastcheck `1794175846879083352` |
| `make test-upstream` | 128 pinned upstream fixtures, 126 Reference files | `test-upstream.log` |
| Pin build and ordinary observation | passed | `qualified-1/receipt-build.json` |
| Census 19 | completed, analysis-only | `qualified-1/receipt-census.json` |

Each fastcheck includes formatting, the architecture audit and strict
Clippy for its selection. All ran on the rebased sources (`9430c64e7` on
`bfbf43049`); the same selections passed before the rebase. The scaling
test's four asserted rows are identical on the pre-rebase binary
(`39b735cf`) and the rebased one (`4085cf49`).

Development failures, each fixed before the passing runs:

- The invariant was first named `pattern_coercions.rs`, the seventh audited
  file with the stem `pattern`, which the architecture audit rejects. It is
  `refined_coercions.rs`, beside `refined_annotations.rs`.
- The first scaling program used `Family<T>: Unsize<Family<U>>` premises:
  "source Unsize nominal-tail goals are not implemented". Its second used
  `*const` families, which overlap core's impl (E0119 in rustc). The
  committed one uses `*mut` families.
- Clippy's `chunks_exact_to_as_chunks` rejected `chunks_exact(2)`.

## Open items

- **Stricter than rustc, by name** (rejects valid; accepted for now):
  - `reference-base`: `&'b T is !null` to `&'a U is !null`. rustc 1.98.0
    accepts it; this analysis rejects the base with "non-null pattern base
    must be a raw pointer or type parameter" (batch 6's open item).
  - `nested-pattern`: a non-null pattern over `*const T is !null`. rustc
    1.98.0 accepts it; rejected with the same base text.
  - `executed-coercion`: a body returns `*const [u8; 4] is !null` as
    `*const [u8] is !null`. rustc 1.98.0 accepts it; this analysis reports
    "incompatible types", because body unsizing has no pattern-type rule.
    Demand it when a core body needs one.
- **The pattern head's cross term.** `Head::Pattern` keys a candidate by its
  constraint only, so each pair of a non-null pattern impl and a consumer
  goal costs about 20 steps (section 4: 8 by 8 is 5,397, 16 by 16 is 13,005,
  32 by 32 is 35,901). Raw pointer impls have the same shape, keyed by
  mutability only. It is bounded for core, which has a single pattern
  `CoerceUnsized` impl. A base-aware key would remove it.

## Storage

- Kept: the census 19 pin `obj/parallel/census-batch16/qualified-1/pin/`
  (`480f4169`, with `.keep-artifacts`), the census baseline.
- Retired by removing `.keep-artifacts` and pruning the directory: census
  18's pin (`census-batch15/qualified-3/pin`, `b5bcfc0f`), after census 19's
  pin was built. Its hash is in
  `obj/parallel/census-batch16/retired-pins.sha256`; its receipt, manifest
  and census remain.
- No analysis-only pin was built in this batch.
- obj was 9.6 GiB before the pin build.
