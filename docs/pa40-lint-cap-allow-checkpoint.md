# Explicit Allow lint cap checkpoint

This finite frontend profile accepts `--cap-lints=allow` and
`--cap-lints allow`. Uncapped compilation remains the default. Other cap values
return an explicit unsupported-profile error. PA34/PA40 remain planned; this
checkpoint establishes neither complete lint parity nor a library artifact.

## Cap-aware macro event work

Source `6697b601a` authenticates the actual producer/package cap and sticky
diagnostic budget before returning from suppressed macro deprecation production.
Macro execution identity and owner parsing stay in their existing passes.
Uncapped lookup, source witnesses, malformed/duplicate annotations, feature gates
and pending expectations retain their checks. No cache, common field, quota or
metadata permission is added.

The original-core immutable debugger observation identifies the expansion call
chain `deprecated_macro → deprecation_in → deprecation_owner` at10,000,463
combined steps and37,128,057 charged bytes. It is a work-limit failure, not the
earlier storage failure. Source owner/module witnesses were being checked for
every macro invocation even though the explicit Allow cap suppresses the event.

The same generated wide-header/many-call source gives these actual policy costs:

| Header attributes and calls | Previous work | Current work | Current bytes |
| ---: | ---: | ---: | ---: |
| 8 | 351 | 127 | 3000 |
| 32 | 3663 | 463 | 8472 |
| 128 | Baseline assertion already failed | 1807 | 30360 |

This removes Cartesian optional validation; it is a deterministic operation-count
result, not a wall-time speedup claim. The unchanged strict/Allow macro programs
and all three scaling sources retain byte-identical full LowIR across immutable
previous/current binaries. All seven output comparisons pass. Independent pinned
Rust matches26/26 metadata/object outcomes, including hard malformed-note,
unsafe-wrapper and duplicate annotation errors under both caps.

Sixteen focused private controls pass160.12s with strict Clippy, formatting and
architecture; CLI/lexer12 and PA8 120/120 pass83.96s. The complete harness/upstream
check passes22.32s. All3258 source/tool fingerprints match the two compiler gates.
Current Config80/Package256/Report3096/Producer368/Owner160/NodeFacts80 widths and
the existing cold cap/debt ownership remain unchanged.

Evidence in the root-owned warm B checkout:

- Private `obj/qualification/fast/1793358222194790640/receipt.json`, SHA
  `dfc571b80ee8fa68da3c9e631bd12afb0b4166dbfd3e34abcf1225e0224080b1`.
- Production `obj/qualification/fast/1793358534008297291/receipt.json`, SHA
  `7e892538f1b7f58c3a48047ec80ed8db8a1c2625df6940837805e647a048ab09`.
- Debugger `obj/parallel/lint-work-recurrence-v1/debugger-v1/receipt.json`, SHA
  `4fb0c2d3bcd7fcb680726811ba618872721319ce0f6e36d8dc3a2baa8aac4518`.
- Output equality `obj/parallel/lint-work-recurrence-v1/equality-v1/receipt.json`,
  SHA `9e4630c130e6531fd5137634048ba11fea7d47a44046fe2ac2e0d8b95e42310d`.
- Independent C-tree reference receipt SHA
  `32f905bf94dd0c9149748ee21f60638752210eb0bb1feb26113d2812b9a995db`;
  successful14 object/metadata artifacts are pruned, raw evidence retained.

Immutable CLI
`b84d289990b937829da0bd45a7a2fa2cf790b88bc223e93d7f53126a4f29a0f5`
passes the earlier policy-work frontier and reaches LIB110,
`#[rustc_confusables("swap")]`, original `library/core/src/cell.rs:508`
byte18838. The types observation takes25.26s, with375 loading opens and all722
source hashes unchanged. Target, explicit cap and source-contract14,500,624-work/
1GiB-attempted/64MiB-retained limits stay fixed; policy limits remain10M/64MiB.
The guarded harness/equality/core batch takes78.38s including admission/cleanup.
Core receipt `obj/parallel/lint-work-recurrence-v1/core-v1/receipt.json`, SHA
`f2a290c932fa15e03328874b2de7d36a2eaed75922ff77d2f766084c6bd5396d`.
No original crate artifact or whole assignment release is implied. Broader
uncapped diagnostic work and strict qualification remain required.

## Initial cap protocol

One typed immutable input travels through Config, Package, staged expansion
requests and semantic request policy. Existing-facts entry and finalization
validate the cap and authentic expectation domain in constant time. Capped facts
cannot be reused with an uncapped report. The cap changes diagnostic levels and
optional diagnostic sinks, preserving retained source attributes and hard type,
syntax, feature, unsafe, API, metadata and CTFE requirements. No source attribute
is stripped and no metadata/body permission is granted.

