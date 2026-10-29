# RustGM Programming Assignment 12: Aggregates and layout

Status: **implemented draft**. Prerequisite: PA11.
The profile remains Rust 1.98.0, edition 2024, Linux x86-64, checked integer
arithmetic and aborting panics. This assignment adds concrete owned aggregates
and an explicit layout inspection view; it is not whole-Rust conformance.

## Required implementation

Extend the canonical type store with nominal identities whose definitions are
completed separately. Reserve identities before elaborating fields, so pointers
can refer to incomplete declarations. Complete fields, constructors, representation
hints and discriminants before caching layouts. Diagnose by-value cycles, including
cycles through zero-length arrays; pointers break storage dependency cycles.
Keep flat field/variant arenas with compact identities and retained source access
facts. Parse source once. Later phases consume selected constructors, fields,
array bounds, casts and layouts rather than repeating lookup or inference.

Support tuples, fixed arrays, record/tuple/unit structs, enums, nested aggregates,
construction, field access, indexing, assignment, returns, direct calls and function
pointers with aggregate arguments/results. Struct update evaluates explicit fields
before its base and reads only inherited fields. Preserve source visibility and
field type checks. An assignment evaluates its RHS before forming its destination;
an index expression executes once. Wildcard field discards need no value read,
but wildcard array indexing still forms the place and checks bounds.

Check the types actually used by a body from its physical module. A public
function returning a private aggregate does not make that aggregate accessible;
private types nested in a callable signature remain private too. A public
function-pointer interface may hide the private function it stores. Consume
completed semantic types and share traversal of repeated type nodes.

Array repetition evaluates its operand once, including lengths zero and one.
Larger repeats require a Copy operand or an eligible constant operand. Keep one
operand and a canonical length, never one body-IR entry per repeated element.
PA16 covers closed constant/static materialization and reference storage with
their lifetime/loan proofs; general constant evaluation remains PA25. The earlier concrete array-length profile
remains: lengths in annotations and repeats are usize-compatible integer literals
or grouped literals. Nonliteral array-length queries belong to PA25.

Resolve `Self` to its enclosing concrete struct/enum in field annotations and
discriminants. It does not cross a nested item's capture barrier. Enum members
are qualified names; they are not automatically lexical bindings inside their
discriminants. Explicit variant imports still introduce lexical bindings.
Configured fields/variants, retained attributes, generated items/expressions and
external source modules use the preceding assignments' identities and worklists.

## Representation and discriminants

The target uses 8-byte pointers, 64-bit isize/usize and 16-byte u128/i128 alignment.
Implement cached, iterative size/alignment/field-offset queries. Array size is
stride times length, with target object-size overflow checks; zero-sized elements
permit huge lengths without expanding a field arena. Retain uninhabitedness and
by-value reference footprints even when size is zero.

For unspecified Rust tuple/struct layout, this course deliberately chooses source
field order: align each field's offset, add its size, then pad total size to the
maximum alignment. This is an allowed Rust layout, not a promise to reproduce
rustc's field reordering or niche optimization. Empty types and zero-sized fields
remain explicit semantic facts. The layout view's exact choices are the course
oracle; compare rustc layout bytes only where the relevant guarantee applies.

Honor `repr(C)`, `repr(Rust)`, `repr(transparent)`, `repr(packed)`/`packed(N)`,
`repr(align(N))` and integer enum representations. Validate applicable owners,
conflicts, power-of-two limits, transparent fields and packed types containing
explicitly aligned fields. C enum layout uses a tag followed by a maximally sized,
aligned payload union. An integer-representation enum uses a union of per-variant
structs with the tag first. Combined C/integer hints use C payload placement with
the selected integer tag. The unoptimized default enum layout uses a 64-bit signed tag
and source-order payload fields; a single default-representation variant has no
physical tag. Source-owned nullable policies may instead use an authenticated
non-null pointer carrier or an exact unsigned `1..=MAX` scalar carrier. The latter
covers u8/u16/u32/u64/u128/usize, including ordinary transparent wrappers, and keeps
zero as the empty variant without reading its inactive payload. Integer carriers
retain integer ABI types; they grant no pointer provenance or dereference rights.
Explicit representations and unsupported scalar ranges retain their separate
physical profiles or explicit wider-niche requirement. General Rust niche selection
is not implied. Transparent ancillary zero-sized fields have offset zero in the
course layout. See the [unsigned niche checkpoint](../docs/pa12-unsigned-niche-checkpoint.md)
for source, CTFE, ABI and native qualification boundaries.

Explicit discriminants have the declared integer representation type, or isize
without one. Evaluate typed scalar constant expressions: integer and floating
literals, booleans, characters, grouping, unary/binary built-ins, numeric casts,
short-circuit operators, conditional expressions and expression-only blocks.
Named constants, immutable Copy statics and eligible field-less enum constructors
can supply values. Integer arithmetic checks overflow/division/shift limits;
numeric casts use Rust truncation, sign extension, rounding and saturation.
Const functions, statements, loops and allocations are PA25 responsibilities.
Validate constant eligibility in both conditional branches, but evaluate only the
selected branch. Reuse ordinary type inference and selected operation facts.

Solve constant/variant dependencies iteratively, caching completed values and
diagnosing cycles. Forward variant references are valid when acyclic. An implicit
discriminant starts at zero or increments its predecessor, with representation
overflow and duplicate-value checks. Only unit-only enums may specify discriminants
without an integer representation. Cast field-less enums to integers only when
non-unit empty variants have no explicit discriminants. Layout caching must never
observe temporary discriminant values.

