# User callable-trait impls

Four no_core source-core libraries for a user impl of a callable trait.
Rustc assembles impl candidates for `FnOnce` beside its builtin closure and
function evidence. A user impl for a struct (core `SyncView`) or for
`&mut Drain` (core `array::Drain`), whose referent is not callable, is
therefore selected, and its own `Self::Output` normalizes. Before, RustGM
answered such a subject only from language evidence: a struct had no
signature and a reference forwarded to its referent, so both were absent.

The opponents are an `Output` returned at the wrong type (E0308) and a
`&mut Plain` with neither a callable referent nor an impl (E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle; see
`outcomes.json`. The implementation change was first probed on
`spec/fn-sugar-impl-parameters/const-call-adapter.rs`, which now passes.

The lang-item, `Tuple`, `Fn`, `FnMut` and `FnOnce` prefix is the exact pinned
Rust core fragment of `spec/fn-sugar-impl-parameters/const-call-adapter.rs`,
from the original core `marker.rs` and `ops/function.rs`. The case bodies are
reduced from the original core `sync/sync_view.rs` and `array/drain.rs`. They
keep Rust's MIT/Apache-2.0 attribution; see the root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/user_callable_output.rs` checks every case.
