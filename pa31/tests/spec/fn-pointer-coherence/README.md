# Function-pointer coherence

Fourteen no_core source-core libraries for impls beside a blanket `impl<F:
FnPtr> Trait for F`, checked against rustc 1.98. They reduce census 10's crate
record, core's `impl<F: FnPtr> PartialEq for F` (`ptr/mod.rs:2592`).

rustc decides builtin `FnPtr` evidence from the outer Self type alone: only a
function pointer implements it. Without `with_negative_coherence`, a knowable
goal over any other type constructor is impossible, but a reference or a
`#[fundamental]` nominal over an unknown type is not knowable, since a
downstream crate could own that Self type (E0119). Core enables
`with_negative_coherence` (`lib.rs:166`). Coherence then also proves a negated
where clause with the intersection's unknowns as placeholders, and the builtin
`impl !FnPtr for T` holds for every rigid type that is not a function pointer.
That is why core's four `&A`/`&mut A` `PartialEq` impls are disjoint from the
blanket, while census batch 9's reduction without the feature was rejected.

- `core-references`: core's exact const `PartialEq` shapes under core's
  features (`cmp.rs:2117`, `2192`, `2265`, `2281`);
- negative coherence accepting a reference, a `#[fundamental]` nominal, a
  trait object and, with `#[rustc_strict_coherence]`, a tuple;
- opponents with negative coherence: a parameter and a function pointer
  (E0119);
- without the feature: the core shapes are rejected, here as an unordered
  specialization overlap, and a reference and a fundamental nominal are
  rejected (E0119), while a tuple, a local nominal, a raw pointer and an array
  are accepted.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run; both are recorded in `outcomes.json` (14/14 agree). The prefix
is `spec/function-pointers/foundation.rs` with its feature line varied, then
core's `FnPtr` declaration with its doc comments. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/refuted_fn_pointers.rs` checks every case.
See `docs/parallel/census-batch10.md`.
