# Actual slice methods with projected elements

These seven source programs preserve the exact attributed
`function-pointers/foundation.rs` prefix. Each explicitly requests
`rustc_coherence_is_core` to author a slice inherent method named `inspect`.
Its real body returns 23. The direct slice, plain/conditional Copy wrappers,
explicit call, inferred/annotated field receivers and concrete element control
all have independently accepted Rust 1.98.0 object outcomes and pass source
analysis. `manifest.json` declares every source, its hash and the frozen profile.

An ordinary builtin slice `len` control does not exercise source candidate
filtering. These actual slice methods expose the nested projected property needed
by core's `guard.array_mut.len()`. The compiler reuses its existing semantic trait
selection when the structural candidate filter needs projection or size work;
all actual caller, formation, body and effect proofs remain required.

Ordinary reductions and method-priority execution belong to
[PA20](../../../../pa20/tests/spec/projected-method-inputs/README.md). Generic
referent field preparation belongs to
[PA18](../../../../pa18/tests/spec/borrowed-place-fields/README.md). The existing
13 `projection-lookup-conditions` sources are reused unchanged: their three known
valid normalization failures now accept, with both genuine rejections retained.
PA31 remains planned. These analysis contracts establish no original library
artifact or additional native source execution.
