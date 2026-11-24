# RustGM Programming Assignment 18 — Generics

Extend [PA17](../pa17/README.md) with type and const parameters and concrete
instances of generic bodies. Use Rust 1.98.0, edition 2024, on Linux x86-64
with aborting panics, checked arithmetic, and disabled debug assertions.

## Required implementation

Extend the checked source-to-LowIR pipeline with type and const parameters,
generic aliases and aggregates, generic function bodies and demanded concrete
instances. Retain PA15–PA17 lifetime, ownership, loan and destruction guarantees.
Check every declaration and source body, including unused generic bodies.
Demand controls executable instantiation, not whether a source body is valid.

Parameters have identities belonging to their declaration and ordinal. Preserve
lifetime/type/const argument kinds and declaration order; type and const
parameters may intermix after lifetimes. Validate duplicate names, arity,
argument kinds, const types and default ordering. Const parameter types are the
integer primitives, bool and char, including aliases to those types. Keep the
type and all scalar bits in canonical identity. Function items with different
type/const arguments remain distinct even when their signatures erase them.

Support explicit arguments, turbofish, inferred arguments and `_` placeholders
where the pinned language permits them. Infer through calls, result annotations,
constructor fields, function pointers and fixed array shapes. Arguments needed
for type formation or executable instance identity must be determined. An ordinary
alias may erase an unused const placeholder: `type E<const N:usize>=u8;` permits
`let value:E<_>=7u8;`. Erasure does not discharge an associated projection's
original input obligations. Distinguish early lifetime arguments from late function
binders: supplying a late lifetime explicitly is invalid. Resolution of a bare
path in an argument follows Rust's type/const ambiguity rules; a const block can
select the value namespace explicitly.

Aliases, structs and enums can declare defaults. Resolve omitted arguments in
one simultaneous substitution environment; a default can use earlier parameters.
Explicit arguments need no unused defaults, but all default declarations still
need validation. Finite applications such as `type A<T=A<u8>>=T` are valid;
actual default or alias-body cycles are errors. Ordinary function and impl
parameters cannot introduce defaults. Alias bounds are checked as declarations,
but do not grant proof assumptions to a caller using the alias.

Classify parameter-default bounds from the original written requirement. A bound
containing any lifetime, including an explicit static reference, is deferred to
actual use. A lifetime introduced only by substituting the default does not defer
an otherwise eligible check: a mutable reference default still fails `T: Copy`.
Keep formation checks separate and enforce every bound when the default is used.

Substitute arguments through signatures, aliases, aggregate fields, annotations,
array lengths and selected body facts without reparsing source. Preserve binders
inside function-pointer types: an inserted reference must not be captured by a
nested callable lifetime. Generic array lengths remain typed const parameters
until substituted. Stable const arguments may use the existing closed constant
expression subset or a standalone const parameter, optionally in one block;
general arithmetic on a parameter in a type-level length remains invalid under
the pinned stable profile. Macro-forwarded literals and permitted expression/block
roots enter this same const-argument grammar from their retained typed captures.
Keep scalar kinds, unary negation, block safety, edition and source provenance;
neither opaque arithmetic nor a type context becomes valid by forwarding. Runtime arithmetic on a scalar const parameter is
ordinary checked arithmetic, including overflow and bounds panics.

