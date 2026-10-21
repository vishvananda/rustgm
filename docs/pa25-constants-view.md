# PA25 constants inspection draft

Status: implemented contract under maintainer qualification; PA25 remains planned.
No generated reference or assignment activation is implied by this document.

`rustgm --emit-constants SOURCE` checks a crate without requiring `main`, consumes
its retained checked body program, and prints the completed constant inventory.
`--count-constants` performs the same checks and constructs the same inventory,
then prints only its counts. An entry-free library needs no crate-type option for
these inspection modes. Artifact generation and dependency CLI options retain
PA29's separate LowIR profile restrictions.

Both modes run the existing source preparation, ordinary closed initializer
validation, unused body/template checks, demanded instance proofs, ownership,
regions, loans, destruction and alignment gates. They do not lower runtime code
or start semantic queries while rendering. An invalid unused ordinary constant
still rejects. Generic/associated initializers remain lazy; inspection does not
evaluate one just to populate a row. Constant execution faults, unsupported
reached operations and exhausted compiler resources exit 1. Source preparation
preserves its original source diagnostic. Display-graph errors are explicit
inspection errors, never an opaque successful value. Output begins only after
the complete graph inventory has been built.

## Row grammar

All rows terminate with LF; fields appear in the order shown. Decimal numbers
are unsigned. `def`, `type`, `instance`, `variant`, closure and expression IDs
refer to the corresponding canonical semantic/body view, within this compilation.
`vN` and `aN` are fresh display IDs, with separate zero-based dense domains.
Private product/reference arena IDs and host addresses never appear.

```text
constants 1
declaration def=D kind=function type=T const=BOOL status=STATUS
declaration def=D kind=KIND type=T const=true status=STATUS
declaration source=FILE:NODE kind=inline type=T const=true status=STATUS parent=FILE:NODE
root initializer def=D type=T value=vV storage=none|aA
root associated-instance instance=D:ARGS type=T value=vV
root inline source=FILE:NODE type=T value=vV
root promotion owner=OWNER instance=none|T expression=E type=T value=vV
value vV type=T PAYLOAD metadata=METADATA
allocation aA type=T STORAGE
eof declarations=N roots=N values=N allocations=N
```

Function `const` reports declaration eligibility only: a true bit does not grant
execution without the selected checked body and closed arguments. Function status
is `checked-body`, `deferred-template`, or `declaration-only`. Initializer kind is `const`,
`static`, or `static-mut`; its `const=true` means its initializer requires constant
execution, not that reading or mutating static storage is a permitted constant
operation. Initializer status is `checked-value` or `deferred-initializer`.
Inline rows follow definition rows in source-key order; their status is
`checked-value`, `deferred-template`, `deferred-unreachable` or
`suppressed-zero-repeat`. Type readiness alone grants no value root.
A deferred row is a declaration, never a checked root/value. Completed associated
values also have their own root carrying the full selected argument identity
`ARGS` and the retained canonical result type. Rendering reads that completed
payload without selecting implementations or executing an initializer.

Owner is `def:D`, `closure:C`, or `anonymous:FILE:NODE`. Metadata is `none`,
`length:N`, or `object:CONCRETE_TYPE:TARGET_TYPE`. Payload alternatives are:

```text
integer bits=HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH
f32 bits=HHHHHHHH
f64 bits=HHHHHHHHHHHHHHHH
bool bits=0|1
char bits=HHHHHHHH
function def=D instance=T signature=S const=BOOL
function closure=C instance=T signature=S const=false
pointer allocation=aA write=writable|shared-derived
address bits=HHHHHHHHHHHHHHHH
text allocation=aA length=N
bytes allocation=aA length=N
byte-pointer allocation=aA length=N
bytes-inline data=HEX
text-inline data=HEX
repeat length=N element=none|vV
product variant=none|VARIANT fields=[vV,vV,...]
```

Hex uses lowercase, fixed-width digits, and preserves exact scalar bits, including
signed integers, negative floating zero and NaN payloads. `product` with no fields
and no variant represents unit. Product fields are in stored declaration/index
order and contain only the active variant. A zero-length repeat has
`element=none`: its discarded fill recipe is not stored data and is not visited.
A nonempty repeat visits its element once, regardless of length. Text/literal
backing is displayed as bytes, never decoded or normalized. Owned byte-array and
unsized text images use the inline payloads, with no invented backing allocation.
The literal address payload length gives its backing extent and, for wide
literal pointer types, their implicit length metadata. The metadata field records
any additionally selected length/object metadata.

Storage alternatives are:

```text
origin=allocation contents=vV
origin=static:D contents=vV
origin=static:D contents=external
origin=projection:aA:INDEX contents=alias
origin=literal bytes=HEX
```

