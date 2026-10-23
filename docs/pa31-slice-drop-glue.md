# Source slice destruction helper

This is the bounded runtime continuation of
[source drop glue](pa31-source-drop-glue.md) and
[actual unsizing roles](pa31-source-unsizing.md). The implementation passed focused source, debug/private, native, strict and
audit qualification on the worker checkpoint. Combined integration/publication
qualification is separate. It does not close
PA27/28, activate PA31 or change the accepted CTFE cleanup boundary.

The exact original source, `drop-glue-slices/original-zst-slice3.rs`, retains
SHA c04f576414232867b062ae3587805059010b1434315d7f84bcf86b60be212eb9.
It already uses zero-sized A elements: its actual source helper must execute
three logical destructor calls. Pinned Rust independently accepts its metadata,
object and native forms and prints3. The original dyn program remains a required
separate positive with stdout1, not a rejection contract.

The new14 frozen sources in `pa31/tests/spec/drop-glue-slices` were independently
qualified against Rust1.98.0 commit88d9e12ae178fab0fb5cc050a94da85685d449ea before
implementation. All28 metadata/object statuses matched, ten native outputs
matched, and there were no ICEs. Four rejections cover shared-to-mutable use,
raw-pointer helper arguments, overlapping unique borrows and lifetime escape.
The immutable source/client hashes and pre-observation statuses are in
[contract.json](../pa31/tests/spec/drop-glue-slices/contract.json).
Three additional sized-array loop regressions are independently qualified
separately (all metadata/object/native outcomes matched); they are not part of
the14-case result.

## Typed recipe and runtime ABI

`body::drops::glue::Shape::Slice { element: Id }` retains one actual element
recipe. Catalog construction visits this dependency once and does not enumerate
runtime elements. `Recipes::slice` validates the full root Slice identity and
its child through the same canonical key policy used by the existing catalog.
Existing nominal region-only recipe sharing is unchanged; equal byte layout
cannot substitute for the actual owner/type application.

The public source helper retains its exact actual DropGlue identity, closed
Instance, completed destructor registration, fully proved original/materialized
Body, source/key/index, and both public and opened signatures. Its selected
Rust ABI still has one native pointer argument: this points to the wide-reference
object. After existing proof checks, the adapter requires actual Slice T,
completed sized element/layout facts, Metadata::Length and the exact retained
16-byte/alignment8 WidePointer object representation of this x86-64 profile.
It loads the data pointer at0 and I64 length at8, then calls the shared internal
`(Ptr,I64)->Void` cleanup function. Sized helper ABI remains `(Ptr)->Void`.
No backend opcode, library API factory or portable C fat-pointer ABI is added.

The runtime loop keeps ascending order and one emitted element body. Length0
executes no destructor; size0 executes one call per logical element at the same
address. Array constant-range validation stays separate from the shared loop.
Stride/alignment use actual retained element layout, and compile-time address
arithmetic remains checked. No pointer end or division by stride is formed.
Reference formation and the helper's unsafe validity requirements still apply;
fabricated lengths, invalid pointers or numeric provenance are not native tests.

## Complete-empty path and proof boundaries

When an ordinary destruction catalog exists, the full selected T must have an
exposed entry, including a real completed None entry. Without a catalog, only
completed-empty registration and the existing bounded sparse concrete NeedsDrop
query can authorize a no-op, after full call and element/layout/ABI gates.
Unknown/pending heads, errors, stale registration or another Store/Definitions
inventory cannot become false. Warm structural answers never grant caller proof.
No Proved flag, verifier drops/catalog pairing, owned-body cleanup plan,
assembly rule or paid-parent proof path is changed for this slice family.

The new slice recipe traversal and emitted loop are measured separately from
the ordinary catalog's inherited dense NeedsDrop state vector, which grows to
Store size. The complete-empty query is sparse. The implementation makes no
Store-noise-independent total-allocation claim for the existing nonempty
catalog, and does not opportunistically rewrite that ordinary query.

Focused controls measure element width and repeated roots, expose exact cold
recipe/query storage, retain failure states, reject wrong child/method/layout
facts, and check invariant loop emission across runtime lengths0/1/8/32/128.
Backend programs are explicitly ignored maintainer tests, with borrowed live
sized C storage and Rust-constructed wide references. Qualification is recorded below. Root owns combined release/text/performance
validation.

