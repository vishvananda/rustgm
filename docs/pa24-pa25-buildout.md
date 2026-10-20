# PA23 through PA25 implementation plan

The starting point is PA22 release `e151b1e9`. PA23, PA24 and PA25 are
planned assignments, not released contracts. This document records implementation
work; it does not change their status or authorize empty test success.

PA23 owns trait objects, including object lifetime bounds, dyn compatibility,
vtable identity, dispatch, destruction and supported upcasting. Complete PA23,
PA24 and PA25 in that order, with `dyn Fn` integration in PA24. The shared parser,
receiver and constant-eligibility prerequisites can be qualified against the
existing PA22 foundation.

## Trait object implementation

1. Retain canonical object bounds, complete trait inputs, associated equalities
   and object lifetimes as typed facts. Keep a trait's higher-ranked binder
   distinct from the object's lifetime. Teach existing substitution, inference,
   region, sizedness and representation queries about that structure.
2. Check dyn compatibility from completed trait contracts and member signatures.
   Cache declaration-level results and traverse shared supertraits once. Reject
   incompatible declarations at object formation without rejecting their
   ordinary static use. Preserve explicitly non-dispatchable members.
3. Prove concrete-to-object and supported object-to-object coercions, including
   object lifetime obligations and inherited associated equalities. Retain the
   selected conversion for lowering; representation alone is not a proof.
4. Add explicit vtable metadata alongside existing slice lengths. Construct
   demanded tables, method entries and destruction shims with stable typed
   identities. Reuse the ordinary method and ownership checks; dispatch consumes
   the chosen slot and concrete implementation.
5. Qualify dispatch, upcasting, generic methods, associated types, lifetimes,
   destruction and unsized storage together. Cover shared trait graphs and
   repeated uses with work counters as well as native execution.

## Closure implementation

1. Complete parser and resolution prerequisites using the existing syntax tree.
   Preserve closure parameter attributes, lexical binding identity and nested
   callable boundaries.
2. Introduce unique closure identities and callable signatures. Infer closure
   inputs and outputs together with their surrounding expression constraints;
   retain late-bound input lifetimes separately from captured lifetimes.
3. Derive captures from typed place uses. Combine overlapping paths in a
   prefix index, with monotone capture-mode joins; avoid pairwise comparison of
   every capture. Preserve wildcard non-reads, shared-reference truncation,
   packed fields, arrays, nested closures and move-capture boundaries.
4. Retain a typed environment and call kind. Reuse ownership, regions,
   initialization, destruction and generic substitution for environment values.
   `move` selects storage acquisition; uses of that storage determine
   `Fn`/`FnMut`/`FnOnce`.
5. Lower environment construction, call shims and noncapturing function-pointer
   coercions using stable identities. Add builtin call-trait identities through
   the existing resolver metadata boundary, respecting user-defined names.
6. Qualify generic, opaque-result and trait-object integration. Async closures
   remain owned by the later async assignments.

## Constant evaluation implementation

1. Retain const-call eligibility as a declaration fact. Check operations retained
   in const function CFGs, including unused functions and skipped conditional
   branches. Pinned rustc accepts otherwise forbidden operations after an
   unconditional return; semantic eligibility must preserve this distinction.
   Do not infer const permission from a function pointer or a trait method name.
2. Reuse typed body operations for execution. Add per-invocation locals,
   continuations and allocation identity; node-only memoization must not reuse
   values across calls or loop iterations. Separate eligibility from demanded
   evaluation and charge explicit work, frame and storage budgets.
3. Add statements, control flow, mutable local storage, recursive const calls,
   complete instance arguments and declaration-readiness continuations.
   Preserve constant-item cycle detection independently of function recursion.
4. Track symbolic allocations, projections, lifetimes and provenance without
   inventing host addresses. Validate escaping allocations and values, preserve
   static identity, and reuse the existing typed persistent-data lowerer.
5. Integrate promotion, statics, associated values, patterns and type-level const
   arguments. Keep unsafe source operations behind their owning assignment.
6. Publish `--emit-constants` from completed typed facts. Freeze ordering,
   vocabulary, diagnostics and count meanings before generating text oracles.

## Qualification and release

Declare outcomes before oracle generation. Every semantic rule needs accepted
and rejected controls qualified with pinned rustc; runtime rules need execution
through both rustc and the CPPGM LowIR backend. Rejections must reach the intended
rule, rather than an unsupported prerequisite. New maintainer cases precede
selection into each owning assignment's manifest and generated sidecars.

During development use focused tests, active PA text comparisons, early strict
Clippy and architecture audits. Use the repository's complete checkpoint and
assignment-release gates before pushing and activating a contract. Compare
immutable compiler binaries with frozen common controls and changed-path scaling
inputs, preserving A/A calibration, ABBA observations, output equality, work
counters and memory observations. Use the storage guard for every build,
validation batch and measurement, and scoped cleanup for known output roots.

No assignment in PA23–PA25 is complete until its typed implementation, published
scope, public fixtures, independently reviewed text oracles, performance evidence
and release checks agree.

## Initial prerequisite checkpoint

The closure parser now attaches parameter attributes to their actual parameter
owners, allowing configuration to elide parameters before resolution. PA5 owns
the corresponding eight syntax fixtures. Closure typing, captures and call-trait
lowering are still required.

Const-function declarations now retain their const permission. Typed bodies and
specializations retain their constant context. Call and pointer eligibility use
the selected CFG; destructor eligibility uses the proved cleanup schedule for
both ordinary and generic bodies. This does not implement compile-time calls,
local execution frames or the PA25 inspection mode.

The source investigation also exposed a PA19 receiver-preparation bug present in
the PA22 baseline: an implicit reborrow of a generic nominal could lack its field
instance even when the equivalent free-function call compiled. The selected
dereferenced receiver now enters the existing preparation path. Ten PA19 cases
cover this prerequisite. The work invariant varies receiver width and call count
independently: fresh borrow lifetimes can cost constant work per call, but must
not cause complete field substitution again.

For execution-frame design, the C++ reference's
`dev/src/sema/constant_frames.h` and `constant_interpreter.cpp` demonstrate
reusable frame/local/object storage, explicit object generations and separate
step/value budgets. Its `analyzer_lambdas.cpp` retains one closure identity and
restores enclosing analysis state. Rust-specific capture precision, call traits,
lifetimes and provenance still require their own typed contracts.
