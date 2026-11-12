# Const closures in const trait defaults

Pinned core's `Iterator::count` 5712 (`iter/traits/iterator.rs:229`) failed
with "trait obligation requires an unambiguous matching assumption or
implementation; diagnostic contract: diagnostic nested trait path formatting is
not implemented". Its body, in the default methods of `const trait Iterator`,
is `self.fold(0, #[rustc_inherit_overflow_checks] const |accum, _elem| accum + 1)`.
`fold` requires `F: [const] FnMut(B, Self::Item) -> B + [const] Destruct`.

The formatting fence hid the failing goal. Reductions whose traits sit at the
crate root show it: `{const closure}: [const] FnMut(usize, Self::Item)` stays
deferred until inference ends. Const-callable eligibility
(`traits/callables/eligibility.rs`) admits a const closure through its owner's
const-fn call effect. A const trait's default method is conditionally const
through the trait's `Self: [const] Trait` premise, not through a const
qualifier, so it is not a `constant_functions` member and never publishes that
effect. Eligibility returned Pending for every such closure. The same closure
in a free const fn or a `const impl` method, and a const fn item in the same
default body, were already accepted.

## Change

- **Defaulted owners.** When a const closure's owner is a const trait default
  body (`FunctionContext::ConstTraitDefault`), eligibility no longer waits for
  a call effect. It requires the closure's defining contract through the
  existing scoped-callable proof. That proof's requirements already add the
  trait's `Self: [const] Trait` premise (`default_self_clause`) and the owner's
  `[const]` predicates, and prove them under the caller's assumptions.
- A non-const closure in the same body is still rejected, and every other
  owner keeps its existing effect lookup.

No new engine, model field, backend, storage policy or compiler limit is added.

Tradeoffs:

- Const fn items in the same owner still stop at "source const callable
  conditional obligations require a scoped continuation", as in core's `last`
  5713. This change covers closures only.
- Source-core mode defers the defining const checks of library bodies. A const
  closure that calls a non-const function is accepted under `--emit-types`.
  Clean 3a1a6772 already accepts a direct non-const call in a const trait
  default body, and the same const closure call in a free const fn. The
  declared `count-nonconst-call` reduction records it
  outside the packet (rustc E0015, RustGM accepts). The previous pin rejected
  it only through the same ambiguity.
- Calling `count` from a root `main` lowers `fold`'s `f(accum, x)` and fails
  with "const invocation differs from its defining call-trait or effect fact".
  The lowered const invocation path supports only by-value `FnOnce` calls. A
  const fn item argument fails the same way, so this is independent of the
  change. `call-count` is checked in types mode, and the core observation is
  analysis-only.
- Both rejections keep the generic unambiguous-obligation text at the `fold`
  call. rustc reports E0277 at the bound or at `clone`.

## Reductions

`obj/parallel/count-fold/core/declared.json` records three declared batches on
the capture prefix, with `const_closures`, `stmt_expr_attributes` and core's
`Add` with its integer and float impls. Rust outcomes were declared before
rustc 1.98.0 and before any RustGM run of each file. Two prefix corrections
keep the declared outcomes:

1. The first prefix lacked core's `const_precise_live_drops` (lib.rs:117).
   rustc rejected every const-trait case with E0493 at `fold`'s `while let`.
2. `count-call`'s object generation needs core's `panic_const_add_overflow`
   lang item for its monomorphized overflow-checked add.

The first prefix's files and outputs are kept in `core/prefix-1/`.

## Contracts

Ten declared reductions, `pa31/tests/spec/default-const-closures/`, pass rustc
1.98.0 10/10 (`outcomes.json`).

| Case | Form | Rust | 3a1a6772 |
| --- | --- | --- | --- |
| `count` | core's form | accept | reject |
| `count-typed` | the closure parameter annotated `usize` | accept | reject |
| `count-suffix` | the initial value `0usize` | accept | reject |
| `count-no-attribute` | the const closure without the attribute | accept | reject |
| `count-plain` | ordinary trait, closure and `FnMut` bound | accept | accept |
| `count-call` | a concrete iterator's `count` in a library function | accept | reject |
| `call-count` | the same called from a root `main` | accept | reject |
| `count-clone-premise` | a const closure cloning its element under `[const] Clone` | accept | reject |
| `count-plain-const-bounds` | a non-const closure for the `[const] FnMut` bound | E0277 | reject |
| `count-clone-missing` | the same clone under only `Clone` | E0277 | reject |

