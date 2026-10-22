# RustGM assignment roadmap

Maintainer reference implementation and course tests grow together. PA1–PA24
have working draft references and executable text-oracle suites; PA25–PA44 are
planned. No planned assignment is counted as passing.

The destination includes compiling original upstream core, alloc and std with
RustGM. Start real-crate acceptance as soon as core works; add alloc and std
as their consumers require them. The compiler supplies genuine language/intrinsic hooks;
ordinary library APIs come from their Rust sources. The
[library bootstrap plan](docs/library-bootstrap-roadmap.md) defines source pins,
acceptance gates, implementation priority and the old-to-new number mapping.
PA1–PA30 retain their current contracts and ownership.

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
| [21](pa21/README.md) | Advanced generic relationships | implemented-draft |
| [22](pa22/README.md) | Unsized types | implemented-draft |
| [23](pa23/README.md) | Trait objects | implemented-draft |
| [24](pa24/README.md) | Closures | implemented-draft |
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

Implementation priority is PA25–PA30 closure, generic cross-crate transport and
source-declared compiler bindings. Develop the Rust 2021/build pipeline alongside
core; exercise itoa/serde_core as soon as their core-only graph works. Then build
alloc and serde_json, followed by std and procedural derives. Develop
independent prerequisites in parallel while keeping cumulative releases honest.
Full unwinding and async follow the first usable abort-profile crate graph.

PA31–PA36 are required, not optional library extensions. An internal or target
feature needed by their actual source must work at that milestone. PA40/PA41
close broader edition/target coverage; they cannot excuse an earlier failed build.
CPPGM remains the explicit LowIR backend. Qualify required operations before use;
backend extensions are separate reviewed infrastructure work.

Optional PA45: reproducible self-hosting. This does not change the current goal
of completing PA1–PA30 or permit incomplete stages to be activated.
