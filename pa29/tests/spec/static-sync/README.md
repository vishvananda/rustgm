# Shared-static Sync packet

`static-sync-outcomes.json` freezes one provider, four clients, their hashes,
edition, target, independent outcomes and the generated controls. They were
declared at 2026-11-08T03:57:43Z, before any oracle or candidate run of these
files.

The provider reuses `spec/lifetime-nominals/provider.rs` (lines 1-147)
unchanged. Those lines carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. They declare no `sync`
lang item. The one authored line after them is `answer`, which returns 42.

rustc 1.98.0 requires every static that is not `mut`, not foreign and not
thread-local to have a `Sync` type, and it requires the `sync` lang item to
prove that. A source client takes every lang item from its source core, so a
client of these lines cannot declare a shared static:

- `plain-static` (a crate-level `static N: u8`) and `local-static` (a
  function-local static) are rejected with rustc's "requires `sync`
  lang_item";
- `mut-static` reads a `static mut N` in `unsafe` and returns 42, because a
  mutable static needs no `Sync`;
- `answer` calls the provider and returns 42.

Unsafe auto traits do not transport yet, so the provider cannot declare
`Sync` and still export. The positive `T: Sync` controls are therefore
single-crate source cores, which `crate_linking::synced_statics` generates
from the shared lines:

- no `sync` item: rejected with "requires `sync` lang_item";
- the `sync` item and `static N: u8`: accepted;
- a record holding `*const u8`, with no negative impl, so the raw pointer is
  structurally `Sync`, as in rustc: accepted;
- the same with `unsafe impl Sync for Raw {}`: accepted;
- the same as `static mut`: accepted;
- `impl !Sync for Raw {}` (E0277 in rustc): rejected with "shared static type
  must implement Sync";
- the negative impl with `static mut`: accepted.

Two controls were amended after their first rustc run and before any RustGM
run, as the outcome file records. The first generated controls redefined the
shared lines' `get` (E0428), so they read `N` through `read_n` instead. The
raw-pointer control was declared rejected, but rustc accepts it because the
shared lines lack core's `impl !Sync for *const T`. The rejected `T: Sync`
control moved to the explicit negative impl.
