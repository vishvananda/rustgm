# Source enum-and-match transport packet

`enum-match-outcomes.json` freezes one provider, seven clients, their hashes,
edition, target, independent outcomes and the wire controls before candidate
execution. The provider reuses lines 1-88 of `spec/callable-roles/provider.rs`,
which reuse lines 2-57 of `spec/source-nominals/provider.rs`: the authored
sizing, receiver, Clone, Copy and Freeze declarations of PA31's
exact-source-linkage and Pin foundations. Their inherited attribution and the
repository NOTICE apply. The enums and their matching functions are authored
controls.

`provider.rs` declares a fieldless `Two { A, B }` with `first`, `second` and a
constant-arm `pick`; a `#[repr(u8)] Tag { Low = 3, High = 39 }` with `low`,
`high` and `code` (`t as u8`); and a generic `Opt<T> { None, Some(T) }` with
`none`, `some`, `or`, `map` (`F: FnOnce(T) -> U`) and `peek`, which binds its
payload by reference (`Opt::Some(ref x) => *x`). `fieldless`, `tagged`,
`generic`, `mapped` and `by-ref` exercise them and return 42. `mapped` passes
both a function item and its own closure to `map`. `wrong-enum` passes a `Tag`
to `pick` and `wrong-payload` passes a `u16` default to `or` over `Opt<u8>`;
both fail with E0308.

Payload values use the braced constructor `Opt::Some { 0: x }`, `map` is a
free function and no frozen fixture carries a match guard: called constructor
items, inherent methods, guards, `match &o` and reference parameters are
outside this checkpoint. The outcome file records these deviations and the
generated controls that fence called constructors and guards by name.

The provider carries the callable-role rustc-only shim under
`--cfg rustc_oracle`. RustGM never configures that cfg. The oracle links the
provider object (`--emit=link,obj`).

Pinned Rust 1.98.0 (`RUSTC_BOOTSTRAP=1`, `-Cpanic=abort`) compiles the
provider; the C observer returns 42 for all five accepted clients and both
opponents fail with E0308. Exploratory candidate probes of similar scratch
sources preceded the freeze; no frozen file was run by the candidate before it.

The candidate profile is `module-bodies-v13`. The wire controls (forged
constructor tests, a foreign variant, forged discriminants and shapes, a
version relabel and every prefix) are mutations of the provider export; their
expectations are in the outcome file.
