# Closure input regions (source core)

Thirteen no_core source-core libraries reduce pinned core's
`SlicePartialOrd::partial_compare` 7660 (`slice/cmp.rs:184`) and
`SliceOrd::compare` 7676 (`slice/cmp.rs:287`). Both bind
`let elem_chain = const |a, b| match PartialOrd::partial_cmp(a, b) { ... }`
and pass it to `chaining_impl`, whose `impl [const] Fn(&'l A, &'r A)` bound
fixes the parameter types later. They are checked against rustc 1.98. Each
case's first line names its prefix:

- Six ordinary cases use the `destructuring-assignment/closure-capture.rs`
  prefix before its closure.
- Seven const cases use `default-const-closures/count.rs` before its
  `ZeroablePrimitive`. That prefix has const closures, `Destruct` and
  `drop_glue`.

A closure without an expected signature gets fresh parameter types. A path call
such as `Cmp::cmp2(a, b)` gives each one a reference type whose region the
body creates. rustc makes that region external to the closure body: the body's
requirements on it go to the enclosing body.

- Positives: `later-sig` and `later-sig-match` are core's form, and
  `typed-params` and `direct-argument` are controls. `static-bound-caller`
  passes `'static` references to a closure that requires them.
- `static-bound` passes local borrows to that closure, and rustc rejects it
  (E0521). Here prepared lowering rejects it in the enclosing body.
- `const-later-sig`, `const-later-sig-match` and `const-concrete` are core's
  const form. `const-later-once` uses a `[const] FnOnce` bound. `const-typed-fn`
  and `const-typed-once` annotate the parameters. `const-nonconst-impl` uses a
  non-const impl, and typing rejects it (E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. All 13
declarations agree. `obj/parallel/closure-infer/core/declared.json` records ten
batches, four of them malformed. It also records rustc-rejected method calls on
unknown receivers, which RustGM types with or without a closure.

The six const accepts type, but lowering stops in a const closure path that
predates inferred inputs, as their annotated controls show.

`dev/tests/unannotated_closure_inputs.rs` checks every case's typing through
the CLI. It also runs prepared lowering, which checks lifetimes, and checks
bounded work. Its ignored native test runs two ordinary-mode programs against
rustc. See `docs/pa31-closure-input-regions.md`.
