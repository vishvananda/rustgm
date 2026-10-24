# Projected Copy, macro allowances and contextual enum aliases

Compiler snapshot `5602dc27` combines the actual declaration/body alias scope
for projected Copy proof, complete local macro allowance lists, and enum-alias
suffix context and privacy. The unchanged upstream core probe now passes the
previous `panic_internals` allowance and stops at `pub macro assert_matches`,
line 170. It still opens the same four original source files. Complete core
expansion, semantic checking and a crate artifact remain unimplemented.

PA1–PA24 remain implemented drafts; PA25–PA44 remain planned. This checkpoint
does not activate an assignment or enable scoped macro execution. The component
contracts are [projected cleanup](pa31-projected-defining-cleanup.md) and
[internal allowances](pa31-internal-allowances-handoff.md).

## Functional qualification

The full release run at `4653f1a7` retained 3,682 passes and ten failures.
Three private and seven frontend projection cases exposed a real interaction:
the callable Copy/Clone shape classifier propagated `Normalization` for a
projection which the ordinary solver still needed to resolve. A bounded trace
of the actual optimized executable confirmed this path. That run is not
reported as passing.

The two-file repair returns "not this callable shape" only for that specific
normalization state, allowing ordinary selection to proceed. It does not cache
absence, grant Copy, or suppress invalid identities and exhausted budgets.
New private controls exercise both Copy and Clone, genuine unresolved
projections, invalid inference identities and materialization-budget errors.

The recovery ran all private, frontend and trait release tests, including their
ignored native cases: **2,084 fresh passes**. The other eleven unchanged
integration targets retain **1,609 passes** from the preceding full run, with
source delta, exact test inventory, binary and output hashes authenticated.
The combined total is 3,693; it is not a fresh execution of every target.

All 8,382 active text cases passed. The affected independent checks cover PA8,
PA18, PA20 and PA21, with native checks for PA18/20/21 and transitive metadata
consumers. Formatting, architecture/file audits, all-target/all-feature strict
Clippy, harness, upstream provenance and backend checks passed. Three release
adapters were rebuilt against the same library. The gate verifies 378 required
test names. No generated reference was changed to mask a failure.

Evidence is retained in the validation worktree:

- `obj/parallel/library-alias-allowances-release-v3/`: original failed full run.
- `obj/parallel/library-alias-allowances-recovery-v1/`: successful recovery,
  exact retained-target bindings and fresh functional checks.
- `obj/parallel/whole-core-blockers-v6/`: unchanged original-core observation.

| Artifact | SHA-256 |
| --- | --- |
| Functionally qualified CLI | `237ea805b979e28ecde796107d3286498e82f38591bd5258c0a15628ceafe9d8` |
| Recovery functional receipt | `5c2ad7794949d88018edfb8148a0a227029a793d7578c525250106aa47eae1da` |
| Recovery checks | `2425da4a1abbfce5f9366b3f802337667b4e79c1ed5cb5f12954e4b774a0f5a6` |
| Retained-target bindings | `76bf677c07d9018dbd7c2e1f0de168874722b016710446054c114d3290417c90` |
| Original failed maintainer log | `aa87a14d8dcdb9b2cea19a2915b2a182e0aee1257eb85c6519cdb7c8536ae894` |

## Performance qualification

The immutable 101-input matrix in
`obj/parallel/library-alias-allowances-perf-v2/` completed against the previous
public compiler, snapshot `6bba6589`. Correctness preflight passed all inputs
and all 76 declared native programs. Its 91 comparable pairs have identical
outputs. Seven projected-alias inputs and three allowance-list inputs are newly
supported and have candidate-only measurements; rejected baselines are never
timed as speedups.

No comparable case crosses the 5% instruction or combined 512 KiB/5% RSS review
threshold. Instruction changes range from −0.30% to +1.29%. Paired median wall
changes range from −11.66% to +11.77%, with A/A calibration spreads as high as
135.62%; these noisy wall observations do not establish an overall speedup.
All raw observations and host-load samples remain retained. The preceding
checkpoint's sixteen RSS triggers remain historical evidence; this comparison
against that checkpoint does not erase them.

**One of 52 growth controls fails its declared threshold.** Increasing actual
projected associated types from 32 to 128 increases instructions from
54,813,625 to 550,412,747 (10.04× for 4× input growth; limit 6×).
The 8-to-32 step is 5.20×. Repeated use sites and allowance-list widths stay
within their bounds. Publication is held while this algorithmic issue is fixed.

Retained Callgrind evidence attributes most of the 128-subject cost to
`normal_form::opening::prepare`: each independent projection maps the complete
caller environment, then constructs its trait-premise index again. Inclusive
costs overlap: opening is about 73% of total instructions, environment
substitution about 52%, and index construction about 21%. These are structural
rebuild costs, not evidence that allocation micro-optimizations are sufficient.
The repair must share structural data while preserving fresh caller-specific
lifetime and selection proof. The immutable inputs and thresholds stay fixed.

Results SHA-256:
`eb51580d30151ac4fd03ddd71037d1bd812428a39b9969900cf2e86587367214`.
Analysis SHA-256:
`0b6997744d324c39efee622df365a6b814b766b598054a93d320c284f19e9fb6`.
Original-core result SHA-256:
`ac6ab9310eb8d92bac2db16aa9c5fe632d27f22b2131a5e4dec6d9de0b1d89a0`.

## Remaining crate work

The [blocker tracker](library-build-blockers.md) separates integrated behavior
from worker experiments. Scoped macro declaration support has a provisional
worker result reaching `cfg_select`, line 235, but is not part of this compiler
snapshot. Its remaining candidate checks must pass before integration.

Scoped invocation and exported macro provenance, hygienic fields/methods and
associated members, early implementation validation, and actual source-body
artifact/import execution remain required. Internal allowance membership alone
does not authorize foreign API use or const calls. The ordinary implementation
of upstream library algorithms must continue to come from their source.

Successful recovery cleanup removed 4,129 generated files, reclaiming 3.91 GiB
of unshared blocks. Current comparison binaries remain pinned; historical
receipts, inputs, diagnostics and raw observations remain available. Resource
limits and the 150 GiB free-space reserve are unchanged.
