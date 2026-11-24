# RustGM Programming Assignment 19 — Traits and methods

Build on [PA18](../pa18/README.md) to resolve inherent and user-trait
methods, prove trait requirements, and lower static calls. Use Rust 1.98.0,
edition 2024, on Linux x86-64 with aborting panics, checked arithmetic,
and disabled debug assertions.

## Declarations and contracts

Collect trait and impl owners once, with distinct Self, declaration and member
identities. Keep nested items in their own scope. Configuration and macro
expansion precede this inventory; inactive members do not impose requirements.
An inventory entry alone is not evidence that an impl or operation is valid.

Inherent methods support value, mutable value, shared, mutable and explicit
reference receivers, including nested references to Self. Reject a receiver
whose complete type differs from its impl's Self schema. Preserve impl captures,
method parameters, early/late lifetime distinctions and receiver output elision.
Check unused methods and reject invalid Self placement, foreign parameters,
unconstrained impl parameters and invalid parameter defaults.

Inside an impl, value-form `Self` selects its actual tuple or unit struct
constructor, including the impl's type, const and lifetime arguments. It may
be called or coerced to the matching function pointer. Constructor visibility
still applies; a record, enum, trait-default Self or ordinary type alias does
not become a tuple constructor. Reject extra arguments on `Self` and invalid
borrows through its fields. A nested item may use a closed enclosing impl's
constructor. Nested items cannot capture
generic parameters from that impl. Check even unused nested items.
`tests/spec/self-constructors/` records these source and native contracts.

Traits support type, const and lifetime arguments, defaults, required methods,
default bodies, associated functions, supertraits and where clauses within the
existing type profile. Preserve the trait's Self identity separately from its
written generic parameters. Type arguments to a user trait have equality
semantics; a subtype or one-way outlives relation cannot stand in for equality.
A user trait named Copy, Clone or Drop does not acquire the built-in identity.

Validate every trait header and default body even when there are no uses. Prove
well-formedness and requirements of trait arguments in their declaring context.
A bound implies the trait's Self requirements and supertraits, but does not grant
arbitrary permissions on other argument types. Detect supertrait cycles by trait
identity and share diamond expansion. Defaults referring to parameters keep the
same capture-safe substitution rules as PA18.

For parameter defaults, any lifetime occurrence in the original requirement
defers that bound to actual use. This includes explicit static trait arguments
and lifetimes inside callable type arguments. Lifetimes introduced only by the
default do not change the original classification. `tests/spec/default-requirements/`
pairs accepted unused declarations with rejected uses of their invalid defaults.
This is separate from the closed-bound definition check below.

A closed bound requires satisfaction at its definition only when it contains no
generic input or nonempty lifetime binder, including binders nested inside
function-pointer subjects or trait arguments. A deferred bound is still required
at each use; declaring an unused function never proves that bound. A callable
with only explicit static inputs has no local lifetime binder and keeps the
definition-time check. Associated equality results follow the same rule in PA20.

Prepare impl headers before declaration proofs that depend on them, including
forward impls and well-formedness cycles. Before publishing usable facts, validate
all impl headers, required members, signatures and bodies. Missing methods need
checked defaults; extra members, mismatched receiver shapes, arity, argument
kinds, safety, return types or lifetime contracts are errors. Method-own type and
const parameters remain rigid during conformance. An impl method cannot require
stronger conditions than the trait promises, except requirements already proved
from the impl environment or other applicable impls.

A source trait function marked `final` must have its real provided body and
cannot be overridden by an implementation. An ordinary provided trait method
remains overridable. The actual keyword requires PA31 feature authority; this
restriction is separate from impl-specialization finality. Final methods use
their provided body for dynamic Self and consume no virtual slot.

Check overlapping same-trait applications and same-name inherent methods against
complete headers. Distinct type/const applications may coexist. Lifetimes cannot
make otherwise overlapping headers disjoint. Reject duplicates and overlaps
rather than choosing by declaration order. This is the current single-crate
profile; associated projections, general normalization, extended coherence and
recursive solver caching belong to PA20.

A checked `min_specialization` request permits nonoverlapping `default fn`
methods on trait and inherent impls. Retain the keyword and the method body,
including unused bodies. Apply ordinary conformance and method selection.
A default body supplied by a trait follows the trait-body rules. Overlapping
specialization families are outside the PA19 profile.

