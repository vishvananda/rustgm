# Guarded-method packet

`../../guarded-method-outcomes.json` freezes one provider and 17 clients:
their hashes, edition, target, independent outcomes, the generated opponents,
the four approved movers and the controls. They were declared before the
rustc oracle, the base-pin baseline and any candidate run, and frozen before
the oracle.

The provider is `spec/callable-fields/provider.rs` (unchanged) followed by
authored lines. The inherited lines keep their attribution, and the
repository NOTICE applies to them. The authored lines are new. They are
shaped as core's `fmt/rt.rs` `Argument::fmt`, `ptr/non_null.rs`
`NonNull::add` and `as_ref`, `option.rs` `unwrap_unchecked` and
`slice/index.rs` `SliceIndex::get_unchecked`, each reduced to an unsafe
associated function over `u8`.

Sixteen clients return 42:

- `method`, `path` and `assoc`: the unsafe inherent `Lane::get`, called with
  method syntax and by path in an unsafe block, and `Lane::make`, which has
  no receiver;
- `relay` and `inside`: a provider safe free fn calling `Lane::get` in an
  unsafe block, and a provider safe method calling an unsafe free fn in one,
  so neither client needs `unsafe`;
- `bare`: `Lane::twice` calls `get` bare under
  `#[allow(unsafe_op_in_unsafe_fn)]`;
- `vtable`: `Table::invoke`, an unsafe method calling an unsafe fn field in
  an unsafe block, over client-local unsafe fns;
- `generic`, `late` and `captured`: an unsafe method of a generic impl at
  `u8`, one with a late region, and one of a lifetime-capturing impl;
- `const`: a const unsafe inherent method evaluated in a client const;
- `trait`, `bound` and `client-impl`: the unsafe trait member `Get::fetch`
  implemented unsafely, called on the concrete type, through a provider
  generic fn's bound, and implemented by the client for its own type;
- `reified`: the unsafe associated fn `Lane::scale` reified by the provider
  to `unsafe fn(u8) -> u8` and called by the client in an unsafe block;
- `formatter`: `Argument::fmt`'s shape, an unsafe method matching a tuple
  variant's unsafe fn payload, called by a safe provider fn in an unsafe
  block.

`missing` calls `Lane::get` from safe code with no unsafe block, and rustc
(E0133) and RustGM both reject it.

The generated opponents and the older-profile controls are listed in the
outcomes file and built by `crate_linking::guarded_methods`:

- a trait's unsafe method implemented safely, and the reverse: rustc
  (E0053) and RustGM's own frontend reject both, and no packet is written;
- an unsafe `extern "C"` inherent method and an unsafe trait method with a
  default body: rustc accepts both and RustGM's version-13 export keeps its
  convention and default-body fences, so they are known rejects-valid;
- an unsafe trait member, an unsafe inherent method and an unsafe call in a
  trait impl member over `spec/source-nominals/provider.rs`: versions 12 and
  10 keep their fences and version 13 exports them.
