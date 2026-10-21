# Inherited closure lifetime scaling

The development study compares immutable compiler `4a449961` (A, SHA256
`38ac5f0d73fc5a40ccb0a6c794d02cead90cf9ea23ecf516fed358f5ff49483a`)
with `18637269` (B, SHA256
`5d62c74e1046eb4aa04c821f08b1fbb38459424981ee4686ec0813fcfd277dc9`).
B includes the PA29 scalar metadata checkpoint as well as inherited closure
lifetime support. This comparison does not isolate one function's cost and does
not measure the later atomic, assignment-operator or constant-view integration.
A was built with all features; B used default features. The only optional feature
controls interop tests, but binary layout may still differ. Retained same-source
default-feature A has SHA256
`2eeb8a1f77562b563196ab8e7f4cf68b7732538c2664c1b70dabb3a3f7341fa2`.

Twelve frozen inputs vary ordinary functions, generic closures, inherited closure
lifetimes and nested inherited closures at widths 8, 32 and 128. Every input is
independently object-qualified by rustc 1.98.0. Repeated calls demand each generic
instance three times. The resource controller granted an exclusive lease;
processes were pinned to one allowed CPU. Every measurement verifies output
bytes. Raw observations, input hashes, A/A calibration, five paired ABBA blocks,
RSS observations and Callgrind files are retained under
`obj/parallel/closure-inherited-performance/measurement-v1/`.

All six behavior-equivalent A/B inputs produce identical LowIR. Their instruction
counts change by less than 0.2% in either direction. The newly supported inputs
are rejected by A, so they have no A/B speedup ratio. Their B instruction counts
are:

| Distinct instances | Inherited closure | Nested inherited closure |
| --- | ---: | ---: |
| 8 | 8,656,658 | 10,552,975 |
| 32 | 29,986,833 | 36,451,339 |
| 128 | 115,744,510 | 140,581,259 |

Fourfold input growth gives approximately 3.45–3.86 times as many instructions,
consistent with linear work plus fixed startup costs. Median B peak RSS for the
128-instance inputs is 11,376 KiB and 12,110 KiB respectively. Its text section
is 38,632 bytes larger (about 0.44%).

Wall-time calibration is noisy on this shared host: the A/A spread reaches 89%
on a shared generic control. Paired wall deltas range from about 0.5% to 12.1%
slower, with inconsistent block-level results; these measurements do not establish
an improvement or a reliable regression magnitude. The new 128-instance medians
of 45.8 ms and 56.1 ms are observations on that host, not performance guarantees
or comparisons against rustc. Instruction growth and output equality provide the
useful algorithmic evidence here. Keep the full release performance matrix as a
separate requirement.
