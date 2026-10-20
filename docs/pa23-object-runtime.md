# PA23 executable object development

This journal preserves prerelease implementation evidence. PA23 now has a
qualified implemented-draft contract; see [the release evidence](pa23-release.md).
PA24 and PA25 remain planned. Historical pending statements below refer to
the named intermediate checkpoint; the published prerequisite was `a8219d0f`.

## Runtime representation and proof boundary

Dynamic references and pointers retain a data pointer and a vtable pointer in
16 bytes. A demanded vtable starts with the destructor pointer, concrete byte
size and alignment, followed by method pointers. Methods with unsatisfied
additional bounds have vacant entries. Ordinary type checking must prove those
bounds before a call or reification; table construction does not assume them.

The existing body proof seals the original lifetime and ownership obligations
before executable dependency discovery. `sema/vtable_plans` then erases runtime
lifetimes while preserving complete trait type/const inputs and associated
bindings. Method targets use the ordinary dispatcher and checked template
instantiation. Closed dispatch proofs and method conditions are discharged in
separate witness namespaces. Callable-local late binders remain in their
templates.

Virtual callable identities remain ordinary selected function items. LowIR
emits one forwarding function per complete erased object shape and method
slot, using the existing aggregate/out-pointer ABI. Concrete method bodies,
including default and generic implementations, are demanded through the normal
instance worklist. An unrelated trait implemented explicitly for an object
continues to use ordinary static dispatch.

Object tables expose their concrete destruction recipes through the existing
checked cleanup catalogue. This discovers destructor methods even when a type
appears only behind a borrowed object. A null destructor entry is reserved for
a concrete type with no destruction work. The supported source profile still
does not supply an owning `Box`; these tests exercise borrowed objects and
ordinary ownership cleanup.

Constant pointers now distinguish length metadata from an object recipe.
Backing allocation/projection identity remains unchanged. Promoted references,
constant objects, byte arrays and absolute raw pointers use the same typed
metadata. The persistent-data writer consumes already selected tables.

## Upcasting

The upcast implementation retains a distinct typed conversion. Its
principal/supertrait goals require intrinsic object evidence; an explicit user
implementation for `dyn A` cannot authorize an unrelated upcast. Object
lifetimes, full trait arguments and associated equalities remain proof inputs.
Quantified object conversions enter their scoped proof even when ordinary
inference sees matching type constructors.

Runtime plans join demanded upcast edges with concrete table instances in a
deduplicated worklist. Both discovery orders are supported. Tables append
pointers to demanded target tables; lifetime-only runtime identities need no
table load. LowIR reserves data identities before emitting dependencies and
drains queues iteratively, including recursive references between tables.

## Evidence

All paths below are under `obj/pa24-pa25-work/`.

- `object-runtime-v1.log`: the initial runtime control exposed a fixed-layout
  request while forwarding a dynamic referent. Its statically guaranteed
  address alignment is now one; its concrete alignment remains in the table.
- `object-runtime-v2.log`: a proposed positive function-pointer test was
  independently rejected by rustc because its object lifetime changed. The
  corrected positive and original negative are retained.
- `object-runtime-v3.log`: the negative initially stopped before the lifetime
  proof. The helper now runs complete instance construction and checking.
- `object-runtime-v4.log`: 17 accepted lowering controls and native comparisons,
  plus the function-pointer lifetime rejection, passed.
- `object-runtime-v5.log`: all 24 runtime controls passed through rustc and
  RustGM plus the qualified CPPGM backend. These include conditional/vacant
  methods, associated aggregate ABI, raw constant metadata, generic/const trait
  inputs, higher-ranked owners and destructor-bearing values.
- `object-runtime-clippy-v3.log` and `object-runtime-audit-v2.log`: strict
  Clippy and the 1,093-file audit passed for that runtime snapshot.
- `object-upcasts-v2.log`: ordinary upcast controls passed, but a higher-ranked
  upcast exposed an inference identity failure. The negative controls passed.
- `object-upcasts-trace-v3.log`: isolated inference imports retained a
  context-local object lifetime ID instead of remapping it. The trace also
  exposed this in a raw-pointer upcast. Temporary tracing has been removed.
- `object-upcasts-clippy-v4.log`: the import fix and invariant test passed
  strict Clippy. The invariant deliberately pre-populates the destination
  context with conflicting IDs and preserves the source inference hole.
