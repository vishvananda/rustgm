# Auto-trait impl transport packet

`auto-impl-outcomes.json` freezes one provider and 26 clients: their hashes,
edition, target, independent outcomes, and the generated and wire controls.
They were declared at 2026-11-08T14:56:06Z, before any oracle or candidate run
of these files. Amendments are recorded beside the entries they amend.

The provider is `#![feature(negative_impls)]`, then
`spec/auto-declarations/provider.rs` (unchanged), then a tail section. The
shared lines carry the attribution inherited from the earlier packets, and the
repository NOTICE applies to them. The tail holds authored auto impls shaped
as core 1.98.0's and alloc's:

- `!Send` and `!Sync` for `*const T` and `*mut T`, `Send` for `&T` where
  `T: Sync` (`core/src/marker.rs`);
- `!UnwindSafe` for `&mut T` and `UnwindSafe` for `&T` where
  `T: RefUnwindSafe` (`core/src/panic/unwind_safe.rs`);
- `!Send` and `!Sync` for `Arguments<'_>` (`core/src/fmt/mod.rs`);
- `Cell<T>`, `Send` where `T: Send`, `!Sync` and `!RefUnwindSafe`
  (`core/src/cell.rs`);
- `Unique<T>`, `Send` and `Sync` under `T`'s own marker
  (`core/src/ptr/unique.rs`);
- `Loc<'a>`, unsafe `Send` and `Sync` (`core/src/panic/location.rs`);
- `PhantomPinned`, `!Unpin` and `!UnsafeUnpin` (`core/src/marker.rs`);
- `Rc<T, A: Allocator>`, `!Send` and `!Sync` (`alloc/src/rc.rs`), with an
  authored `Allocator` trait and `Global`. Every field of `Rc` is Send and
  Sync, so only the negative impls reject.
- authored `Foo<T>` with `Send for Foo<u8>` alone, and `List`, which holds a
  `Unique<List>` of itself.

The core- and alloc-derived shapes follow the Rust project under its
MIT/Apache-2.0 licence, and the repository NOTICE applies to them. Their
bodies are authored: the types are reduced to the fields each control needs.

Seven clients prove an imported auto trait and return 42:

- `explicit-send`: `Cell<u8>`, `&u8`, `Unique<u8>`, `Loc<'static>` and
  `Foo<u8>` are Send, and `Unique<u8>` and `Loc<'static>` are Sync, each
  through its explicit impl;
- `loc-static`: a module static of type `Loc<'static>`;
- `unwind-structural`: `&u8`, `Pair<u8>` and `Cell<u8>` are UnwindSafe;
- `cycle`: `List: Send` through its `Unique<List>` self-cycle;
- `local-cell`: a local struct holding `Cell<u8>` is Send;
- `local-impl`: a local struct holding a raw pointer, with its own unsafe Send
  impl;
- `local-specialized`: a local `W<u8>` with a raw pointer and its own
  `Send for W<u8>`.

Nineteen clients are rejected:

- E0277: `Cell<u8>: Sync` (`cell-sync`), `*const u8: Send` (`raw-send`),
  `Foo<u16>: Send` (`foo-other`, the root-type rule), `Rc<u8, Global>: Send`
  (`rc-send`), `PhantomPinned: Unpin` (`pinned`), `&mut u8: UnwindSafe`
  (`unique-mut`), `&Cell<u8>: UnwindSafe` (`cell-ref-unwind`), a shared
  `Cell<u8>` static (`cell-static`), `Arguments<'static>: Send`
  (`arguments-send`), a local holder of a raw pointer (`raw-holder`),
  `Pair<Cell<u8>>: RefUnwindSafe` (`pair-cell`), a local `W<u16>` beside its
  own `Send for W<u8>` (`local-specialized-other`) and a local negative impl
  (`local-negative`);
- `foreign-pointer`: `Send for *const Mine` (E0117);
- `foreign-reference`: `Send for &'static mut Mine` (E0321);
- `safe-impl` (E0200), `unsafe-negative` (E0198) and `unsafe-unpin-impl`
  (E0199);
- `freeze-impl`: an explicit Freeze impl (E0658).

rustc 1.98.0 gave all 27 declared outcomes, and the seven accepted clients
exit 42 natively. For `foreign-pointer` it prints E0751 before E0117. That is
recorded beside the declared codes.

The generated and wire controls are built from this provider by
`crate_linking::polarity_transport`; the outcomes file lists each with its
rustc result and expected text.
