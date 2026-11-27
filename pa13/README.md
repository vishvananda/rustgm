# RustGM Programming Assignment 13 — Pattern semantics

Build on [PA12](../pa12/README.md) to select pattern bindings, check
exhaustiveness and usefulness, and lower owned patterns. Use Rust 1.98.0,
edition 2024, on Linux x86-64. PA34 adds edition 2021 and PA41 covers the
remaining editions.

## Command-line interface

```text
rustgm --emit-patterns [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
rustgm --emit-lowir [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Use PA11's input, configuration, and output options. `--emit-patterns`
shows selected patterns and pending reference obligations. Use PA11 and
PA12's LowIR conventions for executable cases. Share the typed selection
facts between both modes.

## Semantic and execution profile

Use a sparse selected-pattern representation. PA15 and PA16 validate
lifetime and loan obligations. Simple standalone value
bindings and wildcards use the earlier direct path and have no selected root.
The view covers irrefutable owned products/arrays, explicit reference patterns,
edition-2024 binding modes, irrefutable borrowed slice rests, and owned match
decisions. Boolean, integer, character and floating intervals compose with tuple, record,
enum, fixed-array and reference patterns for exhaustiveness and whole-arm
usefulness. Dynamic slices support exact lengths, prefixes, suffixes and borrowed
rest bindings for typed selection and coverage. Owned matches, including
boolean and conditional guards, and irrefutable constructor bindings can execute.
Owned let-else, if-let, while-let and condition chains also execute. Named
boolean, integer, character and floating-point constants work as patterns and
range endpoints. Floating literals and ranges compose with owned products,
arrays, constructors, guards and conditional bindings. Unit, tuple and array
constants translate into ordinary product/scalar patterns and execute on owned
inputs. Struct and enum constants with derived `PartialEq` also translate and
execute, including nested values, struct updates and field projections.
String and byte-string literals and named constants support typed pattern
selection and coverage. Shared reference constants support promoted values,
immutable statics, nested references and typed field/index projections. Reference
and dynamic-slice execution still require the later region and loan proofs.

Named constants use resolved declaration identities and their canonical types;
pattern endpoints do not coerce between integer types. The shared typed constant
query engine supports scalar literals, dependencies, arithmetic, casts, block
tails and lazy boolean/if expressions, including casts of completed fieldless
enum discriminants. It diagnoses dependency cycles and evaluated overflow.
Tuple/array construction, repeated arrays and selected field/index reads share
that engine. Empty plain blocks produce unit. Demanded product values use flat
storage, and a repeat retains one value and its length. Each use expands into
ordinary selected patterns; repeated product subtrees can share pattern IDs
while their projections still identify distinct places. Generated nodes retain
the pattern use's source location. Empty arrays still require an eligible element
type; nonempty floating leaves must be non-NaN. Expansion has a 1,000,000-work
limit, charging edges before allocation. This is separate from coverage's limit.
Constant-backed ranges, irrefutability and match coverage finish after initializer
typing, retaining each body's file and module context. The resulting intervals
use the ordinary decision and ownership paths, including guard retries.
Nominal values retain selected variant and field identities; translation does
not resolve names or reparse constructors. Type-level equality obligations inspect
all variant fields, while structural value checks inspect the active variant.
The builtin `PartialEq`/`Eq` derive hooks check their field obligations; packed
`PartialEq` additionally requires Copy fields. `Eq` requires `PartialEq` and
rejects floating fields, including zero-length arrays of floats. These hooks do
not implement general traits, equality operators on nominals, methods or
procedural derive expansion; those remain PA19/later capabilities. Recursive
reference obligations use a bounded iterative fixed point and cache complete
positive and negative results. Constant calls validate their callee expression,
and struct updates validate their base even if every field is overridden.

A static cannot be named directly as a constant pattern. Borrowing a static
does not move its value: a shared constant reference may name non-Copy static
storage, and a Copy field may be read independently. Reading through a shared
reference still requires Copy. This check applies even when the containing
constant/static value is already cached. Shared borrows preserve packed-field
alignment and static provenance; an unaligned reference is rejected. An actual
dereference selects the referent's alignment rather than its pointer's container.
Mutable promoted references, mutable static access, cyclic allocation graphs,
general CTFE and expression-position constant execution retain their later-stage
boundaries. Constant references do not authorize runtime reference operations.

Place queries retain values and access provenance in flat arenas. They follow
selected field identities and sparse automatic-dereference counts, without
moving or materializing the containing product. Borrow, dereference and dynamic
slice indexing publish typed operations; `--emit-types` spells them `borrow`,
`deref` and `index length=dynamic`. An adjusted field/index additionally renders
`autoderef FILE:NODE count=N`. Ordinary unadjusted nodes allocate no entry.

Text identity uses exact decoded bytes, including Unicode, NUL and empty values.
Raw and escaped literals with equal bytes select equal patterns; composed and
decomposed Unicode strings remain different. Byte-string literal patterns accept
`&[u8; N]` or `&[u8]`; a named constant preserves its declared type and does not
implicitly unsize while matching. A constant initializer can explicitly declare
`&[u8]` and unsize its byte-string value. Elided references in constant/static
annotations default to `static`; nested function-pointer types retain their own
ordinary lifetime elision rules. Value-pattern type matching does not impose the
constant's storage lifetime on the input. Expression lifetime constraints remain
pending for PA15/PA16.

String coverage uses the current query's distinct observed strings and one
remaining class for every other string. Byte strings use ordinary byte ranges,
array products and exact slice lengths, so they compose with written slice
prefixes/suffixes. Finite string cases require an unconstrained case for
exhaustiveness. Coverage includes strings in supported tuples, arrays and borrowed
slices. Nominal fields containing `&str` or `&[T]` still encounter PA22's unsized
pointer layout boundary; nominal fields containing sized byte-array references
can be inspected. C-string values await the library type assignment. Text value
execution does not bypass reference lowering, region or loan proofs.

Apply the Reference's automatic dereferencing rule when a constant's type
is a non-reference type. For example, the course pattern view accepts a
non-reference constant `C` when matching its corresponding reference input.
Rust 1.98.0's compiler requires an explicit `&C` pattern for these cases.

## Selected-pattern output

Start with `selected-patterns 1`, the canonical type table,
`root FILE:NODE pattern=ID`, and arena-ordered `pattern ID source=FILE:NODE
input=TYPE KIND...` records. Roots sort by file/node; pattern and edge identities
follow deterministic source traversal. A pattern's input type remains distinct
from a binding's local type. Use the following pattern kinds.

- `wildcard` (including a selected rest with no binding operation).
- `range=ID` for a scalar literal, named constant or interval in the separate range arena.
- `text="BYTE_ESCAPED_VALUE"` for an exact string referent.
- `bind def=ID local=TYPE mutable=BOOL capture=value|ref|ref-mut`, followed by
  `region=REGION loan=pending` for borrowed captures, then `subpattern=ID|-`.
- `deref mutable=BOOL implicit=BOOL pattern=ID` for each reference adjustment.
- `product variant=ID|- fields=START:END` with a half-open edge range.
- `slice minimum=N exact=BOOL fields=START:END` with the required length and
  a half-open edge range; `exact=false` permits any greater length.
- `or alternatives=START:END` with shared resolved binding identities.

`field ID tuple=INDEX|nominal=FIELD|index=INDEX|from-end=OFFSET|subarray=START:END|subslice=PREFIX:SUFFIX
pattern=ID` records retain selected places. Subarray endpoints are half-open;
subslice values count excluded elements at either end. `from-end` is a one-based
offset, so `1` selects the final element after the length test. A lone rest is
irrefutable for a dynamic slice; combinations such as `[] | [_, ..]` can also
cover every length. `alternative ID pattern=ID` records retain
each occurrence, including distinct capture operations for a shared binding.

`range ID ranks=LOW:HIGH` uses inclusive, ordered target ranks. Signed integers
use a sign-bit bias; character ranks exclude the surrogate gap. Floating ranks
order negative infinity through positive infinity, collapse signed zeros for
matching, and retain one final unordered NaN coverage class. Numeric ranges
exclude that class, including ranges with an omitted endpoint or infinite
bounds. NaN constants are invalid pattern values/endpoints; a wildcard or
unconstrained binding covers NaN. Captures read the original value and preserve
its zero sign. The input type
determines how to interpret ranks. `match ID source=FILE:NODE input=FILE:NODE
arms=START:END exhaustive=true uninhabited=BOOL work=N` retains the scrutinee and a half-open
source-ordered arm range. `arm ID source=FILE:NODE pattern=ID guard=FILE:NODE|-
useful=BOOL value=FILE:NODE` retains the guard and result identities. Guarded
arms contribute no exhaustiveness coverage. Usefulness refers to the whole arm,
not individual redundant alternatives. Coverage counts visited pattern nodes
and interval-index operations for the scalar fast path. Composite queries count
type/pattern visits and shared decision-graph operations. Both reject work beyond
1,000,000 per query. Cached type facts are keyed by the matching module; enum
domains additionally distinguish valid values from possibly invalid referents.

`uninhabited` is a contextual coverage fact, not a physical layout property or
proof that a place was initialized. Private fields cannot expose an empty-type
proof to an unrelated module, and references to empty types remain possible
coverage inputs. An explicit dereference or automatic field/index dereference
also keeps its root potentially invalid: a variant with an empty field must not
disappear from that constructor domain. Grouping preserves place validity;
a value-producing expression establishes an owned result whose reads still need
independent ownership proofs. A directly empty root enum permits an empty match
even through a reference, while a wildcard on that potentially invalid place
remains useful. Irrefutable bindings retain the same input-validity context,
including when named constants defer their coverage checks.
Constructor decisions retain non-moving `is-variant VARIANT PLACE` body values
and selected field identities.
Variant tests require an initialized owned input, including its tracked fields;
a surviving physical tag does not permit testing a partially moved enum.
Single-variant wildcard patterns need no tag read. An empty match retains a
`validate PLACE` operation in body IR; an uninitialized empty-type local may pass
pattern/body inspection but fails ownership checking before executable IR.

## Coverage and diagnostics

Composite coverage uses reduced binary decisions over normalized scalar and
constructor ranks at sparse typed places.
Union, intersection and difference queries are memoized; products and nested
alternatives are not eagerly expanded into combinations. A missing witness is a
deterministic partial assignment such as `{ $.0 = false, $.1 = true }` or
`{ $ = E::B }`. `$` is the input, `.name`/`.N` select fields, `.*` dereferences,
and `[N]` selects an array element. Unconstrained inputs render `_`; these are
diagnostic constraints, not Rust source to reparse. Whole-arm usefulness still
ignores guard outcomes and does not diagnose individual redundant alternatives.
Retained `allow`, `warn`, `deny` and `forbid(unreachable_patterns)` policies
control rejection of wholly unreachable arms. The default warning level does not
reject them: default and explicit `warn` emit a typed
`warning[unreachable_patterns]: file FILE: BYTE: unreachable pattern` for each
wholly unreachable arm. Successful views can therefore have declared nonempty
stderr while retaining their exit status and stdout. Use `allow` to suppress the diagnostic. Nested `allow` or `warn` cannot
weaken `forbid`. Individual
redundant alternatives within an otherwise useful arm remain outside this
producer.
Item inheritance follows lexical policy independently of body and unsafe
contexts. Warning contracts preserve exact code, count, order and source byte
locations rather than discarding diagnostics during comparison.

Dynamic slices add a length variable to the same coverage graph. A query lazily
collects tested start/end positions by canonical slice type from its own roots.
Only lengths at which those positions can alias need individual specialization;
the intervening intervals share independent start/end projections. Wildcards and
unconstrained captures do not add alias points. At an alias length, a suffix
selects the same canonical start index as the matching prefix. This handles
short slices without enumerating all lengths or element values. Collection,
position pairs, branches and graph operations share the existing work budget.
Diagnostics use `.len` for a length constraint and `[-N]` for an end offset.

## Decisions and captures

Decisions retain ordered alternatives, read the scrutinee only when the
decision or capture requires it, and capture values after a successful decision.
Composite or-patterns save selected alternative identities in body locals.
Outside guarded arms, alternatives with distinct variant tags instead re-test
the unchanged discriminant, which keeps the selected variant's owned fields
initialized. Subsequent capture branches preserve the selected places without
duplicating products of alternatives. Capture projections are formed in their own branches,
so they do not depend on values defined only along an earlier decision path.
The ordinary let/parameter path uses these decisions when an irrefutable pattern
combines refutable alternatives. A declaration without an initializer only
declares its bindings. Field identity retains its owning variant for layout;
constructor tests compare the stored discriminant with the selected layout value.

Conditional selection retains `condition source=FILE:NODE input=FILE:NODE
pattern=ID otherwise=FILE:NODE|-`, sorted by source identity. The failure block
exists only for let-else. Sparse chain identities distinguish conditional `&&`
from ordinary boolean expressions; the builder consumes them directly. Inputs
run once; successful captures become visible to subsequent operands and the
success body. False operands end every earlier chain binding's storage.
While-let recreates bindings at each iteration and balances storage for loop
exit, continue and labeled transfers. Let-else checks its failure block in the
surrounding scope, requires divergence, and preserves success bindings for the
following statements. Owned decisions and delayed captures share the same
constructor worklists as matches.

Even statically impossible pattern failures must pass ownership checks. Body IR
retains `false-edge real=bN imaginary=bN` only when ordinary decision edges
cannot reach the failure. Ownership traverses both successors; executable
lowering follows only the real successor and allocates no storage used solely
on imaginary paths. Non-final irrefutable match arms use the same rule to check
later arms. A diverging input has no such failure continuation. None of these
rules bypass pending region or loan obligations.
Conditional match guards and their let chains can be parsed, resolved and
selected using these same conditional facts. Initializers retain pre-shadowing
binding identities; later operands and arm bodies see the new bindings. This
selection feeds the guarded execution described below.

A full-domain interval requires no value test. Owned guard execution uses
internal read-only views of selected bindings, then commits owning captures
only after success. Guard-let bindings have their ordinary local ownership and
mutability. Failed guards end attempt-local storage and resume the next matching
alternative in written order; the scrutinee is evaluated once. Nested products
traverse fields in written order and keep the last field as the innermost retry.
Each subtree has first-match and resume continuations, so generated decisions
stay proportional to the pattern rather than expanding products of alternatives.
A product field with no first match rejects that product immediately.

Retry every matching alternative when a guard fails. For either signed
zero, `-0.0 | 0.0 if { count += 1; false }` increments `count` twice in this
assignment, following the Reference's guard rule. The pinned Rust compiler
increments it once for opposite signed zeros.

Body IR records `guard-view`, `guard`, `guard-place`, `guard-start`, `bind-view`
and `guard-use` facts. Shallow protection covers tested places across the match;
selected binding views additionally protect the whole bound value. Mutation of
an untested, disjoint field can remain valid. Protection follows future uses:
divergent paths need not preserve values for a guard continuation they cannot
reach. Moving a non-Copy value or assigning through a view is rejected. Packed
origins must support the alignment of an internal reference, even for zero-sized
fields. An empty rest still requires and protects its containing array's shallow
state; moving disjoint elements remains possible. These are internal view checks,
separate from user-reference regions.

Retry paths retain imaginary ownership edges for bypassed guards and later
alternatives. They do not invent a guard-failure edge after an irrefutable
conditional guard. Native lowering follows only real edges. Static retry paths
use `select pN anchor=PLACE`: cached typed offset recipes form fresh addresses
in each use block, preserving dominance after a resume without replaying every
parent projection. Proof-only tested-place metadata never forms addresses for
inactive enum variants. Irrefutable and conditional binding execution can combine
constructors and scalar intervals.

`region-var region:N body=FILE:NODE source=FILE:NODE origin=ORIGIN
pending` gives existential region provenance. IDs are globally allocated from
one and do not alias `static` or signature `bound:DEPTH:INDEX` regions. Origins and pending `region-equal`/`region-coercion` records use PA9's vocabulary.
A capture's fresh region is not itself a loan or outlives proof. PA15/PA16 own
borrowed execution and its required proofs.

The final line is `eof roots=N patterns=N fields=N alternatives=N ranges=N
matches=N arms=N coverage-work=N region-vars=N region-equalities=N conditions=N
chains=N`.
`coverage-work` sums match queries; it does not count ordinary binding checks.
Source errors exit 1 with a diagnostic and no partial pattern view.

## Run the tests

Run the PA13 test target from the repository root.

```sh
make -C pa13 test
```

[The manifest](tests/manifest.json) selects a mode for each case. It covers
exhaustiveness and useful arms, binding and guard behavior, signed-zero
matching, and borrowed patterns with obligations for later phases. For
executable cases, use the [PA11 backend procedure](../pa11/README.md#run-the-tests-and-an-executable).
