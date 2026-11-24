# RustGM Programming Assignment 20 — Trait solving

Build on [PA19](../pa19/README.md) to resolve associated items, normalize
types, and check the coherence of trait implementations. Use Rust 1.98.0,
edition 2024, on Linux x86-64 with aborting panics, checked arithmetic,
and disabled debug assertions.

## Required implementation

PA19 supplies trait identities, methods, dispatch and basic bounds. PA20 completes
associated items, normalized trait obligations and coherence using that machinery.
An inventory entry or a matching impl header alone never proves an operation valid.

### Associated types and their contracts

Support required associated type declarations and their implementation definitions,
qualified projections such as `<T as Trait<Args>>::Item`, and unambiguous shorthand
paths. Validate member identity, visibility, arity and complete mixed arguments;
the same member name in another trait does not supply evidence. Check missing,
extra and conflicting definitions even when no executable root uses the impl.

Retain result bounds separately from assumptions about Self and the trait's inputs.
Support the profile's Sized, Copy, Clone, user-trait and outlives requirements,
including inherited contracts and associated equalities. An impl must establish
the promised result under its actual parameter environment. A caller can use that
promise only after proving the parent trait application. A requirement on an
associated result does not imply the same requirement on its inputs.

Definition-time bound classification also examines the associated equality's
result. A nested nonempty function-pointer lifetime binder defers satisfaction
of that equality until use, even if no lifetime escapes the callable. Such a
declaration grants no proof to callers; incompatible equalities still reject
when demanded. PA19 owns the corresponding subject and trait-argument rule.

Parameter-default eligibility uses the original bound before projection
normalization. An erased lifetime still defers that bound, while a lifetime
introduced by the normalized result does not. Type/const default subjects are
indexed from normalized clauses separately. `tests/spec/default-requirements/`
checks both directions and keeps actual uses subject to the complete bound.

Normalize projections through aliases, nominal fields, callable signatures,
method contracts and demanded bodies. Preserve declaration-time formation and
lifetime obligations even if the resulting type no longer mentions their inputs.
Keep retained declaration field types available for variance while layout consumes
normalized storage. Associated equality is not injective: equal output types do
not establish equality of Self or any trait argument.

