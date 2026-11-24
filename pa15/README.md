# RustGM Programming Assignment 15 — Lifetime contracts

Build on [PA14](../pa14/README.md) to check lifetime contracts using the
canonical type and body facts. Use Rust 1.98.0, edition 2024, on Linux x86-64.

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

```text
rustgm --emit-regions [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Use PA11's input and output options. After checking the typed bodies
and lifetime contracts, begin the output with `regions-crate 1`. A source
error exits 1 with a diagnostic and no partial output.

Bodies follow contract construction order, including initializer and constructor
bodies, and begin `body N source=FILE:NODE`. Write the records below for each body.

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
Count each retained graph node and edge occurrence. Emit records in the
specified order.

## Efficiency

Consume canonical type, variance, substitution and selected-conversion facts;
do not parse source again. Use bounded iterative graph propagation and cache
queries in their exact type/binder environment. Ordinary closed types should
avoid repeated walks. The contract graph caps retained nodes and edges at
1,000,000 per body and charges bounded propagation work. Sort the edge
records only for inspection.

## Run the tests

Run the PA15 test target from the repository root.

```sh
make -C pa15 test
```

[The manifest](tests/manifest.json) covers declarations, elision, outlives
bounds, variance, reference returns, callable binders, casts, and nominal
constraints. When writing additional cases, check that independent calls
produce distinct inference variables and that hidden placeholder lifetimes
stay inside their binders.
