# Declaration-time lifetime contracts

Eleven no_core source-core libraries whose constants, evaluated while
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
  enough;
- waits, added by Lane B: a callee relating `Maybe<&Late<'_>>`, whose `Maybe`
  is complete before `Late` (declared after the length's owner) waits, so both
  outlives contracts must still be demanded; and `Link0<'b>` shortened to
  `Link0<'a>`, where `Link0` reaches `Link2` through two referenced nominals
  declared after the owner. The chain is covariant, and its opponent ends in
  `&'a mut &'a str`, which makes `Link0` invariant.

Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json`, with two amendments. In the first, the first bytes returned
the nominals by value, which these no_core prefixes reject for a missing
`freeze` lang item. The second adds the three wait controls, declared before
their own oracle.
The lang-item prefix is the exact pinned Rust core fragment from the census
batch 2 controls, with its attributes and comments. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/variance_contract_constants.rs` checks every case.
See `docs/parallel/census-batch7.md` and `docs/parallel/linear-waits.md`.
