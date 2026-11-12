# Let-condition captures

Pinned core's `read_radix_max_digits` 6024 (`net/parser.rs:117`) failed with
"this typed capture use is not implemented". Its closure copies `radix` and
`max_digits`, then loops with `while let Some(digit) = p.read_atomically(..)`
and returns early once too many digits are read.

## Diagnosis

Diagnostic probes (`obj/parallel/radix-captures/probes/`) separate the parts of
that shape. The closure's early return, its lazy `&&` and its compound
assignment to a captured local all pass. The `while let` alone fails. The probes
that use `?` on `Option` stop earlier, at ordinary mode's missing source `Try`
catalog, so they show nothing here.

The capture-use walker (`closures/uses.rs`) visits each expression of a
capturing closure's body to record how it uses each captured place. It already
handled a computed `match`, which consumes its scrutinee as a value. It had no
arm for the `if let`/`while let` condition (`Kind::LetCondition`), so the walker
reached its fallback and stopped.

## Change

- **Let conditions.** A `LetCondition` now consumes its scrutinee as a value
  use, exactly like a computed `match`. The pattern's bindings are closure
  locals, and the enclosing `if`/`while` arms already visit the condition and
  bodies.
- **Matched places.** A scrutinee that is itself a captured place, such as
  `if let Some(v) = o` for a captured `o`, keeps the existing explicit
  "capture patterns for a matched place are not implemented" boundary, as for a
  `match`. It needs selected pattern operations, not a blanket value move.

No new engine, model field, backend, storage policy or compiler limit is added.

Tradeoffs:

- A captured-place scrutinee still rejects valid Rust, at the same explicit
  boundary as a computed `match` over a captured place. Core's reached bodies
  have computed scrutinees.
- `if-let-fn-mutation` rejects with the generic unambiguous-obligation text at
  the call, where rustc reports E0594. `while-let-moved` rejects at the second
  read ("local may be uninitialized or moved at this read"), where rustc reports
  E0382.

## Reductions

`obj/parallel/radix-captures/core/declared.json` records three declared
batches. Rust outcomes were declared before rustc 1.98.0 and before any RustGM
run of each file; the RustGM outcomes are observations.

1. Seven PA24 fixtures. `while-let-copy` passes its closure, but its `main`
   compares two `Option<u32>` values, and ordinary mode rejects that by itself
   (`core/eq-probe`, accepted by rustc). It is not part of the packet.
2. `while-let-digits`, the same closure with `match`-based checks. Its bytes and
   name differ.
3. Two captured-place frontier forms and the scaling widths.

## Contracts

**PA24** (`pa24/tests/spec/let-condition-captures/`, seven graded cases).

| Case | Form | Rust | ead8009c |
| --- | --- | --- | --- |
| `while-let-digits` | core's shape: copied captures, `while let`, early return | accept, runs | reject |
| `if-let-copy` | a copied capture in an `if let` condition | accept, runs | reject |
| `while-let-mutable` | an `FnMut` closure mutating its capture in the loop | accept, runs | reject |
| `while-let-owned` | a `move` closure consuming its capture after the loop | accept, runs | reject |
| `if-let-nested-closure` | a nested closure sharing the capture in the scrutinee | accept, runs | reject |
| `while-let-moved` | a capture moved in every iteration | E0382 | reject |
| `if-let-fn-mutation` | an `Fn` closure assigning its capture | E0594 | reject |

The maintainer test (`dev/tests/source_closures/let_conditions.rs`) also
asserts both captured-place forms at the explicit boundary.

## Qualification

Focused receipt 1794495580061342602 passes in 72.4s under external load near
13–28. It covers formatting, the architecture audit, strict Clippy for the
closures suite, and eight exact tests:

- the three new tests: PA24 fixtures through rustc qualification and LowIR
  instance building, the captured-place boundary, and bounded work;
- both computed-match capture tests and the try-block capture test, which use
  the same walker;
- two source capture-mode tests.

Receipt 1794496123240012049 (49.5s) adds PA24's complete text oracle with the
dev-fast CLI to the three new tests. `make test-upstream` verifies the pinned
upstream fixtures.

Text qualification (`obj/parallel/radix-captures/text-review/passed.json`) uses
exploratory candidate 59a51ffa, built from this checkpoint's exact compiler
tree:

- Every PA24 positive crosses the independent Rust/CPPGM object bridge, and
  both rejections are independently rejected by Rust. Before generation, the
  locked CPPGM backend validated, compiled and ran every positive
  (`core/pa24/native.json`).
- All 709 prior PA24 cases reproduce, as do PA1–PA23 and the released PA25
  const-machine unit. All 28,409 prior reference hashes are unchanged.
- Normal scoped generation then adds seven reviewed triples, in batch
  `pa24/reference-batches/1794496092470575797-dabcfb1a5f10.json`.

PA24 now has 716 cases (514 accept). No planned assignment is activated.

Bounded work (`obj/parallel/radix-captures/scaling-raw.txt`), for 8/32/128
functions whose closures each consume a computed `while let` scrutinee beside a
copied capture:

| Closures | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 5574 | 854 | 0 |
| 32 | 21918 | 3398 | 0 |
| 128 | 87294 | 13574 | 0 |

That is 681 inference and 106 canonical steps per closure. Adding 64 unrelated
non-closure `if let` functions adds a constant 3264 inference steps at every
width, and no canonical step: no capture walk visits them.

## Core progress

**Exploratory observation.** Pin `obj/parallel/radix-captures/wip1-dev-fast`
(59a51ffa) carries the change before the test and fixture work; wip2, built
from this checkpoint's exact tree, is byte-identical. The unchanged 120s
observer ends at its deadline after 121.138s, with 22 partial records and
14,470 attempts. The last item is 9102, time.rs:1772. Against clean ead8009c
(20 records, 14,011 attempts, last item 8344) it clears `read_radix_max_digits`
6024. No new failure appears among previously passing reached bodies.

The first remaining failure is `Pin`'s `PartialEq` and `PartialOrd` forwarding
bodies 6456–6464 at pin.rs:1108, which stop at "associated type projection
requires normalization". Next comes `cloned` 6721 ("invalid method trait
selection").

**Clean pin.** Pin 59a51ffa is built from clean source 9663144cf, with 3916
Rust hashes and an empty patch (`obj/parallel/radix-captures/candidate-dev-fast`).
Only test and documentation files changed after the exploratory build, so it is
byte-identical to wip1 and wip2. Its unchanged observation ends at its deadline
after 121.160s, with 24 partial records and 14,894 attempts. The last item is
9964, x86/sse2.rs:487. Against clean ead8009c it clears
`read_radix_max_digits` 6024, with no new failure among previously passing
reached bodies, and the first remaining failure is `Pin`'s `eq` 6456. The
postcheck verifies all 722 source hashes, 375 opens, six configuration
bindings, unchanged immutable bindings and the 3916 head blobs. The progress
SHA is `4c3986b6ad744821b33eb671416ffcd3e2cf55255de32418d742342246b4187b`.
The measurement against ead8009c is pending.

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.
