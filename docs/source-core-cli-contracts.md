# SourceCore CLI contract publication

This checkpoint activates the qualified function-contract API at the real
library driver boundary. Source `41b60e31` passes the selected frontend and
native gates below. It is composed over published main `b830f5dc`, including the
authenticated local crate-root repair (five passes in CI40213866643). The
library checkpoint is source `4311d7ed`, nine selected passes in
CI40205079379; those controls are retained, not repeated for this driver change.

`dev/bin/modules.rs::emit_lowir` selects `FunctionContracts` only inside library
publication when the analyzed Package has an actual SourceCore provider. This
selects tag 10, row version 1 for marked and unmarked providers alike. It does
not inspect attribute presence or retry a failed legacy export. In particular,
a plain provider must transport checked absence so a fresh discarded plain call
does not fail on legacy Unknown metadata. There is no user version flag.

The non-SourceCore path and explicit `build_library_prepared` legacy API retain
their existing wire behavior. The qualified 3,628-byte legacy hex golden stays
version 0. Existing version 0 artifacts still decode; an imported obligation
that was never transported remains Unknown. Provided publication, nominal
profiles, imported UB policy and executable authority are not widened.

The selected API shares the current owned Report/facts/Stage and one charged
selected-header walk for both must-use and inline facts. It preserves the
existing error carrier, warning renderer, metadata write and LowIR write
boundaries. No AST/source scan, metadata-presence shortcut, layout field or
persistent compiler counter is added.

The exact sources and outcomes are frozen in
`pa31/tests/spec/source-attribute-transport/cli-contract.json`. Eight accepted
fresh clients cover plain/marked used and discarded results, explicit let-underscore
discard, allow, warn and inherited deny with inner allow. Each native execution
must print exactly `7\n`, exit zero and have empty runtime stderr. Deny, forbid,
forbid weakening, old Unknown metadata, actual corrupted v1 metadata and a
genuinely unsupported nominal profile retain distinct rejections. Five real
provider sources cover absence, reason, warned wrong crate owner, diagnosed
bodyless inline and an unused private marked function. Provider warnings are
exact and are never replayed in a fresh client.

`scripts/check_source_attributes.py` reuses the shared command recorder and
locked prebuilt backend. The native lane independently builds the two provider
rlibs and eight client objects with pinned rustc, links both implementations
through the unchanged C driver, and checks the three independent lint errors.
It does not build or modify CPPGM. Per-command timeouts are ten seconds with a
240-second global bound. Successful native objects/executables are reclaimed
after each case; raw diagnostics and bindings remain. This complements the
changed existing packet-library native control: nine candidate native paths
in total, with eight independent counterparts in the new lane.

Affected Rust integration selections are the existing packet-library flag and
native controls plus `versioned_clients`' frontend and ignored native groups.
The native group includes the same frontend cases, so do not run both merely
to duplicate observations. Three small Python controls cover exact warning
acceptance/rejection, error masquerade, empty stderr, command environment and
timeout forwarding, and expired global budget refusal.

The source-only text inventory in `cli-text-effects.json` binds all 8,484 cases
in the 24 current manifests. None can select SourceCore: the course loader's
only extra argument pairs are cfg and crate-name; library identity arguments
do not change source mode. Expected course status, warning stderr, LowIR and
reference changes are therefore empty. Retain existing base/scoped receipts
and sidecars; generate no broad replacement references for this change.
The combined candidate's updated 8485-case inventory, including the new PA18
argument-index fixture, also contains zero SourceCore/client-mode commands.
Its argument inventory is verified separately under
`obj/parallel/library-frontier-combined-v1/course-no-source-mode.json`.

C9 printed cold type sizes: Ledger96, Origin40, MustUseView56, InlineView48,
AttributeColumns48 and ValidatedRows120 bytes. C collection capacities,
allocation counts and total retained/work costs were not printed. This driver
checkpoint adds no counter or allocator instrumentation and makes no new claim
about those missing measurements.

The frontend gate [CI40214484371](https://github.com/vishvananda/rustgm/actions/runs/40214484371)
passes both exact integration controls and three Python harness controls. Its
helper takes 112.60 seconds; the 24 candidate commands take 0.216 seconds.
The separate native gate
[CI40214804906](https://github.com/vishvananda/rustgm/actions/runs/40214804906)
passes both exact ignored controls in 90.60 seconds including backend setup and
focused static checks. Its 79 candidate/independent commands take 1.926 seconds.
All 17 native outputs (nine RustGM and eight rustc) are exactly `7\n`, with zero
status and empty stderr; three independent lint-error controls also pass.
Both jobs pass the mandatory CLI build, metadata, formatting, architecture,
strict CLI/integration Clippy, harness, upstream and audit-cache checks. These
are bounded functional observations, not paired performance measurements.

All 2,583 source fingerprints match the frozen product in both receipts. Both
retained CLIs have SHA-256
`b1fb5873b3824265750e47043e39e6dd9f7c36b8b971cf06b01807956611fc39`.
The native receipt is
`b96dcc42641e2be73ad756b78ffd5be4b736419e6ce3e51cbaaa3b4db3790ee9`.
Verified raw artifacts, command logs, bindings and summaries are retained under
`obj/parallel/pa31/cli-source-activation-v2/qualified-frontend-40214484371` and
`qualified-native-40214804906`. Successful native object/executable scratch is
reclaimed between cases. No local Cargo or CPPGM build was used.

Observed metadata sizes are 3,716 bytes for plain v1 (88 bytes above the retained
v0 golden), 3,736 for the reason-bearing provider, 3,722 for bodyless inline and
4,519 with the unused private marked function. A warned invalid crate owner
produces the same bytes as the plain provider; two ordinary exports are exactly
the same 82 bytes. The retained v0 golden hash stays unchanged. These wire sizes
and existing command durations are measured; heap allocations and collection
capacities remain unmeasured. Wider profiles and original upstream crate
artifacts remain required; this checkpoint does not activate an assignment.
