# Caller proofs, mutable iteration, nullable storage and crate arrays

This checkpoint extends public `983f6524` with paid caller lifetime premises,
mutable slice/array iteration, nullable thin-reference/function-pointer Option
layout, and fixed-array/C-record metadata with compressed stored arrays and
typed static element addresses. It preserves the upstream core→alloc→std
implementation priority in [the bootstrap plan](library-bootstrap-roadmap.md).
PA1–PA24 remain active. PA25–PA30 remain planned and incomplete.

## Functional qualification

The tested compiler source commit is
`a6473049c8d225c10ab71c9c0ce52aad178ed922`. The immutable release executable
SHA-256 is
`2912a6b9affeb0ae0901f8ceeee30e0ae17313295b2189c43ecf072495492255`.
Generated references and this report do not change that compiler source.

The validation checkout retains
`obj/parallel/iteration-storage-integration-v3` and `-v4`: 2,710 regular tests,
286 native tests, strict release all-target/all-feature Clippy, formatting,
registration and the 1,818-file syntax/architecture audit pass. The complete
regular run passed 2,704 tests and found five stale expectations. Four measured
Declarations size assertions needed the two new optional Option-layout caches;
one unpaid lifetime case needed the earlier caller-premise rejection diagnostic.
All five exact tests pass after correction, as does the newly published atomic
backend parser test. The production executable stayed byte-identical throughout
these test-only repairs. Full original logs and the test-name reconciliation
remain in the evidence; an interrupted or incomplete run is not counted.

Declarations grows from 8,552 to 8,568 bytes: one optional pointer in aggregate
definitions and one in ABI layouts. Type Kind and IR Value remain 24 bytes each,
Resolution 2,152, and inference Context and Store 448 each. Ordinary compilation
does not allocate the nullable-type catalogs.

All 8,362 active PA1–PA24 text cases are reconciled. The full comparison found
exactly the three predeclared deltas: appended mutable-iterator names in PA6,
the actual caller/selected lifetime identities in one PA20 semantic view, and
an earlier caller-alias rejection in one PA20 negative. All statuses match.
Only those three sidecars were generated through the course runner, after
independent qualification; their exact observed hashes and scoped comparisons
pass. The generated provenance batches and the 44-stage placement check pass.
Evidence is in `obj/parallel/iteration-storage-reference-v1`.

`obj/parallel/iteration-storage-independent-v2` retains 218 PA6 independent
source checks, 1,382 PA20 independent source checks, and 836 PA20 native checks,
including 508 executable comparisons. All pass, along with 98 harness tests and
upstream source integrity. The first runner stopped before testing because Git
quoted a Unicode filename; the retry uses NUL-delimited paths and changes no
compiler or fixture. New planned-assignment source/native controls also execute
through the registered full maintainer suites above.

## Compatibility and measurement inputs

The main checkout's `obj/parallel/iteration-storage-compat-v1` verifies eight
original metadata controls. Both the previous public compiler and this compiler
preserve the archived LowIR and metadata bytes for profiles zero through six,
including the namespace envelopes. Their earlier independently qualified source
inputs are unchanged. The new array/storage family uses profile seven; no old
wire meaning is repurposed.

The measurement matrix retains all 43 frozen inputs: 16 previously supported
controls and 27 newly supported cases. The latter were independently qualified
first: 18 native programs and nine library objects, followed by the same emitted
LowIR through the frozen CPPGM backend. New inputs vary paid callers, mutable
iteration, Option decoding, stored array roots/byte payloads, and zero-sized
repeat extents up to 8,589,934,593 elements. Candidate-only inputs cannot support
a speedup claim against the unsupported baseline.

Thirteen shared inputs preserve byte-identical compiler output. Three original
shared-iteration inputs intentionally change their Option<&u8> representation:
16-byte tagged storage becomes an 8-byte nullable pointer, the return convention
changes accordingly, and tag checks become null checks. Their source bytes and
control-flow topology remain unchanged. Both compilers' emitted programs execute
successfully, as do three additional observable-value controls independently
compiled with pinned rustc. These controls supplement the original inputs;
they do not replace or simplify them.

The first preflight correctly stopped on those three output differences before
any timing. `obj/parallel/iteration-storage-output-review-v1` retains the full
diffs, source declarations, six independent executions and twelve A/B native
executions. The amended measurement protocol retains both raw output hashes and
checks every observation against its own immutable compiler's exact bytes. It
reports the three reviewed layout changes separately from byte-identical output.
The original failed preflight is retained under `iteration-storage-performance-v1`.

## Performance results and attribution

The main checkout retains the complete 43-input run under
`obj/parallel/iteration-storage-performance-v2`. Baseline SHA-256 is
`f72651996370b67c3d2f1af68b8aa1baada6379206bc62e3128a63d563a64e56`;
candidate is the unchanged executable identified above. The protocol uses an
exclusive resource lease, one CPU affinity, two warmups, four A/A observations,
five ABBA blocks with three executions per observation, eight RSS observations,
and instruction profiles. Every observation verifies its frozen output. All raw
samples, calibration, host-load records, profiles and the predeclared analysis
policy remain available. No original observation was replaced or favorably rerun.

