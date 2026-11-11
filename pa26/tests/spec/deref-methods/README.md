# Ordinary Deref method requirements

These seven authored reductions retain six valid ordinary programs and one
unpaid-bound rejection, all independently checked with pinned Rust objects.
General Deref library/operator hooks belong to PA26; earlier PA19 receiver lookup
requires built-in reference dereferencing. The unsized-tail variants also consume
the already active PA22 foundation. Actual source-role counterparts stay in
[PA31](../../../../pa31/tests/spec/deref-methods/README.md).

Current bootstrap lookup does not expose core::ops::Deref. The real ManuallyDrop
case reaches its separate missing method boundary. Those setup failures do not
qualify the rejection opponent; all valid cases remain required positives.
Direct/explicit inner controls are retained in the raw discovery evidence.

The manifest declares every source and expected outcome. The generated
independent receipt binds byte-identical observed inputs and current compiler
limitations. No course oracle, executable comparison or whole-PA activation is
claimed. See [the reduction review](../../../../docs/pa26-deref-method-reductions.md).
