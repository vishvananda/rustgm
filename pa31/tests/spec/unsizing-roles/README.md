# Source unsizing role contracts

These unactivated PA31 inputs retain independently declared exact bytes for
actual source Unsize/CoerceUnsized headers and ordinary trait goals. See
[the typed consumer review](../../../../docs/pa31-source-unsizing.md) and
`contract.json`. Grouped `coercion_provider` maintainer tests read the 19 supported
positives and 24 opposing language negatives; the three valid outside-profile
positives remain required. Scaling sources are separate original inputs.

Header excerpts and declaration contracts are derived from Rust 1.98.0 commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`, core's `marker.rs`, `ops/unsize.rs`
and `intrinsics/mod.rs`; Copyright The Rust Project Contributors,
MIT OR Apache-2.0. The goal/observer reducers are independently authored tests.
No generated references or assignment activation are supplied by this cluster.
