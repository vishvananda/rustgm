# Source formatting builtin implementation contract

LIB-017 is observed on the unchanged original core root with the qualified
`cfg_select` compiler: `macros/mod.rs:994`, byte 35014, claims `format_args` and
stops with an explicit unsupported-builtin diagnostic. The following
`const_format_args` and `format_args_nl` declarations are part of the same source
contract. Core's Rust 1.98 `fmt::Arguments` uses an encoded template and argument
pointers; historical `new_v1` layouts are not its ABI.

The [declaration corpus](../pa31/tests/spec/source-format-macros/declaration-contract.json)
and [literal corpus](../pa31/tests/spec/source-format-macros/literal-contract.json)
are separate preimplementation hypotheses: 14 declaration sources and 12 typed
literal sources, physical editions 2021/2024. The independent oracle now has the
qualified outcomes recorded below. The literal implementation passes all 1,458
private tests, fourteen of fifteen frontend tests and ten native programs. A
test-only correction binds the remaining scoped-declaration rejection to its
actual diagnostic; its narrow qualification and combined release are pending.
Original `format_args` header, feature-list and diagnostics
slices retain exact upstream bytes and attribution. Unknown/malformed owners,
role mismatches, diagnostic-only declarations, duplicate identities, feature
absence and scoped builtin forms remain distinct.

## Declaration tier

Add three source-only roles to the existing Builtin enum. Their declaration-only
state is explicit; the ordinary prelude acquires no name privilege.
`require_invocable` rejects them before arguments, captures, matching or marks;
the defensive builtin dispatcher and Report `FrameView` reject them too. Source
provider, exact active owner/role/header, defining edition and canonical macro
identity still come from the current shared producer and immutable Rc snapshot.
Legacy textual/export publication is unchanged. Scoped formatting claims and
foreign macro transport remain unsupported in this tier.

The `format_args_macro` diagnostic is declaration data and must agree with its
actual local formatting role and owner. The same declared name can be an ordinary
macro when no real builtin claim exists. No placeholder body becomes executable.
This tier may reveal a later original-core declaration frontier, but cannot be
reported as formatting support or LIB-017 closure.

Retain `allow_internal_unsafe` as authenticated no-argument declaration metadata
under its actual feature request. Use the existing boxed SourceCore/contracts
owner, exact lexeme witnesses and borrowed Report session rather than another
Package pointer or a global permission flag. Duplicate handling must follow the
pinned parser; malformed arguments cannot disappear. The published entry/Rc,
wrapper/item owner, full witness, global budgets, actual invocation and raw outer
mark remain necessary. Empty/plain and annotated owner memoization stay distinct.

The pinned consumer of `Span::allows_unsafe` is the `unsafe_code` lint. It does
**not** authorize a raw dereference, unsafe call, or unsafe operation in an unsafe
function. RustGM's semantic `require_unsafe` remains unchanged. The corresponding
lint consumer and foreign grants cannot be claimed until implemented. Existing
complete unstable-feature membership, including `fmt_internals` and
`fmt_arguments_from_str`, remains exact declaration/provenance data.

## Typed literal execution tier

Reuse `Kind::LibraryValue { kind, value }` for a typed formatting result and its
actual decoded string child. A new tag does not add fields to AST nodes, Body,
Code, Engine or frames. Brace/statement categories and opaque argument provenance
follow the qualified `cfg_select`/builtin argument machinery. Literal grammar,
escapes, newline and capture provenance are checked; user expressions are not
silently executed to obtain a string.

Register the real local `format_arguments` nominal language owner, not a factory
nominal or an invented core path. The compiler protocol selects its actual
associated `from_str` member through existing owner/member indexes. Name spelling
alone supplies neither identity nor permission. Check visibility, declaration
stability, actual full generic/region arguments, opened callable signature,
input/output relation and ordinary const eligibility. Missing/wrong/private or
nonconst constructor opponents remain real obligations.

