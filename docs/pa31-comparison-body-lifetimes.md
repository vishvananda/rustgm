# Comparison body lifetimes

Release candidate5's core census (`obj/parallel/capture-match/candidate5-release`,
d4c52a18, clean 8d185ef39) stops `marker/variance.rs:107` at "cannot lower a
captured local lifetime". Two later `eq` bodies of `variance.rs:285` are
blocked. The failing body is the derived `eq` of

```rust
#[derive(PartialEq)]
pub struct PhantomCovariantLifetime<'a>(PhantomCovariant<&'a ()>);
```

Candidate4 reached it once the
[derived tuple field privacy checkpoint](pa31-derive-field-privacy.md) typed
its field.

## Diagnosis

`obj/parallel/variance-eq/core/declared.json` records six declared batches.
Batch 1 was malformed: `Sized` is not in a `no_core` submodule's scope. Batch
2 corrected it. On the base:

- `==` failed on every operand type carrying a body lifetime parameter, a
  function's or an impl's. Examples are `Wrap<&'a ()>` and
  `PhantomContravariant<&'a ()>`.
- `Wrap<&'static ()>`, `Wrap<()>`, `*self.0 == *other.0` on `&'a u8` and the
  path call `PartialEq::eq(&self.0, &other.0)` passed.

A temporary debug pin (`wipdbg6`, now retired) printed the solved
signature, `fn(&r1 Wrap<&r3 ()>, &r2 Wrap<&^0.0 ()>) -> bool`. A body's
lifetime parameters are the bound regions `^0.i` of the body's own binder. An
opened callee signature names them only through free alias regions such as
`r3`. The receiver's `Self` was aliased, but `Rhs` still held `^0.0`, and
publication lowers each input out of the signature's binder.

A source comparison gives `Rhs` a fresh variable and selects `Self:
PartialEq<Rhs>` before the right operand coerces to it, as rustc's
`check_overloaded_binop` does. That goal named the left operand's own type,
so selecting `impl<T> PartialEq for Wrap<T>` (`Rhs = Self`) bound `Rhs` to
`Wrap<&^0.0 ()>`, and `Rhs` sits beneath the method signature's binder.

The first candidate fix aliased the goal's subject. It typed every case, and
the packet test then met prepared lowering's "method borrow differs from
selected receiver place" for the right operand. Batch 6 showed the base
fails the same way for two local borrows, `R(&x) == R(&y)`, with no body
lifetime involved. The right operand was borrowed in the type of `Rhs`. When
its coercion to `Rhs` changes only regions, lowering keeps the operand's own
place, and that place's type differs from `Rhs`. Batch 5 showed that typing
leaves region relations to prepared lowering's lifetime proof. That proof
rejects path calls, method calls and returns that rustc's borrow checker
rejects.

## Change

- **Fresh `Self`.** Unless both operands are primitive scalars, the
  comparison's `Self` is the left operand's type with fresh regions. The goal
  `Self: PartialEq<Rhs>` uses it, and the method's `Self` is bound to it
  before the left operand is borrowed. The left borrow then relates the
  operand's regions to `Self`, as borrow checking relates rustc's
  renumbered regions. No body lifetime parameter can reach `Rhs`.
- **Unconverted right operands.** When the right operand's coercion to `Rhs`
  is an identity, the operand stays in its own type. Its adjustment then
  targets that type, and the right borrow is rebuilt in it with the same
  region. Lowering then borrows the operand's place in its own type and
  constrains the borrow to the `Rhs` parameter, as it already does for a
  receiver. A right operand converted by a real coercion keeps its borrow of
  `Rhs`. The work item that coerces the right operand carries its borrow's
  index, so no search is needed.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- The first candidate, an aliased goal subject, would have pinned `Self` to
  `'a`. Covariant operands with unrelated lifetimes, which rustc accepts,
  would then need `'b: 'a`. Fresh regions match rustc. Each comparison
  allocates the left operand's regions again.
- rustc rejects `invariant-unrelated` and `invariant-static-vs-local` in its
  borrow checker ("lifetime may not live long enough"). RustGM rejects them in
  prepared lowering ("required lifetime does not outlive its target").
  `--emit-types` accepts them, as it accepts batch 5's non-comparison forms.
- An identity adjustment of the right operand is no longer published.
  Its region relation is carried by the right borrow's constraint to the
  parameter.
- This closes the `&T == &T` prepared lowering gap recorded by the
  [always-const premise checkpoint](pa31-always-const-premises.md).
  `ordinary-fn-supertrait` now lowers. `scalar-refs-op`, a const body, now
  stops at that checkpoint's other recorded gap, a trait member call whose
  `Self` an impl proves (here `impl const PartialEq<&B> for &A`).

## Contract

