# Always-const premises (source core)

Eighteen no_core source-core libraries reduce pinned core's two `core_arch`
`simd.rs` `eq` bodies, 9611 and 9643 (`simd.rs:91` and `:304`). Their
`const impl<T: SimdElement, const N: usize> PartialEq for Simd<T, N>` compares
`self.as_array() == other.as_array()`. `SimdElement`'s supertrait is the
always-const `const PartialEq`. They are checked against rustc 1.98. Each
case's first line names its prefix, `comparison-rhs/core-network.rs` through
its `PartialEq`. Core's reference, array and `u8` const `PartialEq` impls
follow it.

rustc lets a `const` clause satisfy a `const` or `[const]` goal, and a
`[const]` clause only a `[const]` goal.

- Positives:
  - `core-form` is core's form;
  - `direct-call`, `element-fn` and `concrete-use` compare arrays and core's
    `Simd` through the supertrait;
  - `scalar-op`, `scalar-refs-op` and `transitive` compare one element, also
    through a second trait;
  - `const-impl-supertrait` uses an impl whose premise is `const PartialEq`;
  - `const-goal-const-premise` and `const-goal-supertrait` call a function
    that requires `const PartialEq`;
  - `ordinary-impl`, `maybe-supertrait` and `ordinary-fn-supertrait` are
    controls.
- Negatives:
  - `plain-supertrait` and `maybe-plain-premise` give only an ordinary
    `PartialEq` (E0277);
  - `const-goal-maybe-premise` and `const-impl-maybe-premise` need `const`
    from a `[const]` premise (E0277);
  - `both-premises` has a `const` and a `[const]` premise for one goal
    (E0284).
- Recorded prepared lowering gaps, which the base shares and rustc accepts:
  - `core-form`, `direct-call`, `element-fn`, `concrete-use`,
    `const-impl-supertrait` and `maybe-supertrait` call a trait member in a
    const body whose `Self` an impl proves;
  - `scalar-refs-op` and `ordinary-fn-supertrait` compare two `&T` operands.
    The comparison body lifetime checkpoint closed this gap later.
    `ordinary-fn-supertrait` lowers, and `scalar-refs-op`, a const body,
    reaches the first gap.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. All 18
declarations agree. `obj/parallel/simd-eq/core/declared.json` records five
batches, including two malformed batches and three ungraded diagnostics. The
cases keep Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/supertrait_const_premises.rs` checks every case's typing through
the CLI. It runs prepared lowering for every accepted case, and checks bounded
work. See `docs/pa31-always-const-premises.md`.
