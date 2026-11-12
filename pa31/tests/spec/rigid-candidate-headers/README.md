# Rigid candidate headers (source core)

Seven no_core source-core libraries reduce pinned core's `Peekable::next_if_eq`
(`peekable.rs:318`), checked against rustc 1.98. The prefix is
`fn-pointer-coherence/core-references.rs`: core's features, its `FnPtr`
declaration, the blanket `impl<F: FnPtr> PartialEq for F` and the four const
reference impls. Here `PartialEq` also has core's provided `ne` and `Not for
bool`, because RustGM validates the comparison family. An `Iterator`-like trait,
a `next_if`-style callback and a `Peekable`-like owner follow.

`&I::Item == &T` under `I::Item: PartialEq<T>` selects `&A: PartialEq<&B>`.
The blanket's header would need `T == <I as Iterator>::Item`. rustc relates that
rigid alias structurally, so the blanket does not apply.

- `projection`, `sized-projection`, `closure` and `method` compare a projected
  item, with `T: ?Sized` or sized, directly, in a closure, or in an inherent
  method over the impl parameter;
- `parameter` and `same-parameter` are controls over a plain parameter;
- `missing-premise` drops `I::Item: PartialEq<T>` (E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run; both are recorded in `outcomes.json` (7/7 agree). It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/associated_declarations/disjoint_aliases.rs` checks every case.
See `docs/pa20-rigid-candidate-headers.md`.
