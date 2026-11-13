# Derived field path bounds

Release candidate6's core census (`obj/parallel/capture-match/candidate6-release`,
54039f5d, clean 0d5530cca) fails five derived bodies of three iterator
adapters:

| Body | Item | Failure |
| --- | --- | --- |
| `clone` | `iter/adapters/peekable.rs:13` | obligation |
| `fmt` | `iter/adapters/peekable.rs:13` | unsize |
| `clone` | `iter/adapters/array_chunks.rs:16` | obligation |
| `fmt` | `iter/adapters/array_chunks.rs:16` | unsize |
| `fmt` | `iter/adapters/intersperse.rs:8` | obligation |

- "Obligation" is "trait obligation requires an unambiguous matching
  assumption or implementation".
- "Unsize" is "conditional Unsize input formation cannot publish an absence
  without paying its debts", for the `&dyn Debug` argument of a field.

```rust
#[derive(Clone, Debug)]
pub struct Peekable<I: Iterator> {
    iter: I,
    peeked: Option<Option<I::Item>>,
}
```

`ArrayChunks<I: Iterator, const N: usize>` has the field
`Option<array::IntoIter<I::Item, N>>`. `Intersperse<I: Iterator>` writes
`where I::Item: Clone`, and its fields are `I::Item` and `Option<I::Item>`.

## Diagnosis

The derived impls bounded only the type parameters, by the derived trait, and
copied the written where clause. Nothing bounded `I::Item`, so `Option<I::Item>:
Clone` and `I::Item: Debug` had no premise.

rustc's derive also bounds field type paths (`find_type_parameters` in
`rustc_builtin_macros/src/deriving/generic/mod.rs`). It walks every field
type. Each path type whose first segment names a type parameter gets a `where`
predicate with the derived trait, unless the path is that parameter alone.
`rustc -Zunpretty=expanded` on the Intersperse reduction prints

```rust
impl<I: crate::fmt::Debug + Iter> crate::fmt::Debug for Sep<I> where
    I::Item: Clone, I::Item: crate::fmt::Debug, I::Item: crate::fmt::Debug
```

