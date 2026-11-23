# RustGM Programming Assignment 3 — Token trees and attributes

Build on [PA2](../pa2/README.md) to group tokens, read attributes, and evaluate
conditional compilation predicates. Use Rust 1.98.0, edition 2024, with the
`x86_64-unknown-linux-gnu` target, baseline x86-64 features, aborting panics,
and disabled debug assertions. The profile covers ordinary libraries and
binaries. Consult the pinned Rust Reference on
[macros](../doc/rust-reference/src/macros.md),
[attributes](../doc/rust-reference/src/attributes.md),
[comments](../doc/rust-reference/src/comments.md), and
[conditional compilation](../doc/rust-reference/src/conditional-compilation.md).

## Interface

```text
rustgm --emit-token-trees [--cfg OPTION]... [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Read and normalize source as in PA1. Capture delimiter trees, discard ordinary
comments/whitespace, retain token spans and arbitrary literal suffixes. Nested
parentheses, brackets and braces must match. A closing delimiter without an
opener, a mismatch or EOF inside a group is an error. Do not decode arbitrary
literal tokens: macro consumers decide when they denote literal expressions.

Desugar documentation comments to typed `doc` attributes with a borrowed body
range. Strip only the three-byte comment introducer and a block comment's final
`*/`; preserve all body whitespace, nested comment spelling and Unicode bytes.
Bare CR is rejected. CRLF has already been normalized. `////`, `/**/` and
`/***/` are ordinary comments, not documentation.

Parse root-level outer `#[...]` and inner `#![...]` attributes, including simple
paths, raw identifiers, optional token-tree input, `= expression-tokens`, and
`unsafe(...)` wrappers. Attribute paths use edition-2024 keyword rules. General
attribute input remains typed tokens; expression parsing, macro expansion,
builtin-attribute semantics and attribute placement are later responsibilities.
Doc comments yield the same typed attribute interface without printing and
relexing a synthetic string.

## Configuration and syntax ownership

Evaluate `cfg` predicates using flags, string-valued options, `true`, `false`,
`all`, `any` and unary `not`. Empty all/any are true/false. Validate every
operand even when the result is already known. Accept trailing commas in
predicate lists, including Rust 1.98's `cfg(unix,)` and `not(unix,)` forms.
A name/value predicate requires a PA2-valid ordinary or raw string without a
suffix. Unsupported operators, wrong arity and malformed arguments are errors.
Predicate and `cfg_attr` nesting each have an explicit limit of 128 levels.
Token-tree nesting itself is iterative and has no such depth limit.

The fixed profile sets `unix`, the Linux/GNU/x86-64 target strings, the features
`fxsr`, `sse`, `sse2`, 8/16/32/64/ptr atomic widths and primitive alignments,
`panic="abort"`, empty `target_abi`, and `target_vendor="unknown"`. Absent options
are false. `--cfg flag` and `--cfg 'feature="name"'` add custom options; values
for one key accumulate, duplicates are harmless, and a key/value does not also
set the bare flag. Raw identifiers allow keyword-named custom options.
Compiler-controlled flags and `target_*` keys cannot be overridden. Invalid
command-line configuration is a usage error, not source rejection.

The fixed course target includes the bare flags `target_has_atomic` and
`target_has_atomic_load_store` as well as their width-valued options.
`target_has_atomic_primitive_alignment` is false as a bare flag and has the
stated width-valued options. The 128-bit width is absent. Compute these facts
from the target profile. PA8 enforces source feature or macro allowances for
load/store predicates. Bare `target_has_atomic` is available directly.

`cfg_attr(predicate, attrs...)` validates attribute syntax and, if selected,
expands its attributes in order. Expansion may produce more `cfg_attr` or `cfg`
attributes; selected expansions cannot produce `crate_name` or `crate_type`.
An unselected expansion does not evaluate its nested cfg predicates. Retain
provenance: generated attributes point to their nearest producing attribute.

**PA3 computes configuration facts; syntax owns attachment and removal.** The
standalone dump interprets only attributes among the root siblings. Groups
retain opaque contents, so an attribute-looking sequence inside macro input is
not eagerly processed. The reusable attribute API accepts a syntax-owned tree
position, allowing the later parser to demand attributes on fields, statements,
parameters and nested items. An attribute set is enabled only if all its cfg
facts are true. PA5 attaches these facts to parsed constructs and omits disabled
constructs from subsequent semantic work. Do not guess item boundaries by
searching for braces or semicolons. PA7/PA8 own macro matching and expansion;
PA40 closes the complete attribute/edition semantics.

## Text contract

This diagnostic view retains one flat preorder forest. IDs are zero-based node
indices, and DEPTH is the number of containing delimiter groups. Group nodes
include their opening and closing delimiter in START..END; there are no
separate closing nodes. AFTER is the first node after the complete subtree.
Offsets are normalized byte positions with exclusive END. Spelling and body
bytes use PA1 quoting. There is no indentation proportional to depth.

```text
group ID DEPTH paren|bracket|brace START END AFTER
token ID DEPTH KIND START END "SPELLING"
doc ID DEPTH outer|inner START END "BODY"
```

Following the forest, emit root attributes and their demanded expansions in
preorder. PARENT is `root` or the producing cfg_attr's attribute index; STYLE
is `outer`/`inner`, SAFETY is `safe`/`unsafe`, EFFECT is `keep`, `cfg=true`,
`cfg=false`, `cfg_attr=true` or `cfg_attr=false`. PATH joins semantic segment
spellings with `::`, dropping raw-identifier prefixes but deferring NFC identity
to PA6. An expanded attribute retains its source attribute's whole span.

```text
attribute ID PARENT STYLE START END SAFETY EFFECT PATH word
attribute ID PARENT STYLE START END SAFETY EFFECT PATH group TREE_ID
attribute ID PARENT STYLE START END SAFETY EFFECT PATH value FIRST_ID AFTER_ID
attribute ID PARENT STYLE START END SAFETY keep doc value "BODY"
eof NORMALIZED_BYTE_LENGTH
```

All lines end in LF. For `(x)` without a newline:

```text
group 0 0 paren 0 3 2
token 1 1 identifier 1 2 "x"
eof 3
```

Success exits 0 with empty stderr. Source errors exit 1 with
`ERROR: OFFSET: MESSAGE` followed by LF, with **empty stdout**. Emit the
forest only after analysis succeeds. The test fixtures give the expected
wording. Usage errors exit 2. An unimplemented mode exits 86.

## Implementation and validation

Use flat arenas/ranges and an explicit delimiter stack, without a Box or Vec
per group, recursive destruction or per-token spelling allocations. The whole
forest here is an authorized diagnostic/macro-fragment representation, not a
mandatory intermediate for ordinary source parsing. PA4's ordinary parser must
consume a bounded token cursor. Reuse typed attribute and configuration facts.

From the repository root:

```sh
make -C pa3 test
```

The [test manifest](tests/manifest.json) covers delimiter nesting, configuration
limits, macro input, documentation comments, expansion provenance, disabled
attributes, and atomic target facts. Run PA1 and PA2 again after changing their
token or literal interfaces.
