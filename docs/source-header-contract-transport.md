# Versioned source header contracts

This is the approved implementation plan for source tag 10, row version 1,
`Source2024FunctionContracts`. The draft is unqualified until its focused
producer, codec, import and consumer controls pass. Version 0 Functions bytes
and imported Unknown behavior stay unchanged. Tag 11 Provided stays closed.

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

Six focused control groups will cover genuine provider/artifact/fresh-client
use, lint levels and canonical origins, inline applicability and completeness,
legacy bytes and closed profiles, codec/remap adversaries, and bounded work plus
layout. Actual minimal SourceCore providers are the inputs; manufactured roles
or packet-only success cannot substitute for publication and fresh proof.
No nominal, closure, Provided, wide-float or namespace eligibility is expanded.
The existing single-source-provider inventory limit remains authoritative;
this checkpoint does not admit a mixed inventory of two source providers.

The draft API is `Declarations::header_attribute_contracts`, returning explicit
`ContractKnowledge` for both families and a sealed `ContractOrigin`. Its source
accessor returns the actual retained declaration/owner/span. Its imported
accessor exposes `coordinate`, checked declaration handle, optional checked
annotation site (absent only for legacy Unknown), and optional real function
`ExternalDef`. The lower opt-in is
`instances::build_source_library_profile(..., SourcePublication::FunctionContracts)`.
Existing library callers still select LegacyFunctions; no CLI flag activates v1.

The six exact private selectors and all provider/client source bindings are
predeclared in `pa31/tests/spec/source-attribute-transport/contract.json`.
They are unrun. Source-only formatting and review do not qualify the new codec,
fresh consumers, warnings or layout/allocation costs. A retained qualified
pre-v1 CLI is available for a separately guarded exact-byte baseline; that
comparison remains pending until it actually publishes the frozen provider.

The full reviewed proposal is retained at
`obj/parallel/pa31/source-attribute-transport-v1/proposal.json`, SHA-256
`c68249bae319ca10e2405696ad932e5ef40939afc0a5a9c7bc44c1f180c1fb2f`.
Choose exact source bytes and expected warning/exit outcomes before executing
the new controls. Keep each development gate below five minutes; do not repeat
unchanged independent observations or full inventories.