- `object-upcasts-audit-v5.log`: the audit passed after naming the invariant
  `binder_import.rs` to reflect its ownership and avoid another repeated stem.
- `object-upcasts-v5.log`: the import invariant, all 36 accepted object/upcast
  controls and eight rejection controls passed. Both native groups matched
  pinned rustc through the qualified CPPGM backend.

These focused runs do not replace the full checkpoint, active byte suites, public fixtures or
immutable-binary performance qualification. The PA19 future-stage object
boundary and its oracle have not been changed or blessed.

Successful native batches use scoped storage cleanup, retaining source inputs,
diagnostics and receipts. The runtime-v4 and runtime-v5 cleanups removed 34 and
48 generated files respectively; receipts are
`obj/cleanup/1792516052038797403.jsonl` and
`obj/cleanup/1792516449673739555.jsonl`. The upcast-v5 batch removed 72 files
(0.15 GiB), recorded in `obj/cleanup/1792517591591822679.jsonl`.

## Dynamic storage tails

The unsized-layout query now represents nested object alignment as a bounded
clamp of the terminal vtable alignment. Prefix/explicit alignment raises its
bounds and packing caps them. Each aggregate retains one such summary and
ordinary field offsets remain compact integers. Selected tail offsets are
typed fixed/dynamic plans; field lookup is constant time. Transparent wrappers
with zero prefix need no runtime rounding, and completed layouts remain cached.

Lowering retains pointer-typed vtable metadata separately from integer slice
lengths. Projection and pattern recipes preserve it through nested tails.
Dynamic offsets load concrete alignment, apply the selected clamp, then round
the prefix once. The frontend still rejects packed object-tail declarations
through their ordinary validity checks.

`object-tails-v1.log` exposed a constructor inference bug before lowering:
the expected object was being used as the stored field type. Constructor hints
now infer that concrete field independently and let the outer conversion prove
its object obligations. Array-tail hints still retain their element context.

`object-tails-v2.log` passed all 50 accepted object/upcast/tail runtime controls,
12 rejection controls and eight layout tests. The new tail controls include
two concrete alignments sharing a function, nested and explicit alignment,
patterns, constants, promotion and nominal upcasts. A 1,024-level typed invariant
checks query work and shared transparent layout; it is not whole-compiler timing
evidence. Its successful cleanup removed 100 files (0.21 GiB), receipt
`obj/cleanup/1792518348781210145.jsonl`.

`object-tails-clippy-v2.log` and `object-tails-audit-v2.log` passed strict Clippy
and the 1,100-file audit. `object-tails-regression-v1.log` records the affected
PA22 maintainer checks, including native structural coercion and constant-tail
groups. Cleanup removed 98 files (0.19 GiB), receipt
`obj/cleanup/1792518464458874963.jsonl`.

Container lifetime defaults consume the already decoded written bounds.
Declaration identities distinguish repeated bounds from ambiguous choices.
The supplying application's lifetime arguments are resolved once and retained
for both the object and the parent annotation, including partial annotations.
Ordinary applications retain their existing inference-origin locations.

`container-defaults/results.json` retains independent pinned-rustc probes:
written parameter/where/alias bounds can supply defaults, but implied field
bounds cannot. Distinct declared lifetime bounds are ambiguous even when actual
arguments coincide or one declared bound is static. No public oracle has been
changed for these probes.

`object-containers-v2.log` passed 13 source controls and five new native
comparisons, alongside all 50 existing runtime controls. Strict Clippy and the
1,102-file audit passed for that snapshot. Successful cleanup removed 110 files
(0.23 GiB), receipt `obj/cleanup/1792518816288207627.jsonl`.
Subsequent edits preserve ordinary annotation diagnostic origins and add
conditional-method rejection and object lifetime-relation controls. Those
follow-up checks remain in progress.

`object-containers-v4.log` confirmed the ordinary-origin adjustment, the three
conditional-method rejections, local reference elision and redundant inherited
binding. It exposed two missing intrinsic lifetime guarantees: a short explicit
object bound was hiding the principal trait's static/early-lifetime promise.

