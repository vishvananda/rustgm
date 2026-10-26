# Source PhantomData checkpoint

Qualified compiler/test source: `79b46b8c2bf9e18edb83edded36059062e617b3d`.
It normally merges public `339bdffb2` and C's separately qualified Foreign source
`27b1712ddc2ed6163dafb02561aefb79ed0ba76d`. This is a local source marker
checkpoint, not a completed Location, panic, formatting or assertion pipeline.

## Real source and authority

The source fixture `pa31/tests/spec/source-phantom/source-marker.rs` keeps the
pinned Rust 1.98 unit declaration `PhantomData<T: PointeeSized>`, unconditional
source Copy/Clone/Freeze impls and the actual Clone body `*self`. Its argument
has a real destructor and no Copy/Clone impl; cloning the marker must not add
such a premise. Real sizing, receiver, const Destruct, Drop and intrinsic
headers remain required. This reduced provider does not replace full core.

`LangItem::PhantomData` appends ordinal31. All old ordinals remain unchanged;
the control checks all32 indices and the full u32 mask without shifting by32.
Source kind/formal/bound/owner validation precedes publication of the existing
aggregate-kind byte as `aggregate::Kind::PhantomData`. No new node, type, Body,
Context, definition field, cache or ownership graph is added. The canonical
type remains the actual nominal `Adt(ItemId, GenericArgs)`.

The borrowed `Definitions::phantom_argument` reads its actual argument. The
existing variance solver makes it covariant; auto-trait and logical dropck
walkers follow it through their existing edges and caller proof. Physical
layout, storage finiteness and emitted destruction keep zero actual fields.
The tag grants no Copy, Clone, Freeze, callable, const or imported permission.
Source impls and their full header/body/instance proofs still decide those.

Nominal role completion runs after the existing deferred bounds/fields phase,
independently of earlier callable-role completion. It uses the same existing
cold role ledger and publishes no partial role on failure. Ordinary crates and
ordinary empty structs create no new owner; a real same-name ordinary control
retains Struct and no source bindings. Existing v0/v1/v2/Provided role whitelists
and profile-local wire IDs deny PhantomData transport.

## Focused evidence

Standard fastcheck receipt:
`obj/qualification/fast/1793043936284694186/receipt.json`, SHA256
`2b87e0d7a696d91ed477703d8825c01f6c0e4dd69edc3b3a542a1ed9928b5d21`.
Metadata, formatting, global architecture audit, private Clippy, private build
and three exact groups pass in114.61 seconds. Production/CLI strict integration
is owned separately by the integrator.

The groups cover actual source constructors, const size/align/needs_drop,
actual Clone call and LowIR; malformed shape/formal/kind and lifetime escape;
logical ownership, physical no-drop, 1/8/64/128 dependency depths, warm reuse,
exhausted work, ordinary namesake and compact containing layouts. Cold query
work stays at most8d+16, and warm completed queries add no steps. The lifetime
opponent uses actual CFG/ownership/region proof through ordinary lowering.
Type analysis alone is not a lifetime proof; bare reference contracts without
checked borrow-place provenance are not a substitute either.

New independent reducers settle covariance, unsized arguments, lifetime escape,
destructor-sensitive dropck, Unpin/Send argument dependence, ordinary unused
formals and no physical destructor. The native std-backed program prints
`0 1 false`. One reducer initially used the reserved keyword `unsized` as a
function name; its failure and the one-source correction remain separate.
The initial no_core source observation lacked an actual trait import and the
`freeze_impls` request. Both diagnostics remain beside the corrected pass.
No old Pattern/transmute/formatter observations were repeated.

The actual reduced source then passes new object/native observation with pinned
rustc SHA256 `3690cc576ede140504698405d5d8fa3826aaadbe71699c6c4ed0a565d6f493e2`.
The same C observer links to candidate LowIR through the unchanged pinned CPPGM
backend. Both native programs exit0 with empty stderr and stdout `0 1 0 7`.
Profile: edition2024, x86_64-unknown-linux-gnu, panic=abort, explicit UB checks
false, default overflow/debug flags, bootstrap only for actual internal source
declarations. This is real runtime evidence, not an object-only bit inference.

Native/before-layout receipt:
`obj/parallel/pa31/source-panic-location-v1/native/verified.json`. The frozen
inputs include2824 candidate source/tool/fixture fingerprints and2781 actual
root production-source fingerprints. The actual immutable root rlib supplies
the before observation; no model structs, private rebuild or binary copy.

| Actual type | Before | After |
| --- | ---: | ---: |
| Aggregate Definition |36|36|
| Aggregate Definitions |264|264|
| Type Kind |24|24|
| Type Store |472|472|
| Package |256|256|
| Report |3040|3040|
| Body |448|448|
| Cold Reservations |888|912|

AST Kind28 and Node36 also pass unchanged. Actual after Catalog is792; a before
private Catalog value was not observed, so no Catalog delta is claimed. The
Reservations growth24 bytes is the real final source-role array-slot cost.
Generated native objects/executables were hashed before official scoped cleanup;
source, streams, failures and receipts remain.

## Remaining source path

Location still requires its genuine NonNull<str>, PhantomData<&str>, NUL-backed
filename allocation, caller_location and track_caller ABI/const semantics.
PanicInfo and real tagged Arguments, handler binding, false Assert and const
failure remain required. Actual Location derives Copy/Clone and PanicInfo's
Debug derive cannot be stripped or replaced. Future role slots beyond31 need a
reviewed bitset-width change. Full imported PhantomData transport, complete
upstream auto-trait families, stability/FFI consumers and may-dangle remain open.
Independent rustc rejects unused ordinary nominal formals with E0392; the
pre-existing candidate rejection gap is source-inferred, not qualified as fixed.
