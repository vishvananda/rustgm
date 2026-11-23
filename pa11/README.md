# RustGM Programming Assignment 11 — Procedural LowIR

Build on [PA10](../pa10/README.md) to lower scalar Rust programs to
[LowIR](../shared/lowir.md) and run them through the CPPGM backend.
Use Rust 1.98.0, edition 2024, on Linux x86-64 with checked integer
arithmetic and aborting panics.

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
with their required proofs in PA16. PA25 covers const validity, general const
functions, and mutable constant memory. Keep obligations for later constructs
pending, even when those constructs are unused.

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

On checked failure call the backend's declared `abort` and end in
`unreachable`. The pinned CPPGM runtime's `abort` exits 134 with empty
output streams. Emit `nowrap`, `typed`, alias, or alignment facts only
when the Rust program supplies the required proof.

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
`-inf`. `nan` is the positive quiet NaN without a payload; a constant holding
any other NaN is stored as its integer bits in a slot and loaded as the float.
The serializer never becomes an input to another frontend phase.

## Command-line interface

```text
rustgm --emit-lowir [--cfg OPTION]... [--crate-name NAME] [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Input files, configuration, and diagnostics follow PA8. Complete semantic
checks and typed lowering before writing output. Success exits 0 with empty
standard error. A source error exits 1 with empty standard output and
the projected diagnostic on standard error. With `-o`, open the destination
before compilation.

## Run the tests and an executable

Use the PA11 test target from the repository root.

```sh
make -C pa11 test
```

[The manifest](tests/manifest.json) defines the expected LowIR, diagnostics,
and exit statuses. To run the assignment's empty `main` fixture in the
student repository, install the pinned backend and use the build below.

```sh
make
make setup-backend
obj/student/release/rustgm --emit-lowir -o obj/program.lowir pa11/tests/spec/entry/empty.rs
python3 scripts/backend.py obj/program.lowir -o obj/program
./obj/program
```

The empty program exits 0. For your own nontrivial examples, check the
program's scalar results and any expected abort behavior. The optional
[C example](../examples/lowir-library/return42.c) shows how to construct
LowIR with the CPPGM library and link its object output.

Lowering must be linear in emitted IR, with iterative demand and CFG
traversal. Use a bounded monotone worklist and packed bitsets for the
initialization analysis. Diagnose exceeding 4,000,000 16-byte state cells
or 32,000,000 charged word and statement steps per body.
