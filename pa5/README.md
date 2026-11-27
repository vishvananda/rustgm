# RustGM Programming Assignment 5 — Items, types and patterns

Build on the [PA4 parser](../pa4/README.md) to parse a crate, give each item
a typed owner, and determine which syntax is active under the selected
configuration. Use Rust 1.98.0, edition 2024, on Linux x86-64. Keep PA4's
standalone expression mode.

## Command-line interface

```text
rustgm --emit-crate-ast [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Use PA1's input and output options and PA3's `--cfg` options.

## Required implementation

Parse a crate once with the ordinary bounded token cursor, reusing PA4's body
parser. Retain typed identities, child ranges and source spans. Item payloads,
generic parameters, function-pointer types, enum variants and use trees occupy
separate dense arrays so uncommon syntax does not inflate ordinary expression
nodes. No node recursively owns another node. Consume typed tokens and
configuration data directly. Implement syntax parsing in your compiler.

### Items and ownership

Implement functions and qualifiers, visibility (including restrictions), generic
parameters/defaults, where clauses, struct/tuple/unit declarations, enums,
unions, constants/statics, type aliases, inline/external module declarations,
use trees and aliases, extern crates/blocks, traits and implementations.
Retain `final fn` as a distinct sparse method modifier with its actual keyword
span. Retain the actual keyword span and item identity for a `default`
function, impl, constant, or type. Later phases check the feature gates,
owners, bodies, and specialization or override rules. Preserve these facts
through syntax copying and fragment transfer. Keep ordinary expression
nodes and unmodified item payloads compact.

Associated functions/types/constants, self parameters, explicit receiver
lifetimes, ABI strings and variadic declarations have typed records. Functions
inside blocks use the same parser as top-level functions.

An owner determines which item grammar is legal. Trait/impl bodies accept
associated items and macro invocations. External blocks accept function/static declarations, opaque `type NAME;`
declarations and macro invocations. Opaque external types have a distinct typed
item kind, accept no generics/bounds/where clause or definition, and retain the
actual `type` token for the later `extern_types` gate. They are not an omitted
type-alias definition. Free and impl definitions require their
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
this draft's cases are completed at PA41.

Syntax captures attributes without depending on expansion. Expansion visits
owners from the root, evaluates their cfg/cfg_attr facts and records an active
node map. A disabled owner stops evaluation of later sibling cfg predicates
and traversal of its descendants: nested cfg
predicates and body literals are not demanded. Grammar still has to parse, so
an invalid disabled `let x = ;` is rejected. Unknown names and type errors in
disabled constructs do not become later semantic work.

Crate parsing initially records ordinary literal tokens as deferred syntax.
Only active traversal decodes them through PA2 into typed literal values.
A literal with an arbitrary suffix can appear in a disabled body. PA1
still checks the token boundaries. Macro captures stay opaque until
PA7 and PA8.

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

An item node is immediately followed by this record.

```text
item ID NAME VISIBILITY GENERICS WHERE_CLAUSE KIND FIELDS
```

When the item has a `final` or `default` modifier, insert
`final START END` or `default START END` after ID and before NAME. START and
END are the keyword's normalized byte offsets. NAME is quoted or `-`. Visibility is `private`, `public` or `restricted:PATH_ID`.
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
An empty crate produces these records.

```text
active [0]
node 0 0 0 crate []
root 0
eof 0
```

Parse and configure completely before emitting output. Source errors return 1,
empty stdout and `ERROR: OFFSET: MESSAGE` on stderr. The required fixtures own
exact diagnostic bytes. Exit 86, panics, signals and timeouts are failures.
Diagnose a crate grammar that exceeds the 128-call nesting budget. The cursor
retains at most three tokens. Long item
lists and captured groups build, walk and drop iteratively.

## Later assignments

PA6 loads external modules, interns normalized identifiers, and resolves
names. PA7 and PA8 match and expand macros. PA9 onward checks types and
semantic validity. PA11 begins lowering Rust to LowIR. PA41 covers the
remaining edition and attribute behavior.

## Run the tests

Run the PA5 test target from the repository root.

```sh
make -C pa5 test
```

[The test manifest](tests/manifest.json) covers item grammar, configuration,
diagnostics, and the text format. Exercise nested syntax and long item lists
when testing your parser. Rerun PA4 after changes to the shared syntax cursor
or body parser.
