# Record-variant transport packet

`../../record-variant-outcomes.json` freezes one provider, two accepted
clients and four opponents: their hashes, edition, target, independent
outcomes, the generated controls and the forged rows. They were declared at
2026-11-09T17:19Z, before the rustc oracle, the base-pin baseline and any
candidate run of these files, and frozen after the oracle and the baseline
and before any candidate run. Its four amendments are recorded beside the
declarations they amend, each before the run it governs.

The provider's lines 1-88 are `../enum-match/provider.rs`'s, which reuse
`../callable-roles/provider.rs` and `../source-nominals/provider.rs`: the
authored sizing, receiver, Clone, Copy and Freeze declarations of PA31's
exact-source-linkage and Pin foundations. Their inherited attribution and
the repository NOTICE apply. Everything after `// Authored record-variant
transport controls.` is an authored control.

`provider.rs` declares `Shape { Unit, Pair(u8, u8), Rect { w: u8, h: u8 },
Empty {} }` with `rect` (shorthand, reordered initializers), `pair` (a called
tuple constructor), `empty`, `width` (`..` and an empty record pattern) and
`height` (a renamed binding and `_`). It also declares `NulError {
InteriorNul { position: usize }, NotNulTerminated }`, the shape of core
1.98's `ffi::c_str::FromBytesWithNulError`, with the `Clone` and `Copy` impls
its derives expand to, `interior` and `position`.

- `construct` builds `Rect` by field name in the client, matches it by name
  and calls the provider's functions; it returns 42.
- `nul` builds `InteriorNul` by field name, uses it after a copy through the
  imported `Copy` impl and matches it by name; it returns 42.
- `record-value` names `Rect` as a value and `record-call` calls it (rustc
  E0533 each), `missing-field` omits `h` (E0063) and `unknown-field` adds `d`
  (E0559).

Clients use `Copy`, not `.clone()`: a method call through an imported `Clone`
impl is a pre-existing fence of fresh consumers. `PartialEq` is not part of
the packet: an exported `==` needs the comparison and `Not` language
declarations, which version 13 does not transport. The outcome file records
both, with the probes that found them.

Pinned Rust 1.98.0 (`RUSTC_BOOTSTRAP=1`, `-Cpanic=abort`, `--cfg
rustc_oracle`) compiles the provider; the C observer returns 42 for both
accepted clients. The candidate profile is `module-bodies-v13`.
