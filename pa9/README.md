# RustGM Programming Assignment 9: Types and inference

Status: **implemented draft**. Prerequisite: PA8.
The source profile is Rust 1.98.0, edition 2024, Linux x86-64.

## Required implementation

Consume PA8's expanded, resolved syntax and publish canonical types and body
facts. Parse no source again. Use resolved declaration identities, decoded PA2
literals, compact type/term IDs, shared child lists and per-body constraint
storage. Intern typed structures; printed type names are never semantic keys.
Separate unification from coercion. Detect cyclic aliases and occurs-check
failures, and diagnose exhausted work budgets without recursion or panics.

The initial semantic profile covers nongeneric aliases, scalar functions,
constant/static annotations and initializer types. Canonical shapes include
unit/never, all primitive types, tuples, arrays/slices, references, raw pointers,
function-pointer signatures and distinct function items. Structural shapes can
exist without being usable: unused aliases are not eagerly rejected for
well-formedness obligations that Rust demands only at a use. Actual function
parameters/results and local bindings must be sized; a function-pointer type
can describe an unsized argument/result without itself being unsized.

Elaborate signature lifetime binders and elision using resolved lifetime
identities. Preserve binder depth, alpha-equivalence and function-item identity.
Elision considers the lifetimes written within each input parameter; it does
not discover hidden candidates by expanding aliases or traversing nested
function-pointer binders. Output-only function lifetime parameters, lifetime
bounds and generic aliases belong to later lifetime/generic contracts.

Body values in this assignment are numbers, booleans, characters, bytes, unit,
function items and function pointers. Type simple identifier/wildcard locals,
parenthesized and attributed expressions, plain blocks, `if`, `loop`, `while`,
labeled blocks, `break`, `continue`, return, known function calls, assignments,
scalar unary/binary operators and casts. Record control targets and selected
assignment places. Numeric shifts may have different integer operand types;
ordinary scalar arithmetic does not implicitly widen numeric types. Compound
assignments select the corresponding built-in operator. Iterator loops,
nontrivial patterns, aggregate values, methods and closures retain their later
owners even though their grammar was already parsed in PA4/PA5.

Definite control transfer is separate from contextual value type. For example,
`fn f()->! { if loop {} {}; }` has a diverging statement and no tail value;
removing the semicolon leaves an explicit unit-valued `if` tail and is rejected.
Control-flow propagation respects short-circuit operators and each loop or
labeled block's own breaks. This transient analysis does not enlarge retained
node facts or replace the expression types consumed by later phases.

Local annotations may contain `_`; item signatures may not. Concrete local
annotations share declaration elaboration, while incomplete shapes become
inference constraints. General, integer and floating variables are distinct.
Apply i32/f64 numeric defaults only after constraints. An unconstrained general
variable is an error, except for edition-2024 fallback at never-coercion sites.

Coercions include never-to-expected-type and function-item reification at an
expected function-pointer type. Compatible different function items can join
in `if` expressions. Propagate expected types through local initializers,
blocks, branches, returns, assignments and call arguments; in particular,
function-item loop results may require an explicit function-pointer expectation.
Numeric conversions require `as`; boolean/character casts follow Rust's
restricted conversion set. Explicit function-item-to-function-pointer casts
check the signature. Other pointer casts and unsafe operations arrive later.

Materialize scalar literal bits using the inferred target type. The course
profile uses the default rejection of out-of-range integer/float literals.
Grouped negative signed-minimum literals are valid; retain their unary-literal
exception for later overflow lowering. Round floating literals directly to the
inferred f32/f64 format. Preserve selected operations, call signatures,
coercion targets, region equalities and initializer identities as typed facts
for later phases.

## Phase boundaries

This view checks types, not complete Rust program validity. PA14 owns definite
initialization and binding write permissions; PA15/PA16 own lifetime contracts
and borrow/region proofs; PA25 owns constant-expression validity; PA27 owns
unsafe access. Their deferred obligations must not be silently treated as
proved. The four `boundary/` fixtures deliberately differ from rustc's final
acceptance and declare the owning later phase in their manifests.

Region equalities are retained with the body identity. Reference borrowing,
dereferencing, coercion variance and lifetime-polymorphic calls require later
contracts. Text literals and aggregate expressions require their later value
and layout semantics. Foreign/async signatures, generic types, associated
methods and general lint-attribute interpretation are outside this profile.
PA9 does not evaluate constant arithmetic or check constant-call eligibility.
PA11 owns procedural runtime arithmetic checks; PA25 owns constant evaluation.

## Tool surface and error contract

