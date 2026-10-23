# Direct generic calls: row version 1

This is a focused implementation checkpoint, not assignment activation or
upstream-library completion. Combined integration qualification remains root-owned. It follows the executable row-version-zero checkpoint in
[pa29-executable-generics.md](pa29-executable-generics.md). The original eighteen
scenarios remain required, including real source trait identities, references,
lifetimes, associated conditions and wider operations. The companion
[source sizing proposal](pa29-source-sizing-next.md) describes the actual core
identity declaration boundary without enabling source-core import policy.

The first family carries module-level safe generic helpers from the actual
Checked originals. One indexed dependency closure retains each actual original
once; all function identities are sorted canonically, including private helpers.
Private dependencies remain private and are restricted to the same actual
module. Callable recursion is permitted; the independently validated lexical
owner forest remains acyclic. No executable proof, permission or synthetic
source function is serialized.

Envelope 9 remains unchanged. The three-byte descriptor is `[1, 0, 0]` for this
new row version: Rust 2024 with the legacy language-role policy. Version zero
keeps its exact byte encoding and rejects these new forms. Versions or policies
other than those explicitly implemented reject before publication.

The existing header tables precede a version-one application table:

- `header_types: u32`, the complete prefix of existing type rows;
- application count and contiguous `{ target: DeclarationRef, arguments: Range }`
  rows, each field encoded as explicit little-endian u32;
- argument count and a flat sequence of `TypeRef` u32 roots.

Each complete application has exactly two ordered suffix recipes: type tag 3
is `FunctionItem(ApplicationRef)`, tag 4 is its `ApplicationSignature`. Its full
actual arguments reference only the header prefix. No application recipe can
refer to another recipe. The target's full formal inventory includes unused
parameters. Every argument must belong to the actual caller's parameter owner;
a root valid elsewhere in the same artifact is insufficient. Exact duplicate
applications and unused or reordered recipes reject.

Existing operation tags 0–2 retain their encoding. Value tag 3 contains a full
application index. Value tag 4 contains `{ signature: TypeRef, callee: u32,
arguments: Range, site: u32 }`. Every original ends with a version-one argument
count and its flat u32 value IDs; version zero has no such suffix. Argument
ranges are ordered, complete and backwards-referencing. The operation source
site comes from the retained typed Evaluate statement, never from reparsing.

Import authenticates all real declaration/body mappings, builds all headers,
derives each exact target/argument signature once, then completes body types
and locals. The canonical application cache shares type structure and pending
goal templates. It shares no caller proof. Import reconstructs formal
well-formedness, default Sized, declared Copy/Sized and every call input/result
coercion as caller goals under that original's exact Environment. Specialization
must still substitute and prove them after clearing original assumptions.
Ordinary original/group proof, instance proof, eligibility, promotion, loans and
exact warm provider lineage remain mandatory.

Counts, retained rows and scratch share the existing component/16 MiB metadata
budget; canonical application folders reserve before allocation and charge both
successful and failed work. Imported bodies stay in the one immutable shared
ImportedOriginals allocation. Value, Body, Contract, Code and their ordinary
execution authority are unchanged. Source language-role contracts, unsafe or
indirect calls, non-generic dependency originals and wider physical ports remain
explicit boundaries; they are required follow-ups, not language rejections.

## Implementation and evidence

The isolated development base is `b43860548059592cbe0393276e556b984b7c8ca6`;
the previous v0 handoff `536d29c8` remains in history. This branch intentionally
does not absorb unrelated production fixes from later integration cuts.

`pa29/tests/generic-call-outcomes.json` freezes twenty independently declared
source outcomes. The first fourteen Rust outcomes and four native executions
passed in `obj/parallel/pa29-direct-calls/rust-first/receipt.json`; the six
additional scaling sources passed full Rust object emission in
`rust-scaling/receipts.json`. All twenty candidate CLI outcomes match their
explicit table, including the safe-only unsafe-call frontier. Runtime and const
clients execute after artifact roundtrip; recursion is accepted as a callable
edge and demanded recursion reaches the actual constant frame limit.

The evidence root is `obj/parallel/pa29-direct-calls/`:

- `fifth`: successful all-target/all-feature strict Clippy and independent six-source
  Rust qualification. The later audit parser/stem failures remain retained.
- `sixth`: architecture/file audit (2,062 files, zero findings), registration,
  three focused integration and eleven private tests. These include missing Copy
  payment, a foreign function's parameter root, a mismatched call result and
  recursive owner separation. Its native launcher failure is preserved.
