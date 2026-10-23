# Source macro lookup through ordinary import dependencies

These authored controls isolate an observed whole-core failure at
`library/core/src/prelude/v1.rs`: a `crate::option_env` import tried to create
the bootstrap Option library solely from its final spelling. Source-provider
mode must instead resolve actual declarations, including forward imports and
aliases, through the existing macro/import dependency worklist.

The six sources were frozen before independent execution. All twelve pinned
Rust 1.98.0 metadata/object observations match the declared four acceptances
and two rejections. There is no native claim. The original contract and the
pre-fix compiler observations remain unchanged.

`contract-v2.json` corrects the candidate view: the earlier PA names view does
not expand every macro call and therefore cannot qualify missing-call semantics.
Candidate checks use complete expansion and resolution, as does `--emit-types`.
Positive cases must retain actual source macros without a synthetic library,
Drop trait or builtin macro prelude. Negative cases require the intended
unresolved-import or unresolved-macro diagnostic.

This correction does not implement a source `option_env!` builtin's Option
result, authenticate a compiler builtin by spelling, or establish a whole-core
build. The original independently observed outcomes and failed whole-core
attempt are under `obj/parallel/source-macro-paths-independent-v1` and
`obj/parallel/library-stability-whole-core-frontier-v1`. Combined candidate
qualification remains pending; PA31 stays planned.
