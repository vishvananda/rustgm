# PA20 fixture ownership

The manifest registers 1,343 exact text-oracle cases from 797 source programs.
The semantic lane has 484 accepted and 62 rejected cases; the LowIR lane has
483 accepted native executions and 314 rejected programs. One semantic-positive
fixture deliberately retains a lifetime error rejected by LowIR and rustc.

Fixture families:

- `projections`: qualified/shorthand paths, complete type/lifetime/const inputs,
  aliases, nominal fields, callable signatures, nested results and repeated uses.
  Include method-local trait Self bounds versus trait-impl Self lookup, reordered
  projected where-subjects, duplicate versus distinct trait applications, captured
  callable lifetimes, alias proof isolation and generic-default rejection.
- `contracts`: result bounds, equalities, inherited promises, valid/invalid impl
  conformance, unused declarations and obligations retained after normalization.
- `constants`: required/default/overridden values, inherent values, generic
  instantiation, selected identity, type errors, invalid values and dependency cycles;
  selected patterns/ranges, structural equality, coverage, unused owner parameters,
  parameter-dependent values and per-caller lifetime/bound obligations; enum
  discriminants, their enclosing-generic capture restrictions and separate
  declaration-time const-argument dependencies, nested arguments and default/
  override declaration-order permutations; selected initializer readiness without
  false cycles through overridden defaults, and invalid unused initializer bodies.
- `selection`: caller assumptions, partial inference, result/argument context,
  unrelated opaque alternatives, ambiguous methods and distinct caller scopes.
- `coherence`: local/foreign identity within the supported profile, covered
  parameters, overlapping and disjoint headers, conditional impls and unused impls.
- `recursion`: successful shared subgoals, inductive cycles, failure-cache isolation,
  retry after inference and query-budget diagnostics.
- `integration`: selected native behavior, fields/layout, borrowing, lifetime
  debts, destruction and constants used by initializers or executable expressions.

Each family includes both semantic inspection and complete LowIR proof cases where
applicable. Declare source acceptance, any later-proof rejection and native
outcomes independently; do not infer them from a generated text reference.
Pair each positive capability with nearby negative controls. Preserve imported
source attribution and source hashes. Do not require PA21 GATs, opaque types or
higher-ranked trait bounds implicitly in a PA20 fixture.

Put required regressions here, in the earliest assignment owning all their
features. Register every input in `manifest.json`; no undiscovered fixtures.

All sources are original RustGM controls; `maintainer_origin` records promoted
tests. Positive programs have separate semantic and LowIR identities so each
mode has its own exact text oracle. No accepted LowIR fixture is skipped or
classified as compilation-only. Three unused-fault fixtures explicitly allow
`unconditional_panic`, preserving their original maintainer wrapper policy while
keeping other default lints enabled. Two originally identically named selection
controls have distinct paths and preserve both source programs.

| Family | Semantic | LowIR | Native executions |
| --- | ---: | ---: | ---: |
| coherence | 14 | 14 | 5 |
| constants | 168 | 261 | 156 |
| contracts | 119 | 167 | 97 |
| integration | 41 | 57 | 39 |
| projections | 77 | 108 | 69 |
| recursion | 27 | 44 | 25 |
| selection | 100 | 146 | 92 |

The maintainer invariants additionally exercise solver budgets, completed proof
reuse, failure-cache isolation, suspended-query ownership and linear work/storage
bounds. These internal algorithm checks supplement the public language contract;
work counters never appear in text oracles.
