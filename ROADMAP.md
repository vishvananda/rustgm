# RustGM assignment roadmap

PA1–PA24 have active, independently qualified text-oracle suites. PA21–PA24 are
**implemented**, with their completed release evidence recorded in
assignments.json. PA1–PA20 retain implemented-draft while individual release
records are reconciled. PA25–PA44 remain planned as whole assignments. No
planned or empty suite counts as passing.

The [adopted assignment-order review](docs/assignment-order.md) replaces the old
numeric work/release chain for incomplete assignments. Keep PA numbers and
fixture paths as ownership IDs; use the machine-checked
[dependency plan](assignment-plan.json) for actual implementation and component
publication order. It has37 phase units,10 implemented finite components and
46 explicitly retained PA25–PA30 requirement families. All component publications
are still pending. Whole-PA completion requires its entire handout and release
checks, not only one working subset.

Source-declaration binding precedes source consumers; runtime memory precedes
symbolic CTFE; declaration metadata precedes body/value transport; required
build/edition/SIMD/assembly slices precede original core. Core-only real crates
start before alloc, followed by alloc consumers and std. Broader closure and
every valid unsupported requirement remain named work. The
[library bootstrap plan](docs/library-bootstrap-roadmap.md) retains original
source pins and the full acceptance ladder; the
[live tracker](docs/library-build-blockers.md) owns concrete failures.

The [core census and transport plan](docs/core-census-plan.md), adopted
2026-11-07, is the current execution path: repair LIB-193, add an analysis-only
keep-going census of original core, and develop branching body transport and
the alloc/std build lanes in parallel. The backend stays frozen. Assignment
requirements and status are unchanged.

The [projected callback-result checkpoint](docs/pa24-projected-callback-results.md)
qualifies83 complete outcomes from84 frozen ordinary/source/native contracts,
including projected array payloads, generic method-value correlations, nonempty
lifetime proof and source native order23/12. One ordinary native artifact and eight override contrasts
remain required. This finite checkpoint does not activate a planned assignment
or establish an original core artifact; the bounded original probe is next.

The [wrapped callback-premise checkpoint](docs/pa24-wrapped-callback-premises.md)
qualifies55 complete outcomes and four additional source type outcomes from68
independent contracts, including source native order23/1234. Thirteen accepting
artifacts remain required. The next unchanged original observation is pending;
no original core artifact or planned-assignment completion is claimed.

The [try-capture checkpoint](docs/pa24-try-captures.md) adds value-use traversal
for captured try blocks,21 complete source outcomes, actual native7/43/23/123 and
one-field scaling. Fifteen ordinary contracts and all earlier requirements remain
open. Preceding CI's16 existing text contracts are independently reviewed and
updated without changing input or acceptance outcomes. The unchanged original
observer clears filter_try_fold and first reaches filter_map_fold at its match
expression. All pinned inputs/configuration/limits verify; no original artifact yet.

The [computed-match capture checkpoint](docs/pa24-match-captures.md) qualifies
40 complete API outcomes from49 independent contracts,19 new PA24 text outcomes,
actual ordinary/source native order7/9/12314 and one-field scaling. Nine entire
positive outcomes and one additional public-helper metadata artifact remain
required. The unchanged original observer clears all three filter-map capture
bodies; first remaining is FlattenCompat::iter_fold at its nested helper call.
Pinned inputs/configuration/limits verify; full successor qualification remains
pending. No original artifact or planned-assignment completion is claimed.

The [helper return-input checkpoint](docs/pa24-flatten-helper-inputs.md) relates
opaque helper callable inputs through published bounds:46 complete API outcomes
from52 independent contracts,23 new PA24 text outcomes, actual ordinary/source
native7/9/1234 and call-linear bounded work. Six entire contracts remain required.
The unchanged original observer clears iter_fold and its five fold/rfold siblings;
first remaining is iter_try_fold's projected helper input. Full 43171710575 passed.

The [helper return-output checkpoint](docs/pa24-flatten-helper-outputs.md) exposes
opaque helper callable outputs after their declared inputs settle. It covers 26
complete API outcomes from 27 independent contracts, 15 new PA24 text outcomes
and observer-linked native 7/9/1234. One borrowed-receiver executable stays
required at the existing opaque-storage gate. An exploratory observation clears
iter_try_fold, iter_try_rfold and four try_fold/try_rfold callers. The first
remaining failure is map_while try_fold's method candidate graph. A clean pin and
the full successor are pending.

The PA25 [const-machine component](docs/pa25-const-machine-release.md) is the first
released work unit: 113 declared outcomes, including four new recursion/work/storage
limit controls, pass text, independent Rust and native checks with frozen a08ab1a4.
Full 43177635950 passed on its release documentation commit. PA25 and every other
parent assignment remain planned.

