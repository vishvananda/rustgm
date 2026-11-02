# Source documentation-label checkpoint

Source d6e091a3f implements the compilation-required rustc_doc_primitive label
that stopped original core in primitive_docs.rs (LIB-115). It qualifies the finite
source/CLI/native contract and advances the observed source frontier. Whole PA40
and original core/alloc/std artifacts remain required.

The existing configured-owner walk validates a real parsed module, rustc_attrs
or the actual macro attribute-envelope allowance, a string value, hard duplicates,
wrong owners and unsafe wrappers. Any string is valid: unknown, empty, Unicode,
escaped and raw inputs match pinned Rust. A sparse prepaid set of FileId/Item
identities catches duplicates across inner/outer captures of the same module.
Per-owner coverage/destructor flags still reset independently. The draft drops
at the end of the pass; retained AST literals already carry their source origin.
No label grants primitive identity, a language item, special layout, const/unsafe
or privacy permission. Ordinary module bodies remain checked. Normal metadata
export stays valid because the label adds no semantic transport contract.

The [42-case packet](../pa40/tests/spec/primitive-documentation/README.md) was
declared before independent execution. Its initial40-case predecessor is frozen
with a passing receipt; the two extra controls exercise mixed capture duplication
and genuine struct layout. An external-module input opens its separately hashed
child source. Broader logical module/edition/attribute closure remains required.

## Qualification and CI repair

All final gates bind the same3307 compiler/tool fingerprints.

| Gate | Actual result | Receipt SHA-256 |
| --- | --- | --- |
| Private/Clippy/static | 14 exact checks,118.59s | `91097d31b9270dfeb7bccbbdb32ebd9b5e61c98974910f9fbde9cc223815f88d` |
| Production/text | 58 checks +PA8120/PA9203=323 text,123.01s | `9f5a6b7b749613fd0c5ec8b513ebc6d354afd110bfabdb76c85a17431673a1cd` |
| Independent Rust | 84 metadata/object outcomes +native7 | `9ebf53895681a3b4686292e82985ebbcc004a062100c5e9c92fe9a4f1725e9eb` |
| CLI/native/upstream | 42 exact CLI inputs, original C observer7,3.81s | `c493f85b163f409030817ae9bb611989b90a0c8ec540161ecf7b58c59daab082` |
| Original core observation | Same722 hashes/finite profile,16.05s | `27c797df63976932486c67c2eb5a817bcc23384d1d8631cb3fff9da79a19ab15` |

Private execution includes five new documentation controls, two prior genuine
SIMD controls and seven exact compilation-owner layout guards. The new controls
cover source forms/targets/gates/macro envelopes, ordinary type/body/unsafe/const/
move/privacy behavior, metadata export, native lowering and bounded/sticky input
costs. A two-field source struct remains an ADT of size16/alignment8 even under
a u64 documentation label. The exact native source lowers through RustGM and
the unchanged pinned CPPGM object bridge and matches Rust's C observer output7.

Widths8/32/128 pay source input-work240/960/3840 and bytes2768/11072/44288.
Sparse duplicate-state growth is separately prepaid. The AST Node36 and Body456
remain unchanged; no permanent per-module label cache was added.

Predecessor [full CI41481896869](https://github.com/vishvananda/rustgm/actions/runs/41481896869)
exposed seven stale size assertions after SIMD added one optional pointer to the
compilation-wide Definitions owner. Actual Declarations size7088 differs from
7080 by8 bytes once per compilation. The other2513 private tests passed. Testing
fix9e12b64fa updates all seven exact formulas to include that pointer and documents
the cost, preserving their other ordinary-record and hot layout guards. All seven
now pass locally together with the new implementation. Full successor CI is
tracked separately; predecessor failure is not called a complete passing run.

The first file audit caught an overloaded primitive filename family; the module
was named documentation_labels to reflect its actual ownership. A mixed
inner/outer duplicate initially bypassed the per-owner flag. The final draft
uses parsed module identity across captures; the independent required negative
was never inverted. Clippy's key-only map warning was corrected with an idiomatic
HashSet, without a lint allowance. All failure logs remain. Every local gate
stayed below300s including admission, with normal resource/storage guards and
no release builds. Successful generated native/object artifacts are scoped-pruned;
inputs, raw diagnostics, receipts and the warm cache remain.

## Current original-core boundary

Immutable CLI6076cc08c184bb7296bd73eada77f547a2bebee1455c2f676891de492a15da96
passes the previously unresolved documentation annotation and now reports
source lint policy storage budget exceeded at file2 (prelude/v1.rs). This is a
recurrence of existing LIB-046, not a new language-feature blocker. The attempt
loads375 original source units, all722 hashes remain unchanged, and the same
capAllow, target and finite work/storage limits apply. No policy cap was raised,
no input source was patched, and no core artifact or full semantic pass is claimed.

Measure retained owner/witness records and cumulative temporary/query allocation
charges in the existing source lint producer. Its policy() currently constructs
and authenticates an ancestry path before consulting the composed-policy cache;
that is a concrete investigation target, not an established cause. Distinguish
actual stored data, conservative allocation charges and repeated query work before
choosing an algorithm. Preserve source incarnation/mutation witnesses, hard
allow/deny/forbid rules, real expectation identity and sticky budgets. Broad
advisory parity stays deferred; compilation-required authority must remain real.

LIB-114 imported SIMD policy, LIB-113 imported optimization intent and LIB-112
receiver/Box/edition/metadata closure remain open alongside all earlier runtime,
ABI, CTFE and original-library requirements. No whole assignment was completed.
