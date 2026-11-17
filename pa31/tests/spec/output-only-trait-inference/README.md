# Output-only associated-result inference

These three authored no_core reductions keep the independently declared outcomes
`0/0/1`: inferred input and explicit `u32` must type; explicit `i32` into
`Sink<u32>` must reject. `outcomes.json` binds every source and keeps the original
oracle declaration and failed candidate observation. Source bytes match the
original tiny reduction exactly.

`Map<I, F>` implements `HasItem` with output-only `B`, constrained by
`F: Call<I::Item, Output = B>`. `Identity` implements `Call<T, Output = T>` and
`Range<T>` requires one of three real `MiniStep` impls. This isolates the output
constraint without builtin `Fn`, actual closures, arithmetic or real core/alloc
inputs. The source foundation comes from the first18 lines of the existing
scalar-niche-suffixes fixture, plus the actual `i32` Clone/Copy impls; its retained
comments do not change this packet's trait-inference contract.

Pinned rustc88d9e12a accepts both positives and rejects the closed mismatch with
`E0277: collecting i32`. Immutable RustGM b77d2e4e instead rejects the inferred
positive at byte1856 with `collecting i32`, while the typed positive accepts.
The closed negative rejects at its call, but its message hits an independently
retained typed-shape formatting limit. This is semantic evidence only; no
lowering, backend or native claim is made.

The existing maintainer traits root's `winnowed_projections::output_parameters`
submodule requires pipeline acceptance and actual literal `u32` facts for each
positive, and a semantic error at the closed mismatch's call. It does not call
rustc to implement or validate RustGM semantics. The local raw evidence and
receipt remain under `obj/parallel/vec-method-lookup/tiny-map-output/`;
The production fix passes all three source outcomes and nine private alias
safety and bounded-work controls (fastcheck receipt `1794901714131978364`).
The four pre-existing projection controls and an ordinary occurrence-check
control pass in that same run. The original real-core/alloc Vec replay and
native observations remain pending.
