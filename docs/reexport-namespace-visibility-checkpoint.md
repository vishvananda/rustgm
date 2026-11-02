# Restricted namespaces through public re-exports

LIB-116's reducer exposes an ordinary PA6 name-resolution defect: a public
re-export of a public tuple-struct type discarded its available crate-visible
constructor. Original core calls `num::TryFromIntError` through that shape.

The existing single-import loop now retains every available namespace at its
effective visibility. A namespace that permits the requested visibility makes
the import valid; another available namespace keeps its narrower source access.
An entirely private widening remains an error, and a name inaccessible at the
import site stays unavailable. The same rule applies to a public type paired
with a restricted function and to a restricted type paired with a public
function. Existing glob imports already use this visibility rule.

Actual source targets, namespace/hygiene keys and import origins remain intact.
The completed resolver still enforces privacy at each use. No field, permission,
new cache, parsing pass, metadata profile or lowering rule is added. The existing
bounded three-namespace loop pays each retained immutable source selection.

## Contract and evidence

The [PA6 packet](../pa6/tests/spec/reexport-visibility/README.md) contains fourteen
authored inputs, nine accepted and five rejected. Independent pinned Rust
metadata establishes all fourteen predictions before text generation. Its
first investigation disproved a proposed private-function/public-type rejection;
the earlier prediction, unchanged input and raw result remain in reference-v1.
Later contracts record the correction and add real call/privacy opponents.

| Gate | Actual result | Receipt SHA-256 |
| --- | --- | --- |
| Private/strict Clippy/static | 12 exact checks,129.98s | `97b6fa97d8bddb93ccd51aebb7bf4a07aa8cf11cd14b4de4e723a88587ded05f` |
| Production/CLI strict checks | 27 checks,78.09s | `bf9a2c07bde26d663bdbe326facad6509c9fe78686f982b8a6ce1f56e5090e41` |
| Independent Rust14 | All accepted/rejected metadata predictions match | `68ec896bc73b7bdef1dc53d8b79d65e419259a06aea0134bc1cb2f0ee18c3e97` |
| Joined consumer/text/upstream/harness/core | Passed,35.26s | `bcd6615bf435fe32d1f79819906f8c0aa157404b8f46d760457944f7c2e7fe30` |

Final receipts live under obj/qualification/fast/1793641935365567358,
obj/qualification/fast/1793642104977683698 and
obj/parallel/reexported-constructors-v1/reference-v3 and core-v2.
They bind identical final compiler/tool fingerprints. All28 CLI names/types
observations match the independent outcomes. Private controls compare actual
import targets/origins/access to immutable source selections, run completed
ordinary resolution for privacy opponents and retain all prior storage/hidden-glob
mutation and bounded-work controls.

Restricted alias widths1/8/32/128 retain2/16/64/256 edges over exactly2 shared
source snapshots, paying520/3112/11368/43816 bytes and16/197/793/3153 steps.
Work remains proportional to actual aliases without growing ordinary records.

The scoped generator publishes fourteen new text oracles and two corrected
existing namespace views. The latter each add exactly one restricted alias
binding, with unchanged definitions, references and outcome. Their original
sources separately pass pinned Rust metadata as libraries. PA6259/259 and
PA8120/120 pass; all275 harness tests,386 maintainer roots,44-stage discovery,
24 pinned upstream fixtures and126 reference files pass. Existing full-stage
reference provenance is preserved by two generated scoped batches.

The first joined run already passed all28 CLI observations, then exposed the
second old namespace view. Its failed receipt remains; the final run verifies
and reuses the exact unchanged observations before checking the corrected view.
Two early test compilation errors were repaired to use the actual boxed Error
and DiagnosticFile API. A privacy control initially stopped at expansion,
which defers ordinary body references; it now calls completed name resolution.
No negative language prediction was changed to hide a compiler failure.

## Original core

Immutable CLI
2cca7d7628ab557f5815da31213d7bbe738e2027fb2aca27013d59f84b693d57 passes
the earlier file55:12164 failure. Its17.40s observation stops at
`ERROR: file 199: 33916: unresolved value name`, with375 opens/all722 original
hashes, unchanged target/config/capAllow/finite limits and no infrastructure
failure. The open-order candidate is atomic_swap in sync/atomic.rs797;
its helper uses bare cfg(target_has_atomic), while the caller uses the "8"
value. Bind that source identity and compare the target predicate to pinned
Rust before changing configuration semantics. Open counts are not a compilation
percentage. No original library artifact or whole assignment is complete.

All local gates stay below300s, without release builds. Successful metadata
artifacts are scoped-pruned. Inputs, raw diagnostics and receipts remain;
superseded binary pins retire only after the successor is qualified and public.
