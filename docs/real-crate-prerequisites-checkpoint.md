# Caller alias checks, constant patterns, integer ranges and enum storage

This checkpoint extends public `11200c52203e729e8354cd29ef9de9ca16fff7b3` with
rigid original type parameters in constant patterns, half-open integer Range
iteration, direct-tag Rust enum byte storage, and a sparse frontier for
proof-only caller alias checks. The [library bootstrap order](library-bootstrap-roadmap.md)
remains source-declared bindings and generic crate transport, followed by
upstream core, alloc and actual crate builds. PA25–PA30 remain incomplete.

The proof optimization follows the previous checkpoint's instruction profiles.
It seeds only rigid nodes with incoming requirements when checking plain alias
graphs. It preserves projection checks, required edges, universe promotion,
entailment and budget accounting. Rich graphs use the unchanged complete solver.
The result carries charged work, not a reusable proof or caller permission.
See [the algorithm and equivalence controls](pa25-alias-frontier.md).

[Rigid constant patterns](pa25-rigid-patterns.md) retain the actual original
caller's parameters and prove its debts before evaluating a parameter-independent
typed body. Generic const parameters, generic callees and conditional nested
output contracts remain required. The new parent-component invariant verifies
why the actual original parent and its contract must accompany a nested body;
it does not implement the full specialized-output consumer.

[Half-open integer ranges](pa26-integer-ranges.md) use checked ordinary next
bodies for the twelve integer types, including inference from the actual Item
relationship. Inclusive, from, character and clone cases remain required. This
is the last in-flight bootstrap Range checkpoint; ordinary upstream APIs should
come from their real Rust source rather than further convenience-method factories.

[Direct-tag enum byte storage](pa27-default-tag-enum-bytes.md) consumes this
compiler's completed enum layout and preserves initialization, validity,
provenance and access restrictions. Physical field addressing does not force
ordinary typed closure-bearing enums into an unsupported byte family. This makes
no claim that implementation-defined Rust enum layouts match rustc byte-for-byte.

## Functional qualification

The tested production source commit is
`5e498da982530c3a4c5e20efeb354dc4a607e3f0`; immutable release executable SHA-256:
`a3c95faac213598806871f5fc79fa176d3500bbaa49cefe1f958d90eece8b162`.
Test-only correction `a8347e4108417c6f9ac3ac1d2ba8848dddd51ced` leaves both
production source and that executable unchanged. Subsequent documentation is
separate from the measured snapshot.

The combined qualification covers 2,739 regular tests and 289 native tests.
The complete regular run passed 2,738 and found one stale expectation in
`c_enum_byte_source_contracts`: the original default-layout enum control still
expected rejection. Its unchanged source was already independently qualified by
pinned Rust, the new enum suite and native execution. Only its two test
declarations changed; exact source and native replays pass. The original failure
is retained, not rewritten as a successful full run.

Formatting, strict release all-target/all-feature Clippy, registration,
the 1,842-file syntax/architecture audit, 98 harness tests and upstream checks
pass. All 8,362 active PA1–PA24 text cases are byte-exact; no reference change is
needed. Planned-stage source/native controls execute through the registered
maintainer suites without activating their assignments.

Affected independent suites pass 2,511 source checks across PA6/12/13/20,
12 PA12 layout checks and 1,464 native checks including 924 executable
comparisons. Functional receipts are in the validation checkout under
`obj/parallel/real-crate-prerequisites-integration-v1`, `-v2` and
`real-crate-prerequisites-independent-v1`. The final functional receipt hashes to
`77aeffa2bfa89bec311640169ae7340ed60a65118e730d9f344efb2561dbaa48`.

The old failed batch's generated artifacts were pruned after diagnosis, freeing
30.40 GiB while retaining receipts, input/output text and current compiler pins.
Eight large diagnostics were losslessly archived. Cleanup is recorded in
`obj/cleanup/1792701991198292676.jsonl`; the focused repair's successful wrapper
also cleaned its new artifacts.

## Measurement contract

The main checkout retains 51 frozen inputs under
`obj/parallel/real-crate-prerequisites-performance-v2`. The prior public compiler
is baseline SHA-256
`2912a6b9affeb0ae0901f8ceeee30e0ae17313295b2189c43ecf072495492255`.
Nineteen shared controls have byte-exact output; 32 newly supported inputs
provide scaling measurements without an unsupported-baseline speedup claim.

Axes vary observer tuple width and context count independently, integer loop
count and unrelated source, enum nesting/repetition/unrelated types, and rigid
parameter width/context count/unrelated source. Original source qualification
is retained, with 14 new inputs independently qualified before observation.
All 50 LowIR inputs pass native checks. Enum inputs whose main is empty rely on
their independently checked compile-time assertions for semantic evidence;
their native runs establish backend compatibility only. The ordinary512 library
control retains its original constants-inspection mode and independent source
qualification; it does not produce a native input in this measurement.

The initial preflight mistakenly classified that library control as a LowIR
executable, causing both compilers to reject its missing main. The second
declaration restores only its original mode/kind, preserving every source,
binary, other command and measurement policy. The failed preflight remains in
`real-crate-prerequisites-performance-v1`; no timings preceded the correction.

