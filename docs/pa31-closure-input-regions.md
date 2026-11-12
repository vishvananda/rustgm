# Closure input regions

Pinned core's `SlicePartialOrd::partial_compare` 7660 (`slice/cmp.rs:184`) and
`SliceOrd::compare` 7676 (`slice/cmp.rs:287`) failed with "invalid inference
identity" at their first closure:

```rust
let elem_chain = const |a, b| match PartialOrd::partial_cmp(a, b) {
    Some(Ordering::Equal) => ControlFlow::Continue(()),
    non_eq => ControlFlow::Break(non_eq),
};
```

The closure has no expected signature. `chaining_impl`'s
`impl [const] Fn(&'l A, &'r A) -> ControlFlow<B>` bound fixes its parameter
types only later.

## Diagnosis

A closure body is checked as a nested body with its own inference context. Its
parameter types are imported from the parent and exported back when the body
is collected. With no expectation they are fresh variables. The path call
`PartialOrd::partial_cmp(a, b)` instantiates `fn(&Self, &Rhs)`, so each
parameter becomes a reference whose region the closure body created.

Export maps every interface region through the closure's closing bound. That
bound knows inherited regions, the call binder's slots and private result
regions, which already become ports. It has no image for a private region in
an input, so `map_region_roots` failed.

Debug pins `wipdbg` and `wipdbg2` were never committed. They traced the failure
to `collect_closure`, then `outgoing_regions`, then `map_types`.

rustc checks the closure in its parent's inference context. Borrow checking
makes every free region of a closure signature external to the closure body,
and propagates the body's requirements on those regions to the creator.

## Change

A closure's private input regions become ports, as its private result regions
already were:

- Each gets a fresh inference region in the parent and a universal slot in
  the body.
- Export maps it to that parent region.
- The body proves its other constraints, and the requirements on the port
  region become the parent's obligations.

Input regions imported from the parent keep their previous identity, so
closures that already exported are unchanged.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- The port slots follow the public call binder. Instantiation reconstructs that
  binder from the body's universals. Inside an instantiated generic function,
  where the closure also inherits lifetimes, it fails closed with "closure
  signature lifetime interface differs from its body". The ordinary-mode
  `generic-match` program in `obj/parallel/closure-infer/native` shows it.
  Non-generic callers compile and run.
- Const closure lowering has two failures that predate this change, and the
  annotated controls reproduce both. A `[const] Fn` call by reference is not
  yet a const invocation. A const closure body's own call requires a function
  owner. Core's census is analysis-only, so these limit the later core build,
  not the census.
- A method call on a receiver whose type is not yet known types here, and
  rustc rejects it (E0282). That is existing leniency, shown without a closure
  by `deferred-receiver`. A closure's method call on an unannotated parameter
  (`method-unknown2`) previously failed only through the export error.

## Contract

**PA31** (`pa31/tests/spec/closure-input-regions/`, thirteen source-core
reductions). All declarations agree with rustc 1.98.0 (`outcomes.json`). The
base, clean candidate 22902d20, rejects every unannotated case with "invalid
inference identity".

| Case | Form | Rust | RustGM |
| --- | --- | --- | --- |
| `later-sig` | `let c = \|a, b\| Cmp::cmp2(a, b); take(x, y, c)` | accept | accept |
| `later-sig-match` | core-shaped `match` body | accept | accept |
| `typed-params` | `\|a: &A, b: &A\|` | accept | accept |
| `direct-argument` | the closure written as `take`'s argument | accept | accept |
| `static-bound-caller` | `'static` caller references | accept | accept |
| `static-bound` | a local borrow reaches `keep5<'x: 'static>` | E0521 | lifetime proof |
| `const-later-sig` | core's const form | accept | types; const `Fn` call gap |
| `const-later-sig-match` | const `match` body | accept | types; const `Fn` call gap |
| `const-concrete` | a concrete `u8` caller | accept | types; const `Fn` call gap |
| `const-typed-fn` | annotated control | accept | types; const `Fn` call gap |
| `const-later-once` | a `[const] FnOnce` bound | accept | types; const owner gap |
| `const-typed-once` | annotated control | accept | types; const owner gap |
| `const-nonconst-impl` | a non-const `Cmp` impl | E0277 | typing |

## Qualification

Focused receipt 1794516672922529260 passes in 120.7s. It covers assignment
metadata, formatting, the architecture audit, strict Clippy for the closures
suite and the CLI, plus five exact tests:

- the two new tests in `dev/tests/unannotated_closure_inputs.rs`. The first
  runs the packet's typing through the CLI and prepared lowering for every case
  that types. It requires the exact lifetime reason for `static-bound` and the
  exact lowering gap for each const case. The second checks bounded work.
- `closure_lifetimes::closure_interfaces_require_their_own_lifetime_proof`,
  `independently_qualified_lifetime_interfaces` and
  `contextual_contracts_match_independent_rustc`, for existing closure
  interfaces.

Native receipt 1794516798427367375 passes in 40.5s with the CPPGM backend:

- the new ignored test, which runs a non-generic unannotated closure through
  `take` and calls one closure twice, against rustc;
- `closure_lifetimes::contextual_closures_execute_their_proved_signatures`.

Earlier attempts:

- 1794515528544514935: the architecture audit rejected a seventh `closure*`
  file stem, so the test is `unannotated_closure_inputs.rs`.
- 1794515910021561004 and 1794516041497321807: they found the const lowering
  gaps.
- 1794516561454756032: Clippy rejected a long `prepare_nested_output`, so the
  observation is now `nested_port_regions`.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/closure-infer/scaling-raw.txt`), for 8/32/128
generic functions, each passing an unannotated closure to `take`:

| Calls | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 22431 | 1056 | 209 |
| 32 | 89127 | 1608 | 833 |
| 128 | 355911 | 3816 | 3329 |

That is 2,779 inference steps, 23 canonical steps and 26 trait selections per
function, linear across widths.

## Core progress

Exploratory pin wip2 (aa8680f6, `obj/parallel/closure-infer/wip2-dev-fast`)
carries this change. No core observation of it has run yet.