Optional unsupported lint entries can pass under the explicit cap only after
validating their retained path/attribute shape. Unknown tools and malformed
attributes remain errors. Unsupported expectations retain their original source
witnesses and a charged cold debt count; they receive no invented expectation ID.
Known expectations retain authentic IDs and remain pending under this profile;
suppression never claims fulfillment or strict completion. Ordinary uncapped
allow/deny/forbid/expect behavior remains covered by the existing protocol.

The initial [source contract](../pa40/tests/spec/lint-cap-allow/contract.json) contains
26 ordinary and actual source-provider inputs. The independent pinned Rust 1.98
join contains 104 matching observations across editions 2021/2024 and uncapped/
Allow configurations. Earlier observations retain two corrected hypotheses:
an unsafe source reducer initially lacked the Copy lang item, and a denied
unsafe-operation lint uses E0133 just as a hard unsafe error can. The compiler
therefore distinguishes typed stages rather than filtering error codes.

Private receipt `1793210348571210440` passes five exact controls in 193.25s.
Production/CLI receipt `1793210576088013880` passes strict Clippy, formatting,
architecture checks and two exact driver controls in 183.02s. These cover both
flag spellings, ordinary and SourceCore entry, default strict rejection, hard
errors, known/deferred expectations, cross-context reuse and an attribute-free
default-policy sink. Warning-only LowIR output is byte-identical with and without
the cap. Earlier failed receipts retain static adapter/style failures and a cold
build timeout; the final unchanged-source warm gate passed.

Initial measured x86-64 widths are Config 72, Package 256, Report 3096, Producer 368,
source Owner 160, cold Policy 152, dense policy ID 4 and NodeFacts 80 bytes.
The existing cold Arc domain now stores an 8-byte cap/debt payload; its complete
24-byte allocation is charged before creation. Repeated authenticated debt reads
retain no additional bytes. An attribute-free capped input creates no producer
or dense policy rows. No per-node cap field or new syntax parse is introduced.

Raw predictions, mismatches, diagnostics, hashes and receipts remain under
`obj/parallel/explicit-lint-cap-allow-v1/`. Full release checks remain CI work.
Original-core capped probes must record the effective cap in both their manifest
and producer binding and remain distinct from the unchanged strict 722-pin
reference. A capped observation cannot close the strict release/core goal.

## Integrated compilation-first qualification

Root normally merged this with source availability v3 and replacement metadata.
The 15 affected private controls pass in 210.05 seconds; production/CLI, metadata
codec and lexer 55 controls, all PA6 and all 805 PA13 cases pass in 210.99 seconds.
Both strict Clippy modes, formatting and architecture checks pass. The first
combined cold harness build exceeded its fixed budget; its infrastructure failure
is preserved, and the unchanged warm retry passed. Receipts are
`obj/qualification/fast/1793211677206940905/receipt.json` (SHA
`d2aa09ae8e3d80d273ac57a53971a525b5d11ff7e646b782bea386ce44e31ec0`)
and `obj/qualification/fast/1793212590516114517/receipt.json` (SHA
`1a03ca0d192260ad96414f3769cac63a5c83b41006103d20264eafeb982a6789`).

Twelve additional independent capped Rust observations verify actual deprecation
suppression while malformed/duplicate replacement strings and missing feature
requests remain errors. Original macro-only allowances do not replace that
language gate. The contract is
`pa40/tests/spec/deprecation-suggestion/contract-capped-v1.json`; oracle receipt
SHA is `13232c6ad7c0d3771e2105f37a12f89e69ebfe0a60de6116093a153535e5ab25`.

Joined CLI `6a5c58a25fa1b78894b88f91594336c5a18c48639df35427489ed1d258361c8d`
has separate authenticated uncapped and Allow manifests/bindings. Both verify
all 722 original pins, open 60 sources and stop at the same hard
`unstable_feature_bound` trait-owner boundary in `mem/transmutability.rs:86`.
Neither emits an original crate artifact. The effective target arguments are
unchanged; the new configuration-source hashes explicitly bind the cap driver.
The seven fresh availability-client programs/metadata/source inputs remain
byte-identical to their independent/native-7 checkpoint. Registration, placement
and upstream integrity pass. Postcheck receipt SHA:
`038181876c10882aeca25cac8003ecedcadde4f3d10cb227587098e3ec44c535`.

Broader optional lint parity is deferred while the next work implements the real
trait feature predicate and source panic-handler execution. Typed feature
requirements stay independent of diagnostic caps.
