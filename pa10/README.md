# RustGM Programming Assignment 10: Typed bodies

Status: **implemented draft**. Prerequisite: PA9.
The profile is Rust 1.98.0, edition 2024, Linux x86-64.

## Required implementation

Construct typed control-flow bodies from PA9's resolved, typed syntax. Consume
its definition identities, types, literal bits, operator/cast selections,
adjustments, signatures and control-transfer targets. Do not repeat lookup,
inference, literal conversion or parsing. This is an executable body model,
not another copy of the syntax tree and not CPPGM's machine IR.

Each function and annotated initializer owns flat locals, evaluated values,
argument lists, statements and blocks. Use compact typed identities, iterative
construction and reusable per-body scratch. Clear only touched scratch slots
between bodies. Printed types/names/text are never semantic keys or transport.

Build bodies for PA9's accepted expressions: scalar/unit values, value paths,
locals, assignments, operators/casts, direct/function-pointer calls, blocks,
`if`, short circuit, loops, labels, breaks, continues and returns. Function
items and reification remain distinct. Constant/static references and initializer
bodies remain explicit for later evaluation. Typed parameter/local reads can
retain shapes whose layout, ownership or runtime support belongs to later PAs.

## Evaluation and storage

Values snapshot reads: `x + { x=2; x }` reads the first `x` before the right
operand executes. Evaluate a call's callee before its arguments and arguments
left to right. An argument that changes a function-pointer local cannot change
the callee already evaluated.

Simple assignment evaluates its right side before its destination. PA9 permits
only simple local/parameter assignment places. A wildcard binding does not read
or move a plain local place (`let _ = x`); other initializer effects still run. Primitive compound assignment
also evaluates the right side before reading/updating the place:
`let mut x=1; x += { x=10; 3 };` leaves x equal to 13. Overloaded/generic
compound assignment needs its later semantic selections.

`&&`/`||` branch and evaluate the right operand only on the required path.
Branches preserve the predicate's true/false sense. Scalar branch/loop results
merge through a local: producing paths store it and the join reads it. Unit
joins need no result local. Diverging paths have no value to store. Every block
has one terminator. An allocated block without an incoming path is empty and
ends in `unreachable`. Statements after an unconditional transfer emit no
executable instructions; PA9 still checks their types.

Loops and `continue` return to the header; `while` reevaluates its condition.
Breaks use the recorded target's exit, including labeled outer loops/blocks.
Do not resolve labels again. Never calls have an unreachable continuation.
Returns retain their value before ending local storage.

Emit `live` when a parameter, binding or merge local becomes available, before
its first store/read. Normal exits end storage in reverse creation order;
transfers end storage they leave. Conditional/condition temporaries end on their
actual paths. `dead` is a storage boundary, not a destructor or initialization
proof. Retain definition identity, declared mutability and local role.
Each local retains its predecessor in the active storage stack when it is
created. Those persistent links skip locals whose nested scopes have already
closed. A multi-local exit is one `end-storage` action, following those links
from `last` back through `first`, inclusively and in reverse live order. It
shares the storage chain without copying a prefix or materializing one `dead`
statement per exited local at every branch. Single-local exits retain `dead`.
Ownership checking and later storage consumers visit the same ended locals;
this compaction does not erase initialization checks or introduce destructors.
Statement-macro expansions add statements to their enclosing scope. The
expansion container does not end binding storage; explicit blocks inside the
expansion retain their ordinary scope boundaries.

## Deferred obligations

By-value reads do not yet decide Copy/move eligibility, definite initialization,
write permission, loans or drops. PA11 adds the scalar executable initialization/
write-permission check; PA14–PA17 and later trait work generalize these rules. Constant
validity/evaluation belongs to PA25. This is not whole-Rust acceptance.

Region equalities remain explicit with their owning source-body identity.
When canonical types differ under pending equalities, PA9 supplies an `equality`
adjustment and the body retains `constrain`. This is neither a numeric cast nor
a region proof. A labeled block's fallthrough may diverge while a break produces
a value; its fallthrough-only never adjustment must preserve the merged result.

