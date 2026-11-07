# Captured-region method transport packet

`impl-region-method-outcomes.json` freezes one provider, seventeen clients,
their hashes, edition, target, independent outcomes and the wire controls.
They were declared before any oracle or candidate run of these files. Later
amendments record what was observed and why.

The provider reuses `spec/lifetime-nominals/provider.rs` (lines 1-147)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The captured-region
method section after them is authored:

- `Held<'a> { text: &'a str, n: u8 }`, covariant in `'a`, whose
  `impl<'a> Held<'a>` has a constructor `new`, `from_static`, a by-value
  `get`, a by-reference `text` that reborrows its field, `count` and
  `pick<'b>`, whose own late lifetime borrows two `Held<'a>`;
- `Cell<'a> { slot: &'a mut &'a u8 }`, invariant in `'a`, with `set`, which
  writes through its slot;
- `Pair<'a, 'b, T>`, whose impl captures two lifetimes and a type;
- `impl<'a> Slot<&'a u8>`, whose Self argument borrows the captured
  lifetime, with `peek_ref`;
- `Info<'a> { held: &'a Held<'a>, flag: bool }`, which borrows another
  lifetime struct as `PanicInfo` borrows `Location`, with `new` and `held`;
- the free functions `make`, which calls `Held::new`, `relay`, which calls
  `Held::text`, and `wrap`, whose struct literal reborrows a `Held`.

The thirteen accepted clients each return 42: `new`, `from-static`, `get`,
`text`, `pick`, `cell`, `pair`, `slot`, `info`, `path` (a path call
`Held::count(&h)`), `make`, `relay` and `wrap`. `text` keeps the returned
`&'a str` after its `Held` is dropped, which the captured result allows.

Four opponents are rejected. Each consumer re-proves the method's declared
captured and late results itself.

- `text-escape` returns `text`'s `&'x str` as `&'static str` ("lifetime may
  not live long enough").
- `cell-shrink` calls the invariant `Cell<'l>::set` with a shorter value
  ("lifetime may not live long enough").
- `info-local` returns an `Info` that borrows a local (E0515).
- `pick-borrow` keeps `pick`'s result past its shorter argument (E0597).

The `crate_linking::lifetime_methods` integration controls generate the
outcome file's other controls in their output directory:

- older-profile and fenced-shape gates on `spec/source-nominals/provider.rs`;
- a minimal forged-wire provider;
- a 6/24/96 scaling provider.

The provider's rustc-only shims (in the shared lines) are configured under
`--cfg rustc_oracle`, which RustGM never sets. The oracle links the provider
object (`--emit=link,obj`).
