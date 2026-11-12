# Premise-selected inputs (source core)

Nine no_core source-core libraries reduce pinned core's `Pin` comparison
forwarding (`pin.rs:1108`), checked against rustc 1.98. They use the
`source-div` foundation prefix, with a reduced `Deref`, `Pin` and comparison
trait added per case.

`Ptr::Target::eq(self, other)` leaves the comparison's `Rhs` to inference. The
caller's premise `Ptr::Target: PartialEq<Q::Target>` selects it, and the
argument `&Pin<Q>` then deref-coerces to `&Q::Target`. That projection must
reach its ordinary normalizer before the coercion reads its head.

- `forwarding` is core's impl over a non-lang trait `E`, and `inherent` and
  `other-trait` make the same call from an inherent method and from an impl of
  another trait;
- `free-deref` and `free-method` pass an overloaded `&**b` as `Rhs` from a free
  function, by path and by method call, and `other-coerces` coerces only the
  `Rhs` argument;
- `qualified` (`<Ptr::Target as E<Q::Target>>::e`) and `coerce-return` (deref
  coercion to a signature's projection) are controls;
- `missing` drops the premise from core's `PartialEq` form (E0599).

The positives use a non-lang trait. rustc 1.98.0 reports an internal compiler
error for the accepted forms with this prefix and a reduced `#[lang = "eq"]`
trait (`obj/parallel/pin-forwarding/core/batch1.txt`), so those are not
reductions.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run; both are recorded in `outcomes.json` (9/9 agree). It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/associated_declarations/forwarded_rhs.rs` checks every case.
See `docs/pa20-premise-inputs.md`.
