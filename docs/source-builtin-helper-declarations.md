# Source builtin derive helper declarations

Status: focused producer qualification passed all six selected controls at source
`eb77e5d729d835902b58826c21ab3490d8a40699`.
This producer slice does not enable Default derive invocation, helper
use, proc-macro hosting or foreign macro transport.

The actual unchanged-core observer now opens 78 of the 722 unchanged pinned inputs
and stops at `library/core/src/default.rs`, byte 3492:
`#[rustc_builtin_macro(Default, attributes(default))]`. Its source declaration is a
real `pub macro Default($item:item)` with a compiler-owned derive role. Source
SHA-256: `6280e537b8ef5d1934b44a77cb1115420d01d8adcd19f4f1bc72a0e968c9b90d`. The old
argument reader explicitly rejected every nonempty helper list.

## Shared producer and consumer

The existing claim reader consumes the retained attribute Forest once. The same
cold `panic_contracts::HeaderData` stores `Vec<HelperAttribute>` with canonical NFC
`Rc<str>` identifiers and original token spans; order and duplicates remain intact.
The existing temporary spelling pool shares canonical/raw witness lexemes. The
existing witness trees still verify exact effect/capture/token data. One actual
`OriginalImage` Arc on that same owner seals its source incarnation; cached checked
and published views reject changed images. No second header owner or map is added.
Empty helper lists retain vector capacity zero. Ordinary Plain rows are unchanged.

Actual declaration queries reuse the existing cached declaration-to-source-owner
route through `checked_definition`: the dense owner index stores attributed
wrappers, while resolver Macro DefIds name their unwrapped items. The lookup checks
the actual declaration key, then reuses the same wrapper, image and witness checks.
Scoped builtin declarations retain their exact item mapping. Legacy `macro_rules!`
declarations use their actual Macro namespace and scope registration; the existing
legacy producer intentionally does not publish a scoped item-table row. Checked
non-builtin rows contribute no helper diagnostics. Ordinary crates without a macro
contract owner retain their existing behavior.

`CheckedHeader::derive_helpers(expected_role)` requires the actual derive role and
positive owned header data, including checked empty data. The read-only same-Rc
`HeaderData` method is consumed only after existing selected-header authentication.
Neither method grants a helper on an attributed owner. B's derive transaction must
bind helper use to its actual selected macro, configured owner, source and effect;
Copy/Clone have empty lists and Default invocation remains unavailable. Existing
scoped publication consumes the checked helper view before publishing a derive.

The diagnostic consumer reuses the existing actual-definition bind walk. It checks
the real Macro DefId/module/scope and reads the same checked header; no source scan,
reparse or new header pass is added. The actual declaration introduction supplies
macro provenance. A sorted snapshot of pinned `rustc_feature::BUILTIN_ATTRIBUTES`
(197 names, primary file SHA-256
`0c503aa7399200dc3eaec7aaa8ab39da4d756d6f6f69d0abb7f0a0a0176cf6eb`)
serves only `ambiguous_derive_helpers` classification. It does not admit active
attributes or install helpers. Work is prepaid before lookup using its actual
binary-search/identifier bound.

The existing lint producer and canonical semantic warning/error carrier implement
default Warn and allow/warn/deny/forbid, including `future_incompatible`. The same
Policy adds one byte; this applies to every retained node whenever a known lint
activates the existing dense policy table. No new warning owner or sink is added.
Old warning paths preserve their original introduction zero and output; this
source family retains its actual definition introduction.

## Independent evidence and validation plan

Nineteen original inputs produce 38 metadata observations, preserving 14 wrong
predictions: strict keyword `type` rejects while raw `r#type` accepts, five malformed
forms carry E0565, and the original Bang source's unused decl_macro request warns.
Binding SHA-256:
`7f2b97037eaece442729239c8a177a915a76e588a831a1dd2b838dce35ac731c`.
A distinct Bang setup successor removes only that unused feature and passes two
new observations (`3b3a1731a490f93bdeedc1ca69e8531746382f2c7c0277cc07c7d0b3bd65ee86`).
Ten new lint/source-origin/mixed-error inputs pass all 20 predictions
(`2454dff0ccd0ac02e754e2e4dc716fc714954e0ce5d0f1119bc147c75806a904`).
All 60 observations are metadata-only, zero infrastructure failures, no native or
previous matrix rerun. Original source bytes and failed hypotheses remain retained.