```sh
dev/rustgm --emit-types [-o OUTPUT] [SOURCE|-]
dev/rustgm --count-types [SOURCE|-]
make test-pa9
```

File loading, `--cfg`, `--crate-name`, expansion, hygiene, includes and name
resolution use PA8's ordinary pipeline. Both modes run identical semantics.
On success stderr is empty and exit status is 0. On a diagnosed failure stdout
is empty, stderr contains the existing `ERROR: file ...` diagnostic with PA8's
expansion/source projection, and exit status is 1. Exit 86, signals and panics
cannot stand for rejection. `-o` opens its destination before compilation; it
is not an atomic replacement protocol.

## Text vocabulary

The version line is `typed-crate 1`. Records are LF-terminated with decimal
IDs and no trailing spaces. Canonical types precede definition, function,
initializer, region-equality and node records. Type IDs reflect deterministic
canonical construction; definition and source-node IDs are PA8 identities.
Definitions are ordered by ID; node facts by file ID, then node ID. Per-node
scalar, operation and adjustment records immediately follow their node record.
Unused type slots on syntax nodes are omitted. Shared canonical types have one
record; equivalent spelling does not create another type.

The first type records are `type 0 never`, `type 1 tuple []`, then primitives
in this order: bool, char, str, i8, i16, i32, i64, i128, isize, u8, u16, u32,
u64, u128, usize, f32, f64. Subsequent forms are:

```text
type ID tuple [CHILD,...]
type ID array element=TYPE length=COUNT
type ID slice element=TYPE
type ID pointer mutable=BOOL pointee=TYPE
type ID reference mutable=BOOL region=REGION referent=TYPE
type ID function binders=COUNT parameters=[TYPE,...] result=TYPE unsafe=BOOL
type ID function-item def=DEFINITION signature=TYPE
```

A region is `static`, `bound:DEPTH:INDEX`, or `region:ID`. Type lists contain
commas without spaces. Definition names use the common deterministic byte
quoting: printable ASCII, escaped quote/backslash/newline/CR/tab and lowercase
`\xNN` for other bytes. The remaining forms are:

```text
def ID type=TYPE name="NAME" [source=FILE:NODE]
function def=ID source=FILE:NODE signature=TYPE body=FILE:NODE
initializer KIND def=ID type=TYPE source=FILE:NODE value=FILE:NODE
region-equal body=FILE:NODE left=REGION right=REGION
node FILE:NODE type=TYPE span=START:END [binding=DEFINITION] [control=FILE:NODE]
scalar FILE:NODE KIND VALUE
operation FILE:NODE DETAILS
adjustment FILE:NODE KIND target=TYPE
eof types=N definitions=N nodes=N functions=N initializers=N
```

Bracketed optional fields above omit their brackets in actual output.
Initializer kinds are `const`, `static`, `static-mut`. Scalar forms are
`integer bits=0xHEX` (32 lowercase hex digits, zero-extended target bits),
`f32 bits=0xHEX` (8 digits), `f64 bits=0xHEX` (16 digits), `bool true|false`,
or `char U+HEX` (6 uppercase digits). Unary signs remain operations.

Operations are `unary NAME operand=TYPE literal=BOOL`,
`binary OP left=TYPE right=TYPE`, `cast KIND source=TYPE target=TYPE`,
`call signature=TYPE`, or
`assign def=ID operator=OP left=TYPE right=TYPE`. Unary names are `negate` and
`not`. Binary OP uses PA4's typed punctuation name (`Plus`, `ShiftLeft`,
`EqualEqual`, etc.); assignment uses the underlying operator or `=` for plain
assignment. Cast kinds are `IntegerToInteger`, `IntegerToFloat`,
`FloatToInteger`, `FloatToFloat`, `BoolToInteger`, `CharToInteger`, `U8ToChar`,
and `FunctionPointer`. Adjustment kinds are `never`, `function-pointer` and
`equality`. An `equality` adjustment records the destination type when equality
depends on pending region obligations; it does not prove those obligations.
The count mode emits just the footer fields, without the `eof ` prefix.

## Validation

The manifest declares every outcome before reference generation. Its full
programs are independently checked with pinned rustc, including default literal
lints. Fixture families cover scalar inference, constraints, aliases, control
flow, demanded well-formedness, lifetime elision, source packages and explicit
phase boundaries. The occurs-check fixture constrains a function-pointer
parameter to contain its own type. Typed Rust invariants additionally verify
canonical sharing, retained operation/control facts, deep iterative algorithms
and unresolved/cyclic constraints.

Read `spec.md`, the pinned Reference and CPPGM architecture guidance. Later
body and LowIR construction must consume these facts without repeating lookup,
parsing, inference, cast selection or literal conversion.
