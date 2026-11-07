# Provider-guard and single-variant transport packet

`provider-guard-outcomes.json` freezes one provider, eleven clients, their
hashes, edition, target, independent outcomes and the wire controls. They were
declared before any oracle or candidate run of these files. Later amendments
record what was observed and why.

The provider reuses `spec/inherent-methods/provider.rs` (lines 1-126)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The provider-guard section
after them is authored:

- `guard`, a by-value guard with a wildcard fallback;
- `pick`, guards over a borrowed scrutinee, where a failed guard retries the
  next arm;
- `keep`, a guard that calls a generic `Fn(u8) -> bool` predicate;
- `impl Opt<u8> { fn big(self) }`, a guard in an inherent method;
- the single-variant enum `One<T>` with `only` (a match), `get` (an
  irrefutable `let`) and `only_large` (a guarded single-variant match).

The eight accepted clients each return 42: `guarded`, `fallback`, `retry`,
`predicate`, `method`, `single`, `single-guard` and `own-single` (the client's
own match over the imported single-variant enum).

Three opponents are rejected:

- `moved-scrutinee` reuses a moved `Opt<u8>` (E0382).
- `wrong-predicate` passes `fn(u8) -> u8` for the `bool` predicate (E0271).
- `wrong-single` binds `only`'s `u8` result as `u16` (E0308).

The `crate_linking::guard_transport` integration controls generate the
outcome file's other controls in their output directory: an older-profile
`source-nominals` guard, the amended enum-match `guarded` control, a
minimal forged-wire provider and a 6/24/96 scaling provider.

The provider carries rustc-only shims under `--cfg rustc_oracle`. RustGM never
configures that cfg. The oracle links the provider object (`--emit=link,obj`).