Built-in unqualified `derive(Clone, Copy)` is supported for these concrete nominal
types. Validate field obligations, Clone prerequisites and packed Clone restrictions.
This is the closed built-in capability profile needed for aggregate operations;
general trait selection, Clone calls and procedural derives remain later work.
PA14 owns the complete move-path/partial-initialization grading contract. The
maintainer checks owned-value uses before executable lowering; this assignment's
native fixtures do not depend on PA14's partial-move exercises.

## Checked aggregate LowIR

Continue `dev/rustgm --emit-lowir [-o OUTPUT] SOURCE` using PA11's deterministic
symbol/block/value ordering, scalar operations and panic convention. Construct
positive-sized object slots as `obj<BYTESxALIGN>` and transfer objects with the
existing `copyobj BYTESxALIGN` vocabulary. Derive actual address alignment for
packed/nested fields; never claim an alignment that the selected address lacks.

Every by-value read snapshots its source. Constructors/calls produce fresh
storage. Transfers are between proven distinct allocations: a snapshot and its
destination, fresh callee parameter slots or a fresh caller result slot. The
physical transfer is not permission to duplicate a non-Copy source value.
Pass nonzero aggregate arguments by pointer to evaluated snapshots and nonzero
aggregate results through a hidden first output pointer. Direct and indirect calls
use that same private convention. This is RustGM's internal ABI, not rustc ABI
compatibility. Scalar-only signatures and outputs retain PA11's spellings.

Zero-sized values have no LowIR object allocation or physical parameter/result,
but keep evaluation effects, initialization/move obligations and bounds checks.
Nonzero array repetition emits a constant-size loop; zero-sized repetition emits
no loop. Indexing uses an unsigned comparison against the canonical array length.
Read enum tags from their selected representation, or materialize the fixed value
for an untagged single variant, then apply the selected integer cast.

References hidden in aggregate bytes cannot bypass region/loan checking. Keep
those obligations and reject demanded executable storage until the owning checks
can discharge them. PA13 owns pattern selection and enum destructuring; PA15–PA17
own lifetime/loan/destruction proofs; PA18 owns type-parameterized aggregates.
The supported concrete functions still use the earlier declaration/body profile.

## Layout text contract

`--emit-layouts` and `--count-layouts` are supplementary PA12 inspection modes.
Complete declaration/body type checking and every fixed-sized canonical layout
before emitting text. This view proves representation facts, not ownership or
loan validity, and does not require a main function. Unsized canonical types are
listed without inventing a size; unsupported sized representations such as a fat
pointer fail the whole view. Failed compilation emits empty stdout and exit 1.
Semantic diagnostics retain preceding file/offset conventions; a newly demanded
layout failure is `ERROR: type TYPE: MESSAGE` followed by LF.

The full view starts with `layouts 1 target=x86_64-unknown-linux-gnu`. Next emit
PA9's shared `type ID ...` table in canonical ID order, then named type declarations
in definition order (type namespace only, excluding variant pseudo-types):

```text
def DEF type=TYPE name="BYTE_ESCAPED_NAME" [source=FILE:NODE]
layout TYPE unsized
layout TYPE size=BYTES align=ALIGN inhabited=BOOL references=BOOL scalar
layout TYPE size=BYTES align=ALIGN inhabited=BOOL references=BOOL array element=TYPE stride=BYTES length=COUNT
layout TYPE size=BYTES align=ALIGN inhabited=BOOL references=BOOL aggregate tag=TYPE@OFFSET|-
variant TYPE:ORDINAL discriminant=BITS
field TYPE:VARIANT:ORDINAL type=TYPE offset=BYTES
eof types COUNT layouts COUNT variants COUNT fields COUNT
```

Emit one layout line per canonical type in ID order. Aggregate variant and field
lines follow their owner in source order; ordinal IDs start at zero. `references`
reports by-value reference presence, including zero-length arrays; raw pointers
do not traverse pointees. `inhabited` distinguishes empty enums from ordinary
zero-sized types. A tag type uses its primitive spelling. Discriminants print
unsigned decimal two's-complement bits at the explicit integer width or default
isize width, even when a physical C tag is narrower. Struct/tuple groups print
discriminant zero. Counts cover completed fixed layouts and their retained groups
and offsets. Count mode prints only the footer without `eof `.

All numbers are decimal; booleans are `true`/`false`. Names quote ASCII printable
bytes, escape quote/backslash and LF/CR/tab, and use lowercase `\xHH` for remaining
UTF-8 bytes. Every line ends in LF. Hash iteration must not affect ordering.
This is an output view of typed facts; no frontend phase reparses it.

## Tests and completion evidence

The manifest declares each input's mode and expected exit before reference
generation. LowIR acceptance requires a declared clean-exit or panic-abort
runtime outcome. Every such input is executed with both pinned rustc and the
qualified CPPGM backend. A supplementary layout case cannot satisfy native
execution requirements. Unsupported later-stage programs carry explicit rustc
acceptance differences; they cannot silently masquerade as invalid Rust.

Layout fixtures declare sizes, alignment and selected offsets independently of
text generation. The layout checker compares those declarations with the
candidate and committed text, then executes rustc size/alignment/offset probes
where guaranteed. Course-specific Rust ordering and transparent zero-sized-field
offsets are explicit exceptions, not claims of rustc equality. Rejected layout
or source programs must return exit 1 with diagnostics; signals, timeouts, exit 86
and empty suites are failures.

Run `make test-pa12 test-layouts test-native-pa12`, all earlier suites,
`make lint audit-rust test-harness test-upstream test-rustc test-backend`, and
debug/release invariants. The maintainer's scaling/equivalence evidence is in
`docs/pa12-validation.md`; required course fixtures live here. Consult
`doc/rust-reference/src/type-layout.md`, `items/enumerations.md`, `const_eval.md`
and `paths.md`, plus the pinned CPPGM contract in `docs/cppgm-spec.md`.
