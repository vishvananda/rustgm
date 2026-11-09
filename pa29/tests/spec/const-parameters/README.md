# Const-parameter transport packet

`const-parameter-outcomes.json` freezes one provider, eight clients, their
hashes, edition, target, independent outcomes and the generated export
controls. They were declared at 2026-11-09T01:47:16Z, before any oracle or
candidate run of these files. Four provider amendments are recorded beside
the declaration, each before the run it governs; none changes an earlier
outcome. The fourth, at 2026-11-09T03:22:33Z, appends the applications below
and declares two clients, `applied` and `misapplied`.

The provider reuses `spec/lifetime-nominals/provider.rs` (lines 1-147)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The const-parameter
section after them is authored:

- `Tag<const N: usize>`, its inherent `impl<const N: usize> Tag<N>` with
  `new`, and `three() -> Tag<3>`;
- `Mixed<const N: usize, T>`, whose const parameter precedes its type slot,
  `impl<T, const N: usize> Mixed<N, T>`, whose const parameter follows it, and
  the generic function `wrap<const N: usize>`;
- `Signed<const V: i8>`, `Flag<const B: bool>` and `Letter<const C: char>`
  with `minus_one() -> Signed<-1>`, `on() -> Flag<true>` and
  `letter() -> Letter<'z'>`;
- body applications with const arguments: `nine` calls `wrap::<9>`, which
  fills an own formal, and `five` calls `Tag::<5>::new`, which fills its
  impl's; `rigid` and `forward` pass their own rigid `N` to the same callees.

`Signed<-1>` needs the `Neg` language item, and its i8 impl needs `Copy`.
Version thirteen transports no unary rvalue, so `neg` subtracts from zero
through a `Sub` impl, which overflows exactly where negation does. Neither
compiler calls `neg` for the literal argument. The oracle-only
`panic_const_sub_overflow` sits beside `panic_const_add_overflow`.

The four accepted clients each return 42:

- `tag` instantiates the imported impl at `Tag::<3>` and binds `three()`;
- `mixed` instantiates both slot orders and calls `wrap::<9>`;
- `scalars` binds `Signed<-1>`, `Flag<true>` and `Letter<'z'>`;
- `applied` runs the imported bodies of `nine`, `five`, `rigid::<7>` and
  `forward::<4>`.

Four opponents are rejected:

- `wrong-value` binds `three()` to `Tag<4>` (E0308);
- `wrong-sign` binds `minus_one()` to `Signed<1>` (E0308);
- `wrong-order` writes `Mixed::<u8, 7>` (E0747);
- `misapplied` binds `rigid::<7>()` to `Tag<6>` (E0308).

The generated controls are written under `obj/parallel/const-parameters` from
lines 1-147 and one declared control each. rustc 1.98.0 accepts both. A body
reading `N` as a value keeps "source rvalue requires a complete wider transport
row", and a const default keeps "source parameter defaults are outside the
initial profile".
