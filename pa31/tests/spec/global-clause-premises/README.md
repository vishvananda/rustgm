# Global clause premises

Twenty-five no_core libraries check a body whose caller declares a global
where clause, one that names no parameter, against rustc 1.98. They reduce
portable-simd's `ops.rs` `int_divrem_guard` `Div` for `Simd<i8, N>`, which
declares `where i8: SimdElement` and selects `Simd::splat(1 as _)` through
`MaskElement`, whose supertrait is `SimdElement<Mask = Self>`. Their trait and
type shapes follow `core_simd`'s `vector.rs`, `masks.rs`, `select.rs`,
`simd/cmp/eq.rs` and `ops.rs`.

Bodies are reduced, and each file's first comment names its origin. Every
file shares one prefix through line 74.

- Positives:
  - core's guard for `i8` and `u8`, both in one library, and without the
    where clause;
  - the guard's parts under the clause: the `&` of two comparisons, then
    `select`; `select` alone; `simd_eq` with and without a splat; a
    `<i8 as SimdElement>::Mask` result, `let` annotation and nested
    projection; `u8`'s `i8` mask; and a generic companion bound;
  - without the clause: `select` with a cast, typed, swapped or UFCS splat, the
    `&` then `select`, `simd_eq(..).any()`, mask values and integer literal
    fallback.
- Negatives:
  - `select-wrong-element` selects a `u16` splat for `i8` lanes (E0308);
  - `select-non-mask` calls `select` on `u8` (E0599);
  - `global-where-wrong-projection` returns `u16` as
    `<u16 as SimdElement>::Mask`, which is `i8` (E0308);
  - `global-clause-select-wrong-output` returns `select`'s `Simd<i8, N>` as
    `Simd<i32, N>` (E0308).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file. `obj/parallel/simd-divrem/declared.json` records
four batches. Batch 1 lacked `Neg` impls for its negated literals and batch 2
the `freeze` lang item that rustc requires for its `const fn`s; both are
recorded as malformed. Batches 3 and 4 are this packet, and every declaration
agrees with rustc. `outcomes.json` records each file's sha256, the base
candidate9 result and RustGM's. The reduced cases keep Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/global_clause_premises.rs` checks typing through the CLI, each
rejection's message at its site and prepared lowering for every accepted
case. See `docs/pa31-global-clause-premises.md`.
