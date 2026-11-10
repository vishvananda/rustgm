# PA21 fixture ownership

The manifest registers 758 cases with committed stdout, stderr and status
sidecars, including the independently checked ordinary result-supertrait regression.
There are 308 semantic inspections and 450 LowIR checks; 264 accepted
LowIR programs declare native execution (260 pass, four destructor panics).
Source outcomes use the pinned rustc mode declared by each independent case.
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

The `contracts/polymorphic-outputs/accepted/semantics/result-supertrait-equality.rs`
regression requires an associated output's genuine equality to reach its inherited
supertrait argument. Both pinned editions accept the exact new source. The prior
qualified compiler rejects its ordinary call, independently of source Try hooks.
The case declares semantic success. Its selected official reference is generated
and reviewed; all 758 text cases pass with the integrated solver correction.
The pinned two-edition metadata observations add no native execution claim.

The `spec/assoc-bounds/certified-premises/` controls (Lane A census batch 17)
test a declared bound on a generic associated type,
`P: Pattern<Searcher<'a>: ReverseSearcher<'a>>`, proving a supertrait
condition whose subject is the same rigid projection at another lifetime.
rustc relates the projection's inputs invariantly, which leaves a region
equality to prove. Their 19 rustc 1.98.0 outcomes were declared before the
oracle ran, and all agree. rustc rejects five: a missing bound, a different
associated item, another lifetime, a one-way outlives and a required
higher-ranked bound. `raw-environment-premise` is accepted by rustc and
stays stricter than rustc by name. The 13 accepted controls declare native
execution. The
private invariant `certified_bound_premises_match_declared_outcomes` checks
each control's first error.

The `spec/assoc-bounds/elaborated-premises/` controls (Lane A census batch 18)
test a supertrait condition that only an elaborated premise proves: a bound
`P: Pattern<Searcher<'a>: DoubleEndedSearcher<'a>>` elaborates to
`Searcher<'a>: ReverseSearcher<'a>`, which the `Walk` impl needs. An
elaborated premise inherits a certified declared bound's certificate when
both have the same subject and binder shape and its trait arguments carry no
projection. Their nine rustc 1.98.0 outcomes were declared before the oracle
ran, and all agree. rustc rejects two: a missing bound (E0277) and another
lifetime (E0803). `elaborated-projection-argument` is accepted by rustc and
stays stricter than rustc by name, because its elaborated premise
`Keyed<'a, P::Key>` carries a projection. The six accepted controls declare
native execution. The private invariant
`elaborated_premises_match_declared_outcomes` checks each control's first
error.

The `spec/assoc-bounds/projection-conditions/` controls (Lane A census batch
19) test a member's signature whose `Self::Item` normalizes through an impl
with a condition on a rigid GAT projection,
`P: Pattern<Searcher<'a>: ReverseSearcher<'a>>`. A member's conformance
contract carries its owner's certified premises when the trait method adds
no clause of its own; a method type parameter's implicit `Sized` is its
parameter kind, not a clause. Their 13 rustc 1.98.0 outcomes were declared
before the oracle ran, and all agree. rustc rejects three: a missing bound
(E0277), another lifetime (E0478 and E0803) and a mismatched item type
(E0053). `cond-method-bound` is accepted by rustc and stays stricter than
rustc by name, because its trait method's own `Q: Copy` bound is a clause of
the contract. The nine accepted controls declare native execution. The
private invariant `projection_conditions_match_declared_outcomes` checks
each control's first error.

`spec/binders/universal-inherent/` freezes nine independently checked PA21
generic-semantic outcomes for closed universal inherent calls: static-only bounds
reject, universal implementations and actual caller universal premises accept.

`spec/binders/scoped-lookup/` adds seven independently checked outcomes for
quantified method applicability, trait fallback and the function-pointer Split
shape. All are checked through types, generic semantics and complete LowIR.