## Qualification

Focused receipt 1794487844058195661 passes in 84.0s. It covers formatting, the
architecture audit, strict Clippy for the closures and frontend suites and the
CLI, PA24's complete text oracle with the dev-fast CLI, and five exact tests:

- the new CLI outcome test;
- the new scaling test;
- the source Copy capture outcomes;
- two PA25 const closure tests: real const closure calls, and unused const
  body checks.

PA25 is still a planned assignment with no accepted text contract, so its oracle
cannot run (receipt 1794487505868234527 records the harness refusal).

Bounded work (`obj/parallel/count-fold/scaling-work.txt`), for 8/32/128 const
traits whose default method passes a const closure, or a const fn item as a
control, to a `[const] FnMut` bound:

| Traits | Closure inference | Canonical | Selections | Item inference | Canonical | Selections |
| --- | --- | --- | --- | --- | --- | --- |
| 8 | 73100 | 8575 | 897 | 69892 | 8592 | 945 |
| 32 | 104276 | 9367 | 1329 | 91444 | 9432 | 1521 |
| 128 | 228980 | 13659 | 3057 | 177652 | 13916 | 3825 |

A closure body costs about 1299 inference steps and 18 trait selections, and an
item body about 898 and 24. Canonical steps grow from about 33 to 45 per body
for both forms. That is a small quadratic term in the const trait and fn
declarations, present without any closure. Adding 64 unrelated const traits and
const fns with their own const closures adds a constant 12,736 inference and 513
canonical steps at every width, and no trait selection.

## Core progress

**Exploratory observation.** Pin `obj/parallel/count-fold/wip1-dev-fast`
(eacd5629) carries the change before the test and fixture work. The unchanged
120s observer ends at its deadline after 120.848s, with 36 partial records and
14,291 attempts. The last item is 8823, str/pattern.rs:691. External load rose
to a load average near 22 during the run, so it reached fewer items. Against
clean 3a1a6772 (43 records, 15,502 attempts, last item 10751) it clears `count`
5712. No new failure appears among previously passing reached bodies.

The first remaining failure is `last` 5713 at iterator.rs:271, which needs a
scoped continuation for source const callable conditional obligations. `max`,
`min`, `max_by_key` and `min_by_key` keep the nested trait path formatting
fence. They pass function items (`Ord::cmp`, a local `compare`) to `max_by`'s
ordinary `FnMut` bound, a separate cause.

**Clean pin.** Pin eacd5629 is built from clean source a468bcc0b, with 3913
Rust hashes and an empty patch (`obj/parallel/count-fold/candidate-dev-fast`).
Only test and fixture files changed after the exploratory build, so it is
byte-identical to wip1. Its unchanged observation ends at its deadline after
120.867s, with 36 partial records and 14,320 attempts. The last item is 8892,
str/pattern.rs:1816. Against clean 3a1a6772 it clears `count` 5712, with no new
failure among previously passing reached bodies, and the first remaining
failure is `last` 5713. The postcheck verifies all 722 source hashes, 375
opens, six configuration bindings, unchanged immutable bindings and the 3913
head blobs. The progress SHA is
`a0be02d133ddc057e01ae550afe5e8dbbf6784337e1f54ac140b286516bd39b1`.

Both eacd5629 observations reached fewer items than 3a1a6772's 15,502. Two
further unchanged observations, one per pin, sampled `progress.tsv` every half
second (`obj/parallel/count-fold/timing-*.txt`). They reverse the order:
eacd5629 reaches 15,220 items and 3a1a6772 13,808. Their first items arrive
after 54.7s and 57.7s of declaration preparation, and both stall 7–9s at
implementation 44720. The partial counts vary between runs by more than the
pins differ, and show no slowdown attributable to the change.

**Measurement.** Sixty exclusive A/A plus ABBA accepting observations against
3a1a6772 preserve complete outputs. The controls are the six specialized trait
input controls, with `count-plain` from this packet in place of `default-body`.
Paired wall ratios are 0.961/1.055/1.122/1.032/0.990/1.013, against A/A
0.848/0.840/1.049/1.061/0.973/1.003, at 10–654 ms. External load stayed near a
load average of 11. No optimized, neutrality or original-core speed is claimed;
the bounded-work test above qualifies the work.

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.
