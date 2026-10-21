# PA24 callable object contracts

This checkpoint admits the typed contracts of `dyn Fn`, `dyn FnMut` and
`dyn FnOnce`. PA24 remains planned. The subsequent
[dispatch checkpoint](pa24-erased-dispatch.md) implements function, pointer and
empty-environment targets. Captured-environment targets and full assignment
qualification remain required before publishing the complete LowIR contract.

Language call traits retain their own complete `Self`/argument application in
their default object-lifetime templates. Parenthesized object bounds use the
ordinary callable `Output` equality builder, including its input lifetime
eligibility checks. Dyn compatibility reads the completed language-trait header;
it does not fabricate a source owner or source method declaration.

Object context can supply a closure signature from the retained `Output`
equality, complete input tuple and object quantifier. This is an inference hint.
Ordinary trait selection, ownership and region proofs still validate the closure
and conversion. A quantified input does not grant a captured value permission
to escape or permit an unrelated result lifetime.

The declared outcomes in `pa24/tests/erased-callable-outcomes.json` contain
12 accepted and 11 rejected complete programs, each independently object-checked
with pinned rustc. Accepted cases check type and constant views. Negative cases
also check LowIR rejection; lifetime and ownership errors may legitimately be
deferred beyond the type view. Controls cover captured and contextual closures,
function items and pointers, higher-ranked inputs, inherited callable bounds,
generic consumers, upcasts, receiver permissions, output binders and escaping
or conflicting loans. No references were regenerated.

`dev/tests/erased_callables.rs` exercises all 23 declared programs. The focused
batch in `obj/parallel/pa24/dynamic-callables/focused-second/complete.json`
passes formatting, strict all-target/all-feature Clippy, the Rust file audit,
upstream/layout checks and 51 tests across eight affected source suites.
Sixteen explicitly ignored native tests in those existing suites were not part
of this batch. This checkpoint does not establish dynamic invocation execution
or a compile-time performance result.

An unannotated closure coerced through an inherited callable bound still needs
an inference-order repair. Its Rust-positive probe remains in the retained
development evidence; the annotated fixture is a separate declared case.
