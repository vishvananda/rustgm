# Source language-role transport packet

`language-role-outcomes.json` freezes two providers, nine clients, their
hashes, edition, target, independent outcomes, two candidate boundaries and the
wire controls before candidate execution. Both providers reuse lines 1-58 of
`spec/source-nominals/provider.rs`: the authored sizing, receiver, Clone and
Freeze declarations of PA31's exact-source-linkage and Pin foundations. Their
inherited attribution and the repository NOTICE apply. The operator traits,
primitive impls, `Counted` and the generic functions are authored controls.

`provider.rs` declares `AddAssign`, `SubAssign` and `MulAssign` with their
`Rhs = Self` defaults and implements them for i32. The Copy-bounded generic
functions `bump` (`+=`), `lower` (two `-=`) and `scale` (`*=`) lower to checked
primitive Add, Sub and Mul. Three clients stay in range (42) and three overflow
(SIGABRT). `mismatch.rs` passes a u8 count (E0308). `own-operator.rs` applies
`+=` itself through the imported role, and `output.rs` with `sum-in-range.rs`
uses `Add` with its associated `Output`; rustc accepts both, and the candidate
boundaries record why RustGM still rejects them.

The providers carry a rustc-only shim under `--cfg rustc_oracle`:
`panic_location`, a safe foreign `abort` and the three `panic_const_*_overflow`
items, which rustc's checked arithmetic requires. RustGM never configures that
cfg, so its provider bytes are the same without the shim; its consumers lower
the shared abort exit instead. The oracle links the provider object
(`--emit=link,obj`).

Pinned Rust 1.98.0 (`RUSTC_BOOTSTRAP=1`, `-Cpanic=abort`) compiles both
providers; the C observer returns 42 for every in-range client, `own-operator`
and `sum-in-range`, and the overflow clients abort. Exploratory candidate probes
of similar scratch sources preceded the freeze; no frozen file was run by the
candidate before it.

The candidate profile is `module-bodies-v13`. The wire controls (unknown,
duplicate and wrong-kind role spellings, a missing role declaration, an
unreconstructable Div flag, a version relabel and every prefix) are mutations
of the provider export; their expectations are in the outcome file.
