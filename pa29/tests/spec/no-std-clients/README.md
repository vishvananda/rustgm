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

`builtins.rs` is used only by the rustc oracle. Without a sysroot, rustc's
injection also needs an `extern crate compiler_builtins`, and this empty
crate stands in for it. RustGM injects no such crate.

The outcomes in `../../no-std-client-outcomes.json` were declared before the
frozen rustc oracle and before any RustGM run of these files. Exploratory
probes of similar shapes ran first.

The lang-item prefix of `provider.rs` is the exact pinned core fragment from
`pa31/tests/spec/fn-sugar-impl-parameters/`, taken from the original core
`marker.rs` and `clone.rs`. It keeps Rust's MIT/Apache-2.0 attribution; see
the root NOTICE and `third_party/sources.json`.
