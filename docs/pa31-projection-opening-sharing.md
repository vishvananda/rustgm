# Shared structural data for projection opening

This checkpoint starts from combined `5602dc27`. Complete source lowering of
the unchanged `opening-subjects` inputs previously grew from 54,813,625 to
550,412,747 instructions between 32 and 128 subjects. Opening the caller
accounted for 73% of the largest observation. The first identity-only draft
passed its functional checks but still grew 9.137 times, above the declared
limit of 6. Both failures, their inputs and all observations remain retained.

The final implementation shares canonical caller clauses and their index,
then maps only premises actually consumed by each fresh projection query.
The unchanged optimized nine-row gate passes: subject growth is 3.040 times
from 8 to 32 and 3.732 times from 32 to 128. The largest input uses
103,096,127 instructions, 81.27% fewer than the immutable baseline. All nine
output pairs are byte equal and all sixteen native contexts pass. This is
focused qualification; the integrator owns combined release coverage.

## Structural ownership and proof boundaries

`normal_form::opening::prepare` still opens the complete requested predicate
and family arguments in a new fold. Each query owns its State, frontier,
obligations and restoration equations. No selected evidence, caller permission,
equality discharge or fresh-region result is shared between queries.

A cold summary authenticates the exact immutable Environment Arc and type
arena. Its first binder-aware inspection covers ordinary abilities and Sized
subjects, builtin clauses, ordinary and quantified trait clauses, projection
equalities, and both sides of lifetime equivalences. Parameter abilities,
const identities and projection builtin promises contain no region-bearing
payload. Failed inspections are charged and never cached. Trait mutation and
substitution invalidate the summary beside the ClauseIndex. A summary occupied
by another cloned or forked owner deliberately falls back; it does not cache
or authenticate that owner.

For an identity opening, the existing Environment Arc and index are retained.
Every distinct free lifetime is still replayed through the fresh fold, retaining
restoration equations and rejecting an ID at or above the fresh interval. The
inspection/replay bound is O(C + Q*R): C is the initial structural inspection,
Q the number of queries, and R the distinct free lifetimes. Combined Q/R work
remains Q*R. Existing per-query sorting, restoration and proof work remain.

For captured openings, the bounded view profile requires nonquantified trait
clauses and no region-bearing builtin, ability, equality or equivalence
partition. All captures must already be mapped by the actual request fold.
The view retains canonical clauses/index and the complete query-local region
mapping. Its source Environment, arena and actual base-index identity are
checked before publication. The actual index head excludes lifetimes and
child arguments, so its buckets remain valid under this mapping. Each consumed
predicate is mapped with charged attempted work and allocation. Canonical
storage grants no proof. Unsupported forms keep complete eager substitution.

The view survives Environment::trait_index and child Proof::new/State::new;
it cannot silently become an empty assumption set. General candidate choice
still uses isolated trials and preserves ambiguity and deferral. The existing
formed-nominal exact-membership path compares the complete mapped predicate
with charged work; it does not broaden general inference equivalence.

Full-clause source consumers now use a fallible iterator. Claims, defining-call
checks and transported source rows authenticate their actual Generics/Template
environment. Body assumptions and method-prefix collection also consume source
state. Dispatch explicitly substitutes before iterating, which materializes a
view with charged work. The iterator error guards internal misuse; a valid
consumer has an explicit source or materialization route. Views remain inside
normal-form/selection state and never enter Body, Contract or exported data.

## Qualification and measured storage

The final source is `1b1983243821ebba3bd886c39c47fea67d05780e`. Fresh
formatting, architecture audit and all-target strict Clippy pass. A single
private binary passes all 1,445 nonignored private tests once, thirteen focused
frontend cases, and thirty-one actual native programs. Harness and upstream
checks pass. Controls cover distinct mappings, child proofs, later non-Open
materialization, full arguments, ambiguity, pending projections, foreign arena
and base indexes, failed work, tiny budgets and eager fallback. Existing seven
full-source inputs and independent oracle receipts are unchanged.

Containing sizes measured from the actual before/after debug binaries:

| Record | Before | After |
| --- | ---: | ---: |
| Environment | 64 | 64 |
| State | 800 | 800 |
| Body | 448 | 448 |
| Contract | 184 | 184 |
| Assumptions | 8 | 8 |
| Cold TraitAssumptions | 176 | 184 |
| Cold ClauseIndex | 112 | 120 |
| Cold Retained accounting | 2 | 16 |
| Cold summary | 48 | 72 |
| New query Opening | — | 240 |
| New mapped-clause owner | — | 16 |

The measurements compare the preceding identity-only draft with the final view.
Retained accounting includes the weak owner, arena identity, shared base and
query mapping/cache allocations. Ordinary closed callers allocate no summary
or arena token. Private counters independently vary clause width, free-region
count and query count, including combined dimensions. They are separate from
the complete-source instruction result.

A bounded no-build observation of the actual qualified private binary found
867 mapped calls and 867 successful mapped openings, with no observer errors
or truncation. All actual captured callers met the declared trait-only profile.
This establishes use of the changed path, independently of timing.

The optimized comparison uses release opt-level 3/debug-info 1, the exact
source-lowering adapter, immutable `5602dc27` binaries, frozen seven opening
inputs and ordinary Copy128/macro controls. It retains A/A calibration, paired
ABBA observations, Callgrind instructions and RSS. No instruction, RSS or
growth trigger fires. Subjects128 wall time falls 75.56%; RSS falls from
16,998 to 13,366 KiB. Ordinary Copy128 and macro instruction changes are
-0.163% and -0.018%, respectively. The original 6-times growth threshold is
unchanged; rejected baselines cannot become speedups.

Evidence roots in the worker checkout:

- `obj/parallel/pa31/captured-opening-focused-v1/`: functional inventory,
  actual-native-programs.json, layouts and source snapshot.
- `obj/parallel/pa31/captured-opening-actual-use-v1/`: bounded actual-use
  observation and exact compiler/source bindings.
- `obj/parallel/pa31/captured-opening-optimized-v1/`: immutable binaries,
  declaration, all raw measurements, equality/native preflights and analysis.
- `obj/parallel/pa31/projection-opening-optimized-v2/`: preceding 9.137-times
  failure, retained separately from the successful final implementation.
