# Struct update fields (source core, bring-up profile)

Four no_core source-core libraries build a struct with functional update
syntax (`S { field, ..base }`), as pinned core's `str::split_terminator`
builds `SplitInternal { allow_trailing_empty: false, ..self.split(pat).0 }`.
They are checked against rustc 1.98. The prefix is the `source-call-members`
`owned` prefix with core's `const_precise_live_drops` feature.

- `generic` updates a generic `Plain<T>`, whose omitted `value: T` field
  must be moved out of the base at the constructed `Plain<T>`'s own field
  type;
- `projected` is core's shape: a generic `split_terminator<P: Pattern>`
  whose omitted `matcher: P::Searcher` field is an associated type
  projection;
- `concrete` is the same update at `P = u8`, where that field's type
  normalizes to `ByteSearcher`;
- `rejected` reads the base's `value` after the update moved it (E0382).

Each field the update takes from its base is read at the instance type the
constructed struct gives it, which type checking records with the update,
not at the declaration's template type, which names the struct's own
parameters.

Each accepted program exports `entry` and `probe`; `observe.c` prints `41 59`.
Outcomes were declared before the frozen rustc 1.98.0 oracle; both are
recorded in `outcomes.json` (4/4 agree). It keeps Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/bringup_stubs.rs` checks every case and runs the accepted ones
natively.
