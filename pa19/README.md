# RustGM Programming Assignment 19: Traits and methods

Status: **implemented-draft**. Prerequisite: PA18. The bounded language profile is
Rust 1.98.0, edition 2024, Linux x86-64, panic abort, overflow checks enabled and
debug assertions disabled. This assignment adds semantic inspection and extends
PA18's checked source-to-LowIR pipeline with inherent and user-trait dispatch.

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

Check overlapping same-trait applications and same-name inherent methods against
complete headers. Distinct type/const applications may coexist. Lifetimes cannot
make otherwise overlapping headers disjoint. Reject duplicates and overlaps
rather than choosing by declaration order. This is the current single-crate
profile; associated projections, general normalization, extended coherence and
recursive solver caching belong to PA20.

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
lifetimes remain distinct. The three explicitly documented redundant-binder
acceptance differences from pinned rustc are part of this draft contract; see
[the validation review](../docs/pa19-validation.md). Higher-ranked trait bounds
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

`dev/rustgm --emit-semantics [-o OUTPUT] SOURCE` and `--count-semantics` run
declaration/body inference and report selected typed facts. They are inspection
modes: region, initialization, move, loan and demanded-instance proofs remain in
`--emit-lowir`. A successful semantic view does not authorize execution. The
paired `inspection/` and receiver-contract controls distinguish these boundaries.

The version-one text starts with `semantic-crate 1`, followed by the existing
canonical `type` table and typed `def` records. It then reports:

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
closes the view. Count mode emits that same counts line without `eof `. Dense
inventories preserve declaration order. Maps are sorted by declaration identity
or `(file,node)` source identity only at this output boundary. Per-owner vectors
preserve elaboration order. Canonical types, arguments and regions reuse the
existing type-view spelling. Work counters and hash-table iteration are excluded.

Successful emission ends in a newline, exits 0 and has empty stderr. Diagnosed
semantic rejection exits 1 with empty stdout and a diagnostic. Output failures
are errors. Signals, timeouts, panics and exit 86 are implementation failures,
never acceptable rejection. Source offsets and macro notes retain earlier rules.
The `.ref`, `.ref.stderr` and `.ref.exit_status` files freeze complete bytes.

`--emit-lowir` and `--count-lowir` retain PA18's full proof and symbol/vocabulary
contracts. Required/default trait dispatch selects ordinary concrete callable
instances, and receiver conversions use the existing place/borrow lowering.
The supplemental LowIR lane includes every native execution control and the
later-proof rejections; it is not optional when implementing this assignment.

## Efficiency, limits and later assignments

Consume the resolved source, typed signatures and semantic inventories once.
Index methods by nominal/name and trait visibility, share inherited lexical
candidate sets, and index trait candidates by complete closed shape or outer
head. Keep assumptions rigid and use canonical identities rather than rendering
types. Exclude impossible receiver depths and outer heads before copying probe
graphs; every surviving candidate still needs complete proof.

Retain bounded iterative worklists and memoized substitutions. The reference's
trait visibility and method candidate work budgets are each 1,000,000; existing
inference, predicate, instance, layout and cleanup budgets still apply. A resource
limit is a diagnostic, never permission to omit checks. Maintainer controls cover
1,024 unrelated traits/call scopes, deep reference chains and predicate diamonds.
Measure immutable binaries and inputs with A/A calibration and paired blocks,
retaining every observation and checking output equality. Follow `spec.md` and
the same safe Rust, Clippy and syntax-aware audit requirements as earlier stages.

Associated types/constants, projections, normalization and extended coherence
belong to PA20. Higher-ranked trait bounds, GATs, opaque captures and advanced
generic relationships belong to PA21. General unsized types, trait objects and
constant statements remain PA22, PA23 and PA25 respectively. The two remaining explicit
later-stage boundary fixtures record rustc acceptance and this stage's rejection.
The higher-ranked bound boundary belongs to the semantic inspection lane: its
ordinary-predicate vocabulary cannot encode quantified clauses, even when later
source-to-LowIR functionality can compile the same program.

## Validation

The manifest contains 1,104 cases: 705 accepted and 399 rejected. There are 440
semantic inspection cases and 664 LowIR cases. The LowIR lane has 269 accepted
programs, all with declared clean execution through rustc and CPPGM, and 395
rejections. No accepted native control is silently skipped.

Six acceptance differences are explicit: two later-stage boundaries, one
pending-lifetime inspection control and three redundant-binder compiler
differences. They are excluded from default-rustc acceptance-parity claims.
The migrated PA17/PA18 inherent boundaries and PA18 trait-method boundary retain
their original source bytes in accepted fixtures.
The former associated-type and associated-constant boundaries are covered by
PA20's associated-item contract. The former trait-object boundary is now an
accepted PA23 fixture, with its source bytes preserved.

Run `make test-pa19`, `make test-rustc`, `make test-native-pa19` and
`make lint audit-rust test-harness test-upstream`. The root report includes PA19;
PA24–PA38 remain planned and fail explicitly. See
[the validation review](../docs/pa19-validation.md) for qualifications, reference
review, native evidence and the distinction between inspection and execution.