The predicates follow the written ones and repeat once per occurrence.
`obj/parallel/derive-projections/core/declared.json` records three declared
batches. Batch 1 confirmed the rule against rustc 1.98.0 on 14 well-formed
cases. It also confirmed what rustc leaves out: `<I as Iter>::Item` gets no
predicate, and an enum's `Default` adds none. rustc rejects a type macro in a
field of an item with type parameters ("`derive` cannot be used on items with
type macros"), from the same walk. Two batch 1 probes were malformed: the
prefix's `usize` had no `ConstParamTy_`, and `a == b` compared references with
no `PartialEq` for `&A`. Batch 2 corrected both.

## Change

- **Walk.** Before writing any member, the derive collects the active
  fields' bounded paths once, in field order. A relative path of two or more
  segments whose first is a type parameter's name is collected. The walk
  descends through references, pointers, tuples, slices, an array's element,
  generic and parenthesized arguments, associated bindings, function pointer
  types, trait object bounds and a qualified path's self type and arguments.
  It does not enter constant expressions. A qualified path is never itself
  collected. The walk runs only when the item has type parameters.
- **Predicates.** Each derived header appends `P: Trait,` for every collected
  path after the written predicates. The path's copied syntax keeps its
  original names and spans, as a field type does. `Trait` is the bound the
  header adds to type parameters: `[const]` for a const derive, and `Copy+`
  first for a union's `Clone`. `StructuralPartialEq` and an enum's `Default`
  add no predicate, as they add no parameter bound.
- **Rejections.** A type macro anywhere in a field of a generic item is
  rejected with rustc's message. A collected path beneath a `for<...>` binder
  is rejected as requiring a wider profile, since rustc's predicate would
  carry that binder.

The type parameter header text is unchanged byte for byte. No new engine,
backend, storage policy or compiler limit is added.

Tradeoffs:

- Predicates are not merged, as in rustc. Selection work stays linear with
  repeated predicates (see the bounded work below).
- rustc also walks types inside constant expressions, such as a turbofish in
  an array length. Such a type needs `generic_const_exprs`, so the walk skips
  constant expressions.
- rustc also adds `Copy` to a packed item's derived bounds, for derives that
  read its fields by value. The source derive writes no packed bound, for type
  parameters or for these paths.
- The walk compares names as rustc does, without resolution, stripping a raw
  prefix.
- A higher-ranked bounded path fails instead of carrying its binder.
- The type macro rejection is new. No existing fixture or core item derives
  over a generic item with one.

## Contract

**PA31** (`pa31/tests/spec/derive-projection-bounds/`, 16 source-core
reductions). All declarations agree with rustc 1.98.0 (`outcomes.json`).
Fourteen come from batch 1 and two from batch 2. Both batches were declared
before rustc and any RustGM run of their files. The base is release
candidate6.

| Case | Form | Rust | Base | RustGM |
| --- | --- | --- | --- | --- |
| `peekable-clone` | core's `Peekable` `Clone` | accept | obligation | accept |
| `peekable-debug` | core's `Peekable` `Debug` | accept | unsize | accept |
| `chunks-form` | core's `ArrayChunks` form | accept | obligation | accept |
| `intersperse-form` | core's `Intersperse` form | accept | obligation | accept |
| `reference-field` | `&'a I::Item` under `Debug` | accept | unsize | accept, lowering gap |
| `enum-variant-field` | `Yielded(I::Item)` | accept | obligation | accept |
| `copy-projection` | `Clone` and `Copy` over `I::Item` | accept | copy | accept |
| `partial-eq-projection` | `PartialEq` over `I::Item` | accept | obligation | accept |
| `two-parameters` | `A::Item` and `B::Item` | accept | obligation | accept |
| `default-enum-skip` | an enum's `Default` | accept | accept | accept |
| `sole-parameter` | a field `T` | accept | accept | accept |
| `type-macro-plain` | a type macro without type parameters | accept | accept | accept |
| `qualified-unbounded` | `<I as Iter>::Item` | E0277 | obligation | obligation |
| `item-bound-required` | a use whose `Item` is not `Clone` | E0277 | obligation in the derive | obligation at the use |
| `unused-projection-bound` | a use the predicate alone refuses | E0277 | accept | obligation at the use |
| `type-macro-generic` | a type macro with a type parameter | error | accept | error |

"Copy" is "source Copy implementation requires every field to implement actual
Copy". The test requires each rejection at the site rustc reports: the use, the
derive or the type macro.

`reference-field` types, but prepared lowering stops at `show`'s
`fmt::Debug::fmt(value, f)` with "generic aggregate requires instantiated
fields". Batch 3's ungraded diagnostics stop the same way with a manual
`Debug` impl and no derive (`d-seen-manual`). The same struct lowers when no
body reads its field, with a local or a `'static` lifetime, and two controls
without a field read lower too. Batch 3 had predicted that those two struct
cases would fail. `peekable-debug`'s impl instance reads projected fields of a
struct without a lifetime parameter, and it lowers. The gap is thus an impl
instance reading a projected field of a struct with a lifetime parameter. A
lifetime struct whose read field is not projected was not separated. The gap
belongs to the core build's later lowering stage, and the test records it in
its gap table.

## Qualification

One focused receipt passes with exact selections on the final source:
1794543500616614716 (60.0s). It covers:

- assignment metadata, formatting and the architecture audit;
- strict Clippy for the frontend, traits and projections suites and the CLI,
  and strict private Clippy;
- the two new tests in `dev/tests/derived_field_bounds.rs`. The first types
  every case through the CLI and runs prepared lowering for every accepted
  case, with the one recorded gap. It requires each rejection's message at its
  site. The second checks bounded work and lowers every width;
- the derived tuple field privacy packet test, `derive_contracts` (two) and the
  `field_normalization` tests (two);
- fourteen private invariants: derived headers (`bounded_headers`, six with
  `associated_equalities`), derived location lowering, the formatter family
  (five) and image accounting (two);
- PA20's text oracles, 1616 of 1616. Its `field-normalization` `copy-result`
  cases derive `Clone` and `Copy` for `W<T: A> { v: T::Out }`, which now carry
  `T::Out: Clone` and `T::Out: Copy`.

Earlier attempts:

- 1794541746796923336 failed formatting.
- 1794541804455458198 failed private Clippy: two image accounting invariants
  built the derive output without its new field.
- The next run stopped at the 16 GiB output cap. The exploratory pin was
  retired and `make tidy-build` retired the Cargo cache.
- 1794542347053102655 and 1794542647756158464 then exhausted the budget in
  cold private Clippy. The caches were warmed once with fastcheck's own
  commands and environment (`obj/parallel/derive-projections/warm.py`).

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/derive-projections/scaling-raw.txt`), for one
derived `Clone` over 8/32/128 fields of type `Opt<I::Item>`, whose impl
repeats `I::Item: Clone` once per field:

| Fields | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 14217 | 3516 | 101 |
| 32 | 40449 | 3516 | 173 |
| 128 | 145377 | 3516 | 461 |

That is 1,093 inference steps and 3 trait selections per field, with constant
canonical work, linear across widths.

## Core progress

Release candidate7 (clean 8f4e2f737) carried this change. Its census ended at
item 14975 on a work budget regression from the first form of
[blanket candidate winnowing](pa31-blanket-candidate-winnowing.md), before
the derived bodies (33847-33913).

Release candidate8 (`obj/parallel/capture-match/candidate8-release`, clean
53667da5b, CLI ae92eca9) carries the bounded scan, the derived field path
bounds and the AVX-512 operands. The unchanged 120s observer ran it
exclusively from load 9.5. It reached the end of the body pass in 23.0s, with
all 35,120 item attempts, 8 failed and 13 blocked bodies, against
candidate6's 27 and 14. Every candidate8 record is a candidate6 record, so no
previously passing body failed. `_mm512_movm_epi16` 14975 passes. The five derived bodies, `ArrayChunks` `clone` 33847 and `fmt`
33849, `Intersperse` `fmt` 33902 and `Peekable` `clone` 33911 and `fmt`
33913, are reached at their candidate6 items and offsets, and none has a
record. `compare-original.py` matches exact progress lines, and the derive's
new bounds renumber these bodies' nodes, so its 15 cleared records omit them.
