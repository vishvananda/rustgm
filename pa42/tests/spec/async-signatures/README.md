# Frozen async signature predictions

Frozen before RustGM implementation or rustc observations, 2026-11-05.
The cases.tsv outcomes predict Rust 1.98.0, edition 2024, Linux x86-64,
metadata/type-checking acceptance with the exact no_core source declarations.
Scalar, borrowed and generic calls must prove Future::Output equality.
Each async declaration keeps a distinct opaque identity; generic and const
arguments remain part of that identity. Required trait methods carry genuine
Future results without invented bodies. Unused bodies remain checked, and
unsafe, const and foreign restrictions apply independently of async formation.
The current lane does not establish state machines, polling, cancellation,
destruction, imported async bodies or original core/alloc/std completion.

The first exact no_core oracle attempt is retained: free async bodies require
the real ResumeTy source role before rustc can check them. No missing-role
rejection qualifies a body control. Explicit independent companions retain
each authored test body byte-for-byte and use the pinned standard core Future;
oracle.tsv binds their predictions. The trait-method no_core consumer itself
also accepts independently. These are two declared source profiles, not a whole
original-core acceptance claim.

The reviewed sized alternatives are separate exact no_core declaration
contracts in reviewed-sized-cases.tsv. Their original prediction and explicit
independent correction are preserved in sized-predictions.md; the advisory
warning is not represented as a hard semantic rejection. Nested authored RPIT
uses real source Copy/Clone declarations in the source prelude; each async
body remains byte-identical to its standard-core companion.
