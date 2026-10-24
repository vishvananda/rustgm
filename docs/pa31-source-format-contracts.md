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
literal sources, physical editions 2021/2024. No oracle or candidate result has
been generated. Original `format_args` header, feature-list and diagnostics
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
