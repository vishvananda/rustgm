# RustGM Programming Assignment 6: Modules and names

Status: **implemented draft**. Prerequisite: PA5. The fixed language profile is
Rust 1.98.0, edition 2024, Linux x86-64. This assignment produces name facts;
type validity, macro expansion and executable lowering belong to later stages.

## Required implementation

Load configured modules, intern semantic names, collect declarations and resolve
lexical references. Preserve PA1–PA5 views. `dev/rustgm --emit-names SOURCE`
produces the view below; stdin uses a virtual source in the current directory.
`--cfg` keeps PA3's explicit arguments. `--count-names` reports sizes for scaling
measurements. No compiler-library code reads the environment or filesystem: the
driver supplies source files through the loader's source-provider interface.

Names use Unicode 17 NFC, remove raw-identifier prefixes and distinguish type,
value, macro, lifetime and label namespaces. Interned names occupy a shared byte
arena with dense IDs. Preserve physical spelling in immutable source images.
Borrowed literal payloads become checked source ranges; decoded payloads share
one pool per file. Loading another source must not invalidate existing facts.

Modules support inline bodies, `name.rs`, `name/mod.rs`, explicit `#[path]`,
configured path attributes and nested directory ownership. Diagnose conflicting
candidate files, missing files and circular inclusion. External modules in
blocks and non-ASCII filesystem module names require explicit paths. An explicit
file path acts like a `mod.rs` directory owner for child lookup. Parse each
canonical source path once, but give each inclusion its own module/scope identity.
The provider supplies both a canonical cache identity and the requested path.
Child directories follow the requested path, including aliases and symlinks;
canonicalization must not relocate them to a physical target directory.
Disabled declarations must never request files or resolve their contents.

Collect forward-visible items before resolving bodies. Keep primitive-type
fallback, block items, generics, parameter/local shadowing, closure capture,
item capture barriers, lifetime binders, labels and their scope boundaries
explicit. Initializers and let-else bodies precede new local bindings; condition
bindings are visible in the consequent, not its else branch or following code.
Guard-chain bindings are visible to subsequent operands and the arm body;
their initializers see earlier bindings, and neither later arms nor code after
the match can see them. Guard bindings can shadow the arm's pattern bindings.
Or-patterns must agree on binding names/modes. Constants and constructors in
patterns are references rather than invented local variables.

Resolve nested use trees, aliases, underscore imports, self imports, globs,
reexports and import cycles. Direct/explicit bindings take precedence over globs;
different glob targets are ambiguous on use. Repeated paths to the same target
retain one definition identity. Imports follow namespace and privacy rules,
including ancestor-restricted visibility and tuple-constructor field visibility.
An explicit import cannot widen its target's visibility. A dependency worklist
must wake affected paths and propagate changed glob bindings; do not rescan the
whole crate. Provisional ambiguity is not a final error while a stronger import
is still resolving. Replaced targets must update downstream aliases and globs.

Use the pinned Reference's [names](../doc/rust-reference/src/names.md),
[name resolution](../doc/rust-reference/src/names/name-resolution.md),
[modules](../doc/rust-reference/src/items/modules.md),
[use declarations](../doc/rust-reference/src/items/use-declarations.md), and
[visibility](../doc/rust-reference/src/visibility-and-privacy.md) chapters.
Directory ownership also follows the pinned compiler's
[module loader](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_expand/src/module.rs).

## Typed interface and exact text view

The maintainer's `resolve::Package` owns source units and module instances.
`resolve::analyze` returns `Resolution`: dense scopes/definitions, namespace
bindings, imports, resolved references and pending expansion tasks. References
store the definition visible at that source position, even after later shadowing.
Later phases consume these records directly; the text view is never reparsed.

IDs are decimal and zero-based. Source node IDs retain PA5 identities within a
file. File/module IDs follow configured source traversal; root module/scope are
0. Name IDs follow first interning (module loading, primitive names, declarations,
then uses). Scope/definition IDs follow deterministic creation order. Emit:

```text
name ID "NFC_NAME"
module ID name=NAME file=FILE scope=SCOPE parent=MODULE|root
scope ID module=MODULE barrier=BOOL parent=SCOPE|root
def ID NAMESPACE KIND name=NAME scope=SCOPE access=ACCESS [source=FILE:NODE] [members=SCOPE]
binding scope=SCOPE NAMESPACE name=NAME def=DEF ORIGIN ambiguous=BOOL access=ACCESS
ref FILE:NODE scope=SCOPE NAMESPACE TARGET
expansion FILE:NODE scope=SCOPE
eof files=COUNT modules=COUNT definitions=COUNT references=COUNT
```

Emit sections in that order. Bindings follow scope order, then first insertion
order; references follow semantic traversal. Name strings use Rust debug-string
escaping. Namespaces are `type`, `value`, `macro`, `lifetime`, `label`. Kinds are
`primitive`, `module`, `function`, `struct`, `constructor`, `enum`, `variant`,
`trait`, `alias`, `const`, `static`, `generic`, `local`, `parameter`, `self-type`,
`macro`, `extern-crate`, `label`, `lifetime`. Access is `public` or `module:ID`.
Origins are `Direct`, `Import(ID)`, `Glob(ID)`; import IDs follow flattened use
leaf order. Targets are `def=ID`, `associated=ID`, `pending-expansion=SCOPE`.
Optional fields are omitted. Boolean spelling is `true`/`false`. End every line,
including the last, with LF. Hash iteration must not affect any output order.

Complete loading/resolution before writing any text. Errors return 1, empty
stdout, and `ERROR: file FILE: OFFSET: MESSAGE` on stderr; offsets are normalized
UTF-8 bytes. Exact required diagnostic bytes live in the fixture sidecars. An
import work budget of 1,000,000 plus 128 times the import count diagnoses excessive
retry work. Earlier parser budgets remain applicable. Long expression chains,
module traversal and arena destruction do not consume proportional call stacks.

## Boundary and validation

Macro captures remain opaque. Invocation owners become explicit expansion tasks;
missing ordinary references that depend on such a scope retain pending queries.
PA7/PA8 own matching, hygiene, macro scope, generated imports/items and completing
those queries. PA6 does not claim that an opaque invocation successfully expands.

Type-directed associated lookup retains its resolved base for later analysis.
Field/method selection, type inhabitation, callable validity, pattern usefulness
and borrow validity are later contracts. A variant can resolve in the type
namespace while failing PA9's requirement to denote a type; the manifest records
that deliberate rustc-acceptance difference. Lifetimes `'static` and `'_` keep
their built-in meaning. Primitive scalar names are available; external crate
metadata and the standard-library prelude are not fabricated here. External
qualified paths report unavailable metadata until the corresponding later stages.

The manifest declares every primary input and auxiliary module file. File cases
run with their source path; stdin cases retain earlier harness behavior. Receipts
hash auxiliary bytes as well as primary bytes and arguments. Independent rustc
checks copy only the declared source package into a fresh case directory.

Run `make test-pa6`, `make test-rustc`, all earlier suites, debug/release typed
invariants, `make lint audit-rust test-harness test-upstream`, and `make perf-names`.
Invariants check definition identity, scope boundaries, moved source ownership,
NFC table growth, module caching, deterministic rendering and long chains.
