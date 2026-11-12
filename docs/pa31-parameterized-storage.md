# Parameterized projected storage

Pinned core's `AtomicPtr::new` 6835 (`sync/atomic.rs:1478`) and `into_inner`
6842 (`sync/atomic.rs:1694`) failed with "constant storage layout requires a
closed normalized type". Each one transmutes between `*mut T` and
`Atomic<*mut T>`. That type's field is `UnsafeCell<T::Storage>`, and core's
`impl<T> AtomicPrimitive for *mut T` sets `Storage = Align8<*mut T>`
(`atomic.rs:342`).

## Diagnosis

`obj/parallel/atomic-storage/core/declared.json` records three declared
source-core batches. Each uses the prefix of
`pa31/tests/spec/default-const-closures/count.rs` through its Fn traits, then a
reduced `AtomicPrimitive`, `Align8` and `#[repr(C)] Atomic<T>`.

The transmute size query asks the shared layout readiness walk for both sizes.
Normalized field storage existed only for closed nominals. A parameterized
nominal kept its raw fields, so the walk met
`<*mut T as AtomicPrimitive>::Storage` and stopped. rustc normalizes that
projection through the implementation, so `Atomic<*mut T>` has a known
eight-byte layout in the generic body.

## Change

- **Premise-free storage.** A transmute of a parameterized type, in a crate
  with projected field templates, first runs the existing stored-field graph in
  a premise-free session (`projections::concrete`, with no caller environment).
  It then discharges that proof closed (`discharge::closed`). A parameterized
  nominal's projected fields normalize there through validated implementations
  only. Their published storage keeps parameters but no projection
  (`complete_storage`). A field that needs a caller premise fails to normalize
  and keeps an error. No later caller replays the proof.
- **Nested nominals.** An ordinary build publishes a parameterized root without
  normalizing nested projected fields. The registry records each parameterized
  root whose premise-free subtree has completed. A premise-free build re-enters
  any other registered parameterized root once, so `Outer<T>`'s
  `Atomic<*mut T>` field is reached.
- **Shadowing premises.** rustc prefers a body's where clause over an
  implementation when selecting a projection's trait, and the projection then
  stays rigid. Some bodies have an elaborated trait premise whose trait declares
  an associated type, or have a quantified premise. If such a body's transmuted
  type reads a parameterized nominal's projected fields, it keeps the closed
  boundary. A bounded structural walk (`reads_parameterized_projections`)
  checks the second condition. The boundary holds even after another body has
  published the same form (`sibling-shadowed`).

No new engine, backend, storage policy or compiler limit is added. The registry
set is bounded by the existing recipe retention budget.

Rejected design: wip1 (ffd1c529) had no shadowing check. Batch 2 shows that it
accepts `shadowed`, which rustc rejects (E0512). wip2 (2b83bbd1) added the check
but missed `Outer<T>` (`nested-new`). It also reported `param-storage`'s failed
normalization text instead of the closed boundary.

Tradeoffs:

- The shadowing check is conservative. Two declared probes that rustc accepts
  are rejected with the closed text, exactly as before this change. They are
  excluded from the packet and recorded in `outcomes.json`:
  - `unrelated-premise`: a premise `U: Other` of another trait with an
    associated type;
  - `equality-premise`: `where *mut T: AtomicPrimitive<Storage = Align8<*mut T>>`.

  No reached core transmute has such a premise. Core's `impl<T> AtomicPtr<T>`
  has none.
- Only the transmute size query produces or gates premise-free storage. Other
  readers of a parameterized shape see the published form without consulting
  their body's premises. Before this change, they had no parameterized storage
  to read.
- `size-mismatch` now reports "source transmute requires equal source and
  destination sizes", where it previously failed with the closed text. rustc
  reports E0512 for both.

## Contract

**PA31** (`pa31/tests/spec/parameterized-storage/`, fifteen source-core
reductions). They agree with rustc 1.98.0 15/15 (`outcomes.json`).