## Proof and method selection

A complete trait goal contains Self, trait identity and every argument, including
const type/bits and lifetime identities. First use actual caller assumptions;
otherwise select a validated implementation and prove its own conditions. Support
blanket and conditional impls, closed bounds, nominal requirements and the
existing generic Drop coverage rules. Unknown Self is not guessed from a registry
of impls. Type/const arguments may be inferred when complete goal equations select
an unambiguous candidate. Defer uncertain alternatives; numeric defaults apply
only after ordinary inference and trait progress stop.

A unique compatible impl may expose a partial type constructor or array length
before every parameter is resolved. Retain that candidate and its unfinished
conditions while other body goals determine the remaining inputs. Partial input
progress is not evidence: all impl conditions and lifetime obligations must still
succeed. A pending or ambiguous alternative prevents publication. Preserve this
behavior under bound reordering, nested impl conditions and const arguments;
`tests/spec/partial-inference/` includes executable results and rejection controls.

Keep candidate equations isolated. Failed candidates cannot publish types,
lifetimes or permissions into the caller. Successful evidence retains directed
outlives and equality requirements for the normal lifetime proof. Assumptions
belong to their original body/callable binder; they cannot escape a function
pointer's local binder or become permissions for its enclosing body.

Function-pointer arguments use binder-aware alpha equality, including reordered
names and unused local binders. Local-to-static and differently correlated
lifetimes remain distinct. The course accepts redundant unused binders where the callable lifetimes
are equivalent. Higher-ranked trait bounds
such as `for<'a> Trait<'a>` still belong to PA21.

Support inherent `Type::method` calls and function values, implicit trait paths
such as `T::method`, `Self::method` and `<T>::method`, and explicit qualification
such as `<T as Trait<U>>::method`. Preserve the written Self and explicit trait
arguments. Unwritten trait arguments are inferred without applying type-path
defaults. Enum variants retain their preceding lookup and constant behavior.

An applicable inherent method precedes trait fallback for a type-qualified path.
Parameter-bound trait methods precede lexical candidates. Consume resolved trait
bindings, including aliases, anonymous imports, globs and macro call-site scopes.
Use the pinned compiler's retained glob contribution for method eligibility;
naming that ambiguous trait directly remains a separate resolution question.
Within a trait impl, its implemented trait also supplies lexical method
eligibility, including for nested functions and macro-generated members. Retain
the resolved trait identity in that impl's scope. It supplies no named import,
bound priority, fixed trait arguments or additional supertraits in scope.
Method-own requirements, later call arguments and expected results cannot choose
between distinct method names. After selecting a name, ordinary inference may
resolve multiple applications of that same trait.

For dot calls, consider each built-in dereference of the receiver, then value,
shared-borrow and mutable-borrow candidates in that order. At each candidate,
inherent methods precede parameter-bound and lexical trait methods. Static
associated functions cannot be called as dot methods. Keep receiver-place
mutability, moves, lifetime and loan legality as requirements of the selected
call; failure does not trigger another method search.

Parameter-bound lookup fixes Self to the parameter supplying the bound and keeps
that bound's complete trait application. An unrelated blanket implementation
cannot replace its type or const arguments during later inference. This
restriction applies exactly to the selected callable's implicit Self predicate;
its additional method requirements retain ordinary proof. Explicit trait
qualification retains ordinary selection. The same trait declaration may remain
a lexical candidate for a different Self, such as a reference to the parameter.

After the final dereference, array receivers can convert to the existing slice
reference profile. Retain the actual array place and borrow and publish its
array-to-slice conversion for the region proof and lowering. Mutable reference
receivers reborrow their source, including reference-valued Self. Built-in
reference dereferencing is required here; general user Deref and unsized value
relationships remain later work.

## Static dispatch and execution

Check every source body, including unused generic/default bodies, with its own
assumptions. Preserve PA14–PA18 initialization, move, promotion, lifetime, loan,
cleanup and instance-growth guarantees. A selected trait operation does not
bypass those proofs or grant stronger permissions.

