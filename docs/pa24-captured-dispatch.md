# Captured callable object execution

Borrowed captured closures now execute through `dyn Fn` and `dyn FnMut`, using
their existing source bodies and receiver adapters. PA24 remains planned.
Higher-ranked public call signatures now retain quantified conversion debts;
see `pa24-quantified-dispatch.md` for the subsequent implementation and evidence.
An unsized `dyn FnOnce` view may exist, but this does not implement consuming an
unsized object by value.

Each source conversion selects entries before lifetime and loan proof completes.
Selection retains the complete closure Instance, actual capture lifetimes,
requested call permission and opened execution signature. It uses the same
normalization, repeated-slot equality and generic source-condition debts as a
direct invocation. Nested callable binders are preserved. Object supertraits
come from the existing complete bound graph; no synthetic declaration or body
owner is introduced. A same-kind target uses the strongest checked source body;
a weaker callable slot uses the existing receiver adapter.

Only dependency collection from a sealed body publishes physical table targets.
Every conversion retains its own debts even when a physical target was cached
earlier. Dependencies demand the exact source or adapter from the original Entry.
Semantic table plans retain a mechanical target descriptor, not a body-layer
proof token. Final lowering must resolve the exact sealed body and check its ABI
before producing a function address. The earlier prototype stored body Entries
in semantic plans; the architecture audit rejected that dependency, and the
descriptor separation fixes it without changing the audit rule.

The ordinary scalar-body path allocates no entry collection. Captured-body entry
collections add a sparse conversion map, including its empty map header when a
body has direct captured calls but no object conversion. Retained-byte accounting
includes that header and allocated map capacity. No extra optional Box around
the collection or Clippy waiver hides this cost.

## Independent development evidence

`pa24/tests/captured-dispatch-outcomes.json` retains 23 declared cases. Sixteen
pass all three frontend views and independently execute with rustc and CPPGM.
They cover shared and mutable captures, weaker receiver permissions, generic
owners and consumers, inherited bounds, upcasts, aggregate inputs/results,
returned external references, destruction exactly once, captured zero-sized
values and repeated use across distinct lifetimes. Six Rust-invalid controls
exercise missing output lifetime, call permission, escaped storage, a later
static requirement after an earlier valid conversion, and conflicting loans.
One Rust-valid quantified captured case remains unsupported, separately from
those language rejections. Type inspection intentionally precedes some executable
lifetime/loan checks.

The initial independent probe corrected two hypotheses. The first result fixture
omitted a required named output lifetime and is rejected by rustc too. Its
explicit-lifetime counterpart succeeds. The first purported captured quantified
fixture used `let _ = n`, which does not capture `n`; it succeeds as an empty
closure. A separate fixture actually reads `n` in arithmetic and reaches the
quantified captured-entry boundary. Both original inputs and observations remain
unchanged under `obj/parallel/pa24/captured-objects/first/`; corrected declarations
and full native evidence are under `second/`.

The promoted batch passes formatting, strict all-target/all-feature Clippy, the
file/architecture audit, 53 source groups, thirteen native groups and four typed
table/dispatch invariants. Evidence is under `checks-fifth/`. Its final upstream
command named a nonexistent script and failed as infrastructure; the corrected
`checks-sixth/` passes upstream24+126 and layout38 without repeating completed
compiler tests. That batch also confirms the two previously reported PA28 slice
consumer boundaries on the combined compiler. `checks-seventh/` passes every
active PA1–PA23 text suite without regenerating references.

At 8/32/128 repeated conversions, one table and two captured targets remain
shared, with planning work 185/641/2,465 steps (nineteen steps per added use).
Distinct captured generic specializations retain 8/32/128 tables and
16/64/256 targets, with 101/269/941 planning steps. Both families share one
physical virtual-call forwarder; each distinct Fn source demands one existing
FnMut receiver adapter. These counters measure table planning rather than total
compiler work or wall time. Combined release checks and immutable performance
measurements remain separate publication gates for this checkpoint.