**PA31** (`pa31/tests/spec/comparison-body-lifetimes/`, 21 source-core
reductions). All declarations agree with rustc 1.98.0 (`outcomes.json`).
Five come from batch 2, six from batch 3, seven from batch 4 and three from
batch 6. Batches 3 and 6 were declared as diagnostics, before rustc and any
RustGM run, and are graded here. Batch 4 was declared after the cause was
found and before any fix was built. Batch 6 was declared after the first
candidate met the right operand's lowering failure. The base is release
candidate4 (82013883, 28c5f9780) for batches 2 and 3, and candidate5 for
batches 4 and 6. Neither carries a comparison change.

| Case | Form | Rust | Base | RustGM |
| --- | --- | --- | --- | --- |
| `core-form` | core's derived `eq` over `fn() -> T` | accept | captured | accept |
| `plain-field` | a derived `eq` over `PhantomData<T>` | accept | captured | accept |
| `manual-eq` | a manual `eq` impl | accept | captured | accept |
| `contravariant` | core's `PhantomContravariant` form | accept | captured | accept |
| `sized-param` | an impl lifetime, a `Sized` parameter | accept | captured | accept |
| `deref-operands` | `*a == *b` through references | accept | captured | accept |
| `local-values` | a function lifetime on two values | accept | captured | accept |
| `unrelated-covariant` | `'a` and `'b`, covariant | accept | captured | accept |
| `invariant-same` | one lifetime, invariant | accept | captured | accept |
| `ne-local` | `!=` through the provided `ne` | accept | captured | accept |
| `impl-lifetime-method` | an inherent method's impl lifetime | accept | captured | accept |
| `static-vs-local` | a `'static` right operand, covariant | accept | captured | accept |
| `value-rhs` | a call result as the right operand | accept | captured | accept |
| `local-borrows` | two local borrows, `a == b` | accept | receiver | accept |
| `static-lifetime` | `Wrap<&'static ()>` | accept | accept | accept |
| `path-call` | `PartialEq::eq(&self.0, &other.0)` | accept | accept | accept |
| `scalar-field` | `*self.0 == *other.0` on `&'a u8` | accept | accept | accept |
| `unit-wrap-field` | a `Wrap<()>` field beside `&'a ()` | accept | accept | accept |
| `local-borrow-values` | `R(&x) == R(&y)` as values | accept | accept | accept |
| `invariant-unrelated` | `'a` and `'b`, invariant | reject | captured | outlives |
| `invariant-static-vs-local` | `'a` against `'static`, invariant | reject | captured | outlives |

- "Captured" is "cannot lower a captured local lifetime", at typing.
- "Receiver" is "method borrow differs from selected receiver place", in
  prepared lowering. It was observed with the first candidate, whose alias
  step changes nothing for a body without lifetime parameters. Batch 6's
  other two cases lowered there.
- "Outlives" is "required lifetime does not outlive its target", in prepared
  lowering.

## Qualification

Two focused receipts pass with exact selections. Each covers assignment
metadata, formatting and the architecture audit.

- 1794538963565047887 (193.7s), with the packet's first 18 cases:
  - strict Clippy for the frontend, memory and traits suites and the CLI;
  - the two new tests in `dev/tests/rhs_body_lifetimes.rs`. The first types
    every case through the CLI and runs prepared lowering. It requires every
    accepted case to lower and each rejection to fail the lifetime proof. The
    second checks bounded work and lowers every width;
  - the derived tuple field privacy packet (two tests), the three
    `comparison_rhs` tests, `derive_contracts` (two) and `language_operators`
    (36).
- 1794539775822772664 (81.6s), on the final source with all 21 cases:
  - strict Clippy for the frontend, traits and projections suites and the
    CLI, and strict private Clippy;
  - the two packet tests, `configured_tails` (two), `gat_region_premises`
    (two) and `supertrait_const_premises` (two), with its updated gap table;
  - six `associated_declarations` tests, from `forwarded_rhs` and
    `disjoint_aliases`;
  - two private invariants: rigid projection sizing and function pointer
    coherence.

1794540042643866653 (51.8s) reran the two packet tests and
`supertrait_const_premises` on the final tree, with the documentation.

Earlier attempts:

- 1794537899533202982 failed formatting.
- 1794537936983952225 failed the architecture audit. A seventh file stem
  `comparison` needed review, so the test is `rhs_body_lifetimes.rs`.
- 1794538010042479741 failed to compile: `Program` has no `Debug`.
- 1794538093099577190 selected `pointer_coercions::comparison_rhs`, which
  names a module, not a test.
- 1794538180233872420 ran the first candidate. Typing passed, and prepared
  lowering failed every accepted case with a body lifetime.
- The broad selection first stopped at the 16 GiB output cap, before the
  traits suite ran. The two exploratory pins and candidate5's stale release
  library were retired. 1794539624916908170 then found
  `supertrait_const_premises`' recorded receiver gap closed.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/variance-eq/scaling-raw.txt`), for 8/32/128
functions, each comparing two operands with a body lifetime:

| Functions | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 7838 | 3795 | 99 |
| 32 | 20510 | 7707 | 195 |
| 128 | 71198 | 23355 | 579 |

That is 528 inference steps, 163 canonical steps and 4 trait selections per
function, linear across widths. The base cannot type these widths.

## Core progress

No core observation of this change has run yet.
