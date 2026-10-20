# PA22 release evidence

PA22 implements the unsized-type contract in its handout. The suite has 328
LowIR text cases: 236 accept and 92 reject. Every source is independently checked
with Rust 1.98.0 object generation. Native qualification separates 102 executions
(97 successes and five expected panics) from 134 compile-only controls. PA23 and
later assignments remain planned.

## Implementation

The earlier [metadata](pa22-validation.md), [storage](pa22-storage-checkpoint.md)
and [projected-proof](pa22-proof-checkpoint.md) checkpoints supply completed-tail
sizedness, bounded metadata/layout queries, structural and projected unsizing,
constant reference backing, coherence and reusable concrete proofs. The release
adds safe raw pointer coercions, address creation, comparisons and explicit casts.

Implicit reference-to-pointer conversions erase the outer reference loan while
preserving embedded lifetime obligations. Mutable references take an immediate
reborrow of the original sized place. Raw address creation checks initialization,
mutability and current loan conflicts without creating a lasting loan or an
alignment promise. Direct raw-dereference address syntax forwards the pointer
without reading memory. Unsafe raw loads/projections remain PA27 work.

Explicit casts select metadata compatibility before lowering. Wide-to-thin casts
remove the length; integer addresses require thin pointers. Pointer comparisons
include length, and ordering compares the address before metadata. Existing LowIR
pointer/integer copies implement address casts: neither CPPGM nor the backend
vocabulary changed.

Constants retain allocation identity or absolute address bits, independently of
the selected metadata. A string-backed pointer reinterpreted as an array and then
unsized carries that array's selected count. Numeric pointer addresses can acquire
length without inventing an allocation. Pointer comparisons and pointer-to-integer
casts remain invalid during constant evaluation, including dead branches.
Mutable statics, general constant statements and raw-pointer constant patterns
are outside this release's grading contract.

## Public fixture and oracle review

The eight families cover sizedness, coercions, projected tails, pointers,
addresses, casts, constants and integration. Course-authored maintainer tables
supply most inputs; each manifest entry preserves its origin. Additional controls
exercise transparent/over-aligned storage, nested prefix offsets, UTF-8 byte
counts, zero-sized and empty tails, bounds aborts and destruction of temporaries.
Compile-only controls reify otherwise unconstructible tuple/packed-tail functions
to exercise their code generation without claiming to execute them.

Source/native outcomes were declared before oracle generation. The first public
probe wrongly predicted that an aligned element behind a packed slice must reject.
Pinned rustc accepts it, as it does the earlier array control. Keeping that result
led to a stronger reified-function test, which exposed a real tail-layout bug:
it reapplied a conservative containment restriction after declaration packing had
already been proved. Tail and sized layouts now consume the same completed
packing check. Unaligned references still reject. The failed plan and both probes
remain in `obj/pa22-work/{public-v1,packed-slice-probe-v1,packed-tail-probe-v1,packed-tail-reify-v1}/`.

The generated output was reviewed for address/length pairs, equal-address length
ordering, UTF-8 byte counts, packed raw-address forwarding, alignment offsets and
qualified abort paths. Positive outputs are parsed and linked by the unchanged
CPPGM backend. Rejections require empty stdout and diagnostics. No earlier PA
oracle was changed; all 7,040 previous byte cases pass.

## Qualification

The 67-gate release batch passed in 988.23 seconds. Candidate SHA-256:
`8e7586011415a27d7bd9dbebc16f3f516381e1be3012a5c22237c2e19837d892`.

| Check | Result |
| --- | --- |
| Release maintainer tests | 1,765 passed |
| Explicit native maintainer groups | 112 passed |
| PA1–PA22 public byte oracles | 7,368 passed; earlier 7,040 unchanged |
| Independent source checks | 7,167 passed; 11 declared stream fragments |
| Independent native checks | 3,877 passed; 2,272 execution and 137 compile-only comparisons |
| Formatting, strict Clippy and architecture/file audit | Passed; 1,061 audited Rust files, zero findings |
| Audit tests, harness, discovery, upstream, backend, token/literal/layout controls | Passed |

These totals retain earlier assignments' declared acceptance/runtime differences;
they do not claim unqualified rustc parity. The exact source and binary identities
and every gate log are in `obj/pa22-work/release-v1/validation-v1/`.
Preliminary public qualification is in `obj/pa22-work/public-v2/`; course-generated
oracles and their receipt are committed under `pa22/`.

Review replaced one negative const-local case: it reached PA25's unsupported
constant statements before testing pointer semantics. The replacement directly
takes a raw address of a temporary and reaches the intended PA22 rejection,
paired with accepted static/reference-backed addresses. Pinned rustc and the
unchanged candidate both reject it. The supplemental focused test, formatting,
Clippy, audit, regenerated oracles, all 328 text cases and discovery passed at
`obj/pa22-work/constant-fixture-review-v1/validation-v2/`. Its receipt verifies
that the compiler and every other fixture retain their qualified identities;
no unrelated full suite was repeated. The first supplemental script expected a
different diagnostic phrase; its failed assertion is retained separately.

## Performance and storage

The frozen release study uses published checkpoint `374ae1df` as its baseline,
SHA-256 `1a0b00f09ce414dfff8aa64463135dc4b7daaaeee336de8fc603d45693eda70a`.
The candidate is the exact qualified binary above. Sixteen shared inputs and
seven new pointer/cast inputs are independently rustc-qualified. The study retains
A/A calibration, five paired ABBA blocks with three executions per observation,
eight RSS observations and Callgrind profiles. New inputs rejected by the baseline
use candidate-only repetitions. Raw plans, hashes and every observation are in
`obj/pa22-work/release-v1/performance-v1/`.

All sixteen shared outputs are byte-identical. Instruction changes range from
−0.217% to +0.316%. The ordinary 256-function control takes 26.58 ms and changes
instructions by +0.150%. Projected depths 16/32 take 16.07/25.15 ms and use
27.21/54.04 million instructions, retaining the earlier approximately linear
proof work over that range. Shared paired wall medians range from −10.74% to
−0.24%, but calibration spreads and individual blocks are noisy; no general
wall-time speedup is claimed.

| New input | Count 32 | Count 128 | Count 512 |
| --- | ---: | ---: | ---: |
| Pointer functions: median ms | 6.55 | 14.31 | 46.82 |
| Pointer functions: million instructions | 16.95 | 63.52 | 249.02 |
| Pointer constants: median ms | 7.84 | 12.15 | 66.56 |
| Pointer constants: million instructions | 12.81 | 48.82 | 190.26 |

The 128→512 instruction ratios are 3.92 and 3.90 for four times the input. These
are measured families, not universal complexity claims. The largest constant
case has a 56.2% calibration spread, so its wall-time ratio is not a complexity
estimate. Median RSS at count 512 is 14,762 KiB for functions and 14,720 KiB for
constants. Reified packed-tail code generation takes 3.84 ms.

Shared-input median RSS increases by 56–1,258 KiB; the largest increase is the
256-tuple layout control. This release does not claim memory-neutral behavior.
GNU `size` reports 32,768 additional bytes across text, data and BSS.

All compilation, execution and measurement batches run through the storage guard
with the 40 GiB reserve. It removes disposable generated binaries while preserving
source, diagnostics, observations and receipts. Full qualification removed 8,700
generated files, reclaiming 8.11 GiB; measurement removed its 23 object files.
The large full scan is reserved for suites with scattered output; focused batches
use explicit cleanup roots. Only current immutable comparison binaries remain pinned. The active C++ and v5rust checkouts were not modified.