Multiple pointers to one static share its display allocation; equal contents do
not merge distinct allocations. Projection rows name subobject aliases of their
base, not additional physical storage. Static contents describe the checked
initializer, not mutable runtime state. A foreign static has external contents;
it is neither implicitly zeroed nor an uninitialized local allocation. Literal
rows use byte element type and an even-length lowercase hex sequence (empty is
allowed). Integer-derived address values do not acquire allocation provenance.

## Ordering and limits

Declaration rows enumerate retained source function and initializer declarations,
then sort by canonical definition ID. Foreign declarations with no initializer
acquire a graph row only when reached as external backing; they do not invent
initializer roots. Ordinary closed initializer roots
follow in definition order, then completed associated roots ordered by definition
and full argument identity, then completed inline roots in source-key order,
then proved promotions in body-key order
(declaration, closure, anonymous source), specialization type, and body value ID.
Deferred initializers add no root. Values and backing nodes receive IDs on first
discovery: seed all roots in this order, then process one iterative FIFO frontier;
visit product fields in index order, pointers before backing contents, and a
projection's base before visiting any further data. Output groups declarations,
roots, value rows in value-ID order, then allocation rows in allocation-ID order.
No hash-table traversal determines output order. Scalar/value sharing uses the
pair of canonical type and constant identity; backing sharing uses its separate
location/static/literal identity.

Default display budgets are 1,000,000 graph nodes, 4,000,000 charged work units,
and 64 MiB logical retained display storage. Work charges include declaration,
body, promotion and edge preparation plus queued-node visits and cache lookups;
sorting comparisons are not counted as graph visits. Logical storage charges
row bytes plus conservative fixed node/index/queue/row-handle allowances; this
is a deterministic display bound, not an exact heap or process RSS measurement.
Literal hex and product edge output are charged before growing their row.
Independent interpreter/type/borrow limits remain in force. Public embedding
`inspect_with_limits` may choose lower display budgets; it cannot authorize an
operation or bypass a source proof.

## Small entry-free example

```rust
const fn add(n: u8) -> u8 { n + 1 }
const NUMBER: u8 = add(6);
static STORAGE: u8 = 7;
const LEFT: &u8 = &STORAGE;
const RIGHT: &u8 = &STORAGE;
```

The example is independently compiled as a Rust library before view review.
It has four initializer roots. `NUMBER` and `STORAGE` may share the same typed
scalar value row; `LEFT` and `RIGHT` point to the one `origin=static:D` storage
row. `add` has `const=true status=checked-body`. The count mode equals the final
`eof` fields byte for byte. Concrete IDs and full candidate output are retained
in maintainer evidence rather than hand-edited reference files.

Inline-const integration and its qualification boundary are described in
[the inline body draft](pa25-inline-const.md). Array-backed slice indexing and
compressed reads are described in [the sequence checkpoint](pa25-sequences.md).
Remaining PA25 work includes dynamic pattern and enum readiness continuations,
sequence rest views, whole-body const eligibility, raw/union
validity and complete promotion/static allocation coverage. This view does not
claim those execution boundaries are implemented.

## Checkpoint qualification

The focused gate passes 15 inspection tests, including two independently executed
RustGM/rustc native programs, plus a typed closure identity/signature unit check.
It covers the four independently object-qualified draft fixtures, exact scalar
bits, literal addresses versus owned inline bytes, static/projection identity,
foreign backing, active products/unit/associated selection, deferred declarations,
empty/large repeats, source-preparation requirements and unused-invalid source.
Repeated rendering starts no initializer execution or index lookup. Distinct
16/64/256 roots fit the declared linear node/work/storage budgets; this is a
charged-visit invariant, not a total scheduling time or RSS measurement.

Strict all-target/all-feature release Clippy and the file/architecture audit
(1,277 files, zero findings), layout and upstream checks pass. Shared root,
initializer selection and validation regressions passed 20 tests before the
final view-only literal/closure row refinements. Evidence is retained under
`obj/parallel/pa25-constant-inspection/`, including the full entry-free output,
count output, independent diagnostics, source hashes and resource receipts.
The small example counts are `declarations=5 roots=4 values=2 allocations=1`.
No assignment activation, generated reference update or broad release matrix was
performed in this worker checkpoint. The deferred PA29 timing study and earlier
unexplained scalar RSS increase remain unresolved; this view adds no timing claim.

Pointer write permission is a value fact, independent of the displayed backing
allocation. `write=writable` retains owned/mutable provenance;
`write=shared-derived` records a monotonic restriction introduced by a shared
borrow or promotion. Copies, calls, casts and projections preserve that fact;
casting to `*mut` cannot restore write permission. Allocation rows and native
pointer bits do not include this field. The initial raw-store profile explicitly
withholds shared-derived interior-mutable writes instead of treating the flag as
a complete UnsafeCell model. These planned view spellings do not bless active
references or expand scalar-only metadata export.
