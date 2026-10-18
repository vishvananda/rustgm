# RustGM Programming Assignment 21: Advanced generic relationships

Status: **planned**. These are implementation and release requirements, not a
released grading contract. Prerequisite: PA20. The bounded source profile is
Rust 1.98.0, edition 2024, Linux x86-64, panic abort, overflow checks enabled and
debug assertions disabled. The stage must continue to fail until its complete
manifest and reviewed text oracles ship.

## Required implementation

PA20 supplies associated items, complete trait applications, ordinary predicates,
normalization and retained proof continuations. PA21 adds quantified relationships,
associated type families and opaque types while preserving those proofs.

### Higher-ranked trait bounds

Support trait-position binders (`T: for<'a> A<'a>`) and where-clause binders
(`for<'a> &'a T: A`). Respect their different lexical scopes. Resolve named
lifetimes once and retain canonical binder identities and complete subjects,
trait arguments and associated equalities. Equivalent binder renaming must not
change meaning; distinct slots and enclosing captures must not collapse.

Prove a universal obligation under fresh scoped universal lifetimes. A valid
implementation for only `'static`, or for one particular caller lifetime, does
not establish a bound for every lifetime. Using a universal caller assumption
instantiates it for the requested application; it does not grant the caller a
new universal assumption or equate unrelated lifetimes. Preserve correlations
across multiple bound lifetimes, associated results and nested callable binders.

Support quantified premises in function and nominal contracts, implementation
conditions, supertraits and associated result bounds. Check unused declarations
and impl conformance as well as demanded calls. A caller bound may justify a
body operation only in that caller's environment. Preserve quantified proof
scopes through normalization, header waits, specialization and concrete body
instance validation. A semantic inspection alone is not a completed region proof.

The initial implementation controls use user-defined traits and scalar/reference
programs. They must not depend implicitly on closures, trait objects or standard
library iterator implementations. See the pinned
[trait-bound rules](../doc/rust-reference/src/trait-bounds.md#higher-ranked-trait-bounds).

### Generic associated type families

Support lifetime, type and const parameters on associated type declarations and
impl definitions. A projection retains its declaring trait/member, complete Self
and trait inputs, and separate family arguments. Validate arity, parameter kinds,
visibility, result bounds and each family's where clauses, including unused impls.
A definition cannot strengthen the declared application requirements.

Apply the pinned rules for required where clauses on GAT declarations, including
requirements inferred from method uses, their intersection across multiple uses
and propagation through associated result bounds. Do not assume every GAT needs
`Self: 'a`, or omit a required clause because the current impl happens to work.
Use the retained typed signatures and contracts to compute these requirements.

Normalize each demanded family application with its own complete substitutions
and proof environment. Repeated applications may share structural work; a result
from one family argument or caller must not authorize another. Associated outputs
remain non-injective. An unused generic definition still has to establish its
result contract. Associated type defaults and inherent associated types remain
outside the pinned stable profile. See
[associated types](../doc/rust-reference/src/items/associated-items.md#associated-types).

### Opaque types and capture

Support argument-position `impl Trait` as an anonymous generic parameter, with
its proper call-site generic argument rules. Support return-position `impl Trait`
in free/inherent functions and trait methods. Give each opaque declaration its
own identity and checked hidden concrete type; callers use the published bounds.
A matching hidden representation does not make two opaque declarations equal.
Return values must establish one compatible hidden type and satisfy its declared
bounds through ordinary body typing.

Implement edition-2024 generic capture and explicit `use<...>` capture bounds
under the pinned rules. Preserve type, const and lifetime capture, including
nested binders and implicit trait Self. Diagnose excluded captures and malformed
capture lists. Hidden types become available to concrete layout, destruction and
lowering only through the checked defining context and complete substitutions;
they must not leak back into generic caller inference.

Type-alias `impl Trait`, local/field opaque type syntax and unstable associated
opaque definitions are not part of this stable profile. Async opaque state and
closure construction remain later assignments. Exercise opaque results first
with ordinary structs, references and user traits. See
[`impl Trait` and capture](../doc/rust-reference/src/types/impl-trait.md).

### Variance and generic destruction

Extend the existing variance, outlives and destruction machinery to the new
binder, family and opaque identities. Reuse declared field structure for variance
and checked normalized storage for execution. Keep covariance, contravariance
and invariance distinct, and preserve nested callable binder depth. A projection
or opaque result is not a structural promise about its inputs.

Complete generic drop checking for the supported safe source profile. Retain the
required lifetime/type dependencies of actual owned storage and selected drop
behavior through family normalization and hidden-type substitution. Preserve
initialization, conditional cleanup, borrow validity and destruction order from
earlier stages. Neither an erased input nor a shared layout cache grants a
caller an outlives proof. Qualify positive and negative source controls against
rustc before choosing a more permissive rule.

Library hooks such as PhantomData and UnsafeCell, unstable `may_dangle` promises,
unsafe operations and trait-object destruction are separate responsibilities.
Do not require them implicitly in the first safe generic controls. The existing
[variance rules](../doc/rust-reference/src/subtyping.md) remain authoritative for
the supported constructors.

## Tool surface

Develop a separate `--emit-generic-semantics` and `--count-generic-semantics` view.
PA19's `semantic-crate 1` and PA20's `trait-semantics 1` bytes remain unchanged.
The new view must expose canonical binder scopes, quantified predicates, family
parameters/contracts, opaque identities/captures and retained proof relationships.
Define the exact vocabulary, deterministic ordering and counts from actual typed
facts before generating public references; no draft row is a grading contract.

The supplemental `--emit-lowir` lane proves complete region, ownership,
destruction and layout obligations before emitting the existing LowIR contract.
Execute accepted controls through CPPGM and compare their declared results with
pinned rustc. The frontend must not invoke either compiler to implement a Rust
phase. Success exits 0 with newline-terminated stdout and empty stderr. Diagnosed
rejection exits 1 with empty stdout. Panics, signals, timeouts and exit 86 are
implementation failures, never accepted negative outcomes.

## Implementation and performance

Reuse the parsed syntax, canonical type graph, binder substitution and lifetime
proof machinery. Represent quantified clauses explicitly, binding both subject
and predicate inputs. Ordinary goals must retain their existing fast path; do not
copy or rescan whole caller environments for each quantified application.

Keep universal proof, assumption instantiation and opaque reveal as distinct
operations. Scope-aware cache keys include the full environment, mode and binder
identity. Pending work retains its exact continuation and owned allocations.
Successful structural sharing never shares caller permissions. Traverse recursive
obligations iteratively with charged work and diagnosed limits; unfinished or
cycle-cutoff answers are not completed negative evidence.

Measure binder depth, quantified premise width, independent callers, GAT argument
count and repeated opaque instances separately. Use internal work/storage
invariants and immutable A/B binaries with A/A calibration, paired blocks,
retained observations and output equality. Follow `spec.md`, safe idiomatic Rust,
Clippy all/pedantic with warnings denied and the file/architecture audits.

## Completion and review

Develop all families in [tests/README.md](tests/README.md) alongside the maintainer
reference. Declare source, stage and native outcomes before generating oracles.
Unsupported-feature rejection of a negative control does not validate its intended
lifetime, capture or conformance rule. Pair positive capabilities with nearby
negative controls and preserve any independently qualified gaps.

Publish the semantic and lowering contracts, complete manifest, sidecars and
validation review together. Run every active earlier stage, strict lint, audits,
harness, upstream/backend checks, independent source/native checks and affected
scaling measurements. Keep planned-stage discovery failing until then. Student
export remains a later operation; this repository owns the full solution.
