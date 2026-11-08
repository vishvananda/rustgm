# Raw-pointer and slice transport packet

`pointer-slice-outcomes.json` freezes one provider, eleven clients, their
hashes, edition, target, independent outcomes and the generated and wire
controls. They were declared before any oracle or candidate run of these
files.

The provider reuses `spec/lifetime-nominals/provider.rs` (lines 1-147)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The raw-pointer and slice
section after them is authored:

- `same`, `same_mut`, `text_ptr` and `generic`, which pass a `*const u8`, a
  `*mut u8`, a `*const str` and a `*const T` through;
- `Raw`, a record with a raw-pointer field, with `raw` and `raw_n`;
- `pass`, `width` and `generic_slice` over `&[u8]` and `&[T]`, and `View<'a>`,
  a lifetime-generic record holding a slice, with `view`.

The seven accepted clients each return 42:

- `same`, `same-mut` and `generic` dereference a passed-through pointer;
- `raw-field` and `raw-literal` build and read `Raw`;
- `text-ptr` and `slices` relay pointers and slices through the provider's
  signatures. The shared lines have no `CoerceUnsized`, so no client builds a
  slice value.

Four opponents are rejected:

- `wrong-pointee` passes a `*const u16` (E0308);
- `const-for-mut` passes a `*const u8` to `same_mut` (E0308); RustGM reports
  that a raw-pointer coercion cannot strengthen mutability;
- `deref-safe` dereferences a returned pointer outside `unsafe` (E0133);
- `slice-mismatch` passes a `&[u16]` (E0308).

The `crate_linking::addressed_views` integration controls generate the
outcome file's other controls in their output directory:

- older-profile gates on `spec/source-nominals/provider.rs`;
- a minimal forged-wire provider of the shared lines, `same`, `text_ptr` and
  `width`;
- a provider of `pair<T, U: ?Sized>`, whose forged `[T]` row names `U`;
- a 6/24/96 scaling provider.

The provider's rustc-only shims (in the shared lines) are configured under
`--cfg rustc_oracle`, which RustGM never sets. The oracle links the provider
object (`--emit=link,obj`).
