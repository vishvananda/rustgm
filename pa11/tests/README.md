# PA11 fixture ownership

The manifest owns every source and declares success, rejection and native
outcome before generating references. `control/` reuses PA10's declared scalar
results as full-value native checks. The former constant/static/reference executable boundaries have moved to
PA16, where the complete compiler proves and lowers those capabilities.
PA10's test interpreter never supplies executable frontend behavior.

`integers/`, `shifts/`, `casts/`, `floats/` and `panic/` cover narrow/wide values,
checked failures, saturation, IEEE special values and remainder. The two fixed
remainder-vector inputs contain 64 finite pairs per width, generated from
Python Random seed 11002352 and raw IEEE bits; expected values come from
`math.fmod` (f32 results rounded to f32). Pinned rustc independently executes
all vector assertions. These are original fixtures, not imported libm code.

`calls/`, `entry/` and `initialization/` cover demand, ABI signatures, zero-sized
arguments, recursion, main selection and converged local state. `reject/`
checks diagnosed errors with no partial LowIR. Positive programs use full-value
comparisons and a failing arithmetic path on mismatch, never truncated exit
codes. Manifest panic outcomes explicitly distinguish backend exit 134 from
rustc's SIGABRT and panic diagnostic.

Only `make ref-test-pa11` writes sidecars. `make test-native` independently
validates and executes the generated LowIR with the qualified backend. A text
oracle, backend smoke test, empty suite or exit 86 cannot substitute for it.
