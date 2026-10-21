# Receiver binding permissions in captures

PA24 remains planned. Capture environments now preserve the distinction between
an immutable `&'a mut self` binding and its mutable referent. An explicit-lifetime
receiver syntax node's `mut` describes the referent; only an actual mutable
binding grants replacement permission. This matches the existing declaration
completion rule and adds no receiver-table scan per capture.

`pa24/tests/receiver-binding-outcomes.json` declares five source controls:
explicit and elided borrowed receivers, mutable owned and explicitly typed
receivers, and rejection of replacing an immutable borrowed receiver. All five
match pinned rustc object compilation, and a semantic invariant checks the
retained binding authority for each. Full executable permission checks await
the capture-runtime integration; successful semantic inference is not claimed
as rejection of the invalid assignment.

Evidence is in `obj/parallel/receiver-binding/gate-v2/`: the source invariant,
strict release all-target/all-feature Clippy, formatting, file audit, layout and
upstream checks all pass. The initial self-assignment controls unexpectedly
triggered rustc E0521 because implicit reborrowing would escape the closure.
Their original bytes and diagnostics remain in `revision1-inputs/`,
`revision2-inputs/`, `gate-v1/` and `rustc-v2/`. The separate eight-case
`rebind-investigation/` establishes a precise pair: first move `self` into an
unannotated local, then assign it back. Rust accepts the mutable binding and
rejects the immutable one with E0594. These independently qualified source
fixtures replace the initial hypotheses; no text oracle was generated.
