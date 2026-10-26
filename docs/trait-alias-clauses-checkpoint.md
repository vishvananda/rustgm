# Canonical local trait alias clauses

This qualified partial maintainer checkpoint implements actual
local canonical alias headers and a bounded ordinary-bound clause consumer.
Complete trait alias semantics, LIB-031, and the original core artifact remain
open. The public declaration pipeline still rejects alias execution/export.

`declarations/aliases.rs` owns a cold optional table in the declaration checker.
Ordinary programs allocate no table. Each actual resolved alias has a separate
rigid Self, full declared lifetime/type/const slots, its own schema, direct typed
clauses, and typed applications of other aliases. The alias owner installs its
schema before the shared Self producer publishes completion. The associated
inventory does not own alias Self: no associated trait schema, impl entry,
vtable principal, or synthetic Self:Alias predicate is created.

The existing declaration worklist prepares actual trait arguments and associated
equalities. Canonical alias headers retain dependency edges instead of copying
ancestor clauses into every declaration. One iterative graph traversal checks
all owners and edges before sealing. An empty contract is explicitly complete;
an unfinished or failed declaration cannot restart as an empty successful one.

The ordinary-bound consumer substitutes the complete subject and every argument
through the existing typed folder, including unused formal slots, captured
lifetimes, const arguments, and associated equalities. A per-use visited set
includes the actual alias declaration, subject, and full arguments. It belongs
to the same checker/Store and one substitution context; it caches no proof
answer. Identical shared applications are visited once; applications with
different arguments remain distinct. The shared argument-list folder now has a
charged entrypoint rather than another structural type walker.

Written Self uses the same lexical schema capture routine as ordinary traits
and impls. Alias lookup authenticates its actual resolved item and Self identities
and completed canonical type. This shares capture mechanics without granting an
alias an associated owner. An existing captured-owner trait-conformance control
protects the ordinary route.

Five actual-source groups cover full unused formals, independent unsized Self,
real equality clauses, complete substitution, distinct argument environments,
the real function-bound worklist, unused arity errors, cycles, empty aliases,
retained quantifiers, failed restart, and repeated shared diamonds. The latter
uses widths8/32/128 plus an ordinary no-alias control. Counters measure logical
graph/substitution work and cumulative storage charges, not physical heap size
or wall-time speed. The existing bounds producer retains its ordinary work
accounting; complete alias-stage retention accounting remains required.

The focused combined gate also selects the existing alias identity/authority
guard, shared Self formals and captured-owner conformance, three fresh-consumer
registration controls, and three source panic/abort declaration controls. It
runs both strict Clippy modes and the global architecture audit before building
or executing tests. No generated references or upstream files are changed.

The following consumers remain closed or incomplete:

- Nested alias applications beneath their own quantifiers need full binder
  composition. Direct quantified leaf clauses are retained; nested binders are
  rejected rather than discarded.
- Lifetime-to-lifetime schema relations need a scoped region consumer. Complete
  original/normalized projection views, shorthand associated lookup, object
  principal selection, and associated constraints on alias uses remain required.
- Unused default well-formedness, complete const qualification, and all source
  sizing defaults need their actual declaration consumers and opposing tests.
- Imported aliases need genuine identity/schema/contract transport. Existing
  public Provided, defaultness, restricted-policy, and wire guards stay intact.

The pinned compiler supplies a specific sizing requirement for the next step:
`rustc_hir_analysis/src/collect/predicates_of.rs` routes both Trait and TraitAlias
through `add_implicit_sizedness_bounds` with `TraitDef`. In
`hir_ty_lowering/bounds.rs`, a trait or alias without explicit sizing bounds gets
the actual MetaSized default; explicit PointeeSized suppresses that default.
These are the original sources at revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`. Unsized Self does not establish
absence of MetaSized. The current alias Self sizing consumer remains open;
the execution/export fence prevents this partial checkpoint granting permission.

The earlier independent alias observations and their wrong predictions remain
in the [contract checkpoint](trait-alias-contracts-checkpoint.md) and
[design](trait-alias-design.md). This checkpoint does not rerun their reference
matrix, claim native alias execution, or count opened core files as compiled
files.

[Combined run40265767585](https://github.com/vishvananda/rustgm/actions/runs/40265767585)
passes all14 selected groups, strict production/private Clippy, formatting,
architecture, harness and upstream checks. The helper takes174.59seconds;
the actual tests take0.11seconds. All2,711 source fingerprints match before
and after execution and the final imported host formatting. Receipt SHA is
`cfb9b6741185f4e58e182bd15574b34477ec26287d1f2f65c5319d5b48f87c64`;
binding SHA is
`4454978b05e59c2ec58131982988ad7e569e6d59a83164ea0aaef02d07278cfc`.
There is no fresh CLI, native, release, course-text or whole-core sweep.

| Shared width | Counted steps | Cumulative logical charged bytes |
| --- | --- | --- |
| 8 | 394 | 12,389 |
| 32 | 1,574 | 51,133 |
| 128 | 6,246 | 202,269 |

Observed layouts are Checker10,232, cold table80, canonical header104 and
application edge28 bytes. These are current containing layouts, not inferred
before/after differences. The imported-registration controls retain51,000 steps
for1,000 warm queries at unrelated widths1/8/32, with no warm retained growth.
The new source-role reservation adds128 logical cold bytes to those component
measurements; it does not add a bootstrap Abort factory or execution permission.

Failed attempts remain evidence: the first collected both Clippy function-size
findings without tests; the next passed static/build but rejected an incorrect
logical test selector before execution. The following13-case run passed9 and
exposed missing alias kinds in the shared formal collector. Its successor
passed12, including all five new groups, and exposed the ordinary-only Self
reader. The final checkpoint shares the real lexical capture consumer and adds
the existing captured-owner conformance control. No expectation was weakened,
reference regenerated, or lint/audit exception added.
