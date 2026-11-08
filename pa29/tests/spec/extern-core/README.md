# `--extern core=` names the direct dependency

Rustc lets `--extern core=PATH` name a crate's direct `core` dependency, and
std's Cargo graph does so for every library unit. The `no_std` injection's
`extern crate core` then resolves to that crate, and so does a 2018+ path
through the extern prelude. RustGM accepts the alias `core`; `std` stays
reserved. The injection reuses the alias's extern-prelude entry instead of
binding the provider root a second time. The edition prelude is read from
the module that entry names.

These cases reuse the `../no-std-clients/` provider and clients, each with
`core` given by `--extern` as well as by the dependency inventory:

- `client.rs` in 2024 and `client-2015.rs` in 2015 run natively with
  `../no-std-clients/native-driver.c` and print `42 40`;
- `no-core.rs` is a `#![no_core]` client that reaches `core::base` through
  the extern prelude, as the `rustc_std_workspace_core` shim reaches core;
- the opponent `../no-std-clients/bare-client.rs` names `no-core.rs`'s crate
  as its `core` alias. That crate has no prelude module, while the source
  provider does. Rustc reports "cannot resolve a prelude import" and RustGM
  rejects it by name.

The rustc oracle passes `compiler_builtins` explicitly. That keeps rustc from
seeing two candidates once later manifests add their own stand-ins. RustGM's
`no_std` clients have the same stand-in in their dependency inventory; the
`no_core` client needs none.

The outcomes in `../../extern-core-outcomes.json` were declared before the
frozen rustc oracle and before any RustGM run of these cases.
