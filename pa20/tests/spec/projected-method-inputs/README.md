# Projected properties in method lookup

This packet has 27 ordinary PA20 inputs: 24 accept and three reject. All outcomes
were independently checked with Rust 1.98.0 object emission, edition 2024, Linux
x86-64, panic abort, overflow checks enabled and debug assertions disabled.
`independent-review.json` records the first 21 cases; `scaling-plan.json` adds six
width/noise cases declared before their execution. Every input is registered in
the owning assignment manifest with its full LowIR outcome.

The minimum is `Wrap<R::Out>.get()` under `R: Family`. The plain and conditional
Copy wrappers, concrete outputs, explicit calls and caller-paid Clone conditions
separate receiver shape from the actual method requirements. Projected built-in
conditions now use the existing non-refining trait selector when the structural
possibility query requests semantic work. That selector retains caller premises,
projection formation, scopes and pending declarations. The chosen callable still
proves its complete bounds, lifetimes and effects.

The priority cases retain both an inherent method requiring `I::Out: Copy` and a
lexical trait method. Generic callers without the bound select the trait body;
paid callers and a suitable concrete output select the inherent body. Rust also
accepts `Hold::<I>::choose` through the lexical fallback. The initial prediction
that this syntax must reject was disproved independently; its original bytes and
prediction remain under `obj/parallel/projected-properties/opponent-*`. Separate
`*-no-fallback` programs reject the genuinely unpaid Copy obligation.

`priority-native.rs` executes six independently declared values:
41, 41, 23, 23, 41 and 31. Rust and CPPGM both pass. The remaining 23 positives are
unused generic/concrete body compilation contracts. Width 8, 32 and 128 cases
bound probes, method steps, inference and trait selections by fourfold growth
plus 2048; adding 128 unrelated nominal declarations leaves those counters equal.

Full LowIR checks exposed an existing referent-field preparation error in five
explicit-call controls. The ordinary minimal forms belong to
[PA18](../../../../pa18/tests/spec/borrowed-place-fields/README.md). All five
remain acceptance requirements and now pass through LowIR. Actual source slice
methods belong to [PA31](../../../../pa31/tests/spec/projected-method-inputs/README.md).
Whole original core/alloc/std artifacts remain required.
