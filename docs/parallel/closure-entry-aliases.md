# Closure entry aliases

This is the Lane B batch of 2026-11-08, on origin/main `a9a66a1ab`. It
repairs audit a08 of [caller-region premises](caller-region-premises.md).
That doc and [library build blockers](../library-build-blockers.md) recorded
a08 as a closure whose declared return type names a caller lifetime:

```rust
pub fn item<'x, F: Out<O = Option<&'x u8>>>(s: &mut Wrap<F>) -> Option<&'x u8> {
    let c = |w: &mut Wrap<F>| -> Option<&'x u8> { w.next() };
    c(s)
}
```

rustc 1.98.0 accepts it. This compiler rejected it with "missing lifetime
argument". The bug predates the alias-view repair: the audit run before that
repair (`obj/lane-b3/t3/base1-audit`) gives the same error.

## 1. Reduction

There are eighteen probes, in `obj/lane-b6/a08/probes`, `probes2` and
`probes3`. Each one is the fixtures' pinned prefix (sha256 `437463f6…`) plus
the a08 base: `Wrap<F>: Iterator` where `F: Out<O = Option<B>>`. Before each
round ran, its outcomes for both compilers were declared in
`obj/lane-b6/a08/predictions.json` (final sha256 `cca5aa50…`). rustc 1.98.0
(lib, metadata) accepts every probe except v10.

| Probe | Closure body or shape | rustc 1.98.0 | This compiler before |
| --- | --- | --- | --- |
| v0 | a08: return annotation naming `'x`, `w.next()` | accepts | "missing lifetime argument" |
| v1 | no return annotation | accepts | "missing lifetime argument" |
| v2 | elided return | accepts | "missing lifetime argument" |
| v4 | by-value parameter | accepts | "missing lifetime argument" |
| v5 | two elided parameters | accepts | "missing lifetime argument" |
| v6 | `'x` in a closure parameter | accepts | "missing lifetime argument" |
| v7 | capturing, no parameters | accepts | "missing lifetime argument" |
| v8 | let annotation naming `'x` in the body | accepts | "missing lifetime argument" |
| v3 | the annotation, no selection | accepts | accepts |
| v11 | `'x` parameter and return, no premise or selection | accepts | accepts |
| v9 | `'static` premise | accepts | accepts |
| v10 | the return names an undeclared `'y` | E0261 | "unresolved lifetime name" |
| v12 | `get::<'x, F>` before `w.next()` | accepts | accepts |
| v13 | `get::<F>` before `w.next()` | accepts | accepts |
| v14 | `w.next()` before `get::<'x, F>` | accepts | "missing lifetime argument" |
| v15 | fn body, by-value `Wrap<F>`, unit return, `s.next()` first | accepts | accepts |
| v16 | closure, premise with a lifetime-free output | accepts | accepts |
| v17 | fn body with an `&'x mut` parameter, `s.next()` first | accepts | accepts |

Round 1 refuted the recorded description. Without the return annotation (v1)
or with an elided one (v2), the closure is still rejected. With the annotation
but no selection (v3), it is accepted. So the trigger is a selection in a
closure body through an inherited premise that names a caller lifetime.

Round 2 tested order. The hypothesis was that the closure's lifetime aliases
are prepared lazily. v12 passes and v14 fails at `w.next()` (offset 6726), so
what decides is whether some body step aliases a caller lifetime before the
first selection. One prediction was wrong. I predicted that v13 would be
rejected, because a turbofish with an inferred lifetime names no caller
lifetime. It is accepted, because instantiating a generic callee also runs
`prepare_universals` through the alias paths of `bodies/calls.rs`. The exact
step was not isolated.

Round 3 tested whether fn bodies share the gap. One prediction was wrong. I
predicted that v15 would be rejected, because its parameter and return name
no lifetime. It is accepted. A fn body prepares its aliases in
`prepare_bound_environment` (`bodies/bounds.rs`) while it aliases its own
escaping premises, before any expression runs. So the gap affects closures
only.

## 2. Cause

1. `switch_closure_body` (`bodies/closures.rs`) builds the closure body's
   state from fresh scratch, so `call_types.universal_regions` is empty.
   - It sets `body_binders` to the closure interface's lifetime count.
   - When the parent's method environment requires region mapping, the child
     inherits the parent's projection view mapped into those interface
     binders (`nested_projection_environment`).
2. The child's first `projection_environment` call
   (`bodies/projections/selection.rs`) folds that view with
   `Bound::new(universal_arguments())`. No alias exists yet, so the first
   interface binder has no argument. `types/substitute.rs` then returns
   "missing lifetime argument".
3. The view is cached per body, so only the first fold matters. An earlier
   aliasing step fills the list in time (v12, v13); a later one is too late
   (v14).
4. The other two kinds of nested body prepare their aliases:
   - Fn bodies prepare them in `prepare_bound_environment`.
   - Inline const bodies call `prepare_universals` right after
     `park_nested` when their environment was mapped (`bodies/inline.rs`,
     from `c8d9724f6`).

   Closure bodies had no such call.

