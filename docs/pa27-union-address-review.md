# Raw addresses into scalar union Images

The PA27 constant machine now forms raw addresses of fields in its existing
all-fields primitive scalar union Image profile. Distinct field ordinals select
the same current parent bytes. Cross-field writes are visible through previously
formed pointers, while copied union values retain immutable prior snapshots.
Every selected read checks its initialization mask and scalar validity. Merely
forming a raw address does not read an invalid alternative representation.

`ProjectionLayout` now carries the exhaustive typed
`ProjectionStorage::{Product, UnionScalar { field: Selection }}` distinction.
The reader uses the retained canonical union owner, field group, ordinal, closed
field type and prepared ABI offset. Union scalar projections require a prepared
Image family and offset zero; enums retain their unsupported boundary. The
bounded `Evaluator::union_address` checks the exact base owner and nominal field,
prepares the existing scalar family, checks its work budget before publishing a
projection, and retains the full Pointer write permission.

`Value`, `View`, `LocationId` and `Origin` variants do not change. The existing
`Origin::Projection { base, index }` records a typed view, rather than an
independent field allocation. A private `StorageTarget` normalizes a terminal
union ordinal to its canonical parent Image. Reads decode that parent's current
bytes; stores use the existing `union_write` snapshot update. The machine never
thaws an Image into product fields and introduces no active-member flag. Nested
ordinary product/array paths retain the existing mutable cell traversal. The
normalizer checks only the leaf projection and performs no ancestry rescan or
new ABI walk on an ordinary read/write path.

All ProjectionLayout consumers were audited: raw physical-offset computation,
the union storage normalizer and private typed invariants consume the new tag
exhaustively. Constant inspection retains explicit Image bytes/mask and
projection-alias rows; `Reader::field_types` still rejects structural union
interpretation. Lowering already follows canonical sole-variant ABI field
offsets for pointer relocations. Native controls prove that differently typed
static field pointers have the same address; no backend operation or relocation
vocabulary changes.

The existing exact-type, bounds, lifetime, shared-pointer and raw alignment
checks remain independent. A packed union-base pointer can store a field by
value, while a separately formed unaligned u32 field pointer cannot read/store.
Mutable-static raw addresses can be stored as constants, but reading mutable
static contents during CTFE still rejects. Runtime unsafe mutable-static pointer
use remains legal and does not inherit a CTFE write grant. A named-constant
union temporary still rejects raw-address formation; no promotion restriction
was relaxed. Ordinary unsafe access and source ownership checks remain intact.

The predeclared corpus has 32 full-object outcomes checked through type, LowIR
and constants views. Its initial 29 baseline observations are retained. Three
later generic/static integration controls were independently declared and
qualified separately; they do not claim a baseline frontend observation.
Opposing controls cover initialized and uninitialized ranges, invalid bool/char,
invalid alternatives without reads, shared-derived writes, alignment, null and
expired roots, nested storage, copied snapshots and generic returned pointers.
Six explicit native groups compare exact values and effects with pinned rustc.
Native tests remain ignored by default. Two private invariants cover permission
retention, canonical owner/field capabilities, budget rejection and cached layout.

At 8/32/128 loop iterations, changed-path execution uses 938/3314/12818 steps
with 42 retained execution slots. Image updates retain the existing bounded
immutable arena, initialized masks and byte/clone work charging. These are linear
work and fixed execution-slot observations, not a wall-time neutrality claim.
The separately rerun ordinary/raw-store controls retain their prior counts.
Strict all-target/all-feature Clippy and the 1,358-file audit pass. The initial
test-only unused-mut warning and duplicated qualifier block are retained as
failed gate evidence; the latter now uses one shared test harness.

The prior scalar static-field-address and raw union-base store boundaries are
explicitly superseded without changing their fixture bytes or historical
receipts. Pointer/reference/function/aggregate-bearing Images, general byte/type
reinterpretation, raw-reference reconstruction and shared-derived interior writes
remain explicit unsupported profiles. No assignment activation, active reference
regeneration, full-suite claim or CPPGM change is included. Exact source, object,
native and gate evidence is in `pa27/tests/union-address-outcomes.json`.
