# Lifetime-nominal transport packet

`lifetime-nominal-outcomes.json` freezes one provider, fourteen clients, their
hashes, edition, target, independent outcomes and the wire controls. They were
declared before any oracle or candidate run of these files. Later amendments
record what was observed and why.

The provider reuses `spec/inherent-methods/provider.rs` (lines 1-126)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The lifetime-nominal
section after them is authored:

- `Arguments<'a> { text: &'a str }`, covariant in `'a`, with the free
  functions `take`, `first` and `text`, and a method in
  `impl Arguments<'static>`, which captures no lifetime;
- `Inv<'a> { slot: &'a mut &'a u8 }`, invariant in `'a`, read by `read`;
- `Ref<'a, T> { r: &'a T }`, whose field implies `T: 'a`, with `new_ref`
  and `get`;
- `enum Text<'a> { Empty, Word(&'a str) }`, with `is_word` and `word`, which
  calls the constructor `Text::Word` in the provider.

The nine accepted clients each return 42:

- `args`, `first`, `field` and `fixed` build `Arguments` literals and call
  `take`, `first`, `text` (reading the first byte through a raw pointer) and
  the `'static` method;
- `shrink` passes `Arguments<'b>` where `Arguments<'a>` is expected, which
  covariance allows;
- `invariant` keeps `Inv<'a>` at its own lifetime;
- `reference` round-trips a borrowed `u8` through `new_ref` and `get`;
- `text` and `word` build `Text::Word` in the client and in the provider.

Five opponents are rejected. Each consumer derives the variance and the
implied outlives requirements from the transported fields itself.

- `inv-shrink` shrinks the invariant `Inv` ("lifetime may not live long
  enough").
- `args-grow` and `text-grow` lengthen a covariant lifetime ("lifetime may not
  live long enough").
- `escape` returns a `Ref` to a local (E0515).
- `outlives` names `Ref<'a, T>` without `T: 'a` (E0309).

The `crate_linking::lifetime_transport` integration controls generate the
outcome file's other controls in their output directory:

- older-profile and fenced-shape gates on `spec/source-nominals/provider.rs`;
- a minimal forged-wire provider;
- a 6/24/96 scaling provider.

The provider's rustc-only shims (in the shared lines) are configured under
`--cfg rustc_oracle`, which RustGM never sets. The oracle links the provider
object (`--emit=link,obj`).
