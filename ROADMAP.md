# RustGM assignment roadmap

Maintainer reference implementation and course tests grow together. PA1–PA4
have working draft references and executable text-oracle suites; PA5–PA38 are planned.
No planned assignment is counted as passing.

| PA | Assignment | Status |
| --- | --- | --- |
| [1](pa1/README.md) | Source and tokens | implemented-draft |
| [2](pa2/README.md) | Literals | implemented-draft |
| [3](pa3/README.md) | Token trees and attributes | implemented-draft |
| [4](pa4/README.md) | Expressions and statements | implemented-draft |
| [5](pa5/README.md) | Items types and patterns | planned |
| [6](pa6/README.md) | Modules and names | planned |
| [7](pa7/README.md) | Macro matching | planned |
| [8](pa8/README.md) | Expansion and hygiene | planned |
| [9](pa9/README.md) | Types and inference | planned |
| [10](pa10/README.md) | Typed bodies | planned |
| [11](pa11/README.md) | Procedural LowIR | planned |
| [12](pa12/README.md) | Aggregates and layout | planned |
| [13](pa13/README.md) | Pattern semantics | planned |
| [14](pa14/README.md) | Moves and initialization | planned |
| [15](pa15/README.md) | Lifetime contracts | planned |
| [16](pa16/README.md) | Borrow checking | planned |
| [17](pa17/README.md) | Destruction | planned |
| [18](pa18/README.md) | Generics | planned |
| [19](pa19/README.md) | Traits and methods | planned |
| [20](pa20/README.md) | Trait solving | planned |
| [21](pa21/README.md) | Advanced generic relationships | planned |
| [22](pa22/README.md) | Unsized types | planned |
| [23](pa23/README.md) | Trait objects | planned |
| [24](pa24/README.md) | Closures | planned |
| [25](pa25/README.md) | Constant evaluation | planned |
| [26](pa26/README.md) | Language and library hooks | planned |
| [27](pa27/README.md) | Unsafe language | planned |
| [28](pa28/README.md) | Intrinsics and atomics | planned |
| [29](pa29/README.md) | Crates and metadata | planned |
| [30](pa30/README.md) | Foreign interfaces | planned |
| [31](pa31/README.md) | Panic and unwinding | planned |
| [32](pa32/README.md) | Async lowering | planned |
| [33](pa33/README.md) | Async ownership | planned |
| [34](pa34/README.md) | Procedural macro host | planned |
| [35](pa35/README.md) | Derive and attribute macros | planned |
| [36](pa36/README.md) | Edition and attribute closure | planned |
| [37](pa37/README.md) | Target-specific closure | planned |
| [38](pa38/README.md) | Frontend acceptance | planned |

Optional later assignments: PA39 pinned core compatibility; PA40 alloc/runtime;
PA41 required std and build ecosystem; PA42 reproducible self-hosting. These are
not prerequisites for the initial source-to-LowIR course.

PA37 requires an explicit capability audit: the current CPPGM backend does not
implement arbitrary Rust inline assembly. Backend extensions are separate
infrastructure work, not an implicit student prerequisite.
