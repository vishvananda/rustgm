# Declaration-time lifetime contracts

Eight no_core source-core libraries whose constants, evaluated while
declarations are still being checked, call const fns that relate lifetime
nominals before the crate-wide variance and outlives inference has run:

- a struct field length, a signature length and a length whose callee names a
  nominal declared after the length's owner, each passing `Note<'static>` as
  `Note<'_>`;
- a covariant shortening of `Note<'b>` to `Note<'a>` under `'b: 'a`;
- `Pair<'a, &'b u8>`, whose inferred `T: 'a` contract alone implies the
  `'b: 'a` that the body needs;
- opponents: widening `Note<'a>` to `Note<'static>`, shortening the invariant
  `Slot<'b>` (over `&'a mut &'a str`) to `Slot<'a>`, and escaping the implied
  bound to `'static`. rustc reports each as a lifetime that does not live long
  enough.

Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json`, with one amendment: the first bytes returned the nominals
by value, which these no_core prefixes reject for a missing `freeze` lang item.
The lang-item prefix is the exact pinned Rust core fragment from the census
batch 2 controls, with its attributes and comments. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/variance_contract_constants.rs` checks every case.
See `docs/parallel/census-batch7.md`.
