# Ordering, slice access and metadata checkpoint

This checkpoint combines shared bounded constant observations, contextual
reference-argument preparation, sequence ordering, raw slice indexing, canonical
DST field metadata, and fixed C callable metadata. It does not activate PA25–PA30.
Their broader constant execution, iterator, unsafe-memory, intrinsic, generic
crate and foreign-interface requirements remain open.

## Candidate and functional qualification

Production cutoff: `213f1e21ccb737048f8ada50a3aa62124901a294`.
Test-only repair: `156d8cc431cce4fb552c8d5085f4cc9c05e1d425`.
Immutable executable SHA-256:
`9c6b81a88558e23cf9d61c9571dd1b9f9f7998c03b6279a50b562d4181faf8fd`.
Evidence is retained under
`rustgm-validation-work/obj/parallel/ordering-slice-metadata-checkpoint-v2/`.

- 2,524 regular tests are qualified. The initial complete release run had one
  failure: a test explicitly required an independently Rust-positive DST example
  to remain unsupported. The repair removes that obsolete expectation and its
  native exclusion, without changing the input or declared outcome. The affected
  inference group then passed all 143 tests; unchanged passing groups and private
  invariants retain their original evidence. `cargo-qualification.json` records
  the original nonzero status and exact replacement group; it does not describe
  the failed command as passing.
- All 257 native maintainer tests pass. The repaired example executes through the
  ordinary native harness.
- Strict all-target/all-feature release Clippy, formatting, the syntax-aware
  file/architecture audit, registration, harness, layout, upstream attribution,
  and both backend capability checks pass.
- The active PA1–PA24 inventory contains 8,360 text cases. Every status and stderr
  matches. Before scoped regeneration, 8,309 stdout files matched exactly and
  51 differed as described below.
- Independent source checks pass for PA6, PA13, PA14, PA16, PA17 and PA22;
  the affected native suites pass for PA13, PA14, PA16, PA17 and PA22.

The default and all-features builds reuse identical production artifacts.
No feature-disabled compiler is substituted for the tested candidate.

## Reviewed text changes

The 51 changes belong to PA6 (1), PA13 (36), PA14 (1), PA16 (4), PA17 (1)
and PA22 (8). Fixtures and declared outcomes remain unchanged.

PA6 appends canonical comparison/Ordering/Option names, scopes and declarations.
Every existing name, scope, declaration, binding and source reference is exact.
Forty-nine LowIR outputs remove unused wildcard address calculations. Review
substitutes pure address/index and unchecked integer-offset expressions and
renames SSA temporaries only to establish equivalence; control flow, checked
operations, calls, loads and stores remain exact. The actual course comparison
continues to compare complete bytes, without normalization.

The remaining PA16 loan view removes one unused indexed place and two wildcard
projection statements. Frozen before/after typed body views prove that all other
operations are unchanged. The same two loans, region membership and activation
states match exactly after applying the resulting place/statement/point map.
`output-review-final.json` binds all original and candidate output hashes and
the retained review scripts. Only these selected cases are regenerated.

After scoped regeneration, all 8,360 active text cases pass byte-for-byte, and
layout, harness and upstream checks pass again. The final qualification receipt
`complete.json` has SHA-256
`9feb37c4438d4cc4e70c2e4931d08da582191c9fa6dbdbb57efd6acf06b6967f`.

## Performance

Baseline: public `17f126b68296f50e3e4190a7f58efc4077ea95f6`, executable
`e25d01ba2b6fbd352880e4397f4fe2b0aebeb8fe9816d5c2b5353e4ebc6d4963`.
All 34 frozen sources independently compile to Rust objects; 13 also compile
with the baseline and have identical stdout/metadata. The other 21 exercise
newly supported behavior and carry no baseline speedup claim.

Evidence: `rustgm/obj/parallel/ordering-slice-metadata-performance-v1/`.
The immutable A/B run retains six A/A observations, five paired ABBA blocks
with three executions per observation, eight RSS observations and instruction
profiles. An exclusive controller lease excludes other registered RustGM jobs;
unrelated host workloads are not isolated.

| Input | Candidate median | Instruction change |
| --- | ---: | ---: |
| Observation graph 8, contexts 128 | 27.39 ms | −6.24% |
| Observation graph 128, contexts 128 | 52.43 ms | −24.02% |
| Ordinary 512-constant control | 12.87 ms | +0.12% |
| Rust scalar metadata, 128 functions | 8.47 ms | +0.21% |
| Ordering, 128 calls | 9.30 ms | newly supported |
| Raw reads, 128 operations | 9.67 ms | newly supported |
| C scalar metadata, 128 functions | 12.44 ms | newly supported |
| C record-reference imports, 128 uses | 12.86 ms | newly supported |

The largest observation input's paired wall median improves 18.17%. Other wall
changes must be read with their retained calibration and block variability; this
does not establish a general compiler speedup. Ordinary/metadata instruction
growth is 0.12–0.30%. The growing 8/32/128 series have incremental instruction
ratios of 3.99–4.56 per fourfold input increase. Extent-only ordering and raw
read controls remain effectively flat. No declared instruction/scaling threshold
is exceeded.

Eight shared cases trigger the predeclared RSS investigation, increasing
574–1,064 KiB (5.5–10.3%). The single declared followup profiles all eight and
the largest observation input with both heap and mapped-page Massif modes.
Metadata and ordinary scalar/pointer heap peaks are unchanged; ordinary float
is +24 bytes, the small observer is −1,272 bytes and the largest observer is
+1,760 bytes. Mapped-page peaks grow 24 KiB, or 12 KiB on the largest observer;
the executable text section grows 15,968 bytes.

These profiles do **not** explain the full native RSS increase. The raw increase
is retained, and this checkpoint makes no memory-neutrality claim. The sampled
allocation profiles show no unbounded heap growth, while native physical page
residency remains an explicit attribution limitation. All observations, profiles,
declarations and hashes are preserved in `attribution-v1/`.

The separately attributed quadratic generic cleanup/liveness algorithm remains
required work; the bounded-observation improvement does not resolve that issue.
