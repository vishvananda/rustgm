# Pointer-call packet

`pointer-call-outcomes.json` freezes one provider and 13 clients: their
hashes, edition, target, independent outcomes, the generated opponents and the
controls. They were declared in scratch before the rustc oracle, the base2
baseline and any candidate run, and frozen at 2026-11-09T13:10Z, after the
oracle and the baseline and before any candidate run.

The provider is `spec/reified-items/provider.rs` (unchanged) followed by
authored lines. The inherited lines keep their attribution, and the repository
NOTICE applies to them. The authored lines are new.

Each provider body calls a closed Rust fn pointer, or returns a constructor
reified to one. All 13 clients return 42:

- `call`: `call_ptr` calls its `fn(u8) -> i32` parameter;
- `twice`: `twice` calls its pointer twice, nested, with a client-local item;
- `generic`: `apply_ptr<T>` calls its `fn(T) -> T` parameter, instantiated by
  the consumer at u8;
- `reified`: `reified_call` reifies `hold` to a local pointer and calls it;
- `block`: a safe fn calls its `unsafe fn` parameter in an unsafe block;
- `safe-block`: a safe pointer in an unnecessary unsafe block, which records
  Safe as a direct call does. rustc warns `unused_unsafe` on the provider;
- `nested`: an unsafe fn calls its unsafe pointer in an unsafe block;
- `bare`: an unsafe fn calls it bare under
  `#[allow(unsafe_op_in_unsafe_fn)]`;
- `formatter`: the fmt-shaped twin, a free unsafe fn calling its
  `unsafe fn(u8, u8) -> i32` formatter parameter in an unsafe block. core's
  `Argument::fmt` stores the formatter in a record variant instead;
- `method`: a safe pointer call in `Runner::run`, an inherent associated fn of
  a unit struct;
- `ctor`, `variant` and `some`: the tuple-struct constructor `P`, the variant
  constructor `E::A` and the generic `Opt::Some`, each reified by a provider
  body and called by the client. The consumer defines each constructor
  instance itself.

The generated opponents and the older-profile control are listed in the
outcomes file and built by `crate_linking::pointer_calls`. RustGM's version-13
export rejects each by name:

- higher-ranked and C pointers;
- an unsafe pointer call in an associated fn;
- the stored formatter in a record variant, and a call through a pointer
  field. Both are pre-existing nominal-header fences outside R2's body scope;
- a const fn calling a pointer, which rustc also rejects.

rustc accepts all but the last, so the first five are known rejects-valid.
