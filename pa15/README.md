# RustGM Programming Assignment 15: Lifetime contracts

Status: **implemented-draft**. This is a bounded course language profile.
Prerequisite: PA14. Profile: Rust 1.98.0, edition 2024, Linux x86-64.

## Required implementation

Retain lifetime identity through type inference and typed bodies. Distinguish
`'static`, named bound parameters, inferred lifetimes and nested callable
binders. Apply function and nominal lifetime arguments without capturing a
nested binder or coupling independent call sites. Diagnose invalid parameter
names, arity, binding-time arguments and ambiguous return elision.

Elided reference inputs introduce independent parameters. A single eligible
input lifetime supplies an elided reference result. Preserve explicitly named
results and captured lifetimes in function-pointer types. Separate early-bound
function arguments from late-bound callable parameters; explicit arguments
cannot instantiate late-bound parameters. Lifetime-only aliases, structs and
enums retain their declared identities. Type/const generics and general traits
remain later assignments.

Collect declared outlives bounds and well-formedness assumptions from signatures.
Function bounds may constrain a lifetime-only type, such as `&'a u8: 'b` or
`[&'a u8; 0]: 'b`. Decompose explicit type bounds through every referent and
structural component, including zero-length array elements. The subject must
also be well-formed; merely mentioning a type in a where-clause does not supply
its well-formedness assumptions. These predicates participate in early/late
lifetime classification and are checked when naming a function item, even when
the item is never called. Symbolic type parameters such as `T: 'a` remain PA18.
A local annotation creates requirements, not new assumptions. Enforce nominal
bounds at uses; a type alias's own declared bounds are not enforced by the pinned
language profile. Infer nominal lifetime variance from typed fields. Shared
reference lifetimes can shorten; mutable referents are invariant. Function
parameters are contravariant and results covariant. Unsizing an array reference
to a slice preserves the element type's required relationships.

A coercion is a directed requirement. Never repair an invalid return by merging
independent universal parameters or by adding the required relationship to the
assumptions. Instantiate higher-ranked source binders with fresh variables and
target binders with fresh placeholders. Reject a placeholder that escapes into
an older universe. Retain constraints from unused calls, casts and function
items as well as demanded return values.

Use typed borrow-place provenance to bound references into local storage.
References into parameters' referents differ from references into the parameter
variables themselves. Static storage and eligible promoted constants have
persistent provenance. Promotion retains source evaluation, effects and runtime
checks. Its closed constant subset does not implement general const functions
or mutable constant memory (PA25).

This assignment proves lifetime contracts. Intrabody last-use liveness, storage
expiry along control-flow paths, conflicting accesses and two-phase activation
belong to PA16. A successful contract view alone does not authorize LowIR
reference erasure. The executable pipeline must also consume PA14/PA16 proofs.

## Text contract

`dev/rustgm --emit-regions SOURCE` runs the typed-body and lifetime-contract
pipeline and emits `regions-crate 1`. `--count-regions` performs the same checks
and emits only the final count line. Both accept `-o OUTPUT`. Language failures
return 1 with no partial stdout; signals, panics and resource infrastructure
failures never count as expected language rejection.

Bodies follow contract construction order, including initializer and constructor
bodies, and begin `body N source=FILE:NODE`. Each body contains:

- `  universal bN=gN` in body-parameter order.
- `  variable rN=gN` in ascending semantic inference identity order.
- `  region gN static|local` or `  region gN universal|variable universe=N`, in
  graph identity order. Static is g0; local-storage upper bound is g1.
- `  assume gLONGER:gSHORTER`, sorted numerically by the pair.
- `  require gLONGER:gSHORTER`, sorted numerically by the pair.

Body parameter indices are binder-local; semantic inference identities retain
crate allocation order. Graph identities are body-local. Nested callable
placeholders and their fresh variables appear as graph nodes with their own
universe, without masquerading as named body parameters. The `local` sentinel
is a contract upper bound, not a runtime CFG lifetime.

The edge records are the retained direct constraints, not a transitive closure.
Implicit reflexive and static requirements are omitted. Explicit assumptions
and distinct generated requirement occurrences retain their multiplicity;
identical edge lines are permitted. Every inference variable includes its local
availability requirement. Assumptions remain separate from requirements even
when their endpoints match. Do not use rendered text as semantic transport.

Each body ends `end-body`. The final line is
`eof bodies=N universals=N variables=N regions=N assumptions=N requirements=N`.
Count mode omits `eof `. Counts cover the same nodes and edge occurrences;
solver work counters and hash iteration order are not part of the text oracle.

## Efficiency and validation

Consume canonical type, variance, substitution and selected-conversion facts;
do not parse source again. Use bounded iterative graph propagation and cache
queries in their exact type/binder environment. Ordinary closed types should
avoid repeated walks. The contract graph caps retained nodes and edges at
1,000,000 per body and charges bounded propagation work. Inspection sorts edge
records; count mode reads retained counts without materializing those records.

The release suite declares 202 cases covering declarations/elision, explicit and
implied bounds, variance, reference returns, callable binders, casts and nominal
constraints. Each expected result is declared before oracle generation and
independently checked with pinned rustc. Structural maintainer controls also
check small graphs against exhaustive set-inclusion models, variable identity,
placeholder universes and a hand-written exact text vector.

Run `make test-pa15`, `make test-rustc`, and the root lint/audit/harness checks.
Only `make ref-test-pa15` regenerates this assignment's references. Activation
requires the complete handout, manifests, independent evidence and reviewed
text oracles together; an empty or planned suite cannot count as passing.
