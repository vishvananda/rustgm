# Source declared-result transport packet

`associated-result-outcomes.json` freezes one provider, five clients, their
hashes, edition, target, independent outcomes and the wire controls before
candidate execution. The provider reuses lines 1-58 of
`spec/source-nominals/provider.rs`: the authored sizing, receiver, Clone and
Freeze declarations of PA31's exact-source-linkage and Pin foundations. Their
inherited attribution and the repository NOTICE apply. The `Add` and `Shape`
traits, their primitive impls and the generic functions are authored controls.

`provider.rs` declares `Add<Rhs = Self>` with its associated `Output` and a
`Shape` trait with an associated `Item`, implements them for i32 and u8, and
exports two generic bodies whose results are projections: `fetch` returns
`<T as Shape>::Item` and `sum` returns `<T as Add>::Output` from `a + b`.
`fetch-hit` and `sum-in-range` return 42, `sum-overflow` aborts in the i32
`Add` body, `fetch-mismatch` binds the i32 result to a u8 (E0308) and
`fetch-unimplemented` instantiates `fetch` for i32, which has no `Shape` impl
(E0277).

The provider carries a rustc-only shim under `--cfg rustc_oracle`:
`panic_location`, a safe foreign `abort` and `panic_const_add_overflow`, which
rustc's checked arithmetic requires. RustGM never configures that cfg; its
consumers lower the shared abort exit instead. The oracle links the provider
object (`--emit=link,obj`).

Pinned Rust 1.98.0 (`RUSTC_BOOTSTRAP=1`, `-Cpanic=abort`) compiles the
provider; the C observer returns 42 for both in-range clients, `sum-overflow`
aborts (observer status 134) and the two opponents fail with E0308 and E0277.
Exploratory candidate probes of similar scratch sources preceded the freeze; no
frozen file was run by the candidate before it.

The candidate profile is `module-bodies-v13`. The wire controls (swapped,
retargeted and out-of-frame definition rows, a value forged to the impl's Self
type, a missing value, dropped result columns, a version relabel and every
prefix) are mutations of the provider export; their expectations are in the
outcome file.
