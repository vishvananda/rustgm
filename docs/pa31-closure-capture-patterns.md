# Closure capture patterns

Pinned core's `Duration` `fmt_decimal` 9097 (`time.rs:1493`) failed with
"capture patterns for a matched place are not implemented":

```rust
let emit_without_padding = |f: &mut fmt::Formatter<'_>| {
    if let Some(integer_part) = integer_part {
        // ...
    }
};
```

The closure captures `integer_part: Option<u64>` and tests it with `if let`.

## Diagnosis

The capture-use walker (`closures/uses.rs`) records how a closure body uses
each captured place. A computed `match`, `if let` or `while let` scrutinee is
consumed as a value, and its bindings are closure locals. A scrutinee that is
itself a captured place stopped at an explicit boundary, set by the
[match capture](pa24-match-captures.md) and
[let-condition capture](pa24-let-condition-captures.md) checkpoints. A `let`
statement with any pattern other than `_` or a plain binding stopped at
"compound capture patterns are not implemented", even when its value was
computed.

rustc's capture analysis (`ExprUseVisitor::walk_pat`) walks a pattern matched
against a captured place:

- A variant test of a multi-variant enum, a literal or range test, or a slice
  length test reads the place.
- A by-value binding copies the place, which is a shared capture, or moves it,
  which is a by-value capture and makes the closure `FnOnce`.
- `ref` and `ref mut` bindings borrow the place. With a guard, rustc also
  borrows each binding's place shared, which the binding's own use already
  covers.

Capture precision stops at an enum variant and at an index projection, so all
of these capture the matched place itself. A tuple or struct field projection
captures the field. A dereference of a captured reference captures the
referent. Edition 2018 captures whole variables.

`obj/parallel/capture-match/core/declared.json` records three declared
batches. Each batch ran rustc and the clean candidate 593be571 (dd984cad9)
before any implementation run. The candidate rejects every graded case at one
of the two boundaries.

## Change

- **Matched places.** A `match`, `if let`, `while let`, let-else or `let`
  whose value is a captured place walks its selected patterns. Each read and
  binding becomes an ordinary use of the scrutinee place: shared, mutable, or a
  value use whose copy check uses the binding's type. A scrutinee that no
  pattern uses is only mentioned. Before 2021, a mention still captures the
  variable.
- **Precision.** A use below a multi-variant enum variant or an index
  projection captures the scrutinee place. A use below a tuple or struct field
  projection, or below a dereference, would capture a more precise place. That
  fails closed with "precise capture patterns are not implemented". Two cases
  capture the variable instead, so they proceed:
  - a whole-variable edition;
  - a `move` closure, for leading dereferences.
- **Computed `let` values.** A compound `let` pattern on a computed value
  consumes that value, as a computed `match` already did.

The walk visits each selected pattern node once and shares the walker's work
limit. No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- Precise sub-place captures fail closed. These are a tuple or struct field
  (`let-tuple-precise`) and the referent of a captured reference matched with
  default binding modes (`ref-scrutinee`). rustc accepts both. Capturing the
  whole place instead would reject valid programs, such as a later use of
  another field, and would change the closure's layout. Prepared lowering also
  has no place for a scrutinee whose capture is a strict sub-place.
- `wildcard` (`match part { _ => 1 }`) captures nothing, as in rustc, and
  types. Prepared lowering stops at "missing match input place" because the
  uncaptured scrutinee has no place in the closure body.
- A single-variant enum is not tested, so its fields are precise projections
  and fail closed. rustc's own precision for that case is not observed here.
- Borrow conflicts are reported by prepared lowering's generic loan and move
  messages, where rustc reports E0382, E0502 and E0505.

## Contract

**PA31** (`pa31/tests/spec/closure-capture-patterns/`, 29 source-core
reductions). All declarations agree with rustc 1.98.0 (`outcomes.json`).
Three cases use edition 2018.

| Case | Form | Rust | RustGM |
| --- | --- | --- | --- |
| `core-form` | `if let Opt::Yes(x) = part` in an `Fn` called twice | accept | accept |
| `match-copy` | `match part` binding a `u8` | accept | accept |
| `match-move` | `match part` moving an `Owned` payload | accept | accept |
| `copy-then-use` | `part` moves after the copying closure's call | accept | accept |
| `ref-mut` | `Opt::Yes(ref mut x) => *x = 1` | accept | accept |
| `let-condition-move` | `if let` moving the payload | accept | accept |
| `let-array` | `let [a, b] = w` | accept | accept |
| `nested-capture` | an inner closure matches the place | accept | accept |
| `move-closure` | a `move` closure matches the place | accept | accept |
| `or-literal` | `Opt::Yes(1) \| Opt::No` | accept | accept |
| `at-binding-move` | `whole @ Opt::Yes(_) => whole` | accept | accept |
| `while-let` | `while let Opt::Yes(x) = part` | accept | accept |
| `let-else` | `let Opt::Yes(x) = part else { .. }` | accept | accept |
| `guard-call` | `Opt::Yes(x) if keep(x)` | accept | accept |
| `computed-let-capturing` | `let (a, b) = pair()` beside a matched capture | accept | accept |
| `nested-enum` | `Opt::Yes(Opt::Yes(x))` | accept | accept |
| `legacy-ref-scrutinee` | 2018, `match part` for `part: &Opt<u8>` | accept | accept |
| `legacy-tuple-copy` | 2018, `let (a, _) = pair` copying `a` | accept | accept |
| `match-move-twice` | the moving closure is called twice | E0382 | lowering |
| `move-then-use` | `part` is used after the moving closure | E0382 | lowering |
| `shared-conflict` | `part` moves while the closure borrows it | E0505 | lowering |
| `ref-mut-conflict` | `&part` while the closure holds it mutably | E0502 | lowering |
| `discriminant-conflict` | `&mut part` while a variant test holds it | E0502 | lowering |
| `at-binding-twice` | the `@`-moving closure is called twice | E0382 | lowering |
| `let-else-move-then-use` | `part` is used after a let-else move | E0382 | lowering |
| `legacy-tuple` | 2018, `pair.1` after the closure moved `pair` | E0382 | lowering |
| `wildcard` | `match part { _ => 1 }` beside `&mut part` | accept | lowering gap |
| `let-tuple-precise` | `let (a, _) = pair`, then `pair.1` | accept | typing gap |
| `ref-scrutinee` | `match part` for `part: &Opt<u8>` | accept | typing gap |

