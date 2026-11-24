# RustGM Programming Assignment 21 — Advanced generic relationships

Build on [PA20](../pa20/README.md) to check higher-ranked bounds, generic
associated types, opaque types, and generic destruction. Use Rust 1.98.0,
edition 2024, on Linux x86-64 with aborting panics, checked arithmetic,
and disabled debug assertions.

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

Use user-defined traits and scalar or reference programs to exercise these
rules. Later assignments cover trait objects, closures, and library iterators.
See the pinned
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
earlier stages. Check outlives requirements in the actual caller environment,
including for inputs erased by normalization.

Library hooks such as PhantomData and UnsafeCell, unstable `may_dangle` promises,
unsafe operations and trait-object destruction are separate responsibilities.
Do not require them implicitly in the first safe generic controls. The existing
[variance rules](../doc/rust-reference/src/subtyping.md) remain authoritative for
the supported constructors.

## Command-line interface

```text
rustgm --emit-generic-semantics [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
rustgm --emit-lowir [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Use PA11's input and output options. The inspection command reports
retained typed facts. Keep PA19 and PA20's existing inspection formats.
Use the complete proof and lowering pipeline in LowIR mode.

## Output format

Begin the inspection with `generic-semantics 1`. Use the following records.

- First come PA19's type, definition, owner, parameter, predicate, implementation,
  dispatch and pending lifetime records, then PA20's associated member, constant
  and normalization records. Quantified predicates use `forall=<quantifier-id>`
  in place of an ordinary predicate. Type rows expose opaque identities with
  their complete mixed capture arguments and projections with separate family
  arguments. All identifiers retain their typed arena identity.
- `quantifier <id> regions=<count> <predicate>` records each canonical quantified
  clause in arena order. Bound lifetime spelling follows the existing type view;
  binder depth and slot distinguish local names from enclosing captures.
- `family def=<id> captures=<count> regions=<count> complete=<bool>` introduces
  a family, sorted by definition identity. Its `family-bound def=<id> index=<n>
  <clause>` and `family-region def=<id> index=<n> longer=<region>
  shorter=<region>` rows retain contract order. Parent captures and total
  lifetime slots are separate counts; ordinary parameter rows supply type and
  const parameter identities and kinds.
- `opaque <id> owner=<def> source=<file>:<node> application=<type> hidden=<type|->
  role=<trait-result|defining-result> storage-proof=pending` records each opaque
  declaration in identity order. `opaque-bound id=<id> index=<n> <clause>` keeps
  the published bound order. The application type contains exactly the selected
  edition-2024 or precise captures. A retained hidden type is private defining
  evidence, not permission for a caller to equate it with the public identity.
- `opaque-body-result def=<id> type=<type>` records the private body result,
  sorted by defining function. `opaque-implementation owner=<impl> id=<opaque>
  type=<type> parent-regions=<count> regions=[<region>,...]` records a proved
  trait-method override template, sorted by impl owner and opaque identity.
  Its ordered region map translates trait method slots into template coordinates.
- `proof <id> body=<file>:<node> source=<file>:<node> pending` introduces a
  retained proof in insertion order. Its ordered `proof-source`, `proof-scope`,
  `proof-owned`, `proof-block`, `proof-relation` and `proof-type` rows preserve
  source indices, parent scopes, universal counts, scope depths, half-open owned
  lifetime intervals, restoration blocks, equalities/outlives and type-outlives
  debts. Scope zero is the body root; nested scopes have explicit parents.
  Block `aliases` and `source` flags describe restoration, not new assumptions.
- `eof <counts>` ends the full view. Counts extend PA20's totals in this order:
  `quantifiers`, `families`, `family-bounds`, `family-regions`, `opaques`,
  `opaque-bounds`, `hidden-types`, `opaque-body-results`, `opaque-implementations`,
  `proofs`, `proof-scopes`, `proof-blocks`, `proof-sources`, `proof-owned`,
  `proof-relations`, `proof-types`. Each counts its matching records;
  `hidden-types` counts opaque records with a non-dash hidden value.

Numbers are decimal, booleans are `true`/`false`, and list commas have no spaces.
Order the records deterministically. The `pending` marker denotes an obligation
that the complete region, loan, or storage proof must still check before LowIR.

The supplemental `--emit-lowir` lane proves complete region, ownership,
destruction and layout obligations before emitting the existing LowIR contract.
Build and run executable examples through CPPGM using the PA11 procedure.
Implement Rust frontend phases within your compiler. Success exits 0 with newline-terminated stdout and empty stderr. Diagnosed
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

When testing efficiency, vary binder depth, the number of quantified
premises, the number of independent callers, and repeated generic or
opaque applications.

## Run the tests

Run the PA21 test target from the repository root.

```sh
make -C pa21 test
```

[The manifest](tests/manifest.json) selects generic inspection and
executable LowIR cases. Pair successful cases with nearby failures to
check that the compiler enforced the intended lifetime, capture, or
conformance rule. Use the
[PA11 backend procedure](../pa11/README.md#run-the-tests-and-an-executable)
to run executable examples.
