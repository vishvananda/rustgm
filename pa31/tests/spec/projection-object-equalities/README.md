# Rigid projection equalities beside trait objects

Four no_core source-core libraries with core's `Try`/`Residual`
`ChangeOutputType` shape. A rigid projection result such as
`<<R as Try>::Residual as Residual<V>>::TryType` takes its `Output` and
`Residual` equations from its declaration, also in a crate that forms a trait
object and therefore has dynamic shapes. The positives use those equations in
a const fn signature and normalize `Output` and `Residual`; the opponents
compare the normalized `Output` with `u16` and the rigid `TryType` with `R`
(both E0308).

Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json`. The lang-item prefix is the exact pinned Rust core fragment
from the census batch 2 controls, with its attributes and comments. It keeps
Rust's MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/residual_object_equalities.rs` checks every case. See
`docs/parallel/census-batch5.md`.