The locate notice also named `bodies/projections/contracts.rs:144` as a
sibling fold. It is not one. Its only caller, the pending projection
completion in `bodies/projections.rs`, calls `prepare_universals` first. No
program reaches it unprepared, so it has no control.

## 3. Repair

When the environment was mapped, `switch_closure_body` now calls
`prepare_universals(source)` after `park_nested`, as `inline.rs` does. The
change is confined to `dev/src/sema/bodies/closures.rs`, and no `sema::traits`
file changes.

Bounded work:
- Each closure interface lifetime gets one `UniversalAlias` body region and
  one `(alias, bound)` region equality.
- This happens once per closure body, and only when its environment was
  mapped.
- Afterwards `prepare_universals` returns at once, so later aliasing steps add
  nothing.
- Closures whose environment needs no mapping keep lazy preparation.

Tradeoff: a closure body with a mapped environment now allocates its aliases
on entry instead of at first use, so region numbering inside such a body can
change. Before the first candidate run, the candidate declaration
(`obj/lane-b6/a08/candidate-declaration.json`, sha256 `08136db6…`) declared
that every existing text reference is unchanged. Section 5 records the result.

Considered alternative: prepare inside `projection_environment`, and only
when it folds. That would also allocate aliases for views that need no
arguments, such as v9's `'static` view. It would also move the closure entry
contract into a query path. Inline const bodies already establish the entry
form.

## 4. Controls

There are seven additions to `pa31/tests/spec/scoped-body-selections`. The
committed bytes equal the probes. rustc 1.98.0 ran again on the committed
bytes and agreed with every declaration
(`obj/lane-b6/a08/rustc-committed.txt`). `outcomes.json` records them under
`closure_entry_declared`.

| Control | Probe | rustc 1.98.0 | Before | After |
| --- | --- | --- | --- | --- |
| `closure-selection` | v0 | accepts | "missing lifetime argument" | accepts |
| `closure-selection-before-alias` | v14 | accepts | "missing lifetime argument" | accepts |
| `closure-annotation` | v3 | accepts | accepts | accepts |
| `closure-explicit-region` | v11 | accepts | accepts | accepts |
| `closure-premise-static` | v9 | accepts | accepts | accepts |
| `closure-no-caller-region` | v16 | accepts | accepts | accepts |
| `closure-undeclared-region` | v10 | E0261 | "unresolved lifetime name" | "unresolved lifetime name" |

`dev/tests/invariants/nested_condition_universes.rs` checks all 39 cases in
the directory. Name resolution rejects the opponent's undeclared lifetime
before semantic analysis. The invariant's `analyze` therefore reports a
resolution error as the case's outcome, with its location
(`file 0: 6629: unresolved lifetime name`, at `'y`), and no longer fails
the test.

## 5. Checks

All checks ran on `a9a66a1ab` plus this batch. Every receipt's source
fingerprint equals the final tree's. The receipts are under
`obj/qualification/fast/`.

| Check | Selection | Result | Receipt |
| --- | --- | --- | --- |
| private | both `scoped_body_tests` cases | 2 passed | `1794181950076235924` |
| before control | the 39-case test with `closures.rs` reverted | fails at `closure-selection`: "missing lifetime argument" | `1794182133245196659` |
| private closures | the 46 private tests whose path names closures | 46 passed | `1794183605254311460` |
| closures | `suite_closures`, 20 roots | 137 passed | `1794182274562544475` |
| regions | `suite_regions`, 23 roots | 142 passed | `1794182554004646789` |
| projections | `suite_projections`, 28 roots | 131 passed | `1794182615637911935` |
| traits 1 | `suite_traits` roots 1–27 | 116 passed | `1794182725154964807` |
| traits 2 | roots 28–54, except `negative_coherence` (native only) | 240 passed | `1794182820614163790` |
| region-id roots | the inference, declarations and lowering roots that print region ids (8) | 55 passed | `1794183544597079257` |
| PA1–12 | course text | passed | `1794182979735024180` |
| PA13–24 | course text, except PA20 | passed | `1794183160247903621` |
| PA20 | course text | passed | `1794183241077017165` |
| upstream | `make test-upstream` | 128 fixtures and 126 Reference files verified | `obj/lane-b6/a08/upstream.log` |
| harness | `make test-harness` | 313 tests, registration and layout passed | `obj/lane-b6/a08/harness.log` |

The candidate declaration holds. No existing text reference changed, and
nothing was blessed. The region, projection and traits counts equal those of
the [caller-region premises](caller-region-premises.md) batch, which did not
run the closure suite. PA25–31 have no course manifest. Their scoped-body
fixtures run through the private invariant.

## 6. Files

- `dev/src/sema/bodies/closures.rs`: `switch_closure_body`.
- `dev/tests/invariants/nested_condition_universes.rs`: seven cases, and
  resolution errors reported as outcomes.
- `pa31/tests/spec/scoped-body-selections/`: seven controls, plus
  `outcomes.json` and the README.
- `pa31/tests/README.md`.
- `docs/parallel/caller-region-premises.md` and
  `docs/library-build-blockers.md`: the corrected a08 entries.
