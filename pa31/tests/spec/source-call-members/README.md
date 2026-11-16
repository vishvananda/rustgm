# Source call members (source core)

Six no_core source-core programs call a generic callable parameter whose
actual type is a nominal receiver with source call-trait impls, as pinned
core's `array::try_from_fn_erased` calls `generator(i)` with
`G = &mut Drain<..>` (`array/drain.rs`) when `[T; N]::map` runs. They are
checked against rustc 1.98. The prefix is the `explicit-call-members`
`native-runtime` prefix, plus core's `FnMut` and `FnOnce` impls for `&mut F`
(`ops/function.rs`) and a `Relay` modelled on `Drain`'s two call impls.

Call syntax on such a receiver executes the impl member that ordinary
dispatch selects for `<G as FnMut<Args>>::call_mut` (or `call_once`),
passing the receiver unchanged and the operands as one `Args` tuple.

- `mutable` calls `G: FnMut(u8) -> u8` with `G = &mut Relay<_>`, and
  `const-generator` does so from a const fn with `[const]` bounds, as core does;
- `once` calls `G: FnOnce(u8) -> u8` with `G = &mut Relay<_>`;
- `nested` passes `&mut &mut Relay<_>`, which selects core's `&mut F` impl;
- `owned` passes a `Count` whose own `FnMut` impl mutates a field;
- `shared` passes `&Relay<_>`, which implements no call trait (E0277).

Each accepted program exports `entry` and `probe`; `observe.c` prints
`41 59`. Outcomes were declared before the frozen rustc 1.98.0 oracle; both
are recorded in `outcomes.json` (6/6 agree). It keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/explicit_call_members.rs` checks every case and runs the accepted
ones natively.