Resolve demanded concrete trait callables to the validated impl override or its
checked default body. Retain trait, impl and method arguments through substitution.
Support generic methods, const arguments, function values, recursion and defaults
that call other trait methods. Receiver evaluation occurs once; returned borrows,
mutable updates and two-phase calls refer to the original source place. Reuse the
existing body-instance, aggregate ABI, cleanup and LowIR pipelines. No backend
code generator or textual intermediate transport is introduced.

## CLI and text contract

```text
rustgm --emit-semantics [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
rustgm --emit-lowir [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Use PA11's input, configuration, and output options. The semantic view
checks declarations and body inference. The LowIR mode also checks
regions, initialization, moves, loans, and demanded instances before
lowering.

The version-one text starts with `semantic-crate 1`, followed by the existing
canonical `type` table and typed `def` records. The remaining record groups follow.

- `associated-owner` and `associated-member`: dense inventory identity, owner
  kind, Self declaration/type, source, safety, trait name and receiver shape.
- `parameter`: declaration owner, binding, canonical argument and optional default.
- `predicate`: owner/source and subject with a builtin, outlives or trait bound.
  Trait bounds include their complete canonical argument list.
- `impl` and `impl-method`: validated user-trait implementation and method mapping,
  including whether the selected body is the trait's default.
- `method-call`: source, selected method and callable, receiver dereferences,
  optional borrow type, converted receiver type and inherent/bound/lexical origin.
- `inherent-path` and `trait-path`: selected declaration and explicit/implicit
  qualification, parameter-bound origin and written Self source.
- `implementation-use`: selected implementation owner and complete arguments.

The existing region-variable, equality, coercion and outlives rows follow.
`type-outlives` rows retain their body, source, type and shorter region. Constraints
marked `pending` have not passed later region/loan proof. The user-trait impl
records count the validated user-trait registry; built-in Drop retains its
existing dedicated implementation and cleanup path.

`eof owners=N members=N parameters=N predicates=N impls=N calls=N paths=N impl-uses=N`
closes the view. Dense
inventories preserve declaration order. Maps are sorted by declaration identity
or `(file,node)` source identity only at this output boundary. Per-owner vectors
preserve elaboration order. Canonical types, arguments and regions reuse the
existing type-view spelling. Work counters and hash-table iteration are excluded.

Successful emission ends in a newline, exits 0 and has empty stderr. Diagnosed
semantic rejection exits 1 with empty stdout and a diagnostic. Output failures
are errors. Signals, timeouts, panics and exit 86 are implementation failures,
never acceptable rejection. Source offsets and macro notes retain earlier rules.
The `.ref`, `.ref.stderr` and `.ref.exit_status` files freeze complete bytes.

`--emit-lowir` retains PA18's full proof and symbol/vocabulary
contracts. Required/default trait dispatch selects ordinary concrete callable
instances, and receiver conversions use the existing place/borrow lowering.
Implement LowIR lowering as well as semantic inspection. The LowIR cases
include execution controls and rejections from the later proofs.

## Efficiency, limits and later assignments

Consume the resolved source, typed signatures and semantic inventories once.
Index methods by nominal/name and trait visibility, share inherited lexical
candidate sets, and index trait candidates by complete closed shape or outer
head. Keep assumptions rigid and use canonical identities rather than rendering
types. Exclude impossible receiver depths and outer heads before copying probe
graphs; every surviving candidate still needs complete proof.

Retain bounded iterative worklists and memoized substitutions. Use the
1,000,000-work limits for trait visibility and method candidates. The
existing inference, predicate, instance, layout, and cleanup limits still
apply. Diagnose exhausted resources.

Associated types/constants, projections, normalization and extended coherence
belong to PA20. Higher-ranked trait bounds, GATs, opaque captures and advanced
generic relationships belong to PA21. General unsized types, trait objects and
constant statements remain PA22, PA23, and PA25 respectively. The PA19
semantic format reports ordinary predicates. Quantified clauses need the
later trait inspection contract.

## Run the tests

Run the PA19 test target from the repository root.

```sh
make -C pa19 test
```

[The manifest](tests/manifest.json) covers proof, dispatch, receiver
selection, function pointers, and diagnostic boundaries. It selects the
inspection or LowIR mode for each case. Use the
[PA11 backend procedure](../pa11/README.md#run-the-tests-and-an-executable)
to run executable examples.
