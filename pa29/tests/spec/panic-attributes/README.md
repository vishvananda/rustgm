# Panic attributes transport packet

`panic-attribute-outcomes.json` freezes one provider, seven clients, their
hashes, edition, target, independent outcomes and the wire controls. They were
declared before any oracle or candidate run of these files. Later amendments
record what was observed and why.

The provider reuses `spec/inherent-methods/provider.rs` (lines 1-126)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The panic-attributes
section after them is authored:

- a `panicking` module whose `panic(&'static str) -> !` carries core 1.98's
  `#[cold]`, `#[inline(never)]`, `#[rustc_const_stable_indirect]` and
  `#[lang = "panic"]`, without its caller tracking and formatting body;
- `check`, which calls `panic` from an `if` branch;
- `#[cold] rare`, `#[cold] #[inline(never)] rarer`, and `relay`, which calls
  `rare`;
- `#[track_caller]` on a free function `traced` and an inherent method
  `Gauge::read`, neither of which calls a tracked function.

The three accepted clients each return 42: `check`, `role` (the client's own
unreached `panic` call) and `rare` (`rare`, `rarer` and `relay`). rustc also
accepts `traced` and `method` and returns 42, but RustGM rejects both by name
until the provider's caller `Location` and the caller ABI transport.

Two opponents are rejected:

- `panic-type` passes an integer to `panic` (E0308).
- `rare-arity` calls `rare` with no argument (E0061).

The `crate_linking::cold_transport` integration controls generate the outcome
file's other controls in their output directory: older-profile cold, caller
tracking and role gates on `spec/source-nominals/provider.rs`, a pending
nonconforming role, a minimal forged-wire provider and a 6/24/96 scaling
provider.

The provider's rustc-only shims (in the shared lines) are configured under
`--cfg rustc_oracle`, which RustGM never sets. The oracle links the provider
object (`--emit=link,obj`).