Use existing selected-call storage (`Methods::Call`, unchanged shape), ordinary
`Operation::Call`, generic function instantiation, bound obligations and binder
opening. `body::Builder::language_call` emits the checked function item and normal
call. Aggregate-only LibraryValue consumers must branch on the actual selected
operation; a formatting tag cannot enter aggregate construction. Constant
construction must take the ordinary function-body evaluation lane rather than
fabricate a field value or skip a reached call's payment.

The initial executable tier handles literal-only formatting by calling the real
source constructor. Native controls observe its actual string bytes through a
thin pointer, preserving source-owned layout. It does not implement a formatting
runtime or grant foreign source API availability.

## Nonempty arguments and full closure

A cold owned template/argument descriptor will retain parsed NodeIds, source
provenance, argument uses and formatting choices. Parse expressions once through
actual registry inputs and parse the template once. Use the actual
`format_argument` owner and its real `new_display`, `new_debug`, numeric, pointer
and count constructors; their ordinary source signatures carry the actual trait
and generic obligations. Calls evaluate each operand once in source order,
while repeated placeholders reuse the correct borrowed argument/trait pair.
Preserve named/captured argument hygiene and temporary lifetime extension.

Rustc's lowering places argument evaluation/bindings outside the generated
unsafe block; only the actual `Arguments::new` call is inside it. Captured user
operations must not inherit that context. Template encoding and source generic
array references must match the real pinned constructor, with no assumed private
Arguments field layout. Full actual-source formatting, relevant permissions,
foreign transport and backend/native observations are required for closure.

No Methods::Call shape, resolver lexical-scope protocol or root-owned member
visibility API changes are proposed. All new expansion publication uses the
existing Paths::invocation(actual Definition) method. Cold containing layouts,
ordinary no-format allocations, template width × argument-use work and warmed
failed-budget controls must be observed during the owning qualification.

## Independent observations before implementation

The first frozen corpus remains unchanged: 140 actual rows of 172 planned,
18 actual native programs, all declaration-language predictions matched. Its
unknown-role predicate used `builtin` while rustc spells `built-in`. All eight
literal-positive hypotheses stopped at missing Freeze in their source const
constructor. Negatives with that extra error were not accepted as qualified
constructor opposition.

A distinct 12-source contract adds the exact qualified Destruct/Freeze source
prerequisites and required feature requests, preserving every original source.
All 48 metadata/object checks then match, with genuine E0308, E0015 and E0624
constructor negatives and no missing-Freeze mask. Fourteen native programs pass.
The raw source has an actual newline, while its initial consumer predicted a
backslash escape; both failed comparisons remain. A separately declared
four-row continuation reuses the exact two compiled objects and compares actual
`610a62` bytes; both native programs pass without repeating language checks.

The [immutable review](../pa31/tests/spec/source-format-macros/independent-review.json)
and [observed cases](../pa31/tests/spec/source-format-macros/observations.json)
bind all roots, source and stream hashes. Across the preserved history there
are 38 unique source inputs, 224 actual observations and 34 matched native
programs. The qualified milestone set is 14 declaration plus 12 complete literal
sources; original incomplete literal inputs and original failed comparisons
remain explicit. No compiler implementation is qualified by these results.

## Source-only literal checkpoint

The current unqualified producer carries an actual constructor NameId in the
LibraryValue tag. Its canonical result is authenticated against the real local
builtin invocation and retained source frame before semantic use. The owner is
the registered source FormatArguments struct; its actual single lifetime slot
is preserved. An isolated body continuation first requests the existing
inherent-header bucket, then retains ordinary PathQuery state across Pending,
and finally demands the selected declaration signature. Constructor selection,
visibility, inherited impl inputs, argument coercion, bound obligations, binder
opening, safety and reached const execution retain their normal owners.

The cold CallTypes plan adds one optional pointer (predicted eight bytes), and
the new Inherent requirement may grow its enum and containing records. The
focused gate must report actual sizes for those owners and syntax Kind/Node;
these are predictions until that measurement. No Methods::Call, Body, Code or
Frame field is added. Ordinary expressions do not allocate the formatting plan.
Generic formatting arguments/placeholders remain explicitly unsupported while
actual argument construction is developed; no empty executable is supplied.

