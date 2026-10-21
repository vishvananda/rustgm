# Initializer ownership publication qualification

The public compiler checkpoint `bbe280e2` passed the full release qualification:
2,107 Cargo tests, 177 native maintainer groups, all 7,787 active text oracles,
all active independent source/native suites, strict all-target/all-feature Clippy,
formatting, architecture/layout audits, harness and upstream verification.
The exact commit and all-feature binary hash are retained in
`rustgm-validation-work/obj/parallel/nested-corrected-v2/release-complete.json`.

Affected-path timing compares default-feature release executables. The prior
integrated candidate is A (`a32e0b10…`); the ownership correction is B
(`5bff94e7…`). The all-feature qualification binary was separately pinned before
building B. Both executables and every input were hashed before and after the
exclusive measurement. No worker build or test overlapped measurement.

The protocol uses CPU 2, two warmups, four A/A observations, five paired ABBA
blocks with three executions per observation, eight RSS observations and
Callgrind instruction counts. All nine supported shared outputs are byte-identical.
The three borrowed-closure inputs are newly supported and cannot supply an A/B
speed ratio. Pinned Rust 1.98.0 independently produced objects for all twelve.

| Family / width | Candidate ms | Paired wall change | A/A spread | Instruction change |
| --- | ---: | ---: | ---: | ---: |
| ordinary-8 | 3.026 | -1.216% | 3.726% | +0.151% |
| scalar-initializers-8 | 3.136 | +0.253% | 1.632% | +0.443% |
| value-closures-8 | 3.794 | +1.215% | 13.836% | +1.355% |
| borrowed-closures-8 | 4.057 | — | 5.121% | new support |
| ordinary-32 | 3.532 | +2.227% | 7.215% | +0.057% |
| scalar-initializers-32 | 3.730 | +1.496% | 31.663% | +0.715% |
| value-closures-32 | 6.256 | +1.341% | 49.192% | +1.770% |
| borrowed-closures-32 | 6.961 | — | 1.720% | new support |
| ordinary-128 | 4.736 | +2.804% | 10.339% | +0.053% |
| scalar-initializers-128 | 6.015 | +1.895% | 8.246% | +0.787% |
| value-closures-128 | 14.334 | +2.524% | 3.450% | +1.658% |
| borrowed-closures-128 | 17.987 | — | 4.185% | new support |

The changed initializer paths add measured work: roughly 0.4–0.8% for scalar
initializers and 1.4–1.8% for value-returning constant closures. Ordinary controls
add 0.05–0.15%. Instruction growth remains approximately linear over 8/32/128
independent initializers; this is evidence at those sizes, not a universal bound.
All shared paired wall medians are inside their corresponding A/A spread, so
these noisy short executions do not establish a wall-time regression or speedup.
Median RSS changes range from -644 to +318 KiB on shared inputs; raw observations
are retained without claiming a causal memory improvement.

The first preflight exposed missing `main` functions in every generated family.
Both compilers rejected those library-shaped inputs in executable mode. Its
declaration and failed receipts remain intact. Revision 2 adds only executable
entry calls, freezes new hashes and preserves the original expected outcomes.
No timing was collected on the rejected revision.

Evidence: `rustgm-validation-work/obj/parallel/initializer-owner-performance/`
contains immutable pins, build provenance, both preflights and `measurement-v2/`
with every observation, output hash, RSS value and Callgrind profile. The input
declarations are in `rustgm/obj/parallel/initializer-owner-performance/`. This
measurement qualifies the initializer ownership correction; later closure,
associated-constant and operator integrations need their own qualification.
