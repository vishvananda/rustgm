# RustGM Programming Assignment 8: Expansion and hygiene

Status: **implemented-draft**. Prerequisite: PA7. The source profile is Rust
1.98.0, edition 2024, Linux x86-64. This stage completes expansion and ordinary
name resolution; type inference begins at PA9. Core/std library hooks, procedural
macros, older editions and final attribute/lint closure retain their later owners.

## Required implementation

Expand configured crate syntax to a fixed point using a dependency worklist.
Generated definitions, imports, modules and invocations participate in the same
textual and path scopes as source-written syntax. Preserve mixed-site hygiene:
local variables and labels use definition contexts, captured syntax keeps its
context, ordinary item lookup follows call sites, and `$crate` has typed defining
crate identity. Honor macro exports, aliases, `macro_use`, local-inner rewriting,
shadowing and scope restoration. Diagnose unresolved calls, ambiguity, invalid
output syntax and recursion limits without repeatedly scanning the whole crate.

Use PA7's typed captures and the ordinary parser for newly generated syntax.
Forwarded opaque fragments are installed from their typed facts. Eager builtins
wait for expanded argument facts; source includes cache parsed fragments. Keep
source provenance, expansion ancestry and syntax hygiene distinct. Every library
I/O or environment dependency is an explicit provider input at the driver boundary.
Read `spec.md` and the pinned Rust Reference before changing these contracts.

## Tool surface and tests

`dev/rustgm --emit-expanded-crate -o OUTPUT SOURCE` emits the deterministic view
below. Earlier modes remain available. Omit SOURCE or use `-` for stdin.
`--count-expanded-crate` reports counts for measurements. Source failures emit no partial stdout.

`make -C pa8 test` runs the manifest cases against committed stdout, stderr
and exit-status text. `make test-rustc` independently checks every PA8 program.
See [tests/README.md](tests/README.md) for fixture ownership and
[the validation record](../docs/pa8-validation.md) for typed invariants and
measurements. The [small cfg fixture](tests/spec/builtins/cfg.rs) and its
[complete output](tests/spec/builtins/cfg.rs.ref) are a compact worked example.
Expected outcomes are declared before reference generation; missing sidecars,
empty suites, timeouts, panics and not-implemented exits cannot pass.

## Text view

The view begins with `expanded-crate 1`. Its sections are:

```text
macro ID name=NAME module=MODULE context=CONTEXT introduction=CONTEXT source=FILE:NODE|builtin rules=COUNT local-inner=BOOL
macro ID name=NAME module=MODULE context=CONTEXT introduction=CONTEXT source=builtin builtin=NAME
invocation ID context=CONTEXT parent=CONTEXT macro=ID source=FILE:NODE original=START:END generated=START:END rule=ID|builtin parser-calls=COUNT
file FILE
[include-template Expression|Statements|Items(Module), only for cached sources]
[PA5 configuration and syntax records]
origin START:END source=FILE:START:END context=CONTEXT|inherit
end-file FILE
[PA6 name-resolution records]
end-expanded-crate files COUNT modules COUNT macros COUNT invocations COUNT captures COUNT definitions COUNT references COUNT
```

The two macro forms distinguish declarative definitions from builtins. Macro and
invocation IDs follow creation/expansion order. Context 0 denotes physical source;
context N identifies invocation N−1. Introduction/parent contexts record expansion
ancestry, while token origins retain syntax hygiene independently. Builtins have
no invented source definition node or matcher rule. `parser-calls` counts ordinary
fragment-parser callbacks from matching, the single argument-list parse for
an eager builtin, and a demanded uncached include-source parse. A forwarded opaque fragment does not need another callback.

Cfg name/value predicates consume captured string literal roots directly,
including literal-root expression and metadata forwarding. They do not promote
arbitrary opaque expressions. Demanded `target_has_atomic_load_store` uses retain
their actual `cfg_target_has_atomic` feature or macro-allowance authority even
when false. Earlier disabled owners and unselected nested `cfg_attr` attributes
retain the ordinary ordering rules.

Per-file syntax IDs index retained arenas. The PA5 `active` record identifies the
configured tree; inactive original nodes may remain as provenance. An
`include-template` unit retains unexpanded source facts for reuse and provenance.
It is not an additional module or executable body: semantic traversal starts
from module roots and reaches the installed copies. File counts include these
cached sources. Offsets use
the normalized, append-only spelling image, with origin records mapping generated
ranges to their parents. These images and records are diagnostic views, never
inputs reparsed by a later phase. Literal values are read directly from the
source unit's typed storage without cloning its syntax arena.

Name IDs use the PA6 spelling table, now including the early builtin prelude and
ordinary item declarations. Name definitions can additionally have kind
`builtin-macro`. Successful output contains fully resolved ordinary names and no
pending expansion references. Source failures emit no view bytes, exit 1 and
report `ERROR: file FILE: OFFSET: MESSAGE` on stderr. Error offsets refer to
normalized physical source, not the appended expansion image. When applicable,
subsequent lines report innermost-first expansion ancestry:

```text
  in macro NAME at file FILE: OFFSET; defined at file FILE: OFFSET
  in macro NAME at file FILE: OFFSET; builtin
```

A captured expression keeps its original physical error location. A failed
expansion can report its selected definition even before an invocation record
has been committed. Module parse errors retain their introducing expansion,
including when the invalid file has not yet entered the source registry. These
are RustGM diagnostic messages; rustc's wording and note layout are not an oracle
contract.

