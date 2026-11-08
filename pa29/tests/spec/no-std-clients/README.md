# no_std source clients

Rustc's `no_std` injection, observed with rustc 1.98.0: a `#![no_std]` crate
without `#![no_core]` gets an injected `extern crate core` with no
`macro_use` and a glob of `core::prelude::rust_20xx` for its own edition.
Rust 2015 can also name that crate as a crate-root item (`crate::core`). A
source client receives the same imports. Its `core` is the authenticated
`--source-client` provider, which must be named `core`. It reaches that
provider through the dependency inventory, because no `--extern` alias may
spell `core`.

`provider.rs` is a no_core source-core provider named `core`. Its prelude has
core's shape: `v1` re-exports `Option` and a `macro_export` row, the modules
`rust_2015` through `rust_2021` glob `v1`, and only `rust_2024` adds
`IntoFuture`. `client.rs` runs as a native object in 2018, 2021 and 2024.
`client-2015.rs` runs in 2015, where it names the injected crate through a
`use` path and `crate::core`. Both link with `native-driver.c`.

The opponents are:

- `future.rs` in 2021, whose prelude lacks `IntoFuture` (E0405); it is
  accepted in 2024;
- `crate-core.rs` in 2018, where the injected crate is not a crate-root item
  (E0433); it is accepted in 2015;
- `names-std.rs`, since a `no_std` crate has no `std` (E0433);
- `bare-client.rs`, whose provider `bare-core.rs` is named `core` but has no
  prelude modules. Rustc reports "cannot resolve a prelude import".

`builtins.rs` is the empty `#![compiler_builtins]` crate. Without a
sysroot, rustc 1.98's crate loader injects `compiler_builtins` after name
resolution into a `no_std` crate whose graph has no such crate, and this
stand-in is that crate. RustGM compiles it too, publishing crate loader flag
bit 0, and each `no_std` client has it in its dependency inventory. The
amended opponents are:

- the 2024 client before any `compiler_builtins` crate exists (E0463);
- `plain-builtins.rs`, a `no_core` crate named `compiler_builtins` without
  the attribute, which a client then names with `--extern
  compiler_builtins=` ("resolved as `compiler_builtins` but is not
  `#![compiler_builtins]`");
- `ungated-builtins.rs`, the attribute without its internal feature (E0658).

The outcomes in `../../no-std-client-outcomes.json` were declared before the
frozen rustc oracle and before any RustGM run of these files. Exploratory
probes of similar shapes ran first. The `compiler_builtins` amendment was
declared on 2026-11-08 before any rustc or RustGM run of the amended cases,
after exploratory rustc probes.

The lang-item prefix of `provider.rs` is the exact pinned core fragment from
`pa31/tests/spec/fn-sugar-impl-parameters/`, taken from the original core
`marker.rs` and `clone.rs`. It keeps Rust's MIT/Apache-2.0 attribution; see
the root NOTICE and `third_party/sources.json`.
