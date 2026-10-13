# RustGM Programming Assignment 11: Procedural LowIR

Status: **implemented draft**. Prerequisite: PA10.
The profile is Rust 1.98.0, edition 2024, Linux x86-64, checked integer
arithmetic and aborting panics. This is the first executable Rust frontend
milestone; it is not whole-Rust conformance.

## Required implementation

Consume PA9's canonical types and PA10's typed bodies. Construct an owned,
typed LowIR program before serializing the shared CPPGM vocabulary. Keep
compact IDs, per-function slots/temporaries/literals/argument arenas, stable
emission identities and a worklist that emits each demanded function once.
No parsing, resolution, inference, literal decoding or rendered phase transport
belongs in lowering. No rustc, C++ source or reference executable may implement
a failed Rust frontend phase. The external backend accepts LowIR only.

The executable profile includes unit, bool, char, all twelve integer types,
f32/f64, zero-sized function items, function pointers, locals, assignments,
scalar operators/casts, calls/recursion and PA10 control flow. Calls preserve
callee/argument evaluation order. Unit arguments occupy no native parameter,
but their side effects still execute. Function items acquire an address only
when reified or called. Parameter/return signatures consume canonical types.

Before lowering, verify reachable scalar local reads and writes on the body
CFG. At a join, intersect definitely initialized locals and union possibly
written locals. A read needs definite initialization; an immutable store needs
no prior possible write in that storage lifetime. Parameters begin initialized;
`live`/`dead` reset local state on each actual path. Converge before diagnosing.
`let _ = local` does not read or move the local; RHS calls/block values still
execute. Check every constructed body, including undemanded procedures.

This scalar check is the executable prerequisite. PA14 extends it to moves,
projections, partial initialization and non-Copy values; PA15–PA17 own regions,
loans and destruction. Pending region conversions cannot become native casts.
Aggregates need PA12 layouts. Closed constant/static materialization and reference erasure are exercised
with their required proofs in PA16. General const functions and mutable constant
memory remain PA25. Their former executable boundary fixtures now live with
the owning proof assignment. Unused later-stage
constructs do not acquire a validity proof through a PA11 pass. Const validity,
full lint parity, unreachable ownership diagnostics and general Rust acceptance
remain outside this profile.

## Arithmetic and casts

Check signed and unsigned addition, subtraction, multiplication and signed
negation at the operand width. The signed-minimum literal exception comes
from PA9's recorded literal selection. Division/remainder reject zero divisors
and signed minimum divided by -1. Shifts reject a negative amount or an amount
at least the left width, even if the amount has a different integer type;
shifted-out bits are not an overflow. Comparisons use Rust signedness.

Integer casts truncate or extend using the source signedness. Bool/char casts
zero extend. Float-to-integer casts saturate at the target bounds and map NaN
to zero; they cannot use unchecked C++ conversion behavior. Preserve f32/f64
rounding, signed zero, infinities and unordered comparisons. Floating `%` uses
truncating remainder, including subnormal operands. The initial backend has no
floating remainder instruction: emit a demanded typed helper that reduces
binary significands, not a rounded floating quotient. Share one helper per
float width. The pinned backend supplies its existing 128-bit runtime helpers.

On checked failure call the backend's declared `abort` and end in `unreachable`.
An unreachable terminator alone is not a trap. The pinned CPPGM runtime's abort
exits 134 with empty streams; rustc with `panic=abort` raises SIGABRT and prints
a panic diagnostic. The independent harness qualifies those outcomes explicitly.
All other compiler/runtime signals, exit 86, timeouts or unexplained failures
fail the suite. No implicit `nowrap`, `typed`, alias or alignment facts are
permitted without Rust evidence.

## Entry, representation and deterministic text

Resolve the root value binding `main`, including an imported binding. Require
a safe, nongeneric, parameterless function returning `()` or `!`. Emit a native
`main` with `[role=entry]`, return type i64, calling the selected Rust procedure.
The adapter returns zero after a unit-returning main; a never main has no return.

Native scalar representations are bool=i1, char=u32, the corresponding signed
or unsigned 8/16/32-bit spelling, 64-bit integers/isize/usize=i64,
128-bit integers=i128, floats=f32/f64 and function pointers=ptr. Signedness for
64/128-bit operations is an operation fact, not a separate LowIR type spelling.
Unit/function items have no runtime storage. All value reads snapshot slots.

Procedure symbols are `rg.<DefId>`; helper names are `rg.rem.f32`/`rg.rem.f64`.
Symbols print in first-demand order, starting with Rust main; the native entry
adapter prints last. Reserve source body block IDs first; emit reachable blocks
in iterative reverse postorder, with guard continuations at their instruction
sites, remaining unreachable blocks and the shared per-function panic block.
Helper CFGs have fixed construction order. IDs restart per function: `%pN`
parameters, `$sN` slots, `^bN` blocks, `%tN` temporaries. Named parameters also
appear in indirect call signatures. Integer literals print unsigned decimal
bits at their width; floats print 17-place scientific decimals or `nan`/`inf`/
`-inf`. The serializer never becomes an input to another frontend phase.

## Tools and tests

```sh
dev/rustgm --emit-lowir [-o OUTPUT] [SOURCE|-]
dev/rustgm --count-lowir [SOURCE|-]
make test-pa11
make test-native
python3 scripts/backend.py output.lowir -o obj/program
```

Input, cfg, source packages and diagnostics follow PA8. Both modes complete
semantic checks and typed lowering before stdout starts. Success is exit 0
with empty stderr; rejection is exit 1 with empty stdout and projected `ERROR:`
text. As before, `-o` opens/truncates its destination before compilation.
Count mode reports `symbols`, `functions`, `blocks`, `instructions`, `slots`
and `temporaries` from the already constructed program.

The manifest declares outcomes before `make ref-test-pa11` creates text
oracles. `make test-native` additionally compiles each full program with pinned
rustc, validates LowIR with the pinned CPPGM tools, and executes both programs
against declared success/panic outcomes. Positive programs check full scalar
values internally; comparing only low exit-status bits is insufficient. Native
evidence stays in `obj/native/pa11/`, separate from byte-exact text grading.

Lowering must be linear in emitted IR, with iterative demand/CFG traversal.
The scalar initialization analysis uses a bounded monotone worklist and packed
bitsets per body: at most 4,000,000 16-byte state cells and 32,000,000 charged
word/statement steps. Exceeding either is a diagnosed resource limit. Measure
frozen binaries with A/A calibration, ABBA blocks and full-output equality.
Run all previous suites, strict Clippy/rustfmt and architecture/file audits.
