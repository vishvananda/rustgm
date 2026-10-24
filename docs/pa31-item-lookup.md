# Context-sensitive item lookup: source checkpoint

This unqualified LIB-001 checkpoint follows the published
`domains-selectors-macros` integration. It does not enable scoped macro source
admission, complete core, or release PA31. The reference algorithms are pinned
Rust `88d9e12ae178fab0fb5cc050a94da85685d449ea`: `rustc_resolve/src/ident.rs`
for lexical and qualified lookup, `rustc_resolve/src/lib.rs::resolve_self`, and
`rustc_span/src/hygiene.rs::reverse_glob_adjust`. The existing PA31 declarative
macro source hypotheses remain the eventual end-to-end contract.

Canonical path names now travel with each identifier's normalized item context.
Body traversal pools the complete scratch buffer; root paths allocate no context
array. Registered macro calls retain immutable slices, and imported paths reuse
their existing shared arrays. Borrowed prefix views retain the corresponding
contexts through builtin preparation as well as ordinary lookup. No source text
or rendered path is reparsed to recover identity.

One optional boxed owner inside `Resolution::introductions` records actual
invocation-to-definition scopes and nonphysical scope introductions. Expansion
publishes its site at the same point as its consecutive ancestry record. A
captured module or item name cannot substitute for the container's introduction.
Physical source allocates no owner. Definition scopes are available during
expansion/import retries, before a final Report exists.

Lexical item queries probe exact context keys and follow actual definition scopes
when removing opaque marks. Qualified segments use the shared container-relative
adjustment; privacy follows the last stripped mark independently of the caller.
Locals retain their separate mixed-site histories and visibility snapshots.
Generic parameters, lifetimes, constant-pattern references and const-argument
disambiguation consume the contextual lookup. Implicit Self retains its explicit
root identity. Each self/super keyword uses its own retained context. Restricted
visibility selects direct module bindings and still requires an actual ancestor.

Import dependencies and selected keys carry the queried context. Successful
lookup is not an absence cache: a later exact publication can wake a missing
query. Globs reverse the synchronized context adjustment before contributing a
binding. Their source journals remain in source identity; destination keys and
withdrawals use the mapped identity. Identical names sharing one source context
share the transformed glob suffix. Privacy is evaluated separately from that
cached mapping. Unknown provenance cannot create a cached successful answer.

## Declared controls and costs

Before execution, the new private controls require:

- Physical allocation freedom and atomic rejection of invalid invocation order.
- Linear scope storage, actual definition-site versus caller privacy, and exact
  dependencies for a missing name.
- Independent contexts on qualified path segments, contextual self selection,
  and visibility of a newly published binding after a missing query.
- The pinned nested-glob literal/captured-name opposition; invalid provenance;
  depth 8/32/128/512 by width 8/32/128 suffix work and storage bounds.
- Actual import journal replacement and withdrawal in the mapped context,
  preserving an identically spelled direct binding in the root context.

These are structural controls using real parser/expansion owners with supplied
opaque token provenance. They are not evidence of scoped macro source acceptance.
The existing item-adjustment/privacy controls and PA6/PA8 semantic suites remain
required, as do all active text oracles.

The declared cold owner change is Resolution 2048 to 2056 bytes. Saved macro paths
replace a 24-byte Vec with two slice handles (32 bytes); ordinary paths have no
context allocation. The boxed mark arena gains the glob cache. Scope, Definition,
Package, Body, Value and Type records gain no fields. Qualification must record
actual layouts and retain focused work counters. These are source predictions,
not measured performance results. Shared lookup changes require immutable A/B
measurements, ordinary controls and the existing macro depth/width matrix before
publication; all regressions must remain visible.

Formatting is the only completed check at this source checkpoint. Architecture
audit, strict all-target/all-feature Clippy, focused debug tests, text regression,
independent/native source controls and combined release qualification are pending.
Legacy textual macro context keys, scoped publication, full member consumers and
foreign macro transport remain separate required work. Do not enable scoped
admission merely because these private controls pass.
