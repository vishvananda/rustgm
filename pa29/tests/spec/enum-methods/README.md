# Called-constructor and imported-variant transport packet

`enum-methods-outcomes.json` freezes one provider, eight clients, their hashes,
edition, target, independent outcomes and the wire controls. They were declared
before any oracle or candidate run of these files.

The provider reuses lines 1-88 of `spec/enum-match/provider.rs` unchanged.
Those lines reuse `spec/callable-roles/provider.rs` and
`spec/source-nominals/provider.rs`, so their inherited attribution and the
repository NOTICE apply. The `bool` Clone/Copy impls, the enums and their
functions are authored controls.

`provider.rs` declares two enums:

- A generic `Opt<T> { None, Some(T) }`. `wrap` builds it with the called
  constructor `Opt::Some(x)`. `or` matches it. `map` (`F: FnOnce(T) -> U`)
  rebuilds `Opt::Some(f(x))`.
- A monomorphic `Pair { One(u8), Two(u8, u8) }`. `two` calls its two-field
  constructor and `first` matches it.

The five accepted clients each return 42:

- `called` calls through `wrap`, `or`, `two` and `first`.
- `mapped` passes a function item and its own closure to `map`.
- `named` writes `base::Opt::None`, `base::Opt::Some(5u8)` and
  `base::Pair::Two(42u8, 1u8)` and matches imported variant patterns.
- `guarded` matches with a guard over imported variants.
- `by-ref` uses `match &o` and a `&base::Opt<u8>` parameter.

Three opponents are rejected:

- `non-exhaustive` omits `base::Opt::None` (E0004).
- `wrong-payload` builds `base::Opt<u8>` from `1u16` (E0308).
- `wrong-arity` calls `base::Opt::Some` with two arguments (E0061).

Inherent methods, reference parameters in provider bodies and provider-body
guards are outside this checkpoint. The outcome file records these deviations,
the three oracle rounds and the amended enum-match `called` control.

The provider carries the callable-role rustc-only shim under
`--cfg rustc_oracle`. RustGM never configures that cfg. The oracle links the
provider object (`--emit=link,obj`).
