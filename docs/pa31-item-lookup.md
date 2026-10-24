# Context-sensitive item lookup: focused checkpoint

This development-qualified LIB-001 checkpoint follows the published
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
Parameter identity remains the original normalized context when ordinary item
lookup removes opaque marks; rewinding must not capture a defining generic.

Import dependencies and selected keys carry the queried context. Successful
lookup is not an absence cache: a later exact publication can wake a missing
query. Globs reverse the synchronized context adjustment before contributing a
binding. Their source journals remain in source identity; destination keys and
withdrawals use the mapped identity. Identical names sharing one source context
share the transformed glob suffix. Privacy is evaluated separately from that
cached mapping. Unknown provenance cannot create a cached successful answer.

Legacy textual macro declarations, single-name calls and import fallbacks now
use the declared name's exact normalized item identity too. The immutable scope
entries and root-name cache keep their existing layouts. One optional cold owner
stores non-root definition contexts and contextual query answers. Pending answers
retain their actual output dependency, so publication resumes each context
without flushing unrelated answers or accepting a sibling name. These names are
never inferred from the expansion that introduced the declaration.

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
- A literal generic parameter cannot be captured by opaque definition-site
  rewinding; a parameter supplied by the caller retains its original identity.
- Textual scopes preserve independent contexts, historical branches and pending
  output wakeups; ordinary queries allocate no context owner. Repeated queries
  with the same context share the reached suffix at depth plus use-count cost.

These are structural controls using real parser/expansion owners with supplied
opaque token provenance. They are not evidence of scoped macro source acceptance.
The existing item-adjustment/privacy controls and PA6/PA8 semantic suites remain
required, as do all active text oracles.

The measured cold owner change is Resolution 2048 to 2056 bytes. Saved macro paths
replace a 24-byte Vec with two slice handles (32 bytes); ordinary paths have no
context allocation. The boxed mark arena gains the glob cache. Scope, Definition,
Package, Body, Value and Type records gain no fields. The focused controls measured
Scope 32, Definition 36, Package 256, the pooled path buffer 56 and saved path 32
bytes. Textual Scopes is 128 bytes, Entry remains 24 and its optional owner is
64 bytes. Shared lookup changes still require immutable A/B
measurements, ordinary controls and the existing macro depth/width matrix before
publication; all regressions must remain visible.

Source `0e356020ff24cc6b1c0de4b8f9acd5cb94560821` passed formatting, architecture
audit, strict all-target/all-feature Clippy, harness/upstream verification,
66 exact private controls, 138 frontend tests and all 8,382 active PA text cases.
The receipt lives in `obj/parallel/item-lookup-v5/` in the integrator's isolated
`rustgm-pa29-work` checkout. Its focused receipt SHA-256 is
`c8d424d2bb6fe23e8a88316451a890ca1566a8443f5309df8af16b42b30bb157`;
the complete check inventory is
`e787008bbdfdd8910bdb71ffa374520b289f35afd40b7eecb9bf350b6bd02f8f`.
Earlier filename-family audit and strict-style failures remain retained.
The subsequent [combined checkpoint](library-lookup-panic-checkpoint.md)
qualifies this implementation at `6bba6589`, including release/native controls
and immutable A/B measurements. It retains all observed performance costs.
Scoped publication, full member consumers and
foreign macro transport remain separate required work. Do not enable scoped
admission merely because these private controls pass.

The textual controls measure reuse of a reached suffix, not linear work for
arbitrarily many distinct cold query contexts. The current persistent-link cache
can visit a scope suffix for each distinct name/context; the broad width/depth
matrix must expose that cost before scoped source admission. Context-sensitive
lookup must not be described as universally constant-time.

## Independent parameter observation

The four inputs in `pa31/tests/spec/declarative-macros/generic-lookup-contract.json`
were declared before execution. The initial literal-generic positive hypothesis
was falsified by both metadata and object compilation: pinned rustc reported
E0425 for the literal `T`. Captured `T`, the legacy macro literal, and a scoped
macro referring to an ordinary defining block type alias all passed both modes.
The original hypothesis and its competing prediction remain unchanged; the
compiler draft now preserves parameter identity rather than accepting that
positive. This is eight independent observations, not scoped-macro acceptance
by RustGM or native execution evidence.

The receipt is retained at worker A's
`obj/parallel/pa29-generic-lookup-oracle-v1/evidence/receipt.json`, SHA-256
`6aa88d05ac02f5a1f33d412093455e3581c36382114312d7af8850290998f702`.
It binds the original contract, all input bytes, diagnostics and rustc executable
`3690cc576ede140504698405d5d8fa3826aaadbe71699c6c4ed0a565d6f493e2`.
