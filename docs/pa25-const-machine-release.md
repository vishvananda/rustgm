# PA25 const-machine component release

This releases the finite **const-machine** work unit. PA25 itself remains
planned: open/conditional initializer contracts, const closures, symbolic
memory and source const calls stay in their own units and requirement rows.
The release does not activate a parent manifest or count PA25 as passing.

## Released contract

The canonical manifest is [const-machine.json](../pa25/tests/units/const-machine.json):
113 independently declared outcomes, 67 accept/46 reject, under Rust 1.98.0,
edition 2024, x86-64 Linux, checked overflow and aborting panics. Ninety-four
cases use `--emit-lowir` and nineteen `--emit-constants`. Its clusters are the
109 cases in [the course corpus](pa25-const-machine-course.md) plus four limit
controls:

| Case | Outcome | Contract |
| --- | --- | --- |
| `limits/frames-unbounded.rs` | reject | Unbounded const recursion: RustGM's 1024-frame limit; Rust E0080 at its CTFE frame limit |
| `limits/frames-bounded.rs` | accept, execute | Depth 100, inside both Rust's 128 CTFE frames and RustGM's limit |
| `limits/steps-bounded.rs` | accept, execute | 100,000 loop iterations; exact value 935003 checked at run time |
| `limits/storage-bounded.rs` | accept, execute | A 100,000-element array written in a loop; exact value 112344 |

Outcomes were declared before either compiler ran
(`obj/parallel/const-machine-release/declared.json`). Pinned Rust and the
frozen candidate agree on all four, and both native programs pass.

Two limit contracts stay at maintainer level by design:

- **Step-limit rejection.** An unbounded loop agrees in outcome with Rust's
  deny-by-default `long_running_const_eval`. But exhausting the 16M-step budget
  takes 14.7–15.3 s on the unoptimized development CLI, which exceeds the
  independent checker's fixed 10 s timeout, so it is not a course case. It
  remains covered by `dev/tests/cfg_execution.rs`. Its declarations and raw
  observations are retained, and no limit or timeout was changed.
- **Storage-budget rejection.** Rust has no deterministic CTFE storage rejection
  comparable to RustGM's slot/byte budgets, so its exhaustion tests stay in the
  maintainer suite.

## Qualification

The immutable candidate is CLI
`a08ab1a46475334d493514fa30ce2d2717f8c8aa6a382638f0239ce68221499b`, frozen
from clean published source `ee8c11abb` with 3907 Rust hashes and an empty patch.
Its selected work and full-CI successor are recorded in
[the helper-input checkpoint](pa24-flatten-helper-inputs.md).

With that candidate, the complete 113-case unit text contract passes (4.1s),
all 113 outcomes agree with pinned Rust object compilation (9.57s), and 94 LowIR
cases pass independent native checking with 56 executable comparisons through
the locked CPPGM backend (12.37s). Unit layout
confirms complete oracle provenance and metadata validation passes. Raw receipts
are under `obj/parallel/const-machine-release/release2/`; the new reference batch
is `pa25/units/const-machine/reference-batches/1794466883882988189-e8e9c61e948e.json`.

Normal scoped generation produced the four new triples; every prior reference
hash stayed unchanged. Text uses a 30 s local runner timeout for the unoptimized
candidate. CI's optimized course step keeps the default.

## Requirements and performance

Requirements 25.1, 25.2 and 25.10 are implemented components covered by this
release. Requirements 25.3 and 25.4 remain partial: their open and conditional
source contracts belong to the unreleased const-callables unit. Existing
scaling evidence (LIB-099) keeps one shared callee CFG, with linear
execution/storage bounds at widths 8/32/128. The new bounded controls take
0.27–0.31 s on optimized pin 21f66, and the frame rejection takes 0.03 s on
the candidate. No optimized or original-core speed claim is made.

Scheduled CI runs the released unit through `--released-units` and the explicit
PA25 candidate-contract step. A full successor after publication qualifies it
on the optimized CLI.
