# NonNull and pattern-type transport packet

`nonnull-pattern-outcomes.json` freezes one provider, ten clients, their
hashes, edition, target, independent outcomes and the generated and wire
controls. They were declared at 2026-11-08T09:40:34Z, before any oracle or
candidate run of these files. Later amendments are recorded in the same file,
each before the run it governs.

The provider is `spec/source-nominals/provider.rs` with
`#![feature(pattern_types)]` added after its third line. Its lines are
otherwise unchanged; they carry the attribution inherited from the earlier
packets, and the repository NOTICE applies to them. The section after them
holds:

- core 1.98.0's `pattern_type` builtin macro declaration (`core/src/pat.rs`)
  and its `NonNull` declaration (`core/src/ptr/non_null.rs`), each without its
  stability attribute;
- core's closed niche types `Nanoseconds`, `NonZeroCharInner` and
  `CodePointInner`, as `define_valid_range_type!` expands them
  (`core/src/num/niche_types.rs`), without their methods and impls;
- two authored closed ranges, `NonZeroU8` (`u8 is 1..=255`) and `Percent`
  (`i8 is 0..=100`).

The core-derived declarations are copied from the Rust project under its
MIT/Apache-2.0 licence, and the repository NOTICE applies to them.

Eight clients each bind `Opt::None` of one imported type, match on it and
return 42: `nonnull`, `nonnull-str`, `holder` (a local struct over
`NonNull<u8>`), `nanoseconds`, `nonzero-char`, `code-point`, `nonzero-u8` and
`percent`. rustc accepts all eight. RustGM admits all except `nonzero-char`,
whose char-range validity still needs its complete scalar ABI profile. Fresh and
single-crate controls retain that named gap. The zero-based integer wrappers
`Nanoseconds`, `CodePointInner` and `Percent` now use suffix niches; their `Opt`
layouts are 4x4, 4x4 and 1x1. Holder retains its separately recorded 16x8 candidate
divergence from rustc's 8x8 layout.

Honest `Percent` and its rewritten zero-excluded wire control both pass as an i8
scalar. Their `Opt::None` encodings differ: the honest 0..100 range uses ff, while
the rewritten 1..255 range uses zero. The manifest retains the historical
amendments and records this current contract; malformed wire opponents stay
rejected. New candidate native checks remain required until executed.

Two opponents are rejected:

- `private` reads `NonNull`'s private field (E0616);
- `mismatch` assigns an `Opt<NonNull<u8>>` to an `Opt<NonNull<u16>>` (E0308).

rustc 1.98.0 gave all ten declared outcomes in its first run, and the eight
accepted clients exit 42 natively. Each client's size and alignment come from
a std replica of the same declarations, because rustc ICEs on a `no_core`
size observer of these types.