- `close`: the corrected native launcher passes four actual native clients;
  all 43 metadata tests pass, retaining the old profiles and stored/variadic
  golden paths. All twenty CLI outcomes pass. The previously qualified v0 pin
  `d0315bf1...` produces exactly the same 488 metadata bytes as this candidate:
  SHA-256 `8ed2e9f4f4c0515f9f6e813ec7b884a4ff4b31df956cb47449c1e344d5d567fd`.
- `final-second`: strict Clippy passes after final sorting accounting/control
  changes; its only subsequent failure is the private test filename prefix.
- `final-third`: renamed-file audit and upstream verification pass; one
  runtime/const integration plus three directly affected private controls pass.
  This reruns actual helper-count/call-count preparation after sorting accounting,
  tests retained work on quota failure and rejects a missing comparison key.
  All twenty CLI outcomes pass again, and every successful artifact is byte-equal
  to the earlier closed candidate. PA24 passes 470/470.

The final immutable debug CLI is
`obj/parallel/pa29-direct-calls/final-third/pins/rustgm`, SHA-256
`82f8dc59d59cf5b26a597b8c7f818b70b465bf99df9e9f1be82de80a7d755fe2`.
Each batch records commands, source maps, executable hashes and exact selected
inventories. Failed launchers and storage refusals remain in the evidence.
There is no full-suite or broad performance claim from this worker.

All PA1–24 text inputs/references come from an immutable archive of the exact
base commit above. The combined local result is **8,381/8,382**: PA1–22 and PA24
match, while PA23's unchanged `explicit-markers/negative.rs` is accepted on this
base despite its declared rejection. That inherited admission defect is fixed
by root's later `9c35d2eb` / `eb95795d`; no fixture/oracle was changed or
acceptance blessed here. The final receipt retains the source hash and output.

## Work and storage

Canonical dependency sorting observes each key size once and charges input
visits plus a conservative complete-key bound per comparison. Missing keys and
exhausted work reject; consumed work is retained. It never walks the body or
recomputes key sizes inside comparisons. Import derives one signature/goal
record per complete application; each use pays only its actual call goals.

The recorded import preparation observations below are from `close` (before
the final producer sorting charge); the directly affected axis/caps/repeated-use
control passes again in `final-third`. These importer algorithms and successful
artifact bytes are unchanged by the producer-only charge. They do not substitute
for the separately rerun sorting budget control.

| Axis | Width | Applications | Header work | Header reservation | Original work | Original payload |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Calls | 8 | 1 | 106 | 9,804 | 305 | 13,104 |
| Calls | 32 | 1 | 130 | 10,572 | 1,073 | 37,824 |
| Calls | 128 | 1 | 226 | 13,644 | 4,145 | 136,704 |
| Helpers | 8 | 8 | 484 | 44,216 | 473 | 29,392 |
| Helpers | 32 | 32 | 1,804 | 162,968 | 1,817 | 115,072 |
| Helpers | 128 | 128 | 7,084 | 637,976 | 7,193 | 457,792 |

Repeated consumer demand (one versus eight uses) leaves preparation counts and
retention exactly unchanged. Work/payload growth stays within the checked
one-axis bounds. Sorting controls use observed full-key sizes, monotone width
checks and the fixed global work limit, including failure work; no enlarged
quota is introduced.

Measured cold layouts: Headers 136 bytes, canonical Application 64, Rows 232,
Payload 336, Calls 56, raw Original 232, raw Value 28 and raw Type 20.
The additional owned records are included in retained preflight and actual
capacity accounting. Ordinary execution models remain Value 24, Body 448,
Contract 184 and Code 304; canonical Predicate remains 12. No-import storage
stays absent, and v0 has no Calls allocation or flat argument payload.

Changed-path performance inputs are the six manifest-backed
`pa29/tests/spec/generic-contracts/call-scaling-{calls,helpers}-{8,32,128}.rs`
providers. Each exports `through<T:Copy>`; the matching client is
`fn main(){let _=base::through(7u8);}`. They vary call count versus distinct
helper count, not caller proof sharing. The original `call-library.rs`,
`call-const.rs` and `pass-helper-unused.rs` supply the runtime/const controls.
Their Rust receipts and candidate artifact paths are retained above. Root owns
immutable performance comparison and combined qualification.
