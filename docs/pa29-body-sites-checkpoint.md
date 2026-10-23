# Live body ownership and proof locations

This is the source-compatibility migration after `f92a205e`, not generic metadata
execution or a new wire profile. The original generic/const source family and
full remaining transport requirements remain in
[the source-free originals design](pa29-source-free-originals.md).

`Body`, lifetime `Contract`, checked region results and CTFE `Code` now retain
`BodyOrigin`. Local/temporary/statement/block locations retain four-byte
`BodySite` values. Body-owned obligations, proofs and errors use twelve-byte
`ProofSite`; semantic AST facts still use actual `NodeKey`. Source construction
converts those facts once. The nested output link keeps its existing shared Arc
lineage, with typed origins rather than a copied wrapper.

A source builder, inline registry, source caller, source computation or source
component must explicitly obtain its actual Source origin. An imported origin
cannot become syntax, a fake function body or a source proof owner. Imported
original validation still rejects until its authenticated header and actual
Template/Contract consumer exist. Imported anonymous/provider/conditional
contexts remain required later consumers, not source compatibility fallbacks.

The diagnostic error remains one boxed pointer. Its cold file payload explicitly
distinguishes Source, Imported and Unlocated. Imported labels/spans come only
from the current checked catalog. Missing/foreign/retired catalog lookups fail
honestly. Expansion projection stays on real source diagnostics; it does not
project metadata coordinates through an invented source unit. The borrowed
`BodyOrigins` writer view is one pointer, supplied at driver/view entrypoints,
with no stored pointer on each body or value. Rendering cannot grant proof.

## Measured layouts

The focused current compiler measures these actual layouts:

| Record | Bytes |
| --- | ---: |
| Value | 24 |
| Body | 448 |
| Contract | 184 |
| Code | 304 |
| Target / Request | 28 / 56 |
| Template | 552 |
| Proved / paired proof | 472 / 496 |
| BodyOrigin / BodySite / ProofSite | 12 / 4 / 12 |
| Catalog / owner row / declaration row | 168 / 40 / 24 |
| Error / cold ErrorData before → after | 8 / 64 → 88 |

The canonical Predicate remains twelve bytes and source sema LifetimeProof
remains 120. The body-owned proof with wider locations is 128, and an obligation
is 32. Owned vectors charge their actual element capacities. Catalog sharing
stays cold; ordinary source compilation does not create a catalog or Rc.

## Qualification receipts

All artifacts below are local evidence, not committed outputs:

- `obj/parallel/pa29-executable-originals/live-sites-eleventh`: strict all-target,
  all-feature Clippy; architecture audit (1,934 files, zero findings); registration;
  66 exact ownership/proof/catalog/diagnostic/layout tests. Parent work bounds,
  complete-group proof ownership, failed ancestors, wrong instances, foreign
  numbering units and scope retirement remain exercised.
- `live-sites-tail`: current immutable debug candidate and source-map hashes;
  seven unchanged independently qualified source cases in types/constants/LowIR.
  Every stage-specific outcome matched. No independent Rust/native run was
  duplicated for unchanged source bytes.
- `live-sites-tail-second`: 38 exact tests across the affected integration groups
  and two private checks, with the same source/binary; the warm batch build took
  0.11 seconds. The initial failed selector remains retained in `live-sites-tail`
  and was corrected from the file name to the actual `formation` module.
- `opaque-location-close`: final strict Clippy and architecture audit, eight
  exact private checks, and all 8,362 active text comparisons pass against an
  immutable archive of qualified public `4f3b4751`. No reference/oracle changed.
  The prior two PA21 diagnostic deltas remain retained in `live-sites-tail-second`.
  They were fixed by carrying the actual failing opaque owner through the cold
  preparation failure. Lowering checks its exact function index/declaration and
  real Source node. Unknown/foreign/non-source owners remain Unlocated; no file-zero
  fallback or proof/publication change was introduced. Cold/warm and nonzero
  source-offset controls pass. The Opaques record stays 144 bytes, its optional
  box stays eight, and only the allocated preparation header grows 176 → 184.
  Existing message-only preparation callers keep their exact API and errors.

The final current pin is in `opaque-location-close/pins/rustgm`, SHA-256
`f0935525e69213a39d48885dcde6ad84b58837fa5be8d9005f5ca37c69a4967a`.
Its complete source map SHA-256 is
`a53c2bfea4e5d66441634f6a4c717e108274f59ffd00459f69c9a830da040528`.
The earlier pin and every failure receipt remain available for comparison.
The successful all-target checks collected every target with `--keep-going`.
The source compatibility tail runs no full Cargo/native suite or benchmark;
combined qualification belongs to integration.

## Next required consumers

After source compatibility closes, merge the agreed combined base, preserving
the body Copy carrier, source effect qualifiers and shared Edition alias. Then
install authenticated complete external declaration remaps and typed headers,
retain source-free originals once in the cold pipeline sidecar, and reprove
originals/instances/CTFE through the real existing proof machinery. No serialized
Checked, Permit, Proved, caller or source-computation handle is accepted. Old
profiles zero through seven and stored-array/variadic bytes stay unchanged; a
new wire profile requires schema/validator review and actual executable generic
and const multi-crate results, not just location vocabulary.
