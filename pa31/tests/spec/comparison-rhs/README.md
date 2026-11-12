# Comparison right operands (source core)

Twelve no_core source-core libraries reduce pinned core's `small_sort_network`
7571 (`slice/sort/shared/smallsort.rs:355`). They are checked against rustc
1.98. Each case's first line names its prefix,
`pa31/tests/spec/source-div/foundation.rs`. The prefix is followed by core's
`bool` `Copy`, `Not` and `PartialEq` fragments, as in
`builtin-binop-types/compare-mixed-width.rs`, and by core's raw pointer
`PartialEq` impls.

`region.as_ptr() != v_base` compares a `*const T` with a `*mut T`. rustc gives
the right operand's type a fresh variable `Rhs`. It selects `*const T:
PartialEq<Rhs>` where it can, so the only impl fixes `Rhs = *const T`. Only then
does it coerce the right operand to `Rhs`. Raw pointers are also builtin scalar
operands, so the coerced right type must then equal the left.

- `core-network` is core's form. `const-ne-mut`, `const-eq-mut` and `concrete`
  compare `*const` with `*mut`, generic or concrete.
- `fresh-rhs` infers a callee's `*mut U` through the coercion. `ref-rhs` and
  `mut-ref-rhs` coerce a reference into the pointer.
- `same-const` and a user `nominal` impl are controls.
- `mut-eq-const` puts `*mut T` on the left, so `*const T` cannot coerce (E0308).
- `premise-rhs` selects `Rhs = *mut T` through a where clause. The builtin rule
  then rejects the differing pointer (E0308).
- `ambiguous-user` adds `impl PartialEq<Local> for *const u8`. That leaves
  `Rhs` ambiguous, so the right operand's own type is unified (E0308, E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run. Both are recorded in `outcomes.json`, where 11 of 12 declarations
agree. `premise-rhs` was declared accept, and its rustc rejection is the oracle.

Two earlier batches had malformed fixtures, which lacked `Not` and then `bool`'s
`Copy`. They are recorded in `obj/parallel/pointer-rhs/core/declared.json`.
The cases keep Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`stale-node/` is a three-file library that rustc accepts. Node identities are
file-local. `a.rs` coerces `*const u8 != *mut u8`, and `b.rs`, which has the
same node structure, compares `u8 != u8` at the same node. Both comparisons
must stay primitive operations at their own types. Exploratory pin a150d249
kept the first body's coerced operand, so `b.rs` lost its erasure there.
Pinned core's `digits_to_dec_str` failed from the same stale table. The
probes are declared in `obj/parallel/pointer-rhs/regress/declared.json`.

`dev/tests/pointer_coercions/comparison_rhs.rs` checks every case through the
CLI, lowers each positive and checks `stale-node`. See
`docs/pa31-comparison-rhs.md`.