Support inherent associated type aliases on concrete closed owners when the
source enables `inherent_associated_types`. A type and function of the same
name in one impl conflict. Distinct owners can use the names independently.
Generic owners, generic or lifetime-bearing members, and extra owner or
member conditions are beyond this subset. See the Rust Reference on
[associated types](../doc/rust-reference/src/items/associated-items.md#associated-types).
PA21 handles generic associated type families and higher-ranked bounds.

### Associated constants

Support required trait constants, default trait values, impl overrides and
inherent associated constants using the existing constant-expression language.
Resolve qualified and unambiguous shorthand uses through typed declaration and
selection facts. Default values and overrides retain distinct selected identities.
Do not implement constant lookup by rendering an associated path and reparsing it.

Validate declaration types and unused definitions; instantiate generic values in
the selected owner's complete argument environment. Share evaluation for repeated
demands of the same identity, diagnose dependency cycles and retain the existing
overflow and invalid-value rules. Preserve parameter bounds and lifetimes when a
selected value reaches an expression, initializer or supported type-level use.
General constant statements and const functions remain PA25 responsibilities.

Use selected values in constant patterns and range endpoints. Constrain their
types during body inference, then evaluate and check structural equality and
coverage after initializer typing. Values must be available before function
specialization: unused owner parameters are permitted, but parameter-dependent
initializers and opaque caller assumptions cannot supply a pattern value. Keep
the requesting body's complete bound and lifetime proof when reusing a value.

### Selection, inference and normalization

Use typed goals, explicit parameter environments and isolated candidate equations.
Caller parameters are rigid; inference variables may settle from ordinary call
arguments or result context. An unresolved candidate can remain possible during
lookup, but cannot authorize the completed body. Retrying after inference must
use the completed goal without retaining assignments from failed probes.

Check every compatible assumption. A blocked opaque equation must not hide an
independent complete match; two complete matches remain ambiguous. Unresolved
inputs cannot be chosen by ignoring an alternative. Method-name ambiguity remains
a lookup error: later call arguments or expected return types cannot choose between
different applicable methods. Selected lifetime equations remain obligations of
the caller's ordinary region proof.

Qualified local projections may contain inference holes in Self and the trait's
type or const arguments. A unique header can refine those inputs while its own
premises remain pending, and its associated result shape can supply later type
constraints. Keep the selected parent proof and original application alive until
every condition completes. Defaults, including defaults involving Self, use those
same inference variables. Nested callable scopes preserve independent lifetime
slots through this process. See `tests/spec/selection/inferred-parents/lowir/`.

Normalize complete subject and argument graphs with bounded worklists. Retain
the selected impl, member, substitutions and proof obligations for later phases.
Keep immutable environments fixed for each query. A compatible candidate
must satisfy its input equations under the current assumptions. Reject a
candidate that would have to equate a rigid associated type with an
unjustified parameter or concrete type. A caller premise can supply the
projection that fixes an inferred trait argument. Preserve that projection
when checking the argument's type and coercion. Do not publish an unfinished
declaration environment to break a dependency cycle. Structural type caches must
not become caches of permission learned from a particular caller.

Projection selection during header elaboration must wait for the exact compatible
impl header and then the selected associated definition. Resume the retained
query, including opened lifetimes and input proofs, rather than restarting the
normalizer. Interleaved body allocations remain outside its proof. Generic and
lifetime-bearing results must retain unused declaration debts after normalization.

### Coherence and recursive obligations

Validate all impls, including unused ones, for orphan legality and overlap within
the supported crate and builtin-trait profile. Keep local trait/type identity and
covered parameter rules explicit. Compare full type and const argument equations;
lifetimes or a different associated value do not make overlapping impls disjoint.

Coherence asks whether headers can coexist, separately from whether a particular
caller proves their predicates. Missing implementations in the current registry
are not general negative evidence about other compatible implementations. Preserve
the existing special rules for builtin traits, derives and Drop conformance.

Traverse recursive obligations iteratively, with explicit pending, active and
completed states and bounded work. An inductive cycle is not a proof. A cycle
cutoff is also not a reusable negative answer for its ancestors. Cache only
completed results under the full environment, query mode and binder scope.
Reused evidence with fresh lifetime identities must be rebased before use.
Repeated subgoals should share work without sharing permissions across callers.

## Command-line interface

```text
rustgm --emit-trait-semantics [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
rustgm --emit-lowir [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Use PA11's input and output options. Keep PA19's `--emit-semantics` format.
Trait inspection reports the associated declarations, result contracts,
selected definitions, and normalized uses. LowIR lowering also requires
the complete lifetime and ownership proofs.

## Output format

Start the inspection with `trait-semantics 1`, followed by PA19's typed
records. Write the additional rows before the `eof` summary.

| Row | Retained fact |
| --- | --- |
| `impl-type owner=O declaration=D definition=S` | The selected implementation of an associated type. |
| `associated-type def=D type=T` | An associated type declaration's formal projection or an impl definition's type. |
| `associated-constant def=D type=T` | The member's declared type. |
| `result-contract def=D projection=T` | The trait member's formal result application. |
| `result-bound def=D index=I subject=T bound=B` | A result requirement in retained contract order, using PA19's predicate vocabulary. |
| `impl-constant owner=O declaration=D selected=S default=BOOL` | A validated impl's selected override or checked trait default. |
| `constant-use source=F:N definition=D arguments=[...]` | A typed associated use with complete lifetime, Self, type and const arguments. |
| `constant-dispatch declaration=D arguments=[...] selected=S arguments=[...]` | A completed value dispatch with the original and selected initializer's arguments. |
| `normalized-node source=F:N original=T result=U` | A retained original source type paired with its current normal form. |
| `normalized-definition def=D original=T result=U` | A retained original declaration template paired with its current normal form. |

Type/member records follow associated declaration order. Impl constants follow
impl and member order. Uses and normalized nodes sort by file/node identity;
normalized definitions sort by declaration identity. Completed value dispatches
sort by declaration and canonical argument identity. Generic argument spelling
uses the existing canonical writer. Hash iteration must not determine any row order.

For a minimal example, [this fixture](tests/spec/contracts/associated-type-declarations/semantics/concrete.rs)
defines `trait A { type Out; }` and implements `A` for `u8` with `Out = u16`.
Its [expected output](tests/spec/contracts/associated-type-declarations/semantics/concrete.rs.ref)
contains the records below.

```text
impl-type owner=16 declaration=15 definition=17
associated-type def=15 type=22
result-contract def=15 projection=22
result-bound def=15 index=0 subject=22 bound=sized
associated-type def=17 type=12
```

The preceding type and definition rows identify `22` as the trait's formal
projection and `12` as `u16`; the implementation mapping connects the declared
member to its selected definition. The expected output also includes the other
shared records and the exact count line.

The count line retains PA19's fields, then appends `type-members`,
`constant-members`, `result-contracts`, `result-bounds`, `impl-constants`,
`constant-uses`, `constant-dispatches`, `normalized-nodes` and
`normalized-definitions`, in that order. Each counts the corresponding added row.

A constant use is not necessarily a completed value dispatch: ordinary executable
uses may wait for specialization, whereas a declaration-time const argument may
already demand a selected initializer. Report only dispatches that actually exist.
A symbolic normal form may remain an opaque projection, including an unchanged
original/result identity. These records expose retained source and declaration
normalizations, not every transient internal fold. Keep their original types so
erased inputs and later proof obligations remain inspectable. Work counters are
not language decisions and must never be serialized into the grading contract.

The supplemental `--emit-lowir` lane is required. It runs complete body, lifetime,
ownership, destruction and layout checks before emitting the existing LowIR
contract. Selected associated values and methods must reach executable code through
the ordinary specialization pipeline. CPPGM remains the external LowIR-to-native
backend; neither CPPGM nor rustc may implement a Rust frontend phase.

Success exits 0 with newline-terminated stdout and empty stderr. Diagnosed
rejection exits 1 with empty stdout. Signals, timeouts, panics and exit 86 are
implementation failures, never acceptable negative language outcomes. Earlier
stage views and their committed byte contracts remain available.

## Run the tests

Run the PA20 test target from the repository root.

```sh
make -C pa20 test
```

[The manifest](tests/manifest.json) selects inspection and executable cases.
Use the [PA11 backend procedure](../pa11/README.md#run-the-tests-and-an-executable)
to run executable examples. Exercise repeated subgoals and shared projections
to check that the implementation stays within its work limits.

PA21 covers higher-ranked trait premises, generic associated types, opaque
types, and generic projection drop checking. PA22 and PA23 add unsized
execution and trait objects. PA25 adds const functions and general
statements in constant evaluation.