Probe `guard` is malformed and not graded. Its prefix has no `PartialOrd` for
`u8`, so rustc rejects `x > 1` (E0369), and `guard-call` replaces it. The base
already accepts the ungraded control `computed-let-tuple`, because its closure
captures nothing. Copies in `obj/parallel/capture-match/diag` append
`fn main() {}`, so the CLI's LowIR path runs prepared lowering.

## Qualification

Six focused receipts pass with exact selections. The first and the last also
cover assignment metadata, formatting, the architecture audit and strict
Clippy for the selected suites and the CLI.

- 1794523502874940074 (228.1s), ten tests:
  - the two new tests in `dev/tests/matched_captures.rs`. The first runs the
    packet's typing through the CLI, in each case's edition, and prepared
    lowering for every case that types. It requires the exact reason for each
    rejection and gap. The second checks bounded work and lowers every width.
  - `source_closures::let_conditions::captured_place_let_scrutinees_lower`,
    which replaces the PA24 boundary test. Its two ordinary-mode programs,
    `if let Some(v) = o` and `while let Some(v) = o` over a captured `o`, now
    lower.
  - `let_condition_captures_lower_like_rustc`, the creator requirement and
    array length packet tests, and the `captured_shapes` and
    `closure_lifetimes` modules.
- 1794523743532895647 (81.6s): the `source_closures` and `nested_evaluation`
  modules, 70 tests.
- 1794523839972783288 (65.9s): eighteen further closures-suite modules, 56
  tests.
- 1794523905954815850 (57.3s): the `environment_execution` module, 37 tests.
- Native receipt 1794523970891329099 (61.8s) runs four ignored tests with the
  CPPGM backend against rustc:
  - the new test's six ordinary-mode programs:
    - core's `fmt_decimal` form;
    - a moved payload;
    - a `ref mut` mutation that the creator observes;
    - let-else with `while let`;
    - array and `@` patterns;
    - guarded, or- and nested patterns.
  - the creator requirement, array length and contextual closure native
    tests.
- 1794524382772091709 (62.8s): the packet test again, and
  `source_closures::parameter_projections::computed_matches_collect_callback_guard_and_arm_uses`.
  It now also requires the PA31 match-capture positives `discriminant-only` and
  `borrow-binding` to complete prepared roots.

The [match capture checkpoint](pa24-match-captures.md) recorded four
captured-place positives per owner as required. Its PA31 `discriminant-only`
and `borrow-binding` now qualify (`manifest.json`, `independent.json`), and
`pending.json` keeps `wildcard-place` and `copy-field`. Those two stop at this
change's recorded gaps. The PA24 owner's four positives are recorded inputs, not
fixtures. On pin wip1, `obj/parallel/capture-match/pending/run.sh` shows the
same split for them.

Earlier attempts:

- 1794523354323487408 failed formatting.
- 1794523420660077639 failed the architecture audit: the new files were the
  seventh `patterns` and `capture` filename stems. They are now
  `uses/matched.rs` and `dev/tests/matched_captures.rs`.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/capture-match/scaling-raw.txt`), for 8/32/128
functions, each matching its captured `Opt<u8>` in a closure:

| Functions | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 32072 | 6852 | 375 |
| 32 | 41312 | 9972 | 375 |
| 128 | 78272 | 22452 | 375 |

That is 385 inference steps, 130 canonical steps and no trait selections per
function, linear across widths.

## Core progress

Clean candidate2 (`obj/parallel/capture-match/candidate2-dev-fast`, 24b7ce91)
is built from e832ec039 with an empty patch. The unchanged 120s observer ran it
exclusively from load 11.8. It stopped at the deadline (121.07s) after 15,111
item attempts, at item 10246 (core_arch `x86/sse41.rs:83`), with one record. It
reached 9097, which passes. No previously passing reached body failed. The only
remaining failure is `as_underlying` 7631 (`slice/cmp.rs:52`, "source transmute
requires equal source and destination sizes"). The census is incomplete, and no
core artifact exists. The progress SHA-256 is
`4e581b3b30081f4fb55d93357da98575d378cac076d65df3e0a0c65f3a229424`, and the
comparison is `obj/parallel/capture-match/original-candidate2-comparison.json`.