Before measurement: the actual qualified predecessor prints Declaration40,
HeaderData112, SourceCore176, Session16. Its test passed; the tiny observer wrapper
incorrectly demanded empty stderr despite the source's eprintln, and that wrapper
failure/raw output remains retained. A second no-build observer on the immutable
qualified C binary reports Effects368, Producer280, Owner104, Witness72, Frame8,
Call40, Site36, Tail4, Policy11 and Work64. The before bindings are
`040955f5ad1197b62a010c88ac7495437f82fc0a7146cfa4112ef074a3c0d965` and
`e3308fec0186b39d55f0e32bd0cb51e16c2e2b4cbd8721e015140a61da0249b6`.
The passing focused run measures the following bytes. HeaderData's increase
applies to every existing Owned header, including allowance-only headers; Plain
entries retain their existing representation. The policy increase applies to each
retained node in the existing optional dense table.

| Record | Before | After |
| --- | ---: | ---: |
| Declaration | 40 | 40 |
| HeaderData | 112 | 144 |
| HelperAttribute | absent | 24 |
| SourceCore / session | 176 / 16 | 176 / 16 |
| Policy | 11 | 12 |
| Effects / Producer | 368 / 280 | 368 / 280 |

Current `Result<Policy, Violation>` is 32 bytes and its composition row is 40;
their predecessors were not measured. The ordinary no-policy control retains no
dense policy storage. The one-lint control retains four nodes/capacity four:
48 payload bytes, versus 44 at the old element width, plus the existing 24-byte
outer allocation. The affected visibility control independently measures seven
nodes/capacity seven: 84 bytes versus 77. Empty helper vectors have capacity zero;
warm queries increase attempted work without increasing retained bytes. Exhausted
work remains failed on both subsequent retries.

| Helpers | Capacity | Helper bytes | Header payload | Steps | Attempted bytes |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 8 | 192 | 1696 | 1027 | 6240 |
| 32 | 32 | 768 | 4790 | 3623 | 19702 |
| 128 | 128 | 3072 | 17202 | 14367 | 73010 |

Item64, Node36, Scope32, resolution Definition36, macro Body96, macro
Definition136 and Report3040 remain unchanged. The existing carrier control also
reports ErrorData128, one-byte RetentionNote and Engine4088. These are measured
type sizes, capacities and conservative accounting totals, not allocator counts.

Four new private groups cover metadata/canonical names/empty lists, exact source
and header opposition, actual lint scope/origin/carrier and cold/dense/budget costs.
Two existing affected policy/layout controls remain selected with the reviewed
12-byte expectation. The warning group also covers the pinned-primary-backed Bang
collision path. Qualification used only this six-group scope: metadata, full
formatting, global architecture audit, private strict Clippy, fresh private build
and six exact passes in 112.04 seconds. Receipt:
`obj/qualification/fast/1793052933296743618/receipt.json`, SHA-256
`04dc3ff3cb16bfa49f6987a609b446cf6a894ae53e625cbf460f5715f30a1211`;
2789 current source/configuration fingerprints, private binary SHA-256
`6b895c323efba8dbee3a8312f4e2e4c89038e9d87d80946ad08d86943f6bcd17`.
Earlier architecture, typing, lint, formatting and owner/registration failures
remain retained under the handoff; none is counted as a pass.

Production/CLI qualification and a fresh unchanged-core observation belong to the
combined integration. Actual selected derive invocation and helper-use scope stay
with B; Default execution, foreign macro transport and proc-macro hosting remain
closed. PA31 remains planned and no generated references are introduced.