## Focused qualification and measured bounds

The final worker compiler source is based on e3db713f. Guard88582 completed0:
format check, architecture/file audit2082/0, all-target/all-feature strict Clippy,
six grouped source/emission tests, two existing recipe tests and nine private
controls. All18 new/preserved source outcomes match the declared contracts
(14 positives, four language negatives); existing11 sized positives/four negatives
also pass. Three ignored native maintainer tests execute25 programs:11 slice,
three shared-array regressions and11 existing sized-helper/order controls. The
unchanged original ZST slice prints3. There are no failed functional controls.
No CLI inspection/text/full-release sweep was run by this worker.

Independent Rust qualification retains the original slice3/dyn1 receipts plus
17 new metadata/object hypotheses (all34 statuses match) and13 new native
outputs. The original14-case declaration and later three-array declaration are
separate immutable plans; no input was silently replaced. No ICE occurred.
The initial audit-only54501 failure (filename stems) and82157 strict failure
(unnecessary semicolon) remain preserved, without functional claims.

Within one catalog, new-root work for element widths8/32/128 is32/101/389 steps;
width8 includes the first cold leaf, later roots reuse that leaf. Recipe counts
are3/5/7. Another128 exposures of each exact root add zero recipe steps or
entries. The same counts hold with8192 unrelated Store types. Total deterministic
recipe charge is2796 bytes; measured capacities are entries8, fields256 and
pending256. These figures are logical recipe payload/capacity, not whole heap.
The inherited dense NeedsDrop query charge grows2908→199516 bytes with that
noise; total catalog allocation is explicitly not Store-noise independent.
The separate complete-empty sparse-query work/readiness controls pass.

Shared slice cleanup emission is exactly8 instructions, four blocks, one index
slot and one call argument for logical lengths0/1/8/32/128. Runtime count is not
expanded into compiler work, including zero stride. Sized-array length1/8 and
nonzero stride/order3 retain their independently checked native behavior.

Measured current sizes: Shape16, Recipe32, Cache280, pending row8, cleanup Work44,
Input1, invocation State32, Context1656 and Body448 bytes. Read-only GDB reports
Shape/Recipe/Cache/Work/Context/Body exactly equal to the retained pre-slice kernel
binary (source b3bde873, SHA97c6e9a5d2c3b2ce033ebcac80fa0646dd914affd5b23bd31d38413de3c117bc).
That historical pin is a layout comparison, not a claim that it is the e3
functional/performance baseline. No global proof/call/frame owner is added.

Tail80854 completed0:164 harness tests,24 pinned upstream fixtures/126 Reference
files, and read-only GDB. Evidence is under
`obj/parallel/pa28/source-drop-glue/{slice-independent-v1,array-regressions-declared-v1,slice-focused-v3,slice-tail-v1}`.
Documentation/contract-state updates after these checks change no compiler or
source/client bytes; their hashes are retained in the handoff delta receipt.

## Required subsequent source-library work

The pinned [drop_in_place/helper boundary](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ptr/mod.rs#L810-L829)
and [Vec destructor](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/alloc/src/vec/mod.rs#L4294-L4307)
are concrete drivers. Full source pointer methods, conditional const/Allocator
obligations, may_dangle/drop checking and allocation semantics remain required.

Dyn cleanup needs the exact proved vtable drop entry, complete concrete exposed
roots and lazy exposed-only recipe packaging even when the local method registry
is empty. A missing proof is not a null entry. Unsized record tails need retained
dynamic offsets/alignment and exact method-receiver ABI adaptation. Actual
[owned_box](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/alloc/src/boxed.rs#L227-L237)
requires genuine Unique/allocator ownership and pointee cleanup before
[Box's deallocation destructor](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/alloc/src/boxed.rs#L1990-L2004).
Ordinary method-before-fields order does not implement that contract.

CTFE helper execution remains explicitly denied; accepted const cleanup sources
remain required positives. Full original Drop/Pin/default override and source
panic/macro requirements remain separate. No result here narrows those goals.
