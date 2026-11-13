# Body projection normalization

Release candidate8's core census (`obj/parallel/capture-match/candidate8-release`,
ae92eca9, clean 53667da5b) fails three portable-simd bodies with "associated
type projection requires normalization":

- `first_set` 17766 (`core_simd/src/masks.rs:395`), at the argument of
  `simd_reduce_min`:

  ```rust
  let masked_index: Simd<T::Unsigned, N> =
      unsafe { core::intrinsics::simd::simd_cast(masked_index) };
  let min_index: T::Unsigned =
      unsafe { core::intrinsics::simd::simd_reduce_min(masked_index) };
  let min_index: T = unsafe { core::mem::transmute_copy(&min_index) };
  ```

- `try_from` 17943 and 17946 (`core_simd/src/vector.rs:1045` and `:1057`), at
  `slice.try_into()?` in `Ok(Self::from_array(slice.try_into()?))`.

rustc accepts all three.

## Diagnosis

`obj/parallel/simd-normalization/declared.json` records ten declared
batches. Batch 1 was malformed: its prefix had no `Copy` impl for `[T; N]`.
Batch 9 was malformed too. rustc required a `freeze` lang item for its
`const fn`, and RustGM required the actual try protocol. Batches 2 and 10
corrected them. Batches 3 to 8 are ungraded diagnostics. A gdb backtrace of
the dev-fast CLI at the error's formatting located each failure
(`obj/parallel/simd-normalization/trace/`), and a temporary instrumented
build, not committed, printed the terms involved.

**`first_set`.** The batches narrowed the form to

```rust
pub fn f<T: Sealed>(x: T) -> T {
    let r: T::Unsigned = convert(x);
    unsafe { transmute_copy(&r) }
}
```

It fails with a direct bound as well as `MaskElement`'s supertrait, and with
ordinary generic functions in place of the intrinsics. It types whenever the
signature also names `T::Unsigned`.

- A path annotation such as `T::Unsigned` resolves to a declaration type,
  here the projection `<T as Sealed>::Unsigned`. Annotation lowering inserted
  it as a known type and registered no body normalization. Only a qualified
  path with inferred inputs registered one.
- When the signature names the same type, normalizing the signature gives it
  a normal form, so the body's term has a head. Otherwise nothing ever
  normalizes it. The next coercion that inspects it, `&r` against
  `&Src`'s pointee or `masked_index` freshened for `simd_reduce_min`'s
  unresolved input, fails.

**`try_from`.** `slice.try_into()` has the type `Result<[T; N], <&[T] as
TryInto<[T; N]>>::Error>`. Its call result's normalization is registered, but
it has not completed when `?` coerces the operand to an unresolved expected
type. Freshening the operand's regions rejects any nested projection. The
explicit `match` and `<[T; N]>::try_from(slice)?` forms already typed.

## Change

- **Resolved projection annotations.** Annotation lowering hands a resolved
  declaration type that still names a projection to its host. The body checker
  registers it for body normalization, as it does a qualified path. An
  isolated method prefix keeps the resolved type, as before.
- **Equality for a nested pending projection.** When a coercion binds an
  actual type to an unresolved expected type, its regions are freshened. If
  that walk meets a projection that has not normalized, the actual type is
  related by equality unchanged. `coerce_one` already does this for a
  projection head: a future normal form can complete the equality, but cannot
  supply a conversion missing at the source point.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- Every body annotation whose resolved type names a projection now has a
  normalization query. One that cannot normalize is an error at typing,
  where rustc also normalizes it, rather than an unnormalized term.
- An actual type related by equality keeps its regions, so its lifetimes
  are equated where rustc, after normalizing, could relate them by subtyping.
  A program can be rejected by the lifetime proof that rustc accepts. None of
  the packet's cases is.
- The reference coercion of `&r` still requires a normalized pointee. The
  annotation fix provides one for the `first_set` form; a pointee projection
  still pending at a reference coercion remains an error.
- `masks-reduce-to-usize`, which rustc rejects with E0599, now fails with "no
  applicable method for this receiver" rather than the normalization error.
- The body query re-proves an annotation whose type the declaration phase or
  a signature already normalized. The proof is idempotent and its equalities
  match the existing ones, but it is extra work and adds one body region per
  such annotation to inspection output. Skipping already normalized roots
  would need a rule in the root collector that qualified paths share. Applying
  the declaration result directly would need a record that a node's stored
  type is a completed normal form, and declaration normalization keeps none:
  it also stores the original type in header and raw-bound contexts. Both are
  left for a later change.

## Contract

**PA31** (`pa31/tests/spec/body-projection-normalization/`, 17 source-core
reductions). The base is release candidate8. Each file is a declared batch
file, byte for byte. Two declarations disagree with rustc 1.98.0, and
`outcomes.json` records both.

