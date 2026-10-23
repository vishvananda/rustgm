# Source dynamic destruction helper

This focused runtime continuation follows [slice cleanup](pa31-slice-drop-glue.md)
and the exact source-bound [DropGlue contract](pa31-source-drop-glue.md). Central
combined release/performance qualification is recorded in the
[integrated checkpoint](source-capabilities-dyn-returns-checkpoint.md). PA31 stays planned.
The unchanged original dyn source has SHA-256
`54ef238fb38e8eaa7a8c9a03cc581aeb430617f820455dc6f233442061ed3deb`.
Pinned Rust 1.98.0 accepts its metadata/object forms and its native observer prints 1.

## Typed runtime and proof boundary

`glue::Shape::Dynamic` is a leaf with the full selected Dynamic TypeId, including
trait arguments, binders and region identity. It does not select a concrete
source Drop method. All existing actual-role, exhaustive destructor preparation,
closed Instance, checked original/materialized body, caller, source/key and dual
signature checks remain mandatory. Fresh registration checks precede warm data
reuse; a completed structural answer grants no caller or const permission.

The public helper still takes one pointer to the actual 16-byte/alignment8 wide
reference object. Its metadata must be exactly `Metadata::VTable(selected_T)`.
Typed loads at 0/8 feed internal `(Ptr data, Ptr table)->Void` cleanup. The leaf
loads the table's existing drop entry at 0: null returns unit; non-null invokes
the complete concrete `(Ptr)->Void` cleanup with the data pointer. Header size,
alignment and method offsets stay unchanged. There is no new backend opcode,
C fat-pointer ABI, source API factory or inference of ownership from pointer bits.

The first nonempty exposure batch activates the existing optional recipe cache,
including programs that construct a vtable before ever demanding DropGlue.
Already-drained table roots are retained once. A concrete table requires its
exact exposed entry, including completed None; missing exposure is an error.
Dynamic itself is never treated as dropless merely because the local registry
is empty. The original body-owned proof mode is unchanged: Verifier still pairs
its drops/catalog inputs exactly, and every Proved flag reflects program.drops.
Exposed-only packaging has zero body-plan capacity and cannot supply a body
cleanup proof. Empty catalogs and invented plans/flags are rejected. Ordinary,
adapter, paired and paid-parent paths pass a catalog to body proof only in that
original owned-cleanup mode; mechanical lowering may also borrow exposed recipes.

## Lazy arena binding and bounded work

The new complete-empty exposure family uses one bounded sparse NeedsDrop query.
Store and Definitions each gain one private lazy Arc identity slot. Mutable arena
access initializes tokens before exposure; retained tokens are compared before
every warm query/recipe/exposed hit. Moves and append-only preparation preserve
identity; Default/replacement and equal numerical IDs in another inventory do
not. No unsafe code, raw-address identity, public token or arena Clone is added.
The cold boxed Scope owns the binding; Cache, Builder and Body gain no fields.
The existing dense cache retains its same-environment caller precondition and
does not acquire this new runtime identity check.

The exact family shares the existing 64MB/one-million-work limits across query,
recipes, maps and pending slots. Growth preflights include temporary replacement
capacity; failed work is charged and cannot revive warm false answers. Capacity
accounting reads a fixed set of collection headers. Charges cover logical payload
slots, cold boxes and conservative Arc control-block retention, with allocator
headers/hash control bytes excluded as in support::retained. These are policy
charges, not whole-heap measurements. With no exposure, no recipe cache or
identity token is allocated. A table-only program now pays the explicitly retained cold cache.

Measured debug layouts: Store 464→472, Definitions 256→264, Scope 48→56, Binding 16;
Declarations 7048→7064. Five dependent aggregate assertions were updated exactly.
Cache 280, Shape 16, Recipe 32, Body 448, Work 44 and Context 1656 retain their prior
sizes. Builder measures 1696; its field inventory is unchanged. A bound query
after one primitive answer charges 400 bytes, including retained token controls.

| Distinct dyn roots | Recipe steps | Query bytes | Combined high-water charge |
| --- | ---: | ---: | ---: |
| 8 | 16 | 752 | 2908 |
| 32 | 64 | 2096 | 8968 |
| 128 | 256 | 7472 | 33208 |

These counts are identical with 8192 unrelated types. Repeating a warm exposure 128
times adds no recipe work or capacity. Existing nonempty dense-query costs remain
explicit: the preserved slice control charges query bytes 2908→199516 with Store
noise. No total-allocation independence is claimed for that legacy path.

## Independent outcomes and focused evidence

[The fixture contract](../pa31/tests/spec/drop-glue-dyn/contract.json) retains 14
exact sources: ten runtime positives and four rejection controls. C clients pass
valid live thin storage; Rust constructs all wide references. Native outputs
cover the original 1, null 0, mixed concrete 23, complete method-before-fields 42,
reified 1, ordinary lookalike 0, no-exposure 0, header-only 7 and paid-parent 1.

Three original hypotheses remain separately preserved: a provider missing every
DropGlue role accepts metadata but rejects object emission; an overly general
reified object pointer rejects E0308; a method-bearing dyn context rejects E0038
without its actual DispatchFromDyn setup. Distinct corrected providers do not
replace those bytes or turn their gaps into broad language rejection contracts.

Worker evidence lives under `obj/parallel/pa28/source-drop-glue/`:

- `dyn-focused-v4`: fmt, audit 2090/0, all-target/all-feature strict Clippy,
  26 exact private controls and 10 grouped source/ownership tests passed.
- `dyn-close-v1`: all ten native outcomes, five dependent layout controls,
  strict/audit, harness 370 roots/13 binaries/44 stages, pinned upstream 24 and
  Reference 126 checks, and read-only GDB passed. Successful cleanup removed 20
  generated native artifacts after their hashes were retained.
- `dyn-native-observation-v1/artifacts.json`: hashes for all ten native/object
  artifacts, LowIR, clients and execution diagnostics before scoped cleanup.

Controls include wrong Store/Definitions, equal-ID replacement, moves/appends,
tiny-budget preflight, failed warm answers, full dyn binder/region identity,
fresh registration tampering, exact ABI metadata, empty/invented proof rejection,
and first table exposure→drain→later helper demand. Earlier audit-name/default
style failures and the reservation refusal remain recorded; no functional
failure remains in the declared qualified family. No worker full release/text
sweep or timing run was performed.

CTFE cleanup still rejects before caches/placeholder execution. Actual owned_box
pointee-before-deallocation ordering, unsized tails, full Drop/Pin/default bodies,
DispatchFromDyn and the other retained PA27/28 requirements remain separate work.
A further nonzero payload-address opponent was declared in obj only and has not
run; it is not included in any qualification total above.
