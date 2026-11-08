# Pattern-type coercions

Sixteen no_core source-core libraries for `CoerceUnsized` and `DispatchFromDyn`
impls between pattern types:

- core's exact `impl<T: PointeeSized, U: PointeeSized>
  CoerceUnsized<pattern_type!(*const U is !null)> for
  pattern_type!(*const T is !null) where T: Unsize<U>` (`pat.rs:80`) over
  core's raw pointer impls, a `*mut` to `*const` pair, a type parameter base
  and core's `NonNull` shape, whose one changed field selects the pattern impl;
- opponents with two different patterns, a `u32` base, a pattern type to a
  plain pointer (E0377), a `*const` to `*mut` pair, no `T: Unsize<U>`, no raw
  pointer impl and `NonNull` without the pattern impl (E0277);
- a `DispatchFromDyn` impl between two equal range patterns and an opponent
  with two different ones;
- three controls rustc 1.98.0 accepts that this analysis rejects by name: a
  non-null pattern over a reference, a non-null pattern over a pattern type,
  and a body that executes the coercion.

rustc 1.98.0 requires one identical pattern, then registers
`base_a: CoerceUnsized<base_b>` as an ordinary obligation. The bases are not
matched against a built-in pointer rule. Outcomes were declared before the
frozen rustc 1.98.0 oracle; both are recorded in `outcomes.json`. All 16
statuses agree. The declared reason for `pattern-to-pointer.rs` guessed E0376;
rustc reports E0377, and the two different-pattern opponents carry no code. The
lang-item prefix is the census batch 2 core fragment used by
`generic-non-null-patterns/`. The pointer, pattern and `NonNull` impls are
core's, without their stability attributes. They keep Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/refined_coercions.rs` checks every case.
See `docs/parallel/census-batch16.md`.