The count mode emits just the footer's counts, without `end-expanded-crate`.
`make perf-expanded-crate` measures independent functions whose macro matcher
rejects its first rule, retains an expression capture, duplicates its typed
syntax and resolves the generated body. It verifies full output hashes as well
as counts, freezes binaries/inputs, and retains A/A and paired observations.

This assignment implements `cfg!`, `line!`, `column!`, `file!` and `module_path!`
using typed literals. Location builtins accept no arguments. `file!` follows the
physical invocation cause and retains the requested source path, including
symlink aliases, leading `./` and `..`; redundant interior `.` components are
normalized. Stdin is named `<anon>`. `module_path!` uses the module receiving the
expansion, including modules generated around a captured invocation. Function
names do not become module-path components.

`--crate-name NAME` supplies that path's first component. Otherwise the first
root `#![crate_name = "NAME"]` wins, then the input filename stem (with dashes
replaced by underscores), or `rust_out` for stdin. An explicit name must agree
with a root attribute. Empty names, characters other than letters/digits/`_`,
malformed duplicate attributes and `crate_name` inside `cfg_attr` are errors.
These identities do not require source reparsing or filesystem calls in the
compiler library.

`stringify!` consumes classified tokens without expanding its arguments. It walks
retained opaque fragments by identity and produces an owned string literal, with
no fragment-parser callback. Captured tokens occupy one shared arena; repeated
or forwarded substitutions do not duplicate that storage. Unicode identifier
spellings normalize to NFC. Source spacing, macro-template spacing, delimiter
edges and invisible fragment boundaries remain distinct. Direct documentation
comments retain their spelling; matching them as token trees can desugar them
into attributes before stringification.

The pinned stringification layout uses a 78-byte target width, with at least
60 bytes available after indentation, consistent brace edges and independently
wrapping contents. Long individual tokens may exceed that target. The iterative
walker diagnoses more than 1,000,000 token visits or 4,096 nested continuation
frames. These formatting decisions follow the pinned compiler's observable
[token printer](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_ast_pretty/src/pprust/state.rs)
and are checked with independent runtime values. Rendered output remains a
builtin value and diagnostic view, never a syntax transport.

`concat!` and `compile_error!` use eager argument expansion. The ordinary parser
reads the comma-separated argument list once. Nested invocations enter the same
resolution worklist; an explicit dependency count resumes the builtin after
those invocations finish. Delayed definitions and imports do not cause another
argument parse. Temporary argument nodes retain their source spans and typed
literal storage and are retired after the resulting value is installed.
For an eager builtin, `parser-calls=1` records that argument-list parse; the
first include in a grammar context adds one for parsing the included source.

`concat!` accepts strings, characters, booleans and numeric literals, including
unary-negative numeric literals. Integer spellings become decimal magnitudes;
float spellings keep their decoded decimal text without suffixes or underscores.
A trailing comma is allowed. Parenthesized literals, blocks, arbitrary constant
expressions, bytes, byte strings and C strings are rejected. `compile_error!`
requires one eagerly expanded string and diagnoses its contents. Both preserve
ordinary macro shadowing. `concat_bytes!` remains unstable in Rust 1.98.0 and is
outside this stable-language profile. Library-dependent builtin expansion is
owned by the later language/library hooks rather than fabricated external names.

`env!`, `include_str!` and `include_bytes!` also expand their arguments eagerly.
`env!` requires a string name and optionally a string diagnostic for an absent
variable. Missing values and non-Unicode values fail; the custom diagnostic
applies only to absence. Each lookup is an explicit source-provider request,
cached for that compilation. The library has no ambient process environment.

Resource includes require one string path. Relative paths use the requested
source filename at the invocation's physical expansion cause, including aliases;
a relative path from anonymous stdin is rejected. Absolute paths are allowed.
Repeated normalized requested paths share a provider read. `include_str!`
requires UTF-8 and preserves all original content, including BOM, CRLF and bare
CR. `include_bytes!` accepts arbitrary bytes and produces a typed byte string.
These resources are not parsed as Rust and do not create source syntax units.
Disabled code and discarded macro input request no resources. Drivers implement
raw-byte reads separately from normalized Rust source reads.

`include!` eagerly obtains one string path, then parses the included source with
ordinary source normalization and a bounded cursor. It supplies module items or
one expression; statement-position includes still require one expression. Type,
pattern, trait/impl/foreign-item positions and inner file attributes are rejected.
An item include strips a shebang; an expression include does not. Trailing tokens
after an included expression are rejected, matching rustc's default lint setting.
The independent fixture selects that setting rather than suppressing the lint.

Canonical file identity plus grammar context keys the syntax cache. Requested
aliases retain their own observable paths while reusing parsed facts. Repeated
insertions copy typed arenas and decoded literals directly into the receiving
unit, then expand independently in each caller's scope. Included syntax is
unhygienic and retains physical file/line origins. Modules declared inside it
start beside the included file; directory ownership is restored after that
subtree. Recursive includes use the same bounded expansion worklist. Inactive
includes perform no reads. Source reads remain injected provider operations.

The [derived constant-subtree checkpoint](../docs/pa8-derived-constants-checkpoint.md)
records typed field copying, selected literal/pending ownership and paid growth.
Clone/Copy and source/SIMD consumers remain in PA26/31/41; copied syntax never
grants their semantic or runtime permissions. Wider field expressions remain
required work.

The [typed-call checkpoint](../docs/pa8-derived-calls-checkpoint.md) extends that
selected traversal to call callees/arguments and generic types. Real const/body
checking remains mandatory; copied function spelling is never an implementation
of size_of or another library helper.

`option_env!`, formatting/assertion facilities and other expansions that need
library identities belong to PA26's language/library hooks. PA8 does not invent
external core/std definitions to make those calls resolve.
