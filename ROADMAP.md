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
publication order. It has37 phase units,9 implemented finite components and
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
