# Pending aggregate inputs during source coercion discovery — PA31

Compiler bddcb51f3 keeps an associated head awaiting normalization unknown to the
unsizing mismatch probe. The probe supplies no equality or conversion proof.
Ordinary comparison retains its normalization fence; body equality and selected
source conversions still require their full existing proofs. No hot record,
cache, compiler limit or backend contract changes.

The original failure is Alignment::try_from at mem/alignment.rs byte11798:
`align.get().try_into()` returns `Result<Alignment, Self::Error>`. Ordinary blanket
conversion, const/conditional traits, generic receivers and associated stored
representations all passed independent reductions. The actual source environment
was necessary to expose the failing operation.

## Actual trace and reduction

Temporary diagnostic CLI feeee5431 captured only file55 normalization failures.
Its bounded graph has inferred Result term20 with unknown target term13. The
TryInto::Error projection2320 forwards to TryFrom::Error projection2324 with that
same unknown target and the actual usize input. The backtrace identifies
`source_nominal_coercion` calling `unsizing_mismatch` before ordinary return
inference fixes the target. The genuine source CoerceUnsized role enables this
probe; earlier reducers omitted it.

The trace is analysis-only, with at most128 inferred and128 canonical nodes,
parameter owner/slot data, argument layouts and backtraces. The three edited
compiler files were restored from exact saved bytes and the temporary helper
removed before implementation and qualification. The diagnostic executable was
retired by scoped prune1794342479655130994; its source, hashes, patch and graph
remain in `obj/parallel/conversion-errors/`.

Adding only the actual CoerceUnsized declaration reproduces the failure in a
small source protocol. Rust accepts it, while immutable baseline9be rejects
inferred ordinary/const conversions with normalization. Explicit target, direct
conversion and no-role controls pass. PA31 therefore owns the protocol fixture;
this is not a generic ordinary-Rust return-inference reduction.

The existing paired graph walk now treats an unnormalized head as unknown only
for unsizing discovery and continues looking for a known mismatch in other pairs.
It neither binds variables nor grants properties. Invalid identities and traversal
exhaustion still fail. Ordinary `incompatible` and shape equality preserve their
raw normalization errors. Existing selected CoerceUnsized, Unsize, formation,
lifetime, move and cleanup consumers remain authoritative.

## Frozen source contract

[The PA31 packet](../pa31/tests/spec/coercion-discovery/README.md) declares22 active
outcomes,16 accept and6 reject, plus one explicitly required diagnostic boundary.
Every source and the C observer have manifest hashes. Rust1.98 independently
checks every final object outcome and executes all16 positives with output7.
Enum/record returns, generic receivers, forwarding, explicit paths, source const
contracts and raw projections retain the actual conversion shapes. Negatives
preserve target/error identity, generic proofs and real coercion evidence.

An initial record fixture retained an enum constructor in its fallback impl;
independent E0599 caught that setup error before implementation. Original bytes,
manifest and error remain under `record-constructor-revision/`.

The initially active nested opaque-result mismatch still reaches the older raw
normalization boundary. Its independently rejected source is retained under
`required/nested-result-equality.rs`, with the desired body missing-equality error
still required. Explicit prefix qualification isolates it from this discovery bug;
baseline and candidate both reject it at byte1818 with normalization. It is not
counted as a completed regression. The active missing-equality negative checks
the opaque result directly at its root. Original failed source and manifest are
retained; no broad body-diagnostic or inference permission change was added.

## Qualification

Evidence is in B's `obj/parallel/conversion-errors/`:

- `public1.log`, receipt1794341517867751947,163.95s: six exact tests pass, including
  all16 positive LowIR outcomes, seven-counter scaling, raw query guards and the
  existing source/nominal coercion controls. The aggregate fails only the nested
  diagnostic boundary retained above. `public2.log`, receipt1794341919377239898,
  38.65s, passes all22 final active LowIR/status outcomes.
- `ordinary-controls.log`, receipt1794342233661394651,50.60s: two exact pointer
  region/ownership and receiver reference/slice controls pass. Tail conversion is
  the other consumer of this discovery query. Strict selected production, CLI,
  frontend, memory and traits Clippy, full formatting and architecture pass.
- `native.log`, receipt1794341999645587392,59.82s:16 new Rust/CPPGM native programs
  and one existing nominal-wrapper transfer/cleanup program pass. No backend
  bridge or serialization contract changed.
- Width8/32/128 work grows at most4x+1024 in inference, canonicalization, trait
  and projection selections/steps, method steps and probes. Adding128 unrelated
  types changes none of those seven call-work counters. Raw API guards preserve
  normalization for ordinary comparison/equality, leave variables unconstrained,
  find a known sibling mismatch and reject invalid identities.
- `inventory/observations.json` binds all22 active CLI source outcomes and the
  separate required boundary to the immutable candidate. The315 harness tests,
  468 roots/13 binaries/44 stages,128 upstream fixtures and126 Reference files pass.
- `measure/passed.json` retains40 immutable same-profile A/A and two paired ABBA
  observations. Complete ordinary LowIR, two source-control type outputs and all74
  frozen survey outcomes are equal. Candidate/baseline ratios are
  1.067/1.008/1.058/1.006; A/A ratios0.869/1.206/0.775/0.973. No speed claim.

No text oracles were generated or updated. Whole PA31 remains planned. No local
release build or repeated full private inventory was added. Full42890284924 on
bb9b13eee covers the preceding scope/private-diagnostic repair. Its release/course
and maintainer native stages pass; the broad native sweep is active. This newer
semantic checkpoint requires its successor after that run terminates.
Never cancel an active full run by redispatching. Earlier optimized nested-proof
and broad performance qualification remain pending.

Immutable dev-fast CLI
`6549fad645f26cfafea63ff3409f258ec50d9d210647ade28b5bd8a7a73ef5a5`
binds bddcb51f3563223bac9f7f0040ee79a9efabbe75, an empty patch and3,886 compiler,
CLI and test Rust hashes. The baseline is9beaaac9; optimized21f66e51 remains pinned
only for the earlier pending optimized controls.

## Original core

The unchanged120-second observation fails its infrastructure deadline at
121.1880seconds. Its partial stream retains235 failure records and4470 item
attempts, through function9063,file241 byte42996 node1298 in time.rs. All722
original source hashes, six configuration bindings, profile and compiler limits
remain fixed. Progress SHA-256 is
`422521024f3190bf44a720fa4dba9f3e8b14e7e8d16ce3fc40dcbf8eef7e84a4`.

No mem/alignment.rs failure remains in the recorded prefix. Alignment::try_from's
old conversion fence is absent. The first retained failure is now
Discriminant<T>::fmt at mem/mod.rs,file54 byte45756: the DebugTuple field call
requires an unambiguous Debug assumption or implementation for its associated
representation. The next retained request_by_type_tag failure has the same trait
obligation wording. Reduce the actual field/object trait proof first; retain the
compiler-provided DiscriminantKind identity if an ordinary associated-type bound
control passes.

The previous and current streams end at different prefixes; their raw counts
are not whole-core speed evidence. No complete census, checked-body count or
original core/alloc/std artifact exists.
