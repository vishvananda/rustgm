# Source-independent original body ownership

This is a typed metadata prerequisite, not generic-body transport or an assignment
release. `support::crates::owner_catalog` now validates artifact-local original
body families without frontend arena IDs, fabricated syntax nodes or proof tokens.
No wire profile is assigned, and profiles zero through seven are unchanged.

`DeclarationRef` addresses the defining artifact's complete declaration table,
including private helpers and associated declarations. `BodyRef` addresses its
body table. Each body row retains its declaration, exact parent if any, body kind
and sibling ordinal. Function, constant and static roots retain distinct kinds;
closure and anonymous children inherit their parent's declaration. Anonymous
roots also represent signature/type-level expressions in declarations without
an executable body. Root/child categories are structural data: the later importer
must validate actual declaration headers and complete typed body contents.

Root rows precede child rows. Roots are ordered by declaration and contiguous
root ordinal; a named declaration body, when present, is root zero. Children are
ordered by parent body ordinal and contiguous child ordinal. Parents precede
children, giving one canonical breadth-first forest. A future exporter must
derive declaration order and lexical child ordinals from retained defining
identities, independently of local arena numbering or demand order. This module
validates that order; it does not implement the exporter that produces it.

One pass checks each row and at most one parent edge. Unknown declarations,
forward/cyclic parents, cross-declaration parents, incompatible body kinds,
duplicate/gapped ordinals and reordered groups reject before publication. No
ancestor walk, recursion or per-body map is needed. Immutable rows use one shared
allocation; queries use direct indexing. Cross-artifact identities include the
defining CrateIdentity, so matching ordinals from different providers do not alias.
The dependency loader still owns rejection of conflicting artifacts claiming the
same defining crate identity.

Limits are 100,000 declaration slots, 100,000 body rows and a 16 MB construction
budget. Preflight includes the input vector's capacity, the simultaneously
allocated immutable row storage and catalog overhead. Validation has constant
scratch. These are ownership-table bounds, not a budget for all generic metadata;
the complete envelope must additionally account its types, clauses, CFGs and
remapping storage.

The six private invariants cover real parent identity, provider isolation,
shared storage, anonymous roots, malformed edges/categories/order, quota overflow
and deep versus wide families through 32,768 bodies. Instrumented validation
visits exactly one row plus one edge for each non-root, including edges rejected
on error. These counters describe the actual table validator, not whole-compiler
compile time or heap neutrality. The first focused run and strict Clippy passed;
its file audit caught a repeated `body` filename prefix. The module was renamed
to `owner_catalog` without an audit allowance. The final focused run passed all
six invariants, formatting, strict release Clippy for all targets/features,
the 1,844-file architecture audit, maintainer registration and layout checks.
Combined release qualification remains separate. Evidence is retained under
`obj/parallel/pa29-body-owners/` in the PA29 worktree.

The next consumer remains the coordinated
[generic transport prerequisite](pa29-generic-transport-proposal.md): reserve
actual imported declaration/body identities, introduce source-independent body
origins and diagnostic labels, then remap original CFGs and exact contracts.
Original proof-context rows must remain distinct from executable-template lookup.
This catalog records neither role as a permission bit, does not create Checked
or Proved values, and does not bypass any original or instance proof. Generic
body export/import and source-built core remain unimplemented.
