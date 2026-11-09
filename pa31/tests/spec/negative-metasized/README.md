# Negative coherence over a `?Sized` negative impl

Thirteen no_core source-core libraries, checked against rustc 1.98. Each has a
negative impl whose condition is `T: MetaSized`, the default of `T: ?Sized`,
and a pair that only that negative impl can make disjoint:

```rust
pub trait Seal {} impl<T: ?Sized> !Seal for &T {}
pub trait Holds<X: ?Sized> {} impl<T: Seal + ?Sized, S> Holds<T> for S {}
pub struct Local; impl<U: ?Sized> Holds<&U> for Local {}
```

rustc's `impl_intersection_has_negative_obligation` negates one impl's
predicates under the other impl's `param_env`, in each order. Here it negates
the blanket's `&U: Seal` under the client's environment, whose `U: MetaSized`
proves `&U: !Seal`, so the impls are disjoint. RustGM rejected this with
"overlapping trait implementation headers require coherence proof": negative
coherence paid only Sized premises.

A negative coherence query now assumes the other header's own `MetaSized`
premises: its Sized parameters, its Sized, Copy and Clone subjects, and its
`MetaSized` clauses, written or the default of a `?Sized` parameter. Each node
an assumed premise proves remains a debt that only those premises can pay.

- Accepted, as rustc accepts them: `metasized-param`, a written
  `metasized-written`, a `#[rustc_strict_coherence]` trait
  (`metasized-strict`), a Sized client parameter (`sized-param`), and
  `supertrait-implied`, whose `U: PointeeSized + Foo` implies `U: MetaSized`
  only through Foo's implicit supertrait. Owners' predicates already include
  their elaborated supertrait bounds, so that premise is assumed too.
- Rejected with E0119 by rustc and with the overlap text here: a
  `PointeeSized` client parameter (`pointee-param`), a negative impl that
  requires `T: Sized` (`sized-negative`), the pair without
  `with_negative_coherence` (`metasized-stable`), and `premise-side`, where
  the negated clause belongs to the blanket. The blanket's own `V: ?Sized`
  default must not pay its own clause's debt. `supertrait-premise-side` is
  the same with the blanket's premise implied by `V: PointeeSized + Foo`: an
  elaborated premise keeps the side of the impl that owns it.
- `edge-pointee` and `edge-specialized` check that the new premise does not
  break specialization. The negative impl requires `U: Cond`, and a
  specializing `impl<T: ?Sized> Cond for T` applies only with a `MetaSized`
  premise for `U`. rustc accepts both.
- One recorded gap, `sized-premise-side` (rustc gives E0119): the same side
  question for a Sized debt. Sized debts are still paid from both headers'
  Sized premises, so it is accepted here, a recorded accepts-invalid.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of these files. Both are recorded in `outcomes.json`, with two
amendments:
- After the first oracle and before any RustGM run, rustc rejected four twins
  for the fixture itself (E0277 and min_specialization's item rule in the edge
  twins, E0637 in the side twins). They were rewritten, and no outcome changed.
- After the first candidate run, `supertrait-implied`, declared fenced, was
  accepted as rustc accepts it. That declaration was a wrong prediction: this
  compiler already elaborates supertrait bounds into owners' predicates. The
  declared fields are kept. `supertrait-premise-side` was then declared before
  its oracle and its run.

Each file is one crate feature line, then the lang-item prefix and the
`Tuple`, `Fn`, `FnMut` and `FnOnce` declarations, which are the exact pinned
Rust core fragment of `spec/fn-sugar-impl-parameters/map-adapter.rs`, as in
`spec/scoped-body-selections`, then the case body. They keep Rust's
MIT/Apache-2.0 attribution; see the root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/negative_metasized_premises.rs` checks every case. It
also checks that a pair with N conditions against N `?Sized` premises costs
linear work for N = 8, 16 and 32. See `docs/parallel/negative-metasized.md`.
