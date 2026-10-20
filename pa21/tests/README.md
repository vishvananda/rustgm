# PA21 fixture ownership

The manifest registers 744 cases with committed stdout, stderr and status
sidecars. There are 298 semantic inspections and 446 LowIR checks; 260 accepted
LowIR programs declare native execution (256 pass, four destructor panics).
All source outcomes are independently checked with pinned rustc object generation.
Fixtures preserve the source controls in the named `maintainer_origin` field.

The seven families are:

- `binders`: both higher-ranked bound spellings, alpha-renaming, independent and
  correlated slots, nested callable scopes, outer captures, arbitrary local
  lifetimes, universal versus ordinary assumptions, inference retries and leaks.
- `contracts`: quantified function/nominal requirements, supertraits, implementation
  conditions, associated result bounds/equalities, unused declarations, header
  suspension and concrete instance premise validation.
- `families`: lifetime/type/const GAT inputs, complete mixed arguments, required
  where clauses and their intersections, result bounds, conformance, repeated
  normalization, cycles, distinct family applications and caller isolation.
- `opaque`: argument-position anonymous parameters, return identity, one hidden
  type, trait-method opaque results, published versus revealed bounds, edition-2024
  capture, precise `use<...>` lists, nested capture and forbidden positions.
- `variance`: shared versus mutable references, callable direction, mixed field
  positions, nested nominal cycles, family projections and hidden types.
- `dropck`: owned projected/opaque storage, retained input debts, copy versus
  possible destruction, unused generic declarations, live borrows, initialization
  and conditional drop order.
- `integration`: full source-to-native programs combining the preceding features
  with earlier macros, method dispatch, constants, fields and borrowing.

Each family needs positive and negative controls, semantic inspection where
applicable, and complete LowIR proof/execution. Include explicit stage-boundary
exceptions only with independently justified manifest reasons. Every accepted
LowIR program needs a declared native outcome or an explicit justified
compilation-only classification. Do not count unsupported-feature rejection as
proof of a negative language rule.

Keep Fn/closure construction, trait objects, unsized execution, async and library
hooks out of the initial quantified/family/opaque controls. Their owning stages
must supply those capabilities explicitly. Preserve pinned upstream attribution
and bytes for imports. Every primary input is registered. The former PA18 opaque-return boundary moved
here byte-for-byte with an accepted oracle; plain reference wildcard controls
belong to PA16. No upstream input was modified or imported for this release.

Maintainer invariants must test proof scope ownership, cache environment identity,
completed versus pending failures, bounded work and structural sharing separately
from language outcomes. Never serialize work counters into text oracles.