| Case | Form | Rust | b41f1792 |
| --- | --- | --- | --- |
| `ptr-new` | core's `const fn new` | accept | reject |
| `ptr-into-inner` | core's `const fn into_inner` | accept | reject |
| `ptr-runtime` | the transmute in a non-const fn | accept | reject |
| `ptr-no-cell` | a projected field without `UnsafeCell` | accept | reject |
| `nested-new` | through a `#[repr(C)] Outer<T>` field | accept | reject |
| `marker-premise` | a premise of a trait without associated types | accept | reject |
| `ptr-direct` | the normalized field written out, a control | accept | accept |
| `usize-new` | the closed `Atomic<usize>`, a control | accept | accept |
| `unshadowed-value` | the projection as a value, a control | accept | accept |
| `param-storage` | `Atomic<T>` under `T: AtomicPrimitive` | E0512 | reject |
| `size-mismatch` | sixteen-byte `*const T` storage | E0512 | reject |
| `shadowed` | `where *mut T: AtomicPrimitive` | E0512 | reject |
| `sibling-shadowed` | the same in `new`'s own impl, after `new` | E0512 | reject |
| `nested-shadowed` | the same through `Outer<T>` | E0512 | reject |
| `shadowed-value` | the rigid projection as a value | E0308 | reject |

## Qualification

Focused receipt 1794502903646219572 passes in 43.2s. It reuses the build from
the attempt before it, which compiled the same tree. It covers assignment
metadata, formatting, the architecture audit, strict Clippy for the memory
suite, the CLI and the private library, plus ten exact tests:

- the two new tests (`dev/tests/atomic_storage/parameterized_fields.rs`): the
  packet through the CLI, and bounded work;
- two atomic storage source contracts and additive factory work;
- the private transmute size budget and sticky payment invariants, closed
  projected alignment and structural readiness, a source transmute's native
  round trips, and unchecked transmute typing.

Earlier attempts stopped at formatting, at Clippy (`needless_question_mark`),
and at a private test path written with the wrong module name.
`make test-upstream` verifies the pinned upstream fixtures. No graded case or
reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/atomic-storage/scaling-raw.txt`), for 8/32/128
generic functions, each transmuting `*mut T` into its own `AtomicPtr<T>`:

| Calls | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 34600 | 10835 | 423 |
| 32 | 51424 | 25643 | 567 |
| 128 | 118720 | 84875 | 1143 |

That is 701 inference steps, 617 canonical steps and six trait selections per
call over the prefix's fixed cost. Adding 64 unrelated parameterized nominals
with projected fields adds a constant 192 canonical steps at every width, and no
inference step or trait selection.

## Core progress

**Exploratory observation.** Pin `obj/parallel/atomic-storage/wip1-dev-fast`
(ffd1c529) carries the change without the shadowing check. The unchanged 120s
observer ends at its deadline with 2 partial records and 13,424 attempts. The
last item is 7258, fmt/mod.rs:2917. Against the
[negative receiver](pa31-negative-receivers.md) exploratory observation
(9c1bd81d) it clears `new` 6835 and `into_inner` 6842. No new failure appears
among previously passing reached bodies.

Pin `obj/parallel/atomic-storage/wip3-dev-fast` (a202701f) carries this
checkpoint's compiler tree before formatting and one lint-only rewrite
(`Ok(normalized?)` to `normalized`). A first run under external load of 49
reached only item 5722, before `new`, and is kept as
`original-wip3-load49`. The rerun ends at its deadline after 121.219s, with 11
partial records and 14,180 attempts, under external load of 19–23. The last
item is 8635, str/iter.rs:1521. Against clean b41f1792 (14 records, 14,292
attempts, last item 8825) it clears `cloned` 6721, `new` 6835 and `into_inner`
6842. No new failure appears among previously passing reached bodies. Its
eleven records are exactly b41f1792's other eleven. Among them, `as_underlying`
7631 still fails with "source transmute requires equal source and destination sizes",
as it did before this change.

The first remaining failure is `poll` 6928 at sync/sync_view.rs:296 ("no
applicable inherent method for this receiver"), then `resume` 6932.

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.
