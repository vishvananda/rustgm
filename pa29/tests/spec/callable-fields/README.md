# Callable-field packet

`callable-field-outcomes.json` freezes one provider and 16 clients: their
hashes, edition, target, independent outcomes, the generated opponents and the
controls. They were declared before the rustc oracle, the base-pin baseline
and any candidate run, and frozen before the oracle. rustc 1.98.0 then hit an
internal compiler error on the `apply` and `pair` clients, at a match on an
i32 value, which the no_core provider gives no Copy impl. Amendment 1
rewrote those two clients to avoid that match before any candidate run.
Amendment 2 records a failed scaling prediction: an empty client's header
work grows faster than declared, and the declared `u8` twins grow the same
way, so the fn fields add only a constant per struct.

The provider is `spec/pointer-calls/provider.rs` (unchanged) followed by
authored lines. The inherited lines keep their attribution, and the repository
NOTICE applies to them. The authored lines are new. They are shaped as core's
`task/wake.rs` `RawWakerVTable`, `io/error.rs` `OsFunctions` and
`marker/variance.rs` markers, each reduced to a zero-slot Rust fn field.

`Pair` is the hand-written expansion of `#[derive(Clone, Copy)]` for a
non-generic struct: `impl Clone` whose `clone` returns `*self`, and an empty
`impl Copy`. The derive's `AssertParamIsClone` field assertions are left out,
because the no_core provider has no such helper. `RawWakerVTable`'s
`derive(PartialEq)` is left out too: comparing fn pointers is unpredictable,
and rustc warns `unpredictable_function_pointer_comparisons`. The variance and
auto-trait shapes do not depend on that derive.

Fifteen clients return 42:

- `via`, `slot-new`, `method` and `store`: a struct's `fn(u8) -> i32` field,
  called by a provider fn and an inherent method, built by a provider struct
  expression, and assigned by the provider;
- `table`: two `unsafe fn` fields over client-local unsafe fns, called by an
  unsafe provider fn in an unsafe block;
- `op` and `op-new`: a tuple struct's fn field, built by the client and by a
  provider call of its constructor;
- `choose` and `computed`: a tuple variant's fn payload, built by the client
  and by a provider call of its constructor;
- `apply`: a generic `Apply<T> { f: fn(T) -> T }` instantiated at u8;
- `pair`: the derived `Pair`, copied, cloned and called through both copies;
- `send`: `Send + Sync` of a struct with a fn field;
- `widen`, `inv` and `markers`: `Contra<T>(PhantomData<fn(T)>)` widened in
  its contravariant direction, `Inv<T: ?Sized>(PhantomData<fn(T) -> T>)` at
  `u8` and `[u8]`, and both built by generic provider fns.

`narrow` narrows `Contra` in the unsound direction, and rustc and RustGM
both reject it.

The generated opponents and the older-profile controls are listed in the
outcomes file and built by `crate_linking::fielded_callables`. RustGM's
version-13 export rejects each opponent by name:

- higher-ranked, early-region and C fn fields, and core's `ArgumentType`
  shape, whose formatter is higher-ranked (outcomes amendment 4).

rustc accepts all four, so they are known rejects-valid. Two former opponents
export: a safe associated fn calling an unsafe fn field in an unsafe block
since the associated-unsafe item (outcomes amendment 3), and a record variant
holding a zero-slot fn since Lane D's (i) (outcomes amendment 4).
