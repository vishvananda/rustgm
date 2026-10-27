# Typed metavariable expressions and the original-core boundary

The source checkpoint at a5dcce0c3 implements count, ignore, index, len and
escaped dollars in the existing typed grammar and transcription stream. It
keeps the actual crate feature independent of concat and preserves the pinned
empty-first-branch rule, invocation-time unknown-variable errors, target-width
depth parsing and definition-site numeric provenance. Original grammar/parser
failures and incorrect oracle predictions remain retained.

The winning matcher trace is the structural owner. One lazy transcription-local
index records preorder subtree intervals and per-repetition iteration positions;
deep counts use rank queries rather than replaying the subtree. Typed driver
sets share child-list unions, and active binding cursors serve repeated ordinary
and concat substitutions as well as numeric expressions. Allocation and failed
work are prepaid. No new semantic proof owner or global cache is introduced.

The exact 69-test gate passes production and private strict Clippy, formatting
and architecture in226.21 seconds. Receipt:
`obj/qualification/fast/1793088351740100153/receipt.json`.
The same source then passes ten lexer controls and every PA7/PA8 text case in
28.01 seconds, receipt `obj/qualification/fast/1793089277981745234/receipt.json`.
That receipt builds the current immutable CLI with SHA-256
`7ecaec04be6d49350536da3e78f5627df62046d3842bfd00f905250fc2a6095e`;
its source binding is `obj/parallel/metavariable-expressions-v1/cli-binding.json`,
SHA-256 `4b03f4afad5a591310e70a571981b0c25089812743b05aaace65067ecdc19ed7`.

Three authenticated private-binary replays retain exact work observations:

| Actual workload | Sizes | Paid work |
| --- | --- | --- |
| Repeated subtree counts | 8 / 32 / 128 rows and queries | 249 /1097 /5001 |
| Nested active count paths | depth8 /16 /32 | 157 /317 /637 |
| Ordinary nested capture emission | depth8 /16 /32, width32 /64 /128 | 588 /1164 /2316 |

Index prepayment/failure and warm reuse pass. Actual Node40, Kind32, Piece20,
Transcription32, Definition72 and Environment64 byte sizes remain unchanged.

Independent source metadata retains104 pinned Rust and12 immutable baseline
observations. Two original missing-Copy predictions are preserved and separately
corrected. The executable companion and identical C observer then produce
`3 3 2 33` with pinned Rust and candidate LowIR through the unchanged locked
CPPGM backend: seven commands, two native executions,4.61 seconds. Evidence is
`obj/parallel/metavariable-expressions-v1/native-v3`; binding SHA-256
`e4b935eec1ea61cb3a2a7101086e16b67d0743689f16db07023e0f7688e166b4`.
The original library-output consumer remains blocked by real source
macro/semantic-name provenance metadata; its failures stay in native-v1/v2.

Immutable development-build A/A plus paired ABBA measurements retain every raw
observation and output equality. Nested depths8/16/32 show candidate median
ratios1.1855 /1.1938 /1.0641 against A/A ranges12.28% /17.82% /4.72%.
These are observed slowdowns, not a wall-time improvement. Depth64 hits the
baseline's existing grammar-nesting128 rejection before candidate comparison;
the failed prediction is retained. The separate unchanged ordinary512 control
has ratio0.9667 with A/A range26.88%, establishing no change beyond that noise.
The complete passing depth groups are not rerun after the depth64 failure.
The bounded-work improvement and wall-time overhead require separate assessment;
ordinary macro overhead remains followup work. Raw evidence is in ordinary-perf-v1
and ordinary-perf-continuation-v1 under the same local checkpoint directory.

The unchanged pinned-core attempt now opens371 source files, up from263, in
6.50 seconds. All722 hashes verify before and after. It stops with
`source macro contracts exceed their work or storage budget` in generated
num/mod.rs syntax. No complete semantic check, artifact, object or library
consumer exists. The immutable debugger probe observes the first rejection:
steps1,000,001, attempted bytes2,085,659, witness payload53,959. Its call stack
is decorated_calls::macro_attribute during the real expansion traversal.
No budget was raised. The first debugger preflight's relative-path containment
failure is retained; the actual probe resolves and verifies the full source root.
LIB-042 owns diagnosis and an input-appropriate bounded-work policy, with
adversarial/scaling controls required before changing a limit.
