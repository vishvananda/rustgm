# Body projection normalization

Seventeen no_core libraries check projections that a body meets before any
normalization, against rustc 1.98. They reduce two pinned core forms in
portable-simd's `core_simd`:

- `masks-*`: `masks.rs` `first_set`, which annotates `let min_index:
  T::Unsigned` through `MaskElement`'s supertrait `Sealed`, and no signature
  names that type. Its trait and type shapes follow `vector.rs` and
  `masks.rs`.
- `try-*`: `vector.rs` `TryFrom<&[T]>` and `TryFrom<&mut [T]>` for `Simd`,
  `Ok(Self::from_array(slice.try_into()?))`. Its conversion, array and
  `Result` try impls follow core's `convert/mod.rs`, `array/mod.rs` and
  `result.rs`.

Bodies are reduced, and each file's first comment names its origin.

- Positives:
  - the masks forms: core's sequence, cast then reduce then
    `transmute_copy(&r)`, one annotated local, a direct `T: Sealed` bound, a
    qualified `<T as Sealed>::Unsigned`, and a control whose signature names
    `T::Unsigned`;
  - the try forms: core's two forms, a `let` annotation, a concrete `&[u8]`
    into `[u8; 4]`, and three controls: an explicit `match`, a direct
    `<[T; N]>::try_from(slice)?` and the array impl's `Error` projection.
- Negatives:
  - `masks-reduce-to-usize` calls the `Sealed` method `to_usize` on
    `T::Unsigned`, which is only `SimdElement` (E0599);
  - `masks-wrong-output` returns a vector as `T::Unsigned` (E0308);
  - `masks-unbounded-projection` names `<T as Sealed>::Unsigned` without
    `T: Sealed` (E0277);
  - `try-wrong-error` applies `?` in a function returning
    `Result<_, Infallible>` (E0271).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file. `obj/parallel/simd-normalization/declared.json`
records ten batches, including two malformed ones, and the ungraded
diagnostics. Two declarations disagree with rustc: `masks-reduce-to-usize`
was declared accepted, and `try-wrong-error` was declared E0277.
`outcomes.json` records both, with each file's sha256, the base candidate8
result and RustGM's. The reduced cases keep Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/unnormalized_body_projections.rs` checks typing through the CLI,
each rejection's message at its site and prepared lowering for every accepted
case. See `docs/pa31-body-projection-normalization.md`.
