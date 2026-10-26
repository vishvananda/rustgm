# RustGM Programming Assignment 5: Items, types and patterns

Status: **implemented-draft**. Prerequisite: PA4. The language profile remains
Rust 1.98.0, edition 2024, Linux x86-64. This adds a crate entry point, typed item
ownership and configuration-aware traversal. PA4's expression-fragment mode
and its earlier oracles remain available unchanged.

## Required implementation

```sh
rustgm --emit-crate-ast [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Parse a crate once with the ordinary bounded token cursor, reusing PA4's body
parser. Retain typed identities, child ranges and source spans. Item payloads,
generic parameters, function-pointer types, enum variants and use trees occupy
separate dense arrays so uncommon syntax does not inflate ordinary expression
nodes. No node recursively owns another node. The reference keeps ordinary
nodes at 36 bytes on the pinned host. Do not implement syntax by delegating to
rustc or by printing and reparsing another phase's output.

### Items and ownership

Implement functions and qualifiers, visibility (including restrictions), generic
parameters/defaults, where clauses, struct/tuple/unit declarations, enums,
unions, constants/statics, type aliases, inline/external module declarations,
use trees and aliases, extern crates/blocks, traits and implementations.
Associated functions/types/constants, self parameters, explicit receiver
lifetimes, ABI strings and variadic declarations have typed records. Functions
inside blocks use the same parser as top-level functions.

An owner determines which item grammar is legal. Trait/impl bodies accept
associated items and macro invocations. External blocks accept function/static
declarations and macro invocations. Free and impl definitions require their
bodies or values; trait declarations may omit them; external declarations may
not contain definitions. Name/type validity and context-dependent language
restrictions beyond those grammar rules remain later responsibilities.

Retain delimited `macro_rules!` definitions and macro invocations as typed
captures. Do not parse their arbitrary bodies as ordinary Rust or expand them
in PA5. Attributes inside a capture are not attributes of surrounding syntax.

### Types, generics and patterns

Extend PA4's type atoms with generic arguments, const arguments, lifetime
arguments, associated equalities/bounds, qualified paths, function pointers,
higher-ranked lifetimes, `dyn`/`impl` trait bounds and precise `use<...>` capture
bounds. Parse path/function generic argument forms and expression/method
`turbofish` syntax. Angle delimiters split existing compound tokens such as
`>>`, `>=`, `<<` and `<-` at grammar-owned boundaries without relexing source.
An identifier in a generic argument is syntax until resolution determines
whether it denotes a type or a constant. Retain the presence of an empty `for<>`
binder as a typed fact. Higher-ranked binders cannot modify lifetime bounds or
relaxed `?` trait bounds; diagnose these combinations in either modifier order.

Extend patterns with record/tuple-struct fields, shorthand bindings, rest
fields, qualified paths, nested alternatives, modern ranges and macro
placeholders. Preserve reference/binding modifiers and range endpoints as
facts. Reject malformed range bounds and obsolete `...` patterns in the
initial edition-2024 profile. Binding interpretation, refutability, range type
constraints, usefulness/exhaustiveness and match ergonomics are PA6/PA9/PA13
and later semantic work.

### Attributes and configuration

Each grammar owner attaches PA3 attribute/doc captures to its typed node.
Support outer attributes on items, fields, variants, function and closure parameters, generic
parameters, local declarations, match arms and the expression positions covered by the
fixtures. Inner attributes attach to the crate, inline module, associated-item
body or block that owns them. Attribute placement and builtin semantics beyond
this draft's cases are completed at PA40.

Syntax captures attributes without depending on expansion. Expansion visits
owners from the root, evaluates their cfg/cfg_attr facts and records an active
node map. A disabled owner stops evaluation of later sibling cfg predicates
and traversal of its descendants: nested cfg
predicates and body literals are not demanded. Grammar still has to parse, so
an invalid disabled `let x = ;` is rejected. Unknown names and type errors in
disabled constructs do not become later semantic work.

Crate parsing initially records ordinary literal tokens as deferred syntax.
Only active traversal decodes them through PA2 into typed literal values.
This admits an arbitrary literal suffix in a disabled body without eagerly
interpreting it. PA1's draft token/literal diagnostic boundary remains in
force; this is not a claim that every malformed-token diagnostic occurs at
rustc's exact phase. Macro captures stay opaque until PA7/PA8.

Configuration effects retain the identity of their original attribute capture
and their cfg_attr parent, even across different owners and capture forests.
Store effect records in a dense array; reuse scratch for owner evaluation.
Later resolution/type/lowering work must consume the active map and typed
edges, not rescan source or infer activity from rendered output.

## Text contract

Use PA4's node IDs, half-open normalized byte spans, list/optional notation,
quoted spellings, literal records, macro capture records, `root` and `eof`.
PA5 adds the records below. Node IDs are still postorder construction IDs;
separate payload-array indices are implementation details except item IDs.

Before the syntax dump, write configuration facts in traversal order:

```text
configured OWNER_ID true|false
attribute-effect OWNER_ID EFFECT_ID CAPTURE_ID root|PARENT_EFFECT_ID EFFECT
active [NODE_IDS]
```

Effects use PA3's `keep`, `cfg=true`, `cfg=false`, `cfg_attr=true` and
`cfg_attr=false`. Effect IDs and parent IDs refer to one global dense effect
array. `active` lists reachable enabled nodes in ascending ID order. Disabled
subtrees are absent; grammar leaves no unreachable construction atoms, so
configuration need not walk enabled syntax. The syntax log still shows
all constructed nodes, including disabled syntax, for deterministic diagnostic
inspection. It is the active map that gates semantic work.

Additional node tags and their fields:

| Tag | Fields |
| --- | --- |
| `crate` | item-node list |
| `item` | item ID |
| `attributed` | first attribute-capture ID, exclusive end, owned node |
| `deferred-literal` | lexical kind, quoted spelling (inactive syntax only after configuration) |
| `applied` | base, generic arguments, optional function-trait output type |
| `qualified` | type, optional trait path, remaining path |
| `constraint` | target, optional equality type, bounds |
| `bound` | optional-bound boolean, lifetime binders, value |
| `generic` | `Lifetime`/`Type`/`Const`, quoted name, optional type, bounds, optional default |
| `where` | lifetime binders, subject, bounds |
| `function-type` | unsafe boolean, optional ABI literal, binders, parameters, optional result |
| `trait-type` | impl-trait boolean, bounds |
| `precise-capture` | captured parameter names |
| `field-definition` | optional quoted name, visibility, type |
| `variant` | quoted name, shape, fields, optional discriminant |
| `use-tree` | optional prefix, children, glob boolean, optional quoted alias |
| `struct-pattern` | path, fields, rest boolean |
| `pattern-field` | quoted name, pattern |
| `pattern-range` | optional start, optional end, inclusive boolean |
| `receiver` | mutable boolean, quoted explicit lifetime (other receivers use PA4 pattern nodes) |
| `generic-method-call` | receiver, applied method node, call arguments |
| `variadic` | no fields |

An item node is immediately followed by:

```text
item ID NAME VISIBILITY GENERICS WHERE_CLAUSE KIND FIELDS
```

NAME is quoted or `-`. Visibility is `private`, `public` or `restricted:PATH_ID`.
Shapes are `Unit`, `Tuple`, `Record`. GENERICS and WHERE_CLAUSE are child lists.
The item kind determines the remaining fields:

| Kind | Fields |
| --- | --- |
| `function` | const, async, unsafe and explicit-safe booleans; optional ABI; parameters; optional result; optional body |
| `struct` | shape, fields |
| `enum` | variants |
| `union` | fields |
| `trait` | unsafe boolean, super-bounds, associated items |
| `impl` | unsafe and negative booleans, optional trait path, implementing type, associated items |
| `const` | type, optional value |
| `static` | mutable boolean, type, optional value |
| `type-alias` | bounds, optional type |
| `module` | item list or `external` |
| `use` | use-tree node |
| `extern-crate` | optional quoted alias |
| `extern-block` | unsafe boolean, optional ABI, declarations |
| `macro-rules` | capture ID |
| `macro-invocation` | invocation node |

After macro captures, write each attribute source capture as
`attribute-capture ID outer|inner START END "SOURCE"`, before `root` and `eof`.
An empty crate is exactly:

```text
active [0]
node 0 0 0 crate []
root 0
eof 0
```

Parse and configure completely before emitting output. Source errors return 1,
empty stdout and `ERROR: OFFSET: MESSAGE` on stderr. The required fixtures own
exact diagnostic bytes. Exit 86, panics, signals and timeouts are failures.
Crate grammar has a diagnosed 128-call nesting budget, tested on the ordinary
debug test-thread stack; the cursor retains at most three tokens. Long item
lists and captured groups build, walk and drop iteratively.

## Boundary and validation

PA6 loads external modules, normalizes/interns identifiers and resolves names.
PA7/PA8 match and expand macros. PA9 onward checks types and semantic validity;
PA11 begins Rust-to-LowIR lowering. No executable output or Rust library
compatibility is claimed here. Planned later stages still fail explicitly.
Full edition/attribute edge-case closure remains PA40; PA5 is an implemented
draft of the concrete grammar/test contract above.

`make test-pa5` runs all manifest fixtures against committed text oracles.
`make test-rustc` checks their full-program outcomes independently, with explicit
reasons for deliberate phase-boundary differences. Tests also assert selected
configured-owner facts; structural tests verify child ordering, source spans,
compact nodes, deferred work, recursion limits and large item lists. Run all
prior suites, debug/release invariants, Clippy, file/architecture audits,
harness/provenance checks and affected immutable A/B scaling measurements.
`make perf-crate-ast` measures parsing plus configuration without rendering,
while separately checking complete output equality.

The `default` item modifier retains its exact keyword span and actual function,
impl, constant or type item identity in sparse typed metadata. Fragment transfer
and original-module copies preserve those facts without enlarging expression
nodes or ordinary item payloads. Ordinary item text stays unchanged. This is
syntax only: feature authority, legal owners and specialization remain later
checks. `tests/spec/items/default-*` preserves independently observed sources.
