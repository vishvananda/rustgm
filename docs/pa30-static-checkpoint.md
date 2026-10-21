# PA30 foreign storage checkpoint

Foreign statics retain declaration-owned safety, mutability, type and exact
linkage in the canonical foreign registry. Imports have no source initializer,
constant value or fabricated body. A selected import emits a real LowIR global
declaration with its checked size, alignment and storage mutability. Local
exports retain their actual initializer and typed constant-storage graph.

The syntax tree retains default, safe and unsafe static qualifiers separately.
Default/unsafe foreign accesses require an unsafe context; explicit safe does
not grant mutable-static access or permission to create forbidden references.
Raw address formation follows ordinary place permissions, and dereferencing the
result still requires its own unsafe permission. Immutable foreign addresses
can occur in constants without reading unknown external storage. Evaluating a
foreign value in a required constant is rejected.

Exact `link_name`, `unsafe(no_mangle)` and `unsafe(export_name)` attributes use
the existing byte-preserving name pool and selector. The function/global LowIR
namespace has one typed canonical index. Aliases reuse storage only when size,
alignment and readonly permissions agree. Function/global kind conflicts,
incompatible storage and repeated initialization fail before modifying the
relevant canonical identity or data. Compatible local/foreign self-aliases emit
one definition. Ordinary static rendering and the public callable `Roots` API
retain their previous contract.

Internal selected roots distinguish callable and global exports. Registry roots
are collected once in deterministic order. Global roots demand actual local
initializers and share the existing constant, callback, generic instance and
object dependency traversal. Entry-free libraries emit exported callback
storage and its private generic dependencies without an entry adapter. Imports
never acquire an initializer merely to pass a demand check.

Thirty independently object-qualified Rust 1.98.0 draft cases are recorded in
`pa30/tests/static-outcomes.json`: 18 accepted, ten language rejections and two
valid Rust cases with an explicit generated-name collision boundary. Those
boundaries reject a conflicting LowIR function/global name without pretending
that Rust rejects the program. Five C-linked programs cover imported scalar,
floating, aggregate, callback and pointer storage; mutation; exported globals;
local/foreign alias identity; an otherwise unused generic callback export; and
a constant address of foreign storage. Aggregate storage uses the existing
layout; this does not implement aggregate function argument/result ABI classes.

The feature-enabled native command is required when selecting these controls:

```sh
cargo test --manifest-path dev/Cargo.toml --features object-interop-tests \
  --test external_storage -- --include-ignored --nocapture
```

The optional pinned object profile must be configured; absence is a hard error.
PA30 remains planned. C aggregate callable classification, variadic interfaces
and static crate-metadata transport remain unfinished. Integration with interior
mutable nominal types must clear readonly storage through the structural type
property while preserving the separate declared-mutable access authority.
PA26's remaining hook families are still tracked in
`pa26-environment-checkpoint.md`.

Strict all-target/all-feature Clippy, the 1,256-file architecture audit with zero
findings, 38-stage layout checks, 24 pinned upstream fixtures and 126 Reference
files pass. There are 64 distinct focused test groups: 52 source/native/storage/
crate-linking regressions and twelve callable-root/canonical-symbol groups.
Four older callable text views and five ordinary-static views remain identical.
A sixth static inspection view shows the already reviewed PA20 constant-dispatch
change from the inherited ValueScope correction; the integrator independently
qualified it and committed reviewed references in `1ee00be9`. No references were
changed in this worker.

Evidence is retained under `obj/parallel/pa30/statics/`: declared hypotheses,
source hashes and qualification, `audits.json`, `regressions.json`, the earlier
focused `final.json`, native outputs and selected text checks. Initial checks
caught two compile-time API mismatches, a test expecting seven declarations
instead of six canonical symbols, function-length lint limits and two placement
audit limits; each was corrected before the final passing gates. The initial
independent hypothesis that a safe mutable declaration itself was invalid was
corrected from rustc evidence before frontend qualification; its accesses still
require unsafe permission. No assignment activation is included.
