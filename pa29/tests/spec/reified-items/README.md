# Reified fn-item packet

`reified-item-outcomes.json` freezes one provider and nine clients: their
hashes, edition, target, independent outcomes, the generated opponents and the
controls. They were declared in scratch before the rustc oracle, the baseline
probe and any candidate run, and frozen at 2026-11-09T04:05Z, after the oracle
and the baseline probe and before any candidate run.

The provider is `spec/callable-roles/provider.rs` (unchanged) followed by
authored lines. The inherited lines keep their attribution, and the repository
NOTICE applies to them. The authored lines are new.

Each provider body returns a fn item reified to a closed Rust fn pointer, and
each client calls that pointer in its own code. All nine clients return 42:

- `take`: the safe item `hold` as `fn(u8) -> i32`;
- `pick`: the instance `id::<u16>` as `fn(u16) -> u16`;
- `erase`: `erase<T: Copy>` reifies `id::<T>` to `fn(T) -> T`, which the
  consumer instantiates at u8;
- `getter`: the inherent method `W::get` as `fn(W) -> u8`;
- `method`: the trait method `<u8 as Tr>::f` as `fn(u8) -> u8`, selected by
  the consumer;
- `make-local` and `make-provider`: the const fn `make<T: Show>` reifies
  `<T as Show>::show` under a rigid `T`, shaped as core's `argument_new!`.
  The client instantiates it at its own type and at a provider type;
- `as-unsafe`: the safe item `hold` as `unsafe fn(u8) -> i32`;
- `raw-ptr`: the unsafe item `raw` as `unsafe fn(u8) -> i32`.

`make` takes `self` by value. core's formatter takes `&T`, which makes the
target higher-ranked, and higher-ranked pointers stay fenced.

The generated opponents and the older-profile control are listed in the
outcomes file and built by `crate_linking::reified_items`. rustc accepts each.
RustGM's version-13 export rejects each of these by name:

- higher-ranked and C pointers;
- closures;
- tracked functions;
- struct, variant and generic constructors;
- calls through a pointer;
- explicit `as fn(..)` casts.
