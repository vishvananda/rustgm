# PA17 release validation

PA17 publishes implicit destruction through the ordinary LowIR CLI. PA15's
region contract and PA16's loan inspection remain separate views; their pending
Drop boundary is explicit in all three handouts. The execution path selects
`prepare_destruction` and `verify_destruction` only for programs with pending
destructors. Other programs keep the existing proof path. Completed plans and
immutable recipes authorize lowering; candidates alone never do.

## Feature and fixture census

The 199 required fixtures declare 132 accepted programs and 67 rejections.
Accepted cases require 131 clean executions and one explicit panic-abort. Two
rejections name later-stage boundaries accepted by rustc: type-generic destructor
instantiation (PA18) and general inherent methods (PA19). There are no other
acceptance exceptions. Every fixture is original RustGM material, either
promoted from maintained invariants or authored for this release.

| Family | Cases | Required evidence |
| --- | ---: | --- |
| identity | 17 | Prelude, imports, reexports, aliases, explicit crate names, macros, disabled prelude and absolute paths |
| headers | 73 | Mutable receivers, lifetime binders/bounds, aliases, conventions, Copy conflicts, duplicate/invalid implementations and forbidden direct calls |
| field-moves | 15 | Whole moves, Copy/borrow/replacement controls and rejected non-Copy projections from custom Drop owners |
| loans | 12 | Scope ends, containing-object obligations, whole/partial moves, declaration order and self-reference |
| replacement-loans | 15 | Old/new loans, moved/copied/live/ended holders, branch joins, reborrows and indirect/projected replacement |
| temporaries | 7 | Extended borrows/constructors, ordinary-call negative control, inner storage and early return |
| counters | 24 | Exactly-once effects, conditional initialization, loops, return, let-else, packed fields, partial arrays/enums, replacement and borrowed-owner order |
| order | 33 | Local/field/array/variant ordering, parameters, guard retries, or-patterns, temporary scopes, nested/labeled exits and zero-sized calls |
| abort | 1 | Declared panic abort while owning a destructible value |
| boundaries | 2 | Explicit PA18/PA19 ownership of unsupported capabilities |

The earlier active assignments contain no obsolete executable custom-Drop
fixtures to migrate. Their 3,094 text oracles are preserved. Header, lifetime and
field-move fixtures intentionally include unused bodies: those bodies still need
proof even when native reachability omits their code. Counter and order cases
exercise runtime cleanup independently of those declaration/proof controls.

## Independent execution and observer limits

The 22 accepted counter programs execute unchanged source through rustc and
CPPGM with in-source value assertions. Their two negative controls are rejected
by both compilers. The explicit maintainer test also relocates one equivalent
return cleanup onto an incoming CFG edge before proof, then exercises both
successors and initialization states. Normal CLI integration tests compare full
and count outputs with unmodified library lowering and require atomic rejection.

The 33 order fixtures define `trace(u8)` as a no-op because language I/O and
shared interior mutability are not part of this profile. Course text oracles and
course native checks use those source bytes unchanged. A separate explicit
maintainer native test changes only observation: rustc prints callbacks, while
after successful RustGM lowering a typed LowIR observer checks each callback
value against the declared sequence. An entry wrapper checks the final callback
count. The original entry still performs its normal initialization and calls.
The source program must compile before any observer is installed; the test
compares full ordinary/probed output before instrumentation to ensure identity
capture has not changed lowering.

Cleanup calls, selected addresses, flags, branches and source operations remain
compiler-generated. Negative controls deliberately give the observer a wrong
value sequence and an extra expected callback, requiring failure in each case.
This evidence is instrumented native order checking, not unchanged-source output
parity. No signal, timeout or missing backend becomes a passed test. Backend
qualification remains explicit through `make test-maintainer-native` locally;
hosted CI runs the default frontend tests and reports eight native tests ignored.

## Representation and performance boundaries

Nominal glue erases lifetime arguments for layout/code sharing; destructor
liveness queries retain actual lifetime arguments. Structural tuples/arrays
currently share by canonical type identity, so lifetime-distinct structural
instances can emit separate glue. A further sharing optimization must preserve
all layout distinctions, including fields that need no destruction. Type-generic
Drop instances, general methods, manual library destruction helpers, unsized
owned values, closures, async cleanup and unwinding remain later assignments.