The three lifetime-observer controls retain a 128-reference generic argument
while varying inline contexts. Their costs increased; this checkpoint does not
claim a performance improvement:

| Shared input | Instructions A → B | Instruction change | Paired wall change | RSS change |
| --- | --- | --- | --- | --- |
| Observer, 8 contexts | 61,352,949 → 62,782,891 | +2.33% | +2.26% | −10 KiB |
| Observer, 32 contexts | 102,075,202 → 107,672,627 | +5.48% | +7.32% | +160 KiB |
| Observer, 128 contexts | 264,809,236 → 287,169,822 | +8.44% | +8.22% | +456 KiB |
| Ordinary 512 | 43,659,120 → 43,686,606 | +0.063% | +0.99% | +654 KiB |
| Flat byte repeats, 128 | 52,060,748 → 52,532,312 | +0.91% | −0.56% | +808 KiB |
| Shared iteration, 128 loops | 223,351,471 → 224,493,678 | +0.51% | −0.009% | +992 KiB |

Across the other thirteen shared controls, instruction changes range from
−0.36% to +0.91%. Wall differences must be read with calibration: observer32
has 15.06% A/A spread and ordinary512 28.96%; observer128 has 1.21%. The exclusive
lease excludes registered RustGM jobs, not unrelated host activity. These are
frontend process measurements, not end-to-end native build times or a new rustc
comparison.

The declared followup, `obj/parallel/iteration-storage-attribution-v1`, reconciles
all instructions in fourteen existing profiles and records eight new allocation
profiles without repeating timings. The observer's new `Inputs::aliases` call
path costs 1,301,718 / 5,230,737 / 20,928,857 instructions at 8 / 32 / 128 contexts,
with 16 / 64 / 256 calls. At 128, its constraint solve costs 14,307,510 and input
seeding 4,077,067 instructions. These are nested inclusive costs, not independent
numbers to add. The path includes some equation handling previously performed
elsewhere, so its entire cost is not claimed as the regression's removable share.

Source inspection explains the avoidable repeated work: each inline caller
constructs a graph for its whole universal prefix, and the solver seeds rigid
self-pairs even when they have no incoming requirement. The fixed-width series
is linear in context count, but repeated prefix work can multiply independent
width and context dimensions. The next proof checkpoint must investigate
demanded graph construction and rigid-frontier work while preserving actual
caller obligations, conditional-output observations and identity ownership.
No successful proof may be cached merely from shared type shape. This measured
cost remains open; the current correctness checkpoint retains the full checks.

The three RSS triggers are also retained rather than dismissed as noise. The
single declared Massif A/B pass, including stack and allocation overhead, reports:

| Input | Peak heap A → B | Peak total allocated A → B |
| --- | --- | --- |
| Ordinary 512 | 2,092,202 → 2,092,202 B | 2,179,064 → 2,178,456 B |
| Flat byte repeats, 128 | 1,893,621 → 1,893,621 B | 1,962,648 → 1,962,072 B |
| Shared iteration, 128 | 3,492,666 → 3,428,842 B | 3,660,664 → 3,596,736 B |
| Ordinary scalar control | 30,870 → 30,811 B | 173,520 → 177,840 B |

Heap and total peaks can occur at different snapshots. These profiles show no
corresponding heap increase in the measured families; they do not explain native
RSS. Executable text grew by 118,048 bytes and data+BSS by 752 bytes, which also
does not establish the cause of the larger resident-set deltas. Stack samples,
code residency, allocator behavior and process mappings are not interchangeable
with retained compiler heap. Native RSS attribution therefore remains incomplete;
the original increases stay reported as a limitation for subsequent checkpoints.

All twelve 8/32/128 scaling series stay below the declared sixfold trigger. Their
incremental instruction ratios range from 3.80 to 4.06 for a fourfold increase
in incremental input work. This establishes only the measured axes. Zero-sized
repeat extents 8 / 1,048,576 / 8,589,934,593 take 1,038,933 / 1,041,334 / 1,042,413
instructions, with 11,462 / 11,824 / 11,502 KiB RSS; emitted output grows from
352 to 406 bytes. The stored representation does not allocate by repeat extent.
New-only workload wall medians range from 3.16 to 62.05 ms and are retained as
candidate baselines, not speedups over an unsupported old compiler.

## Remaining work

The rigid generic-pattern and half-open integer Range worker checkpoints are
reserved for the next integration; they are not included in these counts.
Generic const parameters/callees and conditional parent results, broader
iteration/desugaring, general enum byte consumers, intrinsic/source-library
bindings, generic crate-body/contract transport and remaining foreign-interface
requirements are still required. The atomic matrix published in `983f6524`
establishes backend capabilities, not a completed source intrinsic interface.

Source-built core remains a separate milestone. Work now targets real attributed
intrinsic declarations, source-owned language identities and the distinct sizing
hierarchy, alongside generic metadata. Ordinary upstream APIs must come from the
library sources. This checkpoint makes no claim to compile core, alloc, std,
Serde or their complete dependency graphs.
