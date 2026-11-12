# Conditional const fn items

Ten pinned core bodies stopped at "source const callable conditional obligations
require a scoped continuation":

- `Iterator::last` 5713 (`iter/traits/iterator.rs:271`) passes the nested
  `const fn some<T>(..) where T: [const] Destruct` to `fold`'s `[const] FnMut`
  bound;
- `Option::as_deref` and `as_deref_mut`, and the same `Result` methods, pass
  `Deref::deref` or `DerefMut::deref_mut` to `map`'s `[const] FnOnce` bound;
- `Option::get_or_insert_default` passes `T::default`;
- the slice `SliceChain` impl's `chaining_lt`, `chaining_le`, `chaining_gt` and
  `chaining_ge` pass `PartialOrd::__chaining_lt` (and its siblings) and
  `usize::__chaining_lt` to `chaining_impl`.

Const-callable eligibility (`traits/callables/eligibility.rs`) classifies such a
function item as `Unpaid`. It is a const fn whose own predicates include
`[const]` bounds, or a member of a const trait, which is const only when its
`Self` implements the trait const. Trait selection stopped at that point. A
const closure already had the continuation: the
[default const closure checkpoint](pa25-default-const-closures.md) proves its
defining contract under the caller's assumptions.

## Change

- **Scoped item proof.** An `Unpaid` function item now enters the same scoped
  callable proof (`selection/search/effectful.rs`). Its requirements are the
  item's own `[const]` predicates and, for a const trait member, the trait's
  `Self: [const] Trait` premise. Both are bound through the item's callable
  template, whose parameters include a member's `Self`, exactly as when its
  signature is instantiated. They are then proved one by one under the
  caller's assumptions.
- A `#[rustc_non_const_trait_method]` member is rejected instead of being
  proved.
- A generic member whose callable template is not yet published waits, as
  other generic items already did.
- The closure path shares the same condition fold.

No new engine, model field, backend, storage policy or compiler limit is added.

A first exploratory build (`item-conditions/wip1-dev-fast`, f8440a49) bound the
requirements through the item's own parameters only. A const trait member kept
the trait's rigid `Self`, so `default-item`, `deref-item` and `default-impl`
stayed ambiguous. Binding through the callable template fixes this (wip2,
87a8fc14).

Tradeoffs:

- Rejections keep the generic unambiguous-obligation text at the call. rustc
  reports E0277 for the unmet `[const]` bound.
- As for closures, source-core mode defers the defining const checks of library
  bodies. Lowering a `[const] FnMut` call inside a const trait default body
  still fails, as recorded for `fold`.

## Reductions

`obj/parallel/item-conditions/core/declared.json` records one declared batch on
the `default-const-closures` prefix, with `Default` and `Deref` const traits
added per case. Rust outcomes were declared before rustc 1.98.0 and before any
RustGM run; the RustGM outcomes are observations.

## Contracts

Eleven declared reductions, `pa31/tests/spec/conditional-const-items/`, pass
rustc 1.98.0 11/11 (`outcomes.json`).

| Case | Form | Rust | eacd5629 |
| --- | --- | --- | --- |
| `last` | core's form, a nested conditional const fn for `fold` | accept | reject |
| `last-call` | the same called on a concrete iterator from a root `main` | accept | reject |
| `nested-item` | a nested conditional const fn in a free const fn | accept | reject |
| `default-item` | `T::default` under `T: [const] Default` | accept | reject |
| `deref-item` | `Deref::deref` for a reference, as in `as_deref` | accept | reject |
| `default-impl` | `<u8 as Default>::default` through a `const impl` | accept | reject |
| `ordinary-item` | an unconditional const fn item, a control | accept | accept |
| `last-missing` | `last` without `Self::Item: [const] Destruct` | E0277 | reject |
| `default-missing` | `T::default` under only `T: Default` | E0277 | reject |
| `default-nonconst-impl` | `<u8 as Default>::default` through a non-const impl | E0277 | reject |
| `nonconst-item` | a non-const fn item | E0277 | reject |

## Qualification

