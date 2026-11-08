# Transparent record transport packet

`transparent-record-outcomes.json` freezes one provider, nine clients, their
hashes, edition, target, independent outcomes and the generated and wire
controls. They were declared at 2026-11-08T05:02:15Z, before any oracle or
candidate run of these files.

The provider reuses `spec/lifetime-nominals/provider.rs` (lines 1-147)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The transparent section
after them is authored. It declares four `#[repr(transparent)]` structs:

- `Wrap(pub u32)`, with `wrap`, `unwrap` and `take_u32`;
- `Tagged { v: u8, unit: () }`, whose second field is trivial, with
  `tag_value`;
- the generic `Cell<T>(pub T)`, with `cell` and `into_inner`;
- `Ptr(pub *const u8)`, with `raw`.

It ends with `Clone` and `Copy` for `u32`. Those two lines were added after
the first rustc run and before any RustGM run. Without them, rustc 1.98.0
stopped with an internal compiler error ("error performing operation:
fully_perform") on every client that matches a `u32` call result. The shared
lines declare `Copy` only for `u8` and `bool`. No client or outcome changed.

The seven accepted clients each return 42:

- `wrap` and `wrap-call` build `Wrap` with its constructor and through
  `wrap`;
- `tagged` builds `Tagged` with a record literal;
- `cell` and `cell-wrap` pass a `u8` and a `Wrap` through `Cell<T>`;
- `ptr` reads through a pointer carried by `Ptr`;
- `outer` declares its own `#[repr(transparent)] struct Outer(base::Wrap,
  ())`, which needs the imported `Wrap` to be its only nontrivial field.

Two opponents are rejected:

- `mismatch` passes a `Wrap` where `u32` is expected (E0308): transparency
  changes the layout, not the type;
- `pair` declares `#[repr(transparent)] struct Pair(base::Wrap, base::Wrap)`
  (E0690), so it has two nontrivial imported fields.
