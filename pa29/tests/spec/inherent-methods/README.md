# Inherent-method and late-region transport packet

`inherent-methods-outcomes.json` freezes one provider, ten clients, their
hashes, edition, target, independent outcomes and the wire controls. They were
declared before any oracle or candidate run of these files. Later amendments
record what was observed and why.

The provider reuses lines 1-92 of `spec/enum-methods/provider.rs` unchanged.
Those lines carry the attribution inherited from `spec/enum-match/provider.rs`,
`spec/callable-roles/provider.rs` and `spec/source-nominals/provider.rs`, and
the repository NOTICE applies to them. The checked-arithmetic shim comes from
`spec/associated-results/provider.rs`. The `Add` role, `impl Add for u8`, the
generic enum `Opt<T>`, the generic struct `Slot<T>`, their inherent impls and
`present` are authored controls.

`provider.rs` declares these inherent impls:

- `impl<T> Opt<T>` with `map` (by value, generic over `F: FnOnce(T) -> U`),
  `is_some(&self)` and `unwrap_or(self, default: T)`.
- `impl<T> Slot<T>` with the associated constructor `new` and `get(self)`.
- `impl Slot<u8>` with `peek(&self)`, which is available only on that one
  instance.

It also declares the free function `present<T>(o: &Opt<T>)`, which has a
late-bound region parameter.

The six accepted clients each return 42:

- `map-call` calls `base::Opt::Some(5u8).map(|x| x + 1)`.
- `auto-ref` calls `o.is_some()` by auto-ref and `r.is_some()` through `&&n`.
- `unwrap-or` calls `o.unwrap_or(0)` on `Some` and on `None`.
- `path-call` calls `base::Opt::<u8>::is_some(&o)`.
- `slot` uses `Slot::new`, `get` and `peek`.
- `present` passes `&o` to the free function `present`.

Four opponents are rejected:

- `missing-method` calls a method that does not exist (E0599).
- `wrong-default` passes a `u16` default (E0308).
- `moved-receiver` uses a receiver after moving it into a by-value method
  (E0382).
- `foreign-instance` calls `peek` on `Slot<u16>` (E0599).

The `crate_linking::inherent_transport` integration controls generate the
outcome file's other controls in their output directory:

- an older-profile `source-nominals` inherent impl;
- three late-region providers;
- a 6/24/96 scaling provider.

The rustc oracle saw the same source bytes.

Provider bodies call no inherent method and contain no match guard.

The provider carries rustc-only shims under `--cfg rustc_oracle`. RustGM never
configures that cfg. The oracle links the provider object (`--emit=link,obj`).
