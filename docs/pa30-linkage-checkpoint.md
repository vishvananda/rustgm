# PA30 exact function linkage checkpoint

Retained `link_name`, `unsafe(no_mangle)` and `unsafe(export_name)` attributes
select exact function spellings in the existing registry-owned symbol pool.
String literal tokens are decoded once; lowering consumes typed naming and
linkage records. Configuration and macro expansion preserve the owning
function's attributes. Duplicate `export_name` attributes retain the first
spelling, and `export_name` takes precedence over `no_mangle`, matching the
independent Rust 1.98.0 controls.

Imported functions keep real foreign ownership and no body. Ordinary exported
functions retain their local bodies, ownership/borrow proofs and ordinary demand
graph. The shared executable/library roots API includes every registry export,
so C can reach an otherwise unused Rust function. Metadata retains exact bytes
and remaps identities; it never transports a local symbol-pool ID.

`LowIR` declarations now use one private name-to-symbol index. Compatible
parameter/result/noreturn/entry declarations reuse the first symbol and signature
without changing declaration order. Distinct structurally equal signature IDs
are accepted without broad signature interning. `Program::define` checks the
canonical identity and installs one body only. Incompatible aliases and repeat
definitions fail before mutation, including export collisions with generated
helpers in either installation order. Compatible C import/export self-aliases
serialize one function declaration/definition and call the local body.

Twenty draft source outcomes distinguish Rust language rejection from explicit
backend boundaries. Unicode and hyphenated exact names remain valid Rust but
cannot enter the pinned LowIR spelling vocabulary. Exact names are never
normalized or replaced. Generic exports require stable external argument
identity and remain unsupported; generic rejection fixtures enable Rust's
`no_mangle_generic_items` deny lint. Function-only linkage is this checkpoint's
scope; foreign statics and aggregate/variadic C interfaces remain unfinished.

The explicit optional object profile is still required for C interoperability:

```sh
cargo test --manifest-path dev/Cargo.toml --features object-interop-tests \
  --test foreign_linkage --test foreign_declarations -- --include-ignored --nocapture
```

Selecting this test without its configured immutable backend is a hard error.
PA30 activation must install that profile and exercise this feature in its
release target and CI. No assignment activation or oracle regeneration is part
of this checkpoint. PA26's remaining hook families are tracked separately in
`pa26-environment-checkpoint.md`.

Strict all-target/all-feature Clippy and 15 focused test groups pass, including
20 independently object-qualified linkage outcomes, seven C-linked native
controls across the import/export suites, six shared-root controls and three
canonical symbol tests. The syntax audit reports 1,224 files and zero findings. Another 18 test groups
pass, including five earlier convention natives, persistent data/backend tests
and typed LowIR construction. All 7,781 active fixture cases pass discovery;
24 pinned upstream fixtures and 126 Rust Reference files verify. Four earlier
callable text views remain byte-identical.

The complete layout check stops at PA12's reference-generation receipt because
the inherited `2395c0bd` fixture migration changed active manifests. The
coordinator confirmed this expected baseline mismatch; reviewed receipt updates
are awaiting publication. No references were regenerated here, and the failed
check is retained separately from passing discovery. Evidence is in
`obj/parallel/pa30/linkage/`; focused receipt
`b14273d83e5d4172abd8d8f1d9138402` records strict Clippy and the first 15 groups.