| PA | Assignment | Status |
| --- | --- | --- |
| [1](pa1/README.md) | Source and tokens | implemented-draft |
| [2](pa2/README.md) | Literals | implemented-draft |
| [3](pa3/README.md) | Token trees and attributes | implemented-draft |
| [4](pa4/README.md) | Expressions and statements | implemented-draft |
| [5](pa5/README.md) | Items types and patterns | implemented-draft |
| [6](pa6/README.md) | Modules and names | implemented-draft |
| [7](pa7/README.md) | Macro matching | implemented-draft |
| [8](pa8/README.md) | Expansion and hygiene | implemented-draft |
| [9](pa9/README.md) | Types and inference | implemented-draft |
| [10](pa10/README.md) | Typed bodies | implemented-draft |
| [11](pa11/README.md) | Procedural LowIR | implemented-draft |
| [12](pa12/README.md) | Aggregates and layout | implemented-draft |
| [13](pa13/README.md) | Pattern semantics | implemented-draft |
| [14](pa14/README.md) | Moves and initialization | implemented-draft |
| [15](pa15/README.md) | Lifetime contracts | implemented-draft |
| [16](pa16/README.md) | Borrow checking | implemented-draft |
| [17](pa17/README.md) | Destruction | implemented-draft |
| [18](pa18/README.md) | Generics | implemented-draft |
| [19](pa19/README.md) | Traits and methods | implemented-draft |
| [20](pa20/README.md) | Trait solving | implemented-draft |
| [21](pa21/README.md) | Advanced generic relationships | implemented |
| [22](pa22/README.md) | Unsized types | implemented |
| [23](pa23/README.md) | Trait objects | implemented |
| [24](pa24/README.md) | Closures | implemented |
| [25](pa25/README.md) | Constant evaluation | planned |
| [26](pa26/README.md) | Language and library hooks | planned |
| [27](pa27/README.md) | Unsafe language | planned |
| [28](pa28/README.md) | Intrinsics and atomics | planned |
| [29](pa29/README.md) | Crates and metadata | planned |
| [30](pa30/README.md) | Foreign interfaces | planned |
| [31](pa31/README.md) | Compiler/library bindings | planned |
| [32](pa32/README.md) | Pinned core bootstrap | planned |
| [33](pa33/README.md) | Allocator and alloc bootstrap | planned |
| [34](pa34/README.md) | Edition and crate build integration | planned |
| [35](pa35/README.md) | Upstream crate acceptance | planned |
| [36](pa36/README.md) | Std and platform bootstrap | planned |
| [37](pa37/README.md) | Procedural macro host | planned |
| [38](pa38/README.md) | Derive and attribute macros | planned |
| [39](pa39/README.md) | Panic and unwinding | planned |
| [40](pa40/README.md) | Edition and attribute closure | planned |
| [41](pa41/README.md) | Target-specific closure | planned |
| [42](pa42/README.md) | Async lowering | planned |
| [43](pa43/README.md) | Async ownership | planned |
| [44](pa44/README.md) | Frontend acceptance | planned |

The goal remains all assignment requirements needed to build original pinned
core, alloc and std and execute independent real-crate consumers. Actual active
source determines required language/runtime/target capabilities; a later number
never excuses a failed library path. Full unwinding and async remain separate
from the initial abort-profile graph. Optional PA45 reproducible self-hosting
remains outside this goal. CPPGM is the explicit LowIR backend; backend extensions
require separate reviewed capability work.

The [overflow/automatic/CI checkpoint](docs/parallel/overflow-auto-ci-main-join.md)
clears the authenticated const-closure overflow attribute frontier. The actual
joined CLI now reaches LIB189, a cold closure in str/pattern.rs1858,19.5022s with
all722 inputs/profile/limits unchanged. Nine focused controls, strict selected
Clippy, architecture and282 harness checks pass. Full successor CI, genuine
const-closure callable/capture/CTFE semantics, Field consumers and all original
library artifact gates remain required; no new whole assignment is complete.

The [actual Field operation checkpoint](docs/parallel/field-proof-projection-checkpoint.md)
qualifies builtin proof, Base/Type, generic/move/lifetime constraints and actual
native projection/callback values with11 focused controls and strict metadata.
Provided OFFSET/intrinsic/layout execution, full reflection and transport stay
required. Whole-PA statuses and original core/alloc/std artifact gates are unchanged.

The [provided-default join](docs/parallel/field-provided-default.md) executes the
actual alternative Field constant and qualifies cold positions and const closure
creation/originals. The [source-header allocation repair](docs/parallel/source-header-allocation.md)
clears the measured local byte failure with exact owner buffers and real-row
accounting. The unchanged-source probe now reaches ptr_guaranteed_cmp declaration
policy in18.1196s, with no limit increase. The [PA20 oracle profile repair](docs/parallel/pa20-oracle-profile-repair.md)
preserves all oracle bytes and passes complete independent checking. Field-offset
intrinsic/layout, actual captured const execution and original library artifacts
remain required; whole-PA statuses are unchanged.

The [complete Field offset/native checkpoint](docs/parallel/field-offset-execution.md)
executes the actual provided initializer and reached intrinsic/layout, preserving
ordinary/generic/lifetime proof. Its full original native observer, eight private
controls and15 regular/scaling/initializer checks pass. Full reflection/field_of,
transport and original library/whole-assignment gates remain required.

The [captured closure/pointer integration](docs/parallel/captured-pointer-main-join.md)
qualifies actual shared/mutable/owned const calls and existing-component reuse,
pointer CTFE and real native fallback bodies together with Field. Six private,
28 regular and6 native controls pass; the unchanged original-core source now
reaches LIB192 staged deprecation in19.1083s. Conditional/generic caller proof,
receiver adapters, bootstrap and authenticated transport remain required. No
whole-assignment status or original library artifact gate changes.

The [cold/deprecation integration](docs/parallel/cold-deprecation-main-join.md)
qualifies actual prototype owner binding and staged release semantics, clearing
LIB192 to an unlocated shared work-budget guard193. Four private,23 regular and
six native controls pass on the same compiler snapshot; no limit/source change
or original artifact is claimed. Assignment status is unchanged. Root handoff.md
records the user-requested pause and exact restart boundary.
