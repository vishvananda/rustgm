# Source object calls (source core)

Four no_core source-core programs call a `dyn Fn` or `dyn FnMut` object whose
call traits the library declares itself, as pinned core does, checked against
rustc 1.98. The prefix is the `const-eval-select-ctfe` prefix up to its `Fn`
trait, plus core's `&T` to `&U` `CoerceUnsized` impl and its `DispatchFromDyn`
declaration with the two reference impls.

An object call selects its vtable slot through the call trait's application.
With a source core that trait is the library's `#[lang = "fn"]` or
`#[lang = "fn_mut"]` declaration, not a compiler-provided one.

- `shared` calls `&dyn Fn(u8) -> u8` and `mutable` calls
  `&mut dyn FnMut(u8) -> u8`, each made from a capturing closure;
- `nested` calls through `&&dyn Fn(u8) -> u8`;
- `shared-as-mutable` calls a `&dyn FnMut(u8) -> u8` (E0596).

Every case has a root `main` and is checked with `--emit-lowir`. Outcomes were
declared before the frozen rustc 1.98.0 oracle; both are recorded in
`outcomes.json` (4/4 agree). It keeps Rust's MIT/Apache-2.0 attribution; see
root NOTICE and `third_party/sources.json`.
`dev/tests/source_closures/dispatched_calls.rs` checks every case.
