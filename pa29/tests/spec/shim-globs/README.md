# Glob re-exports of a dependency

The std graph's `rustc_std_workspace_core` is a `#![no_core]` crate whose
only item is `pub use core::*;`. Crates such as `cfg_if`, `hashbrown` and
`unwind` name it as their `core`. Rustc 1.98.0, observed here, binds through
such a glob every name of the target module that is visible outside its
crate, in the type, value and macro namespaces. An explicit item of the shim
shadows a glob name. Two globs that give one name different targets make it
ambiguous, which is reported only where the name is used.

`core.rs` is a no_core source-core provider named `core`, with the
`../no-std-clients/provider.rs` prelude shape and these glob controls:

- `internal` is `pub(crate)`, `private` is private, and `sealed` is a
  private module, so no glob of `core` carries them;
- `left::shared` and `right::shared` are different functions.

The shims are compiled with `--extern core=` naming `core.rs`:

- `shim.rs` has the upstream shape without its `compiler_builtins` import;
- `shadow-shim.rs` adds its own empty `option` module;
- `ambiguous-shim.rs` also globs `core::left` and `core::right`.

Each client names a shim as its `core`, and `core.rs` is in its dependency
inventory. `../no-std-clients/client.rs` (2018, 2024) and `client-2015.rs`
run natively with `../no-std-clients/native-driver.c` and print `42 40`.
`macro-client.rs` calls `core::forty_two!` through the glob. The opponents
are:

- `hidden-client.rs` names `core::internal` (E0425);
- `sealed-client.rs` names `core::sealed` (E0433);
- `shadow-client.rs` names `core::option::Option` through the shadowing
  shim (E0425, E0433); it is accepted through `shim.rs`;
- `used-ambiguity-client.rs` calls the ambiguous `core::shared`.

`unused-ambiguity-client.rs` is the control for the ambiguous shim: it
calls `core::only_left` and `core::right::shared`.

Rustc reports a used ambiguity across crates through the deny-by-default
future-incompatible lint `ambiguous_glob_imports` ("`shared` is ambiguous").
The oracle runs under `--cap-lints=allow`, so rustc accepts the call and
selects one glob's target. RustGM rejects it by name, as it rejects a used
glob ambiguity within one crate (E0659).

Each client names the `../no-std-clients/builtins.rs` stand-in, built over
`core.rs`, as its `compiler_builtins`: rustc with `--extern` and RustGM in
its dependency inventory. `upstream-shim.rs` is the exact rustc 1.98.0
`library/rustc-std-workspace-core/lib.rs`, which also brings
`compiler_builtins` into the graph with `use compiler_builtins as _;`.
Through it, the 2024 client passes no `compiler_builtins` of its own to
rustc and runs natively.

The outcomes in `../../shim-glob-outcomes.json` were declared before the
frozen rustc oracle and before any RustGM candidate run of these cases.
Exploratory probes of similar shapes ran first. One control was amended
after candidate round 1. The first `shadow-client.rs` returned
`core::option::Option<u32>` from a public function. Through `shim.rs` it
resolved, but the client's export then failed with "generic nominals are
outside the metadata profile", a limit unrelated to the glob. `Option` is
now used only by a private function. The rustc oracle was rerun for both of
its cases before the next candidate run, with unchanged outcomes.

The lang-item prefix of `core.rs` is the exact pinned core fragment of
`../no-std-clients/provider.rs`, taken from the original core `marker.rs`
and `clone.rs`. `upstream-shim.rs` keeps the original bytes of
`library/rustc-std-workspace-core/lib.rs`. Both keep Rust's MIT/Apache-2.0
attribution; see the root NOTICE and `third_party/sources.json`.
