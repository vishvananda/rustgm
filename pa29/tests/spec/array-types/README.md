# Array-type transport packet

`../../array-type-outcomes.json` freezes one provider, nine clients, their
hashes, edition, target, independent outcomes and the generated export
controls. They were declared at 2026-11-09T09:31:35Z, before any oracle run
and before any T code. The indexing control's text was the one outcome left
to an approved probe on T's first development build. At 2026-11-09T09:41Z,
before any other T run, that build (SHA-256 `5f97486d...`) rejected it with
the baseline's local text, and the text is frozen as observed.

The provider reuses `spec/lifetime-nominals/provider.rs` (lines 1-147)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The array-type section
after them is authored:

- `impl<T: Copy, const N: usize> Clone for [T; N]` and `Copy for [T; N]`,
  as `../../../../pa41/tests/spec/asm-native/native.rs` declares them;
- `keep`, `empty`, `copied` and `store` over `[u8; 4]` and `[u8; 0]`: whole
  values in signatures, a copy out of `*a` and a store to `*p`;
- the generic `pass<T, const N: usize>`;
- `Buf { bytes: [u8; 4], n: u8 }` with `bytes`, and the tuple struct
  `Grid([[u8; 2]; 2])` with `rows`;
- the const-generic record `Wide<T, const N: usize> { items: [T; N] }` with
  `wide` and `unwide`.

No provider body builds an array, indexes one or unsizes one. Those forms
stay with Lane C's arrays item.

The five accepted clients each return 42:

- `values` round-trips `[u8; 4]` and `[u8; 0]` values;
- `generic` instantiates `pass` at `[u8; 4]` and `[bool; 2]`;
- `fields` moves array fields out of `Buf` and `Grid`;
- `wide` forms `Wide<u8, 2>` and `Wide<bool, 1>`;
- `copies` reuses a copied local array, calls `base::Clone::clone` and runs
  the imported `copied` and `store` bodies, through the imported `[T; N]`
  impl headers.

Four opponents are rejected:

- `wrong-length` binds `[u8; 4]` to `[u8; 3]` (E0308);
- `wrong-element` passes `[u16; 4]` (E0308);
- `moved` reuses a moved `[Opt<u8>; 1]` (E0382);
- `wide-length` binds `Wide<u8, 2>` to `Wide<u8, 3>` (E0308).

Clients build arrays locally and read lane 0 through a raw pointer, which
needs no indexing language item.

The generated controls are written under `obj/parallel/array-types` from
lines 1-147 and one declared control each:

- list and repeat rvalues;
- an element pattern;
- unsizing, which rustc also rejects without `Unsize`;
- runtime indexing behind an oracle-only bounds-check item, which both
  compilers stop without an `Index` language item (rustc with E0608, a
  recorded failed prediction);
- the older-profile gate on `spec/source-nominals/provider.rs`;
- the 6, 24 and 96 function scaling providers.