Counted element loops keep array length out of compiler work and emitted code
size. The aligned zero-sized array control compares lengths eight and one
billion: both are independently accepted and emit the same instruction count
with bounded output and stable aligned backing. Execution still performs the
required number of calls. Existing retained-data and loan/plan work budgets,
resource controls, Clippy all/pedantic and the architecture/file audit apply.

The earlier staged lowering measurements and implementation evidence are in
[the buildout journal](pa17-buildout.md#native-cleanup-lowering-from-completed-destruction-proofs).
Final CLI regression, native corpus and measurement receipts are recorded below.


## Release gates

All local release gates passed:

- `make test`: 3,293 PA1–PA17 text cases and 516 frontend tests, with eight
  explicitly backend-dependent tests ignored in the default frontend run.
- Release-mode Cargo tests: the same 516 frontend tests passed.
- `make test-native`: eight maintainer native tests and 849 course executions.
  The existing 16 PA13 Reference/rustc runtime differences retain their explicit
  outcomes; they are not same-outcome parity claims. PA17 adds no runtime
  differences. All backend invocations use the qualified pinned tool bytes.
- `make test-rustc`: every active independent suite, including PA15, PA16 and
  PA17, passes with its documented phase boundaries.
- Clippy all/pedantic with warnings denied, rustfmt and the 350-file architecture
  audit pass with zero findings. All 25 harness tests pass. Upstream verification
  preserves 24 source fixtures and 126 pinned Reference files.
- All 199 new oracles were generated after outcome declaration and independent
  acceptance/native qualification. Status and stream checks pass, every accepted
  program has one entry, and counter/order/identity runtime cases demand glue.
  No earlier oracle changed.

`obj/pa15-work/pa17-release-v1-validation.json` indexes the retained logs and
hashes. The final pushed commit's CI result is recorded there separately.

## CLI integration measurements

Immutable binaries, frozen inputs and source hashes were used on CPU 2 with
two warmups, four A/A calibration observations and three paired ABBA blocks
per workload/size. All 162 observations are retained, including scheduling
outliers. No own builds or tests ran during timing; unrelated shared-host work
continued. Full LowIR and count outputs agree byte-for-byte for every comparison.

For ordinary programs the baseline is the preceding CLI. For destruction the
baseline is the preceding library driver, since that CLI deliberately rejected
pending destruction. The latter comparison measures integration of the same
proved pipeline, including the drivers' different arena lifetimes, rather than
a new algorithm's speedup. Driver dependencies match the course lockfile.

| Workload / functions | Baseline median | Candidate median | Candidate RSS |
| --- | ---: | ---: | ---: |
| thin references / 256 | 30.75 ms | 29.16 ms | 9,228 KiB |
| thin references / 1,024 | 112.68 ms | 113.14 ms | 22,644 KiB |
| thin references / 4,096 | 487.81 ms | 475.30 ms | 76,308 KiB |
| wide references / 256 | 53.80 ms | 53.33 ms | 12,078 KiB |
| wide references / 1,024 | 211.07 ms | 215.11 ms | 34,878 KiB |
| wide references / 4,096 | 878.95 ms | 866.53 ms | 124,826 KiB |
| destruction: library → CLI / 256 | 164.03 ms | 163.66 ms | 22,650 KiB |
| destruction: library → CLI / 1,024 | 628.33 ms | 623.16 ms | 75,856 KiB |
| destruction: library → CLI / 4,096 | 2714.56 ms | 2738.43 ms | 290,044 KiB |

At 4,096 functions, wall-time changes are -2.57% for thin references, -1.41%
for wide references and +0.88% for destructor CLI integration. Paired ranges
overlap in every case. Ordinary peak RSS changes by 54 KiB and 240 KiB;
the new destructor CLI uses 9,258 KiB less than the preceding library driver.
These measurements show no material regression in the tested workloads.
The largest destruction output still has 368,671 instructions, 73,739 blocks
and 8,192 globals; array length does not expand glue instructions.

`obj/pa15-work/pa17-release-v1-perf-results.json` indexes every raw receipt.
Source, binary and input hashes were verified after measurement. Binary hashes:

- `drop-native-candidate-v1`: `4c1503906b98310473546971658c11c8216d28ee1cecbb86f8dd8342566e81de`.
- `drop-native-driver-bin-v2`: `bb3e6c85227602d59e4a2daf74eba96904f7e1f08b1111270737948873438181`.
- `pa17-release-candidate-v1`: `77719e8870b9018274272cd3c889443d64fab4ec1a2ecdfc7e74cc6b3e2ac97c`.
