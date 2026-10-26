# Alias naming and source Self sizing

Canonical alias clauses now supply actual shorthand associated-type naming
inputs. The same expansion graph supports two views: complete requirements for
proof, and declaration-Self clauses for naming. No alias becomes a trait,
associated owner, implementation or proof token. Full alias execution/export
and original core/alloc/std builds remain incomplete.

## Actual typed consumer

The existing written-bound index retains alias paths as well as ordinary trait
paths. A cold naming seed prepares the complete actual arguments once and moves
the existing expansion vector into its typed input record. Ordinary bounds keep
their single-clause representation without allocating a vector. Both paths use
the same binder opening, capture mapping, member selection and normalization.
Each actual leaf retains its real trait/member/argument identities; distinct
applications remain ambiguous and identical shared paths do not invent another
member. The private signature worklist proves the retained equality and preserves
the original real projection inputs.

Naming filters in the declaring alias, before substitution. In
`trait A<X> = Left where X: Other`, supplying `X = T` does not make the written
`X: Other` a naming path for `T::Item`. The full requirement still participates
in proof. A written `where Self: Other` does supply a naming path. An associated
subject that normalizes to Self also retains its original subject. Alias edges
therefore keep their written subject; direct clauses reuse existing unreduced
facts. Both views share the same iterative application graph and substitutions,
without a second graph, source replay or flattened ancestor paths.

Pinned rustc implements this distinction with
`collect/predicates_of.rs::explicit_super_predicates_of` and
`PredicateFilter::SelfOnly`; `traits/util.rs::expand_trait_aliases` instantiates
that declaration view. Original typed subjects, rather than equality of actual
arguments, determine eligibility.

The integrated [source Self-sizing checkpoint](alias-source-sizing.md) also
retains direct-RHS sizing separately from where/alias-mediated bounds, using the
same decoded bound scan and real role identities. It adds no owner or hot field.
Actual unused formals, relaxed-bound rejections, source ownership and ordinary
absence/legacy bytes remain covered.

## Evidence and costs

[Run 40281334667](https://github.com/vishvananda/rustgm/actions/runs/40281334667)
passes 20 exact private controls in 0.05 seconds. The helper takes 133.09 seconds,
including a 76.35-second unoptimized build and 44.21-second private strict Clippy.
Formatting, full architecture, harness, upstream and audit-cache checks pass.
Production strict Clippy and metadata retain their actual successful steps from
40280720758: every production/tool/config byte is unchanged; the sole subsequent
private helper change supplies the required original-subject argument. All
2,735 fingerprints match before/after and the committed source.

Eight authored sources have pinned Rust metadata observations, zero infrastructure
failures and no native sweep. The first six ran once in 40278863399; one prediction
was wrong (`X = T` still rejects shorthand lookup). The source, original prediction,
E0220 diagnostic and failed qualification remain retained. Two new observations
in the successful run accept written Self and reject the erased associated
subject. The first six were authenticated and retained rather than rerun.

The initial successful static checks were followed by the independent-prediction
failure, before any compiler tests. The next attempt passed production/static
checks but found one missed private sizing helper call after the internal API
change; again no tests ran. Both failures are preserved. No lint/audit allowance,
reference change or upstream-source patch was introduced.

Checker 10,232, alias Pending 80 and Header 104 bytes are unchanged. Edge grows
28 to 32 bytes for its original subject. A demanded typed naming entry's Inputs
is 24 bytes, versus the former single Clause's 12 bytes; its vector is allocated
only for an alias. This is a real selected-table cost, not a zero-cost claim.

| Shared width | Ordinary work / logical bytes | Quantified work / logical bytes |
| --- | --- | --- |
| 8 | 394 / 13,653 | 610 / 29,549 |
| 32 | 1,574 / 56,589 | 2,392 / 122,757 |
| 128 | 6,246 / 224,013 | 9,448 / 487,045 |

These are existing graph controls and cumulative logical charges, not wall-time
or physical-memory measurements. No broad performance/release/native/text matrix
was repeated. The receipt SHA-256 is
`7a52fd71b99600416c30438011504add9d82f864c0265c158e01bf63ee6bb4e8`;
binding SHA-256 is
`7589a3e09d4325caef56fe97cf4199b5bf4d09861476ad45854271edb81657ca`.
Raw evidence is in `obj/parallel/alias-projection-inputs-v1/`.

## Remaining consumers

Header-time alias/supertrait naming, complete original/normalized application
formation, scoped lifetime relations, associated-family input prefixes, object
principal multiplicity/safety, unused/default well-formedness, const qualification
and genuine imported contracts remain required. In particular, adding all alias
clauses must advance an associated family's input prefix before result bounds;
the early alias continuation does not yet do this. Object naming must not infer
principal multiplicity from the deduplicated proof/naming view. The global alias
execution/export fence stays closed. There is no new whole-core observation or
artifact, and no assignment release.