Root's newer member lookup uses an actual Use context rather than this isolated
branch's older ScopeId API. A separable helper patch against956c9904 retains that
Use through identical candidates, visibility, conditions and polling. Its
consumer uses the actual formatting result key/caller scope, never an invented
prefix syntax node. B's ImplementationValidated dependency is a separate enum
variant which integration must preserve alongside Inherent.


## Source argument construction checkpoint

The default-Debug nonempty tier now executes the real pinned
`Arguments::new`, `Argument::new_debug` and `argument_new!` bodies. It encodes
`{:?}` and positional Debug placeholders, escaped braces and UTF-8 literal
pieces, evaluates original parsed operands once in source order, and borrows
those values through the actual source constructors. Display/numeric formats,
named/captured formatting arguments and format options remain required later
work. The independently qualified constructor provider explicitly reduces its
Write/Debug dependency declarations; it does not replace the retained complete
`Write::write_fmt -> SpecWriteFmt -> fmt::write` source graph.

The generated wrapper is an authenticated expansion sequence with two super-let
bindings. The same scope owner supplies their actual default-statement or outer
let endpoints. An ordinary generated block incorrectly ended those locals at
its closing brace; that failed loan observation is retained. The narrow unsafe
block encloses only the actual unsafe `Arguments::new` call. Adopted user inputs
remain outside it and keep their original nodes, spans and contexts. No generic
storage cleanup, borrow check or source-body requirement is suppressed.

Function-item formation checks the actual public Rust ABI, arity and returned
one-lifetime nominal. It does not export the function's bound output region into
the caller. The actual parent Call opens that binder and retains ordinary input,
result, trait, const, instance, ownership and body checks. The failed predecessor
that equated the unopened result is retained.

Two prerequisites use their existing owners. Genuine crate-root
`extern crate self as NAME` declarations populate the same extern prelude;
lexical bindings, privacy and disabled-prelude rules retain authority. Such an
alias borrows its lexical stability frame and cannot overwrite that frame as an
owned declaration scope. `cfg(sanitize = ...)` uses the existing cfg-use and
feature owner (Feature47 and the sixth known allowance bit). It evaluates the
configured predicate data, with actual feature/provenance validation; this
provides no sanitizer instrumentation option or permission.

The frozen [constructor provider](../pa31/tests/spec/source-format-macros/arguments/constructor-real-contract.json)
and [statement observer](../pa31/tests/spec/source-format-macros/arguments/constructor-statement-contract.json)
preserve the original constructor bodies. Independent Rust and locked CPPGM
native execution both produce:

- Stored arguments: `e1;e2;t1;r7,9;d9;d7;7 9`.
- Statement arguments: `e1;e2;t1;d9;d7;r7,9;7 9`.

These traces observe single evaluation, template bytes, real argument storage
and reverse destruction at distinct endpoints. Escaping return/block borrows,
borrowed input returned as static, raw unsafe input and const misuse have actual
opposing controls. The original const case first reaches API-policy denial; a
separate explicit unstable const-policy/no-Drop case checks semantic non-const
call eligibility. Earlier invalid FnPtr Copy/Clone setup, missing dyn metadata,
source-assembly, lookup, feature, lifetime and loan failures remain evidence.
Existing authenticated FnDef/FnPtr marker proof supplies Copy/Clone marker
requirements; explicit Clone method dispatch and the full FnPtr family are
separate obligations.

Actual `LibraryValue` width is 8 -> 12 bytes; Syntax Kind28, Node36, Body456,
Rvalue20, Resolution2056 and Package256 retain their measured containing widths
at this checkpoint. The existing cold formatting plan1064, path query1032,
CallTypes704 and body work40 remain unchanged. No second scope/permission/proof
owner or retained call/body field was added. Current producer N8/32/128 work is
1471/4925/18777 and retained charge is 28092/106148/417524 bytes; exact root
authentication costs22 work and no new allocation at every width. The prior
1459/4913/18765 work observations precede the added cfg allowance classification
and remain historical. No wall-time speedup or no-op code-erasure claim follows.