| Case | Form | Rust | Base | RustGM |
| --- | --- | --- | --- | --- |
| `masks-core-form` | core's `first_set` sequence | accept | normalization | accept |
| `masks-cast-reduce-transmute` | cast, reduce, then `transmute_copy(&r)` | accept | normalization | accept |
| `masks-one-let-transmute` | one annotated `T::Unsigned` local | accept | normalization | accept |
| `masks-body-only-direct` | a direct `T: Sealed` bound | accept | normalization | accept |
| `masks-body-only-qualified` | `<T as Sealed>::Unsigned` written out | accept | normalization | accept |
| `masks-signature-control` | the signature names `T::Unsigned` | accept | accept | accept |
| `masks-reduce-to-usize` | `to_usize` on `T::Unsigned` | E0599 | normalization | method |
| `masks-wrong-output` | a vector returned as `T::Unsigned` | E0308 | result equality | result equality |
| `masks-unbounded-projection` | `<T as Sealed>::Unsigned` without `T: Sealed` | E0277 | implementation | implementation |
| `try-core-form` | core's `try_from` for `&[T]` | accept | normalization | accept |
| `try-mut-form` | core's `try_from` for `&mut [T]` | accept | normalization | accept |
| `try-let-annotated` | `let array: [T; N] = slice.try_into()?` | accept | normalization | accept |
| `try-concrete-slice` | `&[u8]` into `[u8; 4]` | accept | normalization | accept |
| `try-explicit-match` | a `match` instead of `?` | accept | accept | accept |
| `try-direct-try-from` | `<[T; N]>::try_from(slice)?` | accept | accept | accept |
| `try-projection-type` | the array impl's `Error` projection | accept | accept | accept |
| `try-wrong-error` | `?` into a `Result<_, Infallible>` | E0271 | normalization | obligation |

- "Normalization" is "associated type projection requires normalization".
- "Method" is "no applicable method for this receiver".
- "Result equality" is "body types require a provable associated-result
  equality".
- "Implementation" is "associated type requires an unambiguous
  implementation".
- "Obligation" is "trait obligation requires an unambiguous matching
  assumption or implementation".

The declarations mispredicted `masks-reduce-to-usize`: `T::Unsigned` is only
`SimdElement`, and `to_usize` is a `Sealed` method. They mispredicted
`try-wrong-error` as E0277; rustc reports E0271. Every rejection is at typing,
where rustc rejects.

## Reference update

Six existing references change. An independent read-only review accepted each
against the implementation and fixtures. Before regeneration, release
candidate8 reproduced all four PA20 references byte for byte
(`obj/parallel/simd-normalization/reference-review/`).

- Five inspection references gain the new body query's records. Each
  `--emit-trait-semantics` dump of PA20's `captured-result-default`,
  `captured-result-generic`, `unique-result-generic` and
  `member-indirect-lifetime`, and PA21's `--emit-generic-semantics`
  `shorthand-local`, adds:
  - one inference region at the annotation's source;
  - that region's equality, to the same bound region or annotation region as
    the existing region at that source;
  - in the three header-projection cases, one retained projection type with
    that region.

  Later type ids shift by one. `reference-review/check.py` rebuilds each new
  output from its old reference by that shift and those insertions alone. No
  other line, stderr or exit status changes.
- PA24's `flatten-helper-outputs/unconstrained-output` still rejects with exit
  1. Its message changes from the normalization error at `&mut g` to "type
  annotations needed" at the body, RustGM's E0282 placement. rustc 1.98.0
  reports E0282 at the `flatten::<I::Item,B,_>` path, matching the case's
  declared reason. The coercion of `&mut g` no longer stops at its nested
  pending projection, so the unconstrained `R` is reported.

Scoped generation with the WIP CLI (sha256 69dd8fed) wrote only these six
triples, with batches `pa20/reference-batches/1794553045365728841-6a882697de3d.json`,
`pa21/reference-batches/1794553055412468469-ac7474b01dc4.json` and
`pa24/reference-batches/1794553065016115974-248394a184c9.json`. The new files
equal the reviewed outputs byte for byte. No input, outcome or other reference
changes.

## Qualification

One focused receipt passes on the final source and references:
1794553083123832053 (75.3s). It covers:

- assignment metadata, formatting and the architecture audit;
- strict Clippy for the projections suite and the CLI. The same source passed
  it uncached in 1794551238325107136, and this receipt reuses that cache;
- `body_projections_normalize_like_rustc`, which types all 17 cases through
  the CLI, checks each rejection's message at its site and lowers every
  accepted case;
- `body_projection_work_tracks_annotations`;
- ten `projection_normalization`, three `normalization_occurrences` and two
  `partial_projection_selection` tests, and `definition_readiness`;
- every text case of PA10 (81), PA20 (1,616), PA21 (867) and PA24 (716).

Before regeneration, the same CLI passed 9,301 of the 9,307 active text cases
in PA1 to PA24 (`obj/parallel/simd-normalization/text-all.log`). The six
others are the reviewed references. The first attempt,
1794551134903490980, failed the architecture audit's repeated `body` filename
stem, so the test is `unnormalized_body_projections.rs`. The second,
1794551238325107136, passed everything except the four PA20 references above.

Typing work is linear in annotated bodies:

| Functions | Inference steps | Canonical steps | Trait selections |
| ---: | ---: | ---: | ---: |
| 8 | 10,052 | 3,462 | 184 |
| 32 | 39,812 | 12,438 | 736 |
| 128 | 158,852 | 48,342 | 2,944 |

Each quadrupling of the method count multiplies every measure by 3.6 to 4.0.
The rows are in `obj/parallel/simd-normalization/scaling-raw.txt`.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
is added, and no planned assignment is activated. Full release regular and
native tests, including PA24's accept/reject integration tests for
`unconstrained-output`, are left to scheduled CI.

## Core progress

No core observation of this change has run yet.