Focused receipt 1794489666159404183 passes in 132.8s. It covers formatting, the
architecture audit, strict Clippy for the closures and frontend suites and the
CLI, PA24's complete text oracle with the dev-fast CLI, and seven exact tests:

- the new CLI outcome test;
- the new scaling test;
- both default const closure tests, since the closure proof now shares its
  condition fold;
- the source Copy capture outcomes;
- two PA25 const closure tests.

Bounded work (`obj/parallel/item-conditions/scaling-work.txt`), for 8/32/128
const fns passing `T::default` (member) or a nested conditional const fn
(nested) to a `[const] FnOnce` bound:

| Uses | Member inference | Canonical | Selections | Nested inference | Canonical | Selections |
| --- | --- | --- | --- | --- | --- | --- |
| 8 | 62725 | 8018 | 901 | 63565 | 8066 | 933 |
| 32 | 81325 | 8258 | 1405 | 84685 | 8450 | 1533 |
| 128 | 155725 | 9218 | 3421 | 169165 | 9986 | 3933 |

That is about 775 inference, 10 canonical and 21 selection steps per member
use, and 880, 16 and 25 per nested use. Adding 64 unrelated types with their
own `const impl Default` adds a constant 128 inference and 768 canonical steps
at every width, and no trait selection. The default const closure scaling test
reproduces its previous work exactly
(`obj/parallel/item-conditions/closure-scaling-recheck.txt`).

## Core progress

**Exploratory observation.** Pin `obj/parallel/item-conditions/wip2-dev-fast`
(87a8fc14) carries the change before the test and fixture work. The unchanged
120s observer ends at its deadline after 121.005s, with 29 partial records and
15,102 attempts. The last item is 10217, x86/ssse3.rs:228. Against clean
eacd5629 (36 records, 14,320 attempts, last item 8892) it clears all ten bodies
above. No new failure appears among previously passing reached bodies.

The first remaining failure is `max` at iterator.rs:3255, followed by `min`,
`max_by_key` and `min_by_key`. They pass function items (`Ord::cmp`, a local
`compare`) to `max_by`'s ordinary `FnMut` bound, behind the nested trait path
formatting fence.

**Clean pin.** Pin 716cbe11 is built from clean source be48c2501, with 3914
Rust hashes and an empty patch (`obj/parallel/item-conditions/candidate-dev-fast`).
Its source differs from wip2 only in the rustfmt wrapping of one match guard,
so only source positions change its binary. Its unchanged observation ends at
its deadline after 120.876s, with 32 partial records and 15,395 attempts. The
last item is 10644, x86/avx2.rs:389. Against clean eacd5629 it clears all ten
bodies above, with no new failure among previously passing reached bodies. The
first remaining failure is `max` 5777 at iterator.rs:3255. The postcheck
verifies all 722 source hashes, 375 opens, six configuration bindings,
unchanged immutable bindings and the 3914 head blobs. The progress SHA is
`9503a99e7dab28d68aa62dc8306b473aadc598dff04c92ad678c39e32c183d03`.

**Measurement.** Seventy exclusive A/A plus ABBA accepting observations against
eacd5629 preserve complete outputs. The controls are the previous checkpoint's
six, plus `ordinary-item` from this packet. Paired wall ratios are
1.038/1.252/1.196/1.023/1.010/1.010/1.028, against A/A
0.908/1.570/0.933/0.912/1.049/0.915/0.951, at 9–672 ms.

A recheck of the two widest families, with six ABBA blocks
(`obj/parallel/item-conditions/measure-recheck/`), gives `parameter` 1.005
(A/A 1.044). `trait-blanket` gives 1.093 (A/A 0.743), about 1 ms at 10 ms.
User-space instruction counts (`instructions.txt`, three runs per pin) differ
by 0.016% for `trait-blanket` and 0.003% for `parameter`. The remaining wall
difference is therefore not work. `trait-blanket` is an ordinary program with
no const callable bound. External load stayed near a load average of 6. No
optimized, neutrality or original-core speed is claimed; the bounded-work test
above qualifies the work.

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.
