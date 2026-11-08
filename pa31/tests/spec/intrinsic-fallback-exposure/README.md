# Intrinsic fallback exposure

Five no_core source-core libraries in a staged crate, where every unmarked
const fn is implicitly const-stable-indirect and its body is checked
recursively:

- core's exact `cold_path` fallback intrinsic and core's `likely` and
  `unlikely`, reached from an array length and from a runtime caller. The
  unmarked fallback is itself implicitly const-stable-indirect, so it is safe
  to expose;
- a bodiless `assume` marked `rustc_intrinsic_const_stable_indirect`;
- opponents: the same bodiless `assume` without its marker, which has no
  fallback to expose, and a `cold_path` that is explicitly
  `rustc_const_unstable`. rustc reports that a const function that might be
  exposed to stable cannot use the feature.

Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json`. The lang-item prefix is the exact pinned Rust core fragment
from the census batch 2 controls, with its attributes and comments; the
intrinsic and hint declarations are core's, with only their feature name
changed. They keep Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/fallback_exposure.rs` checks every case.
See `docs/parallel/census-batch7.md`.