PA11 consumes this model for typed LowIR, target representations, checked
procedural arithmetic and an entry shim. The builder invokes no external
frontend/backend. The integer CFG executor under `dev/tests/` is independent
test instrumentation; it is not linked into the reference frontend.

## Tool surface

```sh
dev/rustgm --emit-body-ir [-o OUTPUT] [SOURCE|-]
dev/rustgm --count-body-ir [SOURCE|-]
make test-pa10
```

Both modes perform identical expansion, resolution, type checking and body
construction. Files, includes, `--cfg` and `--crate-name` follow PA8. Complete
semantic/body work before stdout starts. Success is exit 0/empty stderr;
diagnosed rejection is exit 1/empty stdout with the existing projected
`ERROR: file ...` diagnostic. Panics, signals and exit 86 cannot count as
rejection. As earlier, `-o` opens/truncates its destination before compilation.

## Deterministic text vocabulary

Start with `body-crate 1`, then PA9's `type` table and `region-equal` records.
Functions precede initializer bodies, each in semantic order. Body headers are:

```text
body ID function def=DEFINITION signature=TYPE result=TYPE source=FILE:NODE
body ID const|static|static-mut def=DEFINITION result=TYPE source=FILE:NODE
```

Each body resets local (`lN`), evaluated-value (`vN`) and block (`bN`) IDs to
zero. IDs follow deterministic construction; `b0` is entry. Print locals then
blocks in ID order. Statements within a block execute in order; block ID order
does not imply execution order. Records have LF, decimal IDs and no trailing
spaces. Local/block lines use two spaces; statements/terminators four:

```text
  local lN type=TYPE parameter=INDEX|binding|merge [def=DEFINITION] mutable=BOOL storage-parent=lN|- source=NODE
  block bN:
    live lN source=NODE
    dead lN source=NODE
    end-storage first=lFIRST last=lLAST source=NODE
    store PLACE vN source=NODE
    vN type=TYPE = RVALUE source=NODE
    jump bN source=NODE
    branch vN bTRUE bFALSE source=NODE
    return vN source=NODE
    unreachable source=NODE
```

Optional `def` omits brackets. PLACE is `lN` or `static:DEFINITION`. Source
nodes belong to the body's file. RVALUE forms are:

```text
unit
integer bits=0xHEX | f32 bits=0xHEX | f64 bits=0xHEX | bool BOOL | char U+HEX
read PLACE
function def=DEFINITION
constant def=DEFINITION
reify vN
constrain vN
unary NAME vN literal=BOOL
binary OP vLEFT vRIGHT
cast KIND vN
call signature=TYPE vCALLEE [vARG,...]
```

Scalar bits, unary NAME, punctuation OP and cast KIND use PA9's vocabulary.
Signed-minimum literal exceptions remain on unary operations. Argument lists
use commas without spaces. `reify` converts a function item to a pointer;
`constrain` retains selected type/region conversion obligations. An inferred
reference local has its own region identity; reading and returning it can add
conversion values even when the source and result have the same referent type. Unit/never calls are evaluated
statements; a never call ends its block with `unreachable`.

End each body with `end-body`. The footer is
`eof bodies=N locals=N values=N statements=N blocks=N`, summing its arenas.
Count mode omits `eof `. Empty crates are valid bodies=0 inputs; the assignment
suite itself must never be empty.

## Validation

The 47 declared fixtures cover evaluation order, short circuit, nested/labeled
control, divergence, calls, storage, scalar values, expansions and packages.
All outcomes are independently checked with pinned rustc. The 32 execution
fixtures also declare scalar results: a test-only CFG executor verifies dynamic
storage state and compares results with rustc. Deep-body and deterministic-view
invariants exercise iterative construction/count output. A late-error fixture
checks empty stdout. All earlier suites, Clippy, architecture/size audits,
harness checks and scaling measurements remain required.
