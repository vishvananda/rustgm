# SourceCore CLI contract publication

This source checkpoint activates the already qualified function-contract API
at the real library driver boundary. CLI and native qualification remain
pending. The source is composed over published main `b830f5dc`, including the
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

C9 printed cold type sizes: Ledger96, Origin40, MustUseView56, InlineView48,
AttributeColumns48 and ValidatedRows120 bytes. C collection capacities,
allocation counts and total retained/work costs were not printed. This driver
checkpoint adds no counter or allocator instrumentation and makes no new claim
about those missing measurements.
