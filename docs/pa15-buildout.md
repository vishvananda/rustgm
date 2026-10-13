# PA15–PA17 buildout

PA14 is published at `f940b309477a300f0332b159c71d0405921437bd`.
The active work is PA15 lifetime contracts, PA16 borrow checking and PA17
destruction. These assignments remain planned until their contracts, independent
checks and course fixtures are complete.

## Lifetime representation

Nominal types retain their declaration identity and an interned lifetime-argument
slice. Empty arguments preserve the existing nongeneric path. Physical layout,
Copy and structural-equality queries depend on declaration fields; lifetime
substitution and variance must separately retain the actual argument identities.
Declaration field types are templates, not the types of selected instance fields.

Canonical type and region lists share a typed flat-slice interner. Canonical type
properties summarize the number of enclosing binders needed by free bound
lifetimes. Substitution can return an unchanged type in constant time when it
contains no lifetime escaping into the substituted environment.

`types::substitute::Bound` removes one enclosing binder with an iterative worklist.
It preserves nested function binders, shifts replacement lifetimes under them,
and adjusts references to more distant binders. Its cache belongs to one store
and one fixed argument environment and is keyed by type and binder depth. Reuse
that cache across an instantiation's parameters, result and demanded fields;
never reuse it with another caller's arguments. Work and queued dependencies
have an explicit budget. Nominal argument rewriting reuses scratch storage.

The maintainer tests exercise distinct nominal arguments, retained equality
obligations, nested-binder capture avoidance, separate substitution environments,
10,000-level shared type graphs, missing arguments and resource/depth errors.
Ordinary node facts retain their existing 80-byte layout.

## Declaration and proof responsibilities

PA15 owns lifetime-only nominal and alias parameters, including the boundary
currently declared by PA12. General type and const parameters remain PA18.
Implement explicit and implied outlives environments, field substitution,
variance and fresh call instantiation before publishing a region-proof view.
Early-bound item arguments and late-bound function lifetimes have distinct
instantiation times; function-item coercions must preserve that distinction.

The existing elision implementation counts candidates per input parameter.
Pinned-rustc probes confirm that two parameters with the same named lifetime
still require an explicit output lifetime. One tuple parameter containing that
same lifetime twice permits elision. Preserve those independently checked rules.

The current type view retains pending region equalities. The ownership view
proves moves and initialization only. Neither is evidence of valid reference
returns. Return checking must distinguish borrowing a parameter's local slot
from reborrowing its referent, and must account for control flow and legitimate
constant promotion. Inferred local reference types need inference regions;
equating them rigidly to the first initializer's universal lifetime is incorrect.

PA16 adds loan and storage proofs to typed reference operations before those
operations may reach LowIR. PA17 consumes initialization and move facts when
elaborating conditional destruction and cleanup. Passing lifetime contracts alone
must not erase either of those later proof obligations.

## Evidence

The independently declared starting probes are retained under
`obj/pa15-work/contract-probes-1791871629076194885/`: 34 programs with explicit
expected acceptance, pinned rustc results and existing type/ownership outputs.
The original, disproved two-parameter elision hypothesis is preserved separately;
it was corrected before creating any PA15 text oracle.

Implementation and validation are still in progress. No PA15 acceptance totals
or executable-reference claims are published by this buildout record.

The representation checkpoint passes all 2,622 existing course cases with no
oracle byte changes, 352 release tests, the full previous 351-test debug run plus
the final five-test substitution suite, Clippy, rustfmt, 219 audited files and
25 harness tests. Logs are `obj/pa15-work/foundation-*.log`. The frozen checkpoint
binary is `obj/pa15-work/foundation-candidate`, SHA-256
`46dd80470b8fdb1745405c13cd8c0daba0a7ed8c3418f584213b89725903d51e`.

Two sequential CPU-10 performance runs retain 108 observations, A/A calibration,
three paired ABBA blocks per scale, and frozen inputs/binaries with full/count
output equality. No local build or test overlapped timing. Verification is in
`obj/pa15-work/foundation-perf-verified.json`; raw runs are
`obj/perf/1791872583387466624/` and `obj/perf/1791872656600046189/`.
The baseline is PA13's published compiler (also used unchanged by PA14).

| Largest workload | Baseline median ms | Checkpoint median ms | Baseline/checkpoint median KiB |
| --- | ---: | ---: | ---: |
| 64k ordinary type-checked functions | 2,397.77 | 2,396.39 | 464,016 / 464,184 |
| 16k shared nominal constant matches, LowIR | 1,494.80 | 1,456.76 | 239,994 / 240,756 |

These measurements show no material regression in the sampled existing paths.
Host variation is visible in the retained observations; the small timing
differences are not a precise speedup claim. Source-level lifetime instantiation
and proof workloads still need their own measurements when implemented.
