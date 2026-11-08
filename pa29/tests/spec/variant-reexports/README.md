# Variant re-exports from a source core

Core's `prelude::v1` re-exports `Option` together with its variants:
`pub use crate::option::Option::{self, None, Some}`. Rustc binds a tuple or
unit variant in both the type and value namespaces, so the re-export names
both. Version 13 transports the value binding as an ordinary module binding
row whose target is the variant's constructor declaration. A consumer
re-derives that binding from the row and checks its namespace against the
declaration. The type-namespace alias is not transported. Older profiles
reject a variant re-export by name.

`provider.rs` is `../no-std-clients/provider.rs` with the variant re-export
in `v1` and a `reexports` module that renames `Some` and `None` to `Present`
and `Absent`. `client.rs` constructs and matches `Some` and `None` from the
injected edition prelude in 2021 and 2024, and the renamed variants through a
module path. It links with `../no-std-clients/native-driver.c` and prints
`42 40`.

The opponent `variant-type.rs` names `None` as a type. Rustc finds the
variant in the type namespace and reports E0573. RustGM finds no
type-namespace binding, because none travels, and reports an unresolved type
name.

The `compiler_builtins` stand-in `../no-std-clients/builtins.rs` is built
against this provider by rustc and RustGM. Rustc's clients name both crates
with `--extern`, so rustc never loads the `nostd-core` stand-in beside this
one; RustGM's clients have both in their dependency inventory.

The outcomes in `../../variant-reexport-outcomes.json` were declared before
the frozen rustc oracle and before any RustGM run of these files. An
exploratory RustGM probe of a similar provider and client ran first.

The lang-item prefix of `provider.rs` is the exact pinned core fragment from
`pa31/tests/spec/fn-sugar-impl-parameters/`, taken from the original core
`marker.rs` and `clone.rs`. It keeps Rust's MIT/Apache-2.0 attribution; see
the root NOTICE and `third_party/sources.json`.
