# Source wrapper identity and validity checkpoint

The maintainer binds actual source `ManuallyDrop` and `MaybeDangling` declarations.
The pinned source registry requires a struct with exactly one written generic
parameter, counting lifetimes and constants as well as types. The role itself
requires no transparent representation or particular field count. Repeated language
attributes on one item and duplicate declarations reject. Actual fields, visibility,
constructors, representation, generic bounds and source identity remain authoritative.
No bootstrap library type substitutes for the source declaration.

`ManuallyDrop` suppresses both its fields' cleanup and an authored destructor.
This uses a distinct `Never` destruction policy. A union's existing `Suppressed`
field policy continues to preserve its own destructor. Actual aliases retain the
role's identity; an ordinary struct named ManuallyDrop keeps ordinary cleanup.
Moving out an owned field preserves that field's later destruction.

The first actual stored field of `MaybeDangling` introduces scoped pointer-validity
relaxation. Typed byte encode/decode work, suspended reference duties and final
value traversal retain this context. Null, alignment, inhabited pointee layout,
metadata/size, scalar validity, unsafe calls, lifetime proof and actual dereferences
remain separate requirements. The context adds no allocation or write provenance
and is absent after a reference is extracted. Direct ManuallyDrop storage gains
no pointer relaxation; its actual MaybeDangling field supplies that behavior.
Owned interior-mutable storage remains interior-mutable. Reference inspection
includes the validity context in its visited key and does not follow relaxed
references. Numeric thin/slice and symbolic pointer paths are exercised locally;
broader numeric object-pointer metadata and imported nominal domains remain open.

## Typed algorithm and storage

A generic unsized-tail destructor fixture exposed an existing source sizing error:
the solver required concrete pointer metadata where the caller supplied an actual
MetaSized/PointeeSized bound. Unknown terminals now descend the existing iterative
trait stack with their exact source goal, scope and child evidence. Ordinary size
obligations retain their separate query. No recursive solver or shared Frame field
is added.

Result validation uses canonical field-list/array recipes, selects an active
product once, and visits `(value, type, validity context)` keys. Image relocations
have distinct provenance work. Queue/table growth is prepaid, temporary capacities
remain charged while other evaluation work runs, and slots restore on failure
without refunding attempted work. Repeated arrays retain one recipe.

Shared graph depths8/32/128 charge116/502/2022 work and peak at87/330/1302 slots.
Repeat lengths1 and1000000 both charge8 work/12 slots. These are bounded-work
observations, not a wall-time speedup claim. Ordinary syntax36, NodeFacts80,
Body456 and Rvalue20 are unchanged. The two appended roles occupy indices78/79;
prior ordinals stay fixed. Optional source Catalog grows1752 to1800 bytes and
Reservations2456 to2496. The existing cold nominal policy table holds destruction
and dangling properties; ordinary aggregate records gain no field.

## Qualification

- Thirty-three frozen source outcomes match pinned Rust, including real constant
  pointer bits without allocation/write provenance, malformed roles, scalar and
  pointer opponents, extraction, sibling memoization and lifetime escape.
- Full maintainer proof/lowering verifies all33. The lifetime rejection requires
  the existing CFG/region/loan boundary; the first test accidentally stopped at
  type analysis. Correcting that test changes no source or expected outcome.
- Four genuine C observers match independently compiled Rust: cleanup/ordinary
  lookalikes `23 7 0 1 1`, own destructor `0 7`, owned move `4 7`, and const/layout
  `7 8 8`. The production suite registers both source and periodic native tests.
- Private19 passes123.82s; production8 and PA12 125/125 pass204.85s. Both gates run
  strict all/pedantic Clippy, formatting and architecture checks. Harness275 and
  upstream verification pass. No release or broad local sweep is used.
- Sixty-four capped/uncapped CLI constant observations pass for32 cases. The full
  lifetime case is qualified by the maintainer pipeline. The library CLI encounters
  general nominal export admission first, so that diagnostic is not counted as a
  lifetime result. The debug loans view also retains its earlier generic-body
  boundary. These failed CLI attempts are preserved.
- An ordinary pre-existing const program produces byte-identical output with the
  prior qualified CLI and the new immutable CLI.

The final private receipt is1793670897191154625,
SHA2560d25c0d31771b53c637751e13a6bad01f38019e244bf4841e78c9455cb86dc7d.
Production1793671099463965960 has
SHA2563791651ffa9ddef5704699fddb3bd4f8701e1c0fe78c85ad839f838922bfc4e7.
Joined-v3 passes35.62s,
SHA25678392028be7f050d6b33c0570e6d2bdb4bd35da72c93faba46baf1b574c61a21.
Raw receipts, failed predictions, the reference fixture ICE, earlier fixture bytes,
static-check failures and unsuccessful CLI modes remain under
obj/parallel/source-wrappers-v1. The packet preserves final source/native contract
hashes and independent receipt1d72f78a9f4f6909da42306d9fcfaf88bfa2238fcd63b2a68a417f1288455070.

## Original core and artifact requirements

CLIe659b8005c6048d7cabc1769f977cb976791e42aef6cb307165cf52f11576e7e
clears the original manually_drop claim. The same original core probe takes
17.474319s, opens375 sources, and verifies all722 frozen input hashes and all six
configuration source bindings unchanged. Target/arguments/capAllow and finite
work14,500,624/attempt1GiB/retained64MiB limits are identical. Its next error is
file57:14080, unsupported language item `maybe_uninit` in
library/core/src/mem/maybe_uninit.rs352. That file has
SHA2564189eaa95d41706cd506ef9dfdb81478640a53ccacc4d88f96329ad3a27e90fb.
The observation receipt has
SHA2565aaeae399a2ed496b6448a41c4fcc5f447998b4788a5e8c64e8c30190e433fd9.

LIB-122's local source component is qualified; its imported/artifact requirement
remains open. Source publication currently has no struct/enum/union definition
kind in the general declaration slots and errors before real library publication.
PA29 must emit, validate, reload and remap actual nominal fields, variants,
generics, constructors and policies, with genuine fresh consumers. Use one general
typed nominal transport model. The existing general error is preserved; no new
wrapper-specific metadata profile is introduced. This is an artifact prerequisite
for core and ordinary source wrappers alike.

LIB-123 owns genuine source MaybeUninit union identity, real transparent storage,
initialization masks and validity, unsafe assume-initialized operations, destruction
and imported authority (PA26/12/27/25/29/31). Existing bootstrap MaybeUninit is not
source identity. Continue source admission and general nominal transport while
retaining full panic/intrinsic, other source hooks, ABI/runtime and original
core/alloc/std requirements. No library artifact or whole PA completion is claimed.