Measurements use an exclusive resource lease, one CPU affinity, two warmups per
binary, four A/A observations, five paired ABBA blocks with three executions per
observation, eight RSS observations and instruction profiles. Every observation
checks exact output. The predeclared attribution gates cover instruction/RSS
regressions and superlinear scaling separately; original observations are never
replaced with favorable retries.

## Measured results and attribution

All 51 measurements completed. The four declared triggers were two instruction
regressions, one RSS regression on the same family, and one observer-width
scaling trigger. No samples were replaced or favorably rerun.

| Input | Instructions A → B | Instruction change | Paired wall change | RSS change |
| --- | ---: | ---: | ---: | ---: |
| Observer, width 128 / contexts 8 | 62,749,449 → 62,062,896 | -1.09% | -0.40% | +18 KiB |
| Observer, width 128 / contexts 32 | 107,659,737 → 104,695,537 | -2.75% | -0.89% | -120 KiB |
| Observer, width 128 / contexts 128 | 287,277,761 → 275,098,745 | -4.24% | -2.64% | -238 KiB |
| Ordinary512 library | 43,686,792 → 43,854,605 | +0.38% | -1.47% | -198 KiB |
| Ordinary record address, depth 8 | 10,583,052 → 11,526,206 | +8.91% | +9.86% | -6 KiB |
| Ordinary record address, depth 32 | 24,080,388 → 27,054,800 | +12.35% | +13.30% | +1,064 KiB |

The nine observer cases all reduce instruction counts (0.71–4.24%). Other shared
controls, excluding the ordinary-record address family, range from −0.62% to
+0.55%. Wall differences retain their calibration limits: the largest observer
has 1.64% A/A spread, ordinary512 15.15%, and depth 32 addresses 10.70%. These are
frontend-process measurements, not a rustc comparison or native build times.
The exclusive lease excludes registered RustGM work, not unrelated host activity.

The single declared attribution pass under
`obj/parallel/real-crate-prerequisites-attribution-v1` reconciles all 70 retained
instruction profiles and takes exactly four allocation profiles: baseline and
candidate for the triggered address family and an ordinary scalar control.
It performs no new timing run. Inclusive call costs and exclusive instruction
costs are retained separately and must not be added together.

The sparse caller-alias path at width 128 now spends 127,290 instructions in
`Constraints::check_aliases` for 16 calls at 8 contexts, and 2,083,421 for 256 calls at
128 contexts. The separate width-axis trigger remains: at 8 contexts, width 8/32/128
costs 9,859,824 / 15,968,950 / 62,062,896 candidate instructions. Incremental
ratios are 7.51 for the baseline and 7.55 for the candidate. Source inspection
identifies a separate preexisting producer/consumer path: liveness enumerates
active region/parameter components at each point, then loan preparation remaps
all pairs and point inference hashes them into sparse words. On width 128 at
8 contexts, the point solver's insert path consumes 24.45 million inclusive
instructions and loan finishing 32.64 million; these nested costs are not additive.
An interval or word-based producer/consumer contract should be investigated,
with exact CFG, cleanup, imaginary-edge and borrow-birth semantics preserved.
Changing only the hash implementation would leave the materialized product.

The newly regressed ordinary-record address family has a different cause.
Keeping address-mode selection separate from cached logical enum-field selection
correctly avoids reusing an inappropriate address. The implementation currently
bypasses projection caching for every address, including ordinary record chains.
For 128 repetitions at depths 8/32, the candidate invokes `Evaluator::projection`
1,152/4,224 times and retains new reference-path descriptors. Massif peak heap
grows from 963,635 to 1,618,995 bytes at depth 32; peak total grows from 1,062,240 to
1,718,384 bytes. The scalar control's peak heap stays 25,338 bytes. This is a measured
regression with a concrete repeated-work path, not an unqualified noise claim.
Native RSS and Massif allocation accounting remain different measurements.

The next memory checkpoint must preserve separate logical/address cache identity
and all existing invalidation, initialization, provenance and access checks while
reusing completed address paths. It should verify actual cache/Frame layouts and
vary depth and repetitions independently. This regression and the preexisting
liveness width cost remain open in this publication; correctness qualification
and the caller-alias improvement do not claim overall performance neutrality.

Source-built library bindings, generic cross-crate bodies and their complete
original/instance contracts remain the implementation priority. The separate
closure-storage checkpoint and source-binding work are not included in this
compiler hash. The [generic metadata proposal](pa29-generic-transport-proposal.md)
now distinguishes original proof-context rows from executable template lookup;
its implementation remains required.

Frozen measurement plan SHA-256: `fe408a263cfe808defa4e59049d4c105977b8c835b99bdc2d426351b3557d9b6`.

All raw result summaries SHA-256: `2dec6a6c9ac214f13df5f14fc8635753078c41064b2951751d108e207d9ec9d0`.

Declared-gate analysis SHA-256: `ea84768b8acafab527a59f14f5bb526e5a90acd5de034112e059f61e840f6889`.

Attribution completion SHA-256: `2b43bf3ee543737158a413c433c6587637a00af7bf2ba8c76c4184645ef75788`.