Partial callable inference also preserves the distinction between a hole for an
entire higher-ranked function pointer and holes inside its parameters. For
example, passing a callback with independent borrowed inputs through
`fn pass<U>(f: fn(U)) -> fn(U)` with `U = (_, _)` must retain valid source lifetime
relationships. Inferring `U` as `for<'a> fn(&'a u8)` must preserve that binder.
Neither inference path permits returning a caller's borrow as `&'static u8`.

Tuple fields use context established by earlier field coercions. For
`type Pair<T> = (T,T)`, `let pair: Pair<_> = (x,y)` moves an initially
unconstrained mutable reference `x` but reborrows `y` after the first field fixes
the shared type. This context propagates through nested tuples and block tails.
Separate holes in `(_,_)` remain independent; later inference does not undo an
earlier move.

## Template proofs and concrete execution

Resolve a borrowed generic field's instance before checking move and lifetime
obligations. Reborrowing the field preserves ownership of its referent, including
its destructor obligation.

Check bodies with rigid type/const parameters and their own assumptions. The
built-in Sized, Copy and Clone identities and the existing supported derives
provide the bounded capability profile; Copy implies Clone, while Clone alone
does not authorize duplication. Support inline and where-clause bounds,
`?Sized` declaration relaxation, directed outlives requirements and implied
reference well-formedness. An assumption about a composite type is not a proof
of an arbitrary stronger element bound. Imported or reexported built-ins keep
their identity; a user trait with the same spelling does not acquire it.

Composite builtin assumptions respect proved lifetime identity. For example,
`(&'x u8, &'x u8, T): Copy` permits copying `(&'x u8, &'y u8, T)` when
`'x: 'y` and `'y: 'x` establish equality. Transitive, static and implied
relationships use the same proof. One-way bounds do not establish equality.
Retain inferred call requirements until the body lifetime proof can validate
them, including nested composite uses and unused generic bodies. Concrete
specializations must still satisfy their substituted requirements.

Perform initialization, move, promotion, region and loan checks on unused
bodies too. Reject double moves, uninitialized reads, invalid field moves,
escaping local references and overlapping live loans before specialization.
A later concrete Copy argument cannot make an invalid unconstrained template
valid. Conversely, a possible generic destructor cannot be discarded merely
because another instantiation has no cleanup. Retain const-dependent index
bounds and eligible promotions until concrete substitution makes them known.

Starting from the ordinary entry, demand direct callees, reified function-item
values, function pointers in constants/statics and selected destructor methods.
Support implicit tuple-struct and tuple-variant constructor factories, including
references to them as function values. Instantiate their typed bodies and use
the existing aggregate ABI. Deduplicate a repeated complete instance identity;
ordinary and mutual recursion can refer to an already reserved instance.
Increasing recursion such as `f::<(T,)>()` requires a bounded diagnostic.

Generic structs and enums use PA12's layout rules for their actual field types,
including zero-sized fields, const lengths, representations and conditional
derives. Generic patterns, moves, borrows, references and function-pointer calls
use the preceding assignments' selected operations. Owned generic values pass
through the same initialization and destruction checks as ordinary values.

Extend the built-in Drop implementation rules to type/const parameters. An impl
must cover its complete nominal schema with distinct parameters; reordered type,
const and lifetime parameters and aliases can express that same schema. Reject
specialization, repeated parameters, stronger impl/method bounds, duplicate impls
and Copy conflicts. Retain the checked inverse argument mapping for method
selection. Prove each destructor template under its declared lifetime
assumptions. Resolve concrete dispatch and build the cleanup plan for each
demanded instance. A partial move keeps the containing object's remaining
cleanup and lifetime requirements.

Demand destructor code only from actual selected cleanup roots. A cached recipe,
an unused ordinary owner or a path that diverges before cleanup does not itself
require executable destructor code. A selected method's dependencies participate
in the same instance-growth bound as ordinary generic calls. Automatic field and
array cleanup retains PA17's ordering, flags, packed-field handling and panic
abort behavior.

## CLI and text contract

Use PA11's `rustgm --emit-lowir` interface for executable code. Finish
template checks and concrete instance selection before writing output. Success
exits 0. A source error exits 1 with empty standard output and a diagnostic
with the source and macro expansion locations. A library case can produce
empty LowIR when it demands no concrete instance.

`--emit-patterns` inspects coverage for canonical instantiated fields and
module visibility. It uses PA13's selected-pattern format. The other
inspection commands retain their original obligations.

The LowIR dialect, entry adapter `main`, aggregate calling convention, globals,
scalar operations and cleanup vocabulary are inherited from PA11–PA17. Ordinary
bodies retain `rg.D` names, where D is the resolved declaration identity.
Instantiated functions, constructors and custom destructors use `rg.D.iB`, where
B is their index in the proved concrete body list. Original ordinary bodies
precede appended instances. Instance discovery is depth-first in retained body
use order, followed by selected cleanup dependencies; repeated identities reuse
their first index. This is deterministic construction order, not lexical sorting
of rendered types. Callable symbol reservation and LowIR emission use the existing
lowering queue. Glue names remain `rg.drop.N`; globals remain `rg.data.N`.
Hash-table iteration must not determine any emitted identity or ordering.

For example, `fn id<T>(x:T)->T{x}fn main(){let _=id(7u8);let _=id(9u8);}`
requires one `u8` instance. Adding `id(300u16)` demands another instance.
See the [identity fixture](tests/spec/callables/identity.rs) and its
[expected LowIR](tests/spec/callables/identity.rs.ref).

## Bounds, efficiency and later assignments

Keep source parsing, parameter elaboration and template checking shared. Canonical
argument equality must not render types. Use bounded iterative worklists and
memoized substitutions, field/layout queries and cleanup recipes. Retain compact
IDs, dense CFG joins and sparse active-loan enumeration. A concrete body carries
its immutable proof and selected plan through executable assembly. After explicit
and implicit constructor template discovery and all ordinary proofs, an empty
template set permits ordinary lowering to follow dependencies without a second
generic demand walk.

Allow up to 128 simultaneously active distinct instances per declaration,
100,000 instantiated bodies overall, and 64 MB of retained body and contract
storage under its accounting. Callable dependency traversal and selected cleanup
recipe discovery each have a 1,000,000-work bound. Existing type substitution,
layout, promotion, ownership and cleanup budgets also apply. A bound is a diagnosed
resource limit. Report the failure if the work cannot be completed within it.

General inherent/trait method lookup belongs to PA19; associated items,
normalization and general coherence to PA20; higher-ranked trait obligations,
GATs and opaque identities/captures to PA21. PA18 uses existing lifetime/binder
machinery as required by its signatures; it does not claim those later features.
Unsized executable values and pointer metadata remain PA22, trait objects PA23,
closures PA24, and general const functions PA25.

## Run the tests

Run the PA18 test target from the repository root.

```sh
make -C pa18 test
```

[The manifest](tests/manifest.json) includes executable programs, growing
instance graphs, library units, and pattern inspections. Use the
[PA11 backend procedure](../pa11/README.md#run-the-tests-and-an-executable)
for examples intended to terminate. For an intentional divergence case,
inspect the generated LowIR or object without running it.
