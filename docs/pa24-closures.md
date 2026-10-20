# PA24 implementation handoff

PA24 remains planned. This is a maintainer plan, not an implemented grading
contract. PA23 functional release validation and its frozen performance matrix have passed;
no closure compiler changes are mixed into that qualification.

The parser and resolver already retain closure parameter/result/body nodes,
`move`/async flags, closure scopes and label barriers. Implement semantics from
those nodes and resolved identities. Do not reparse or generate replacement
Rust source. The supported edition is 2024; async closure execution belongs to
PA32/PA33 and library owning pointers remain PA26.

## Required phase boundaries

1. Give each closure expression a distinct type identity with its enclosing
   generic inputs, callable signature and typed capture environment. Use the
   existing type/substitution/inference machinery; source identity is not a
   reusable function instance or permission to erase lifetimes.
2. Infer parameters and results with contextual callable expectations, retaining
   quantified inputs and the enclosing body's inference dependencies. An ordinary
   closure remains monomorphic after inference. Nested closures need distinct
   locals, returns, break targets and body contracts.
3. Collect capture paths from typed place uses. Merge overlapping prefixes with
   the required capture-mode ordering using an indexed path structure, rather
   than comparing every capture with every other capture. Keep declaration/use
   order deterministic and count actual path work and retained storage.
4. Apply precision rules: disjoint fields, shared-reference truncation, array/
   slice indexing, discriminant and wildcard reads, packed fields, and move
   contexts. Unique immutable capture is a distinct permission; replacing it
   with an ordinary shared borrow would permit invalid aliasing.
5. Construct the environment with existing aggregate, borrow, move and cleanup
   facts. Rewrite captured place accesses through the typed environment mapping.
   The environment's actual capture fields determine Copy/Clone/auto-trait
   behavior. Capture creation and closure calls must pass the ordinary region,
   initialization, loan and destruction gates.
6. Add resolved identities for `Fn`, `FnMut`, `FnOnce`, their associated output
   and the stable parenthesized bound syntax. Implement invocation through shared,
   mutable and consuming receivers, respecting the hierarchy and complete input/
   output obligations. `move` does not by itself make a closure `FnOnce`-only.
   A same-spelled user trait remains a user trait.
7. Emit callable adapters through ordinary checked body/instance discovery and
   the existing LowIR ABI. Noncapturing closures may coerce to function pointers;
   captured closures, including captured zero-sized values, may not. PA23's
   dynamic dispatch should consume proved callable-object targets without a
   second ad hoc invocation path.
8. Qualify source and native behavior, publish the owning fixtures and byte
   oracles, then run the release checks and immutable performance comparisons.
   Retain all earlier byte contracts except explicitly reviewed representations.

The pinned rules are in `doc/rust-reference/src/types/closure.md`, especially
capture precision, unique immutable borrows, call traits, other traits and drop
order. Existing special cases for unsupported source constructs must not serve
as the semantic evidence for a new rejection control.

## Initial declared controls

`obj/pa24-pa25-work/pa24-controls-v1/plan.json` records 32 outcomes before the
implementation: noncapturing/contextual calls, first-use inference, immutable/
mutable/move captures, once-only invocation, closure copying, precise field
borrows and moves, loan end points, unique immutable borrows, all three call
traits, dynamic Fn/FnMut calls, quantified input/output identity, returned
closures, function-pointer restrictions, capture destruction, shared-reference
truncation and nested captures. All 32 declarations match pinned rustc 1.98.0 object compilation, recorded in
`pa24-controls-v1/rustc-v1/`. No RustGM result or text oracle is claimed for them.

Extend the matrix with paired packed, pattern/discriminant, nested-generic,
partial-move, promotion, opaque-output, marker-bound and method-adjustment cases
as the owning typed interfaces become available. For scaling, vary independent
closures, fields per environment and repeated paths separately, with an ordinary
nonclosure control. Avoid rebuilding crate-wide indexes or scanning unrelated
locals for every closure.
