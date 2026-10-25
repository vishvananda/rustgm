# Versioned source header contracts

The library implementation of source tag 10, row version 1,
`Source2024FunctionContracts`, passes nine exact private controls in
[CI 40205079379](https://github.com/vishvananda/rustgm/actions/runs/40205079379).
Metadata, formatting, architecture, strict private Clippy, build and execution
take 131.84 seconds. Version 0 Functions bytes and imported Unknown behavior
stay unchanged. Tag 11 Provided stays closed. CLI activation is a separate,
pending checkpoint; this does not complete an assignment or upstream crate.

Every admitted function and trait header has a complete parallel contract row.
KnownAbsent requires a completed source producer witness; an absent map entry,
unvisited owner or older artifact remains Unknown. Present must-use retains its
optional reason. Inline retains Hint/Always/Never and actual body applicability;
required bodyless members retain the provider warning and DiagnosedNoBody fact.
Warned attributes on invalid owners produce no effective contract. A 64 KiB
reason limit belongs to this transport profile, not the source language.

Source queries authenticate actual declaration and configured owner against
their retained OriginalImage incarnations. Imported queries use the current
Catalog's declaration handle, canonical crate/declaration/edition coordinate
and owner-checked DeclarationSite. Functions may additionally expose their
already real ExternalDef. No imported syntax nodes or trait linker identities
are invented. Contract presence never grants call, body, intrinsic or UB policy
authority.

The producer reuses existing owner/effect and selected-header work. The new
columns live in cold versioned metadata, without changing SourceRows or hot
function/trait/AST/body/definition fields. Publication opts in explicitly at the
existing lower boundary; Stage inspection and the legacy plain-attribute guard
retain their existing behavior. Imported unused-result diagnostics use the
consumer's lexical lint policy and use location. Provider declaration warnings
are not replayed by imports.

Six focused control groups cover genuine provider/artifact/fresh-client
use, lint levels and canonical origins, inline applicability and completeness,
legacy bytes and closed profiles, codec/remap adversaries, and bounded work plus
layout. Actual minimal SourceCore providers are the inputs; manufactured roles
or packet-only success cannot substitute for publication and fresh proof.
No nominal, closure, Provided, wide-float or namespace eligibility is expanded.
The existing single-source-provider inventory limit remains authoritative;
this checkpoint does not admit a mixed inventory of two source providers.

The API is `Declarations::header_attribute_contracts`, returning explicit
`ContractKnowledge` for both families and a sealed `ContractOrigin`. Its source
accessor returns the actual retained declaration/owner/span. Its imported
accessor exposes `coordinate`, checked declaration handle, optional checked
annotation site (absent only for legacy Unknown), and optional real function
`ExternalDef`. The lower opt-in is
`instances::build_source_library_profile(..., SourcePublication::FunctionContracts)`.
Existing library callers still select LegacyFunctions; no CLI flag activates v1.

The six exact private selectors and all provider/client source bindings are
predeclared in `pa31/tests/spec/source-attribute-transport/contract.json`.
All six pass with the existing selected-publication guard, legacy Unknown
control and wide-type hot-layout control. The source-qualified receipt is
`b73b6ce12d9b871c50cfb9fe4a3d96a9277b4d6db970c7dced1bfadee25909a0`.
Integration `dd1ef564` preserves every production fingerprint; its only three
changed test files have separately qualified assertion-only repairs. The
source comparison is retained under `obj/parallel/source-contracts-publication-v1`.

The qualified pre-v1 CLI has now published that baseline: tag 10/version 0,
3,628 bytes, SHA-256
`81fd7dc86d906ef7ba5fd4abb92a0a24f6f9548c473b9b8a43a62dd7c1f234c0`,
using the unchanged minimal provider and `minicore@attribute_transport`.
The candidate's explicit legacy API reproduces those bytes exactly. Two new pinned-rustc observations
(2021 and 2024) reject inline on the actual Copy trait as a hard error with no
warning; the initial warning-only predictions were wrong. Retain their frozen
plan and raw JSON in `obj/parallel/pa31/source-attribute-transport-v1/baseline-and-owner-v1`.
Results SHA-256 is
`201c40fd977fa844752187d622e7d177096c3383f10f023d8b1778daed97c76d`.
The corrected consumer rejects this trait placement before completion and
publication. This does not change must-use wrong-owner warnings, establish
inline module/ADT semantics, or alter the independently observed bodyless-member
warning and retained DiagnosedNoBody hint.

Measured cold sizes are Ledger 96, ContractOrigin 40, ContractView 56,
InlineView 48, HeaderAttributeColumns 48 and ValidatedRows 120 bytes. Existing
hot record layout controls pass. Warm-query controls charge 128 actual queries
and two failed limit reads; source scaling controls cover 3/10/34 function rows.
Collection capacities and allocation counts were not measured. These controls
establish bounded work and layout, without claiming wall-time nonregression.

The combined anonymous-function gate subsequently exposed a genuine compatibility
regression: a local crate-root definition has no AST declaration node, so using
node absence alone to select imported contracts misclassified that owner. The
repair authenticates the resolver's exact local root, canonical module/scope,
package source and absence from the imported catalog before preserving its
legacy optional query result. All other missing imported producers still fail;
this compatibility path does not manufacture complete header absence.

[CI 40213866643](https://github.com/vishvananda/rustgm/actions/runs/40213866643)
passes the old plain/configured-publication control, two root/import/forgery
regressions, legacy imported Unknown and fresh version-one consumers. All five
execute with metadata, formatting, architecture, strict private Clippy and build
in 131.15 seconds. Harness, upstream and audit-cache checks also pass. Source
`2b3dff02` is integrated at `b670e3f0`; the earlier 12/13 failure remains retained.
CLI activation remains a separate pending checkpoint.

The full reviewed proposal is retained at
`obj/parallel/pa31/source-attribute-transport-v1/proposal.json`, SHA-256
`c68249bae319ca10e2405696ad932e5ef40939afc0a5a9c7bc44c1f180c1fb2f`.
Choose exact source bytes and expected warning/exit outcomes before executing
the new controls. Keep each development gate below five minutes; do not repeat
unchanged independent observations or full inventories.
