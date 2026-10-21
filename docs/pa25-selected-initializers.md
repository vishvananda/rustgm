# Selected associated initializer execution

Status: implemented and focused-qualified draft; PA25 remains planned. This
checkpoint does not activate assignments or publish generated references.

A selected associated constant can now execute its checked initializer CFG,
including statements, mutable locals, loops, nested inline constants, const calls
and suspended static reads. The original `inline-specialized/associated.rs`
fixture is unchanged and promoted from its recorded implementation boundary to
an accepted source/native control.

## Identity and proof boundary

`Target::Associated` and `Requirement::AssociatedBody` carry the full semantic
`values::Instance` plus the expected result `TypeId`. Before provider cache reuse,
the complete arguments must be concrete and the result must equal substitution
of the completed canonical declaration type. An erased owner parameter is still
part of this identity; an unresolved erased parameter cannot authorize execution.
The existing selected-result normalization and projection debts remain in the
ordinary proof route.

`Owner::InitializerInstance { instance, kind }` and
`BodyKey::InitializerInstance(instance)` distinguish these bodies from ordinary
declarations, anonymous instances and closure adapters. Callable `Body.instance`
stays empty: no callable execution permission is manufactured. The original
initializer template and the selected materialized body both pass ordinary
region, ownership, loan, storage and destruction checks. Const eligibility runs
on the selected CFG independently of runtime branch reachability. Conditional
output contracts remain an explicit specialization boundary.

The provider retains one checked original template per declaration and uses the
existing prepared/template store when that original already exists. The ordinary
verifier receives the retained original, rather than reconstructing its CFG.
Each selected body is catalogued under `(Instance, result TypeId)`. A reached
associated load either reads its completed value, enters its catalogued frame,
or suspends with the exact body requirement. Preparation and resumption remain
iterative. Associated activity is separate from the existing monomorphic
`DefId` static-initializer activity set.

Both AST and CFG completion publish through one checked method into the existing
full-instance table. Its internal payload is now `{ ty, value }`; the public
`Reader::associated` signature is unchanged. Publication rejects a different
canonical result, a changed value after completion, or exhausted retained-value
storage before insertion. No selected value is installed in the ordinary
definition-value cache.

## Retained work and inspection

`associated_templates` counts prepared originals; `associated_instances` counts
materialized selected CFGs. The 16/64/256 family uses every selected constant
twice, retains one original, and publishes exactly one CFG per complete instance.
Original-template bytes remain constant across that family; code and completed
value charges have linear asserted bounds. These are work/storage assertions,
not wall-time or actual allocator/RSS measurements.

Template storage charges retained owned CFG/destruction collections and the
contract payload before cloning. Early handoff charges both consumer-owned
template copies; late lookup charges the additional provider copy. Shared Arc
payloads are excluded from repeated ownership charges. Execution catalog and
completed-value maps have conservative entry charges, with independent 64 MiB
limits; completed values add `associated_value_bytes`. Existing suspended-session
and execution budgets still apply. This is the repository's charged retained
storage contract, not an exact heap census.

The unreleased `constants 1` grammar adds:

```text
root associated-instance instance=DEF:ARGS type=TYPE value=vID
```

Rows follow ordinary initializer roots and precede inline roots, sorted by
definition and full argument identity. `ARGS` refers to the canonical generic
argument identity in this compilation. Declaration rows remain
`status=deferred-initializer`; a completed selection is not completion of every
selection. The renderer consumes the typed payload without semantic queries.
The independently object-qualified `associated-execution/const-machine.rs`
example produces two distinct associated roots sharing the containing ordinary
constants' values, 3 and 4. Two independent inspection runs are byte-identical.

## Qualification and remaining work

`associated-execution-outcomes.json` declares 17 inputs: ten accepted
and seven rejected by pinned Rust 1.98 full object generation and the checked
RustGM pipeline. All ten accepted programs execute against both implementations.
Controls include erased type parameters, selected const arguments, canonical
projected results with lifetime debt, forward callees, trait selection, nested
inline bodies, shared-reference results, suspended static reads, arithmetic
faults, dependency cycles, dangling references, mutable static reads and non-const
calls in unused/skipped bodies. The original distinct-argument recursion
hypothesis was rejected independently with E0391; its original source and first
observation are retained, rather than relabelled as a supported positive.

The final bounded batch passed three private identity/publication/budget
invariants, 26 nested-evaluation groups including ignored native controls, five
selected initializer groups, 15 existing inspection groups and 116 trait groups
(unrequested ignored tests remain ignored). Strict release all-target/all-feature
Clippy, the 1,383-file architecture audit with zero findings, layout and upstream
checks passed. Evidence is under `obj/parallel/pa25-associated/`: `final-*.log`,
object diagnostics, native cases, deterministic inspection output and
`source-sha256.json`. The resource receipt is
`obj/parallel/resources/4d10df8d7ed74be49ffc1babaf806f9d.json` (exit 0).
Compiler source digest:
`2a6545df9510754e3c6a8a8bb9e734ec0869c0cd316a3d417e2780f77f643a71`.

This checkpoint does not resolve specialized conditional output interfaces,
generic captured execution on this integration base, dynamic cross-body pattern
or cross-enum discriminant continuations, all remaining allocation/slice/reference
validity families, or released inspection/oracle coverage. No full matrix,
benchmark, reference generation or activation ran in this worker slice. Changed-
path wall/RSS qualification remains pending integrator scheduling, and the earlier
unexplained scalar RSS increase remains unresolved.
