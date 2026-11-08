# Callable sugar in coherence

Nine no_core source-core libraries beside core's blanket `impl<F> MultiCharEq
for F where F: FnMut(char) -> bool`, checked against rustc 1.98. They reduce
the record that followed census 10's once its `FnPtr` coherence and
specialization header lifetimes were repaired: core's `impl<const N: usize>
MultiCharEq for [char; N]` (`str/pattern.rs:645`).

Parenthesized callable sugar writes an ordinary trait predicate under a binder
that binds no variable. rustc's overlap check evaluates `T: FnMut<(char,)>` for
the intersected Self type like any other where clause. For an array, a slice
or array reference, a primitive or a local nominal, no impl or builtin callable
candidate applies and the goal is knowable, so the impls are disjoint.

- `core-pattern`: core's four `MultiCharEq` impls (`str/pattern.rs:633-664`)
  with core's `#[fundamental]` callable traits, their const `&F` and `&mut F`
  impls (`ops/function.rs:253-311`) and core's coherence features;
- the blanket beside an array, an array reference, a slice reference, `u8` and
  a local unit struct;
- a guard: the explicit `F: FnMut<(char,), Output = bool>` form, which has no
  binder;
- opponents: a second blanket over `FnMut(u8) -> bool` and an impl for `&T`,
  which a downstream type could make callable (each E0119).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run; both are recorded in `outcomes.json` (9/9 agree). Its one
amendment, after the first oracle and before any candidate run, adds `Sized`
to the `use` of the callable reference impls, whose `?Sized` bounds rustc
rejected as unresolved (E0405) in four fixtures; no case body or outcome
changed. The prefix is the `spec/fn-sugar-impl-parameters/const-call-adapter.rs`
foundation, which includes core's callable trait declarations. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`core-pattern`, `array-reference` and `slice-reference` were recorded gaps
in census batch 10: their `MultiCharEq` impls were disjoint, but the const
callable reference impls did not prove their `[const]` supertrait requirement
from a `[const]` premise, which rustc accepts. Census batch 12 proves a const
impl's `[const]` requirements as `[const]` goals, and all three pass.
`dev/tests/invariants/vacuous_callable_coherence.rs` checks every case. See
`docs/parallel/census-batch10.md` and `docs/parallel/census-batch12.md`.