The real CLI semantic view and eight ordinary alias naming outcomes also pass
on identical production/private source. The full generic-body loan inspector
still rejects the retained original `Clone::clone_from` at its existing
parameterized ownership/destruction guard. This is separate from the proved
selected constructor/callee/drop instances. Unused generic-body inspection is
not reported as a whole-provider borrow proof; the native receipt has an
immutable scope clarification preserving that failed inspection.

Full Alignment formatting, complete Formatter/Arguments helper closure,
PanicInfo/handler execution and a false assertion reaching actual Abort remain
pending. The constructor checkpoint does not prove whole original core or an
artifact publication profile.


## Default Display/Debug pairs

The next constructor tier accepts default Display (`{}`, `{0}`, `{:}`) beside
Debug (`{:?}`, `{0:?}`). The once-decoded template retains first use of the
actual `(input, formatting trait)` pair. Repeating one pair reuses its argument
slot; using one input through both traits creates two slots. Original eager
inputs are still adopted and evaluated once in input order. A direct two-slot
transient table per input supplies this identity without another retained map.
Other formatting traits/options and named/captured arguments remain required.

The generated calls select the real `Argument::new_display` or `new_debug`
through the existing actual FormatArgument owner, signature, generic, trait,
body, ownership and instance proofs. The original `argument_new!` and
`Arguments::new` bodies are unchanged. Every special use authenticates its real
root/site/member and immutable generated constructor spelling. The inner unsafe
scope still encloses only `Arguments::new`; user input expressions retain their
own safety context. No Body, Rvalue, Code, callable or permission field changes.

The [frozen mixed provider](../pa31/tests/spec/source-format-macros/display/source-shared-refs-contract.json)
adds core's verbatim shared-reference Copy/const Clone impls to the earlier
reduced dependency foundation. Pinned Rust (2021/2024) and the locked CPPGM
backend produce the exact same trace:

```
e1;e2;t1;p0M9;p1D7;p2D9;d9;d7;3
```

The observer checks encoded template bytes, three real first-use pair slots,
the actual distinct Display/Debug method pointers, single input evaluation and
reverse destruction. It does not fabricate a Formatter or invoke either
formatter method. Independent std-backed tests also check actual formatting
output/order and missing Display/Debug obligations. This is constructor evidence,
not full Formatter, original core, PanicInfo, handler or false-assert execution.

The [earlier mixed source](../pa31/tests/spec/source-format-macros/display/source-contract.json)
without those shared-reference impls remains independently Rust-positive.
Candidate gate1793139599944943129 passed five of six controls, failing its reused
shared-reference read during ownership. Ordinary call reborrowing remains a
required PA16/PA31 issue; the real core impl successor neither closes nor
relabels that positive. The preceding gate1793139181896431855 passed four of six
and exposed the corrected input-adoption assumption that input and constructor
counts were equal. Both failures and raw observations remain retained.

Private gate1793140162078991306 passes all six exact controls; separate
production/CLI gate1793140305019800762 passes strict production and ten lexer
controls on the identical 2,991-source map. Actual CLI semantic inspection of
both mixed providers passes; selected borrow/native proof is separate from the
still-unsupported whole generic-body loan inspector. Upstream verification
passes without original-source changes.

Measured transient pair row width is16 bytes; constructor name arrays are8 to12,
Template48 and Output416 (Output's predecessor was not measured). LibraryValue12,
Syntax Kind28/Node36, plan1064, path query1032 and CallTypes704 retain measured
widths. Current integrated Resolution2032 is inherited from root's cold namespace
change; the earlier2056 observation remains scoped to its checkpoint. N8/32/128
producer work is1619/5409/20605 and charged bytes28372/107100/421164. Root
validation remains22 work with no new allocation. Pair preparation is O(inputs
plus literal bytes plus emitted pairs); all two-slot/table/vector storage is
prepaid. These measurements make no wall-time or no-op-erasure claim.