The first attempt used conditional object outlives proofs. It passed strict
Clippy, the 1,104-file audit and all 465 library tests, but the integration run
in `object-guarantees-v2.log` exposed an existing argument-component invariant
and one incorrect proposed rustc rejection. Further independent probes are in
`container-defaults/guarantees/results.json`: reference formation can imply
argument bounds, and an explicit object lifetime must satisfy the principal's
lifetime requirements. A function's input can therefore imply a static bound;
its callers must prove that bound and cannot pass a short local borrow.

The conditional-proof changes were reverted. The pending replacement closes
principal lifetime templates once per declaration and retains argument slots
independently of concrete object applications. It maps the object's explicit
lifetime to the principal's required lifetimes during the existing type-formation
walk. The ordinary object argument components and caller proof boundary remain
in force. The updated controls pair the accepted function definition with the
rejected local-borrow call, and distinguish static from shorter input references.
`object-guarantees-clippy-v3.log` and `object-guarantees-audit-v3.log` pass
strict Clippy and the 1,104-file audit; the updated tests are running.

The v3 source run found a caller formation gap: instantiated call signatures
published nominal requirements only. Their full well-formedness requirements now
enter the existing body proof alongside expression and implicit receiver types.
`object-guarantees-v4.log` passes all object source controls and 56 independent
native comparisons, including a valid local borrow of a value implementing a
static trait, an invalid local call to a statically constrained input, and an
invalid local object annotation. Strict Clippy v4 passes. The full regression
checkpoint and higher-ranked lifetime probes remain outstanding.

`container-defaults/higher-guarantees/results.json` confirms that a lifetime
local to a quantified principal/supertrait does not grant an outer static
promise. Three rejection controls now distinguish these local quantifiers and
unrelated argument requirements from intrinsic object lifetime guarantees.

`object-checkpoint-v8/` ran the broad regression checkpoint. All 1,806 default
maintainer tests passed. Native checks passed 115 tests and failed two groups,
both on array-backed slice reborrows. The active byte suites had the same seven
PA16 source failures, one duplicated PA15 formation row, and the PA19
future-stage object boundary. These failures remain recorded without changing
any oracle.

The pending fixes recover array length metadata when a selected reborrow forms
a fat pointer directly from array storage, and deduplicate instantiated call
input formation with ordinary value types. The call result already appears
among expression types; publishing the whole signature had duplicated its
formation requirement. Input types are lowered out of the opened empty callable
binder before joining the shared check.

A new typed ABI fixture inspects the destructor header, then builds an entry
function that loads and invokes that address on a valid transparent object.
It compares the resulting counter with rustc's ordinary destructor execution.
This exercises the emitted table entry; it does not claim an owning Rust object
pointer or `Box` implementation.

`object-regressions-v9/` passed the affected object, slice-pattern and wide
reference native suites, including direct invocation of the destructor table
entry. PA15 and PA16 returned to exact byte equality. It exposed one duplicate
nominal proof for a diverging call argument in PA19. The file audit requested
an ownership-specific filename: the ABI test is now `erased_cleanup.rs`.

`call-inputs-v10/` passed source controls, the callable-contract subset,
strict Clippy and the 1,105-file audit, but removing the complete signature's
nominal proof changed existing PA19/PA20 semantic rows. The follow-up retains
that proof and publishes only the additional lifetime-formation obligations
for each instantiated input, sharing their deduplication with actual values.
Diverging arguments do not request new concrete field storage. That correction
is under validation in `call-inputs-v11/`.

The failed full checkpoint's disposable artifacts have been investigated and
pruned: `obj/cleanup/1792521335665451042.jsonl` records 9,637 removals and
13.70 GiB recovered. The focused v9 runner names its native output directories
under the existing storage lock; its scoped cleanup removed 210 files and
0.44 GiB, receipt `obj/cleanup/1792521784826475376.jsonl`. Sources, diagnostics,
timing observations and current pinned binaries remain intact.

`call-inputs-v11/` passes the object source controls, six callable-contract
tests, strict Clippy and the 1,105-file audit. All 7,386 active byte cases were
compared: 7,385 match exactly; only the deliberately unchanged PA19 trait-object
boundary differs. No old oracle was regenerated. `object-final-native-v11.log`
then passes all five object native groups (57 runtime controls including the
table-destructor fixture). Cleanup removed 114 generated files and 0.24 GiB,
receipt `obj/cleanup/1792522332875496954.jsonl`.
