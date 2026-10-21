# PA30 foreign function import checkpoint

Foreign functions retain the canonical declaration's actual external-block
owner through indexing and macro expansion. The owner selects the inherited
ABI and default unsafe permission, including `extern "Rust"`. Explicit `safe`
and `unsafe` qualifiers remain separate metadata. Ownership resets at actual
item boundaries; it is never reconstructed from diagnostic provenance or names.
Empty blocks receive the same ABI and edition-2024 unsafe validation.

`sema::foreign::Registry` is allocated only for retained foreign records. Its
canonical `DefId` keys select function signatures, safety, naming and linkage.
The registry owns exact symbol bytes. Source-free metadata may insert a checked
function record directly; exact spellings must be reinterned in this pool.
Metadata transports bytes and remaps local identities rather than serializing
`DefId`, `TypeId` or `SymbolNameId` values.

Registered imports have no synthetic body or initializer and are not marked as
checked function bodies. Runtime body/template demand skips only exact import
records; lowering emits genuine LowIR function declarations. Imported functions
remain ineligible for constant execution, while constant function pointers to
imports use ordinary selected function values. Generic callers, imported
function pointers and C callbacks follow the same typed calls and demand graph.

All 21 predeclared draft outcomes in `pa30/tests/import-outcomes.json` agree with
independent Rust 1.98.0 object generation. Five native controls compile the Rust
frontend's LowIR through the reviewed optional object profile, link with a real
C provider and execute. Independent rustc executables link against the same C
provider. Controls cover integer and floating imports, callback invocation,
function pointers, constant function pointers and a generic caller.

The object interoperability test is enabled explicitly while PA30 remains
planned. Configure the reviewed optional profile as described in
`pa30-object-bridge.md`, then run through the storage/resource guard:

```sh
cargo test --manifest-path dev/Cargo.toml --features object-interop-tests \
  --test foreign_declarations -- --include-ignored --nocapture
```

Compiler functionality is unconditional. Selecting this test without the pinned
object profile is a hard error; it does not silently skip or select another tool.
PA30 activation must install the profile and run this feature in its required
release target and CI. Default maintainer-native validation keeps its earlier
backend requirements while the profile is optional.

Strict all-target/all-feature Clippy and ten focused test groups pass, including
five earlier convention native controls. The syntax audit reports 1,213 files
and zero findings; layout/discovery, 24 pinned upstream fixtures and 126 pinned
Reference files pass. Four earlier callable text views remain byte-identical.
Evidence is in `obj/parallel/pa30/imports/` and resource receipt
`cfdaa2c12b074e04990ce707e344b2c8`. An early test version used `include_str!` for
fixtures; the audit required reviewed vocabulary provenance, so tests now load
the declared files at runtime. No audit exceptions or oracle changes were added.

Remaining PA30 work includes foreign statics, linkage attributes and export
roots, the PA29 crate-qualified Rust naming provider, C aggregate classification,
variadic calls and the full release/performance qualification. Imported Rust
metadata requires a real symbol mapping; there is no missing-body or name-based
fallback. PA26 remains incomplete; its remaining hook families are recorded in
`pa26-environment-checkpoint.md`.
