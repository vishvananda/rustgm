# Source-library prerequisites checkpoint, 2026-10-23

This checkpoint qualifies the compiler source at `b6e30beddbc1b27afb514ca63b2f9fc19bbb3792`.
Later integration adds the already tested PA34 planning adapter and this record;
the compiler source remains byte-identical. PA25–PA44 remain planned. Original
upstream core, alloc and std and their real crate consumers are still required.

## Included behavior

- Constant preparation retains the actual defining parent and complete paired
  proof while reusing original components. Structural completion does not grant
  caller permission. Repeated inline constants preserve lifetime and trait debts.
- Source-library feature gates now check actual configured syntax roles for
  `never_type` and `stmt_expr_attributes`. An experimental assembly request grants
  no capability by itself. The ordinary source path retains its cold-state fast path.
- Opaque preparation validates complete original function identities before
  publication and uses an append-only ready frontier keyed by the exact opaque
  declaration. It does not rescan the entire type store after each completion.
- A private source/imported ownership catalog establishes compact, checked
  identities and lazy diagnostic labels. This is foundation work: it does not
  enable imported generic bodies or assign a new metadata wire profile.

The next work remains source-free generic/const body transport, actual source
Copy/Clone/callable and const-trait consumers, string byte-storage consumers, and
complete crate-build/edition integration. The source-library implementations
must replace the bootstrap library factories; ordinary library algorithms do
not move into the compiler.

## Functional qualification

One frozen release Cargo invocation ran ordinary and ignored maintainer tests
together: **3,158 passed**, without a duplicate full private-library pass. All
**8,362** committed active assignment text cases matched. Formatting, strict
all-target/all-feature Clippy, architecture, harness, upstream and backend checks
passed on the same frozen snapshot.

Affected independent suites passed:

| Suite | Independent source checks | Native qualification |
| --- | ---: | --- |
| PA21 | 748 | 450 checks, including 264 executable comparisons |
| PA24 | 470 | 470 checks, including 313 executable, 3 codegen-only and 10 object comparisons |

The existing independently observed new-feature inputs and worker receipts are
retained. Three newly frozen parent-port scaling sources additionally compiled
and executed with pinned Rust 1.98.0 and with the combined RustGM/CPPGM path.
Executable results were zero with empty output. These are bounded checkpoint
controls, not whole-library acceptance or an assignment release.

The immutable release CLI SHA-256 is
`68298a44a57966dc802a1dc0ea76292ec3c822d1c5f3c0e3040627b5a44ce7a5`.
Local evidence is under `obj/parallel/std-prerequisites-combined-v1/` in the
validation checkout. Its `functional.json` binds the source snapshot, candidate
and complete check receipts. Post-run cleanup reclaimed 16.87 GiB and losslessly
archived one large diagnostic while retaining inputs, observations and pins.

## Focused performance qualification

Compare the immutable candidate against the published `427a7877…` compiler with
frozen inputs, one exclusive resource lease, A/A calibration, five paired ABBA
blocks, retained raw observations, output equality, RSS and instruction counts.
Eight shared controls cover ordinary constants/scalars, generic instance widths
and opaque byte reads. All shared outputs match byte for byte. Instruction
deltas range from **−0.77% to +0.35%**. Maximum positive RSS delta is **282 KiB**.
No declared instruction, joint RSS or isolated-width scaling trigger fired.
Wall-time medians vary within the observed calibration noise; they are not
evidence of a speedup. Opaque width and repeat count vary together, so this
series is not an isolated queue-width complexity measurement.

The published compiler rejects all three parent-port sources. Their results
are candidate-only capability/scaling observations, with no A/B speedup claim:

| Parent ports | Median compile time | Instructions |
| ---: | ---: | ---: |
| 8 | 11.25 ms | 18,723,263 |
| 32 | 35.71 ms | 63,880,358 |
| 128 | 96.49 ms | 246,117,526 |

Instruction ratios are 3.41 and 3.85 for successive fourfold input growth.
Private invariants independently retain the actual preparation-work counts and
prove that original-component reuse does not introduce growing work per port.
All observations, including baseline rejections, are retained under
`obj/parallel/std-prerequisites-perf-v1/`. `measurement-complete.json` binds the
analysis and immutable measurement scripts; no favorable retiming was performed.
