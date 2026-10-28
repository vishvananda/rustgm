# Source deprecation replacement metadata

The next strict original-core boundary after the actual ineffective unstable
impl producer was `mem/alignment.rs:212`, whose deprecation annotation contains
`suggestion = "as_nonzero_usize"`. The compiler now parses that named string,
retains its decoded value and exact argument span in the existing cold owner,
and requires the actual `deprecated_suggestion` crate feature. Ordinary crates
retain that request in the existing local feature context. No source-provider
or API authority is created.

The existing deprecation event remains unchanged. Malformed, duplicate and
ungated arguments reject. Independent pinned Rust also rejects macro output
that has only `allow_internal_unstable(deprecated_suggestion)`; the original
incorrect hypothesis and failed oracle remain recorded. The authoritative
corrected contract is
`pa40/tests/spec/deprecation-suggestion/contract-v2.json`; `contract.json`
preserves the original predictions. All 12 corrected observations match Rust
1.98.0, editions 2021/2024. Oracle receipt SHA:
`a4788b2bdc41d1f79140bfeec6225d592e100cb1fe2eb07b080db4b31a1758e3`.

A monotonic producer hint makes the no-suggestion path constant time. An actual
global feature request avoids an owner scan; rejected reads remain charged.
The optional cold deprecation record grows 48 to 72 bytes; Producer 368 and
Owner 160 bytes remain unchanged. Parsing reuses retained attribute trees and
the existing string decoder. Fix-it rendering is deferred.

Root merged this with the actual impl producer, four stale CI layout assertion
repairs and the source availability v3 transport. Combined private 21 controls
pass in 274.35 seconds; production/CLI, codec and lexer 53 controls plus PA6 pass
in 189.28 seconds. Both strict Clippy modes, format and global architecture
audits pass. Receipts are
`obj/qualification/fast/1793208884366089282/receipt.json` and
`obj/qualification/fast/1793209286589598953/receipt.json`.

The postcheck authenticates all 28 fresh LowIR/metadata/source outputs from the
seven availability clients against the independent Rust and CPPGM native-7
checkpoint. Suite registration, placement and upstream integrity also pass.
Postcheck receipt SHA:
`9945a1cae3db8c005a975520b360211d55c2930d3517fc4922ba24871bf12288`.

Strict CLI `a946b4b32a42f464271cb976f2f64eade42f79baf3dd5db12eda828695d5f250`
keeps all 722 source pins and effective target arguments unchanged. The manifest
explicitly refreshes the driver source binding for its added availability
publication selector. The probe opens 60 sources and stops at
`mem/transmutability.rs:86`: `unstable_feature_bound` on the actual unsafe trait
is outside the current impl-only consumer. Opened files are not compiled files;
no original crate artifact exists.

Compilation-first work retains allow/deny/forbid/expect and defers broader lint
parity unless it blocks compilation. Explicit capped bring-up remains separate
from final strict qualification. The observer now requires matching explicit
cap, manifest and producer identities; four protocol/input-integrity tests pass.
PA40 release, complete diagnostic transport and original core/alloc/std artifacts
remain required.
