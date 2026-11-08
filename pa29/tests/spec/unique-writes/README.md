# Unique-path write transport packet

`unique-write-outcomes.json` freezes one provider, eight clients, their
hashes, edition, target, independent outcomes and the wire controls. They were
declared before any oracle or candidate run of these files. Later amendments
record what was observed and why.

The provider reuses `spec/lifetime-nominals/provider.rs` (lines 1-147)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The unique-path write
section after them is authored:

- `Count<'a> { n: &'a mut u8 }`, with `bump`, which writes through `c.n` from
  an immutable binding, and `lend`, which reborrows `&mut *c.n` and writes
  through the reborrow;
- `Cell<'a> { slot: &'a mut &'a u8 }`, whose method `set` writes through its
  immutable by-value receiver, and the free function `put`;
- `Outer<'a, 'b> { inner: &'a mut Count<'b> }`, with `deep`, which writes
  through two unique references.

The six accepted clients each return 42: `bump`, `after` (which reads the
written byte after the call), `lend`, `set`, `put` and `deep`.

Two opponents are rejected by the client's own borrow checker, as rustc
rejects them:

- `shared-path` writes through `c.n` behind a shared `&Count` (E0594);
- `field-write` assigns the field `c.n` of an immutable binding (E0594).

The `crate_linking::unique_writes` integration controls generate the outcome
file's other controls in their output directory:

- an older-profile gate on `spec/source-nominals/provider.rs`;
- a minimal forged-wire provider;
- a 6/24/96 scaling provider.

The provider's rustc-only shims (in the shared lines) are configured under
`--cfg rustc_oracle`, which RustGM never sets. The oracle links the provider
object (`--emit=link,obj`).
