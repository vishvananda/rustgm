# RustGM Programming Assignment 4: Expressions and statements

Status: **implemented-draft**. Prerequisite: PA3. The profile is Rust 1.98.0,
edition 2024, Linux x86-64. This assignment parses one expression fragment,
including a block containing statements. PA5 adds a separate crate/item entry
point; PA4's fragment mode and its oracles remain stable.

## Required implementation

```sh
rustgm --emit-ast [--edition=2024] [-o OUTPUT] [SOURCE|-]
```

Consume PA1 tokens through bounded lookahead, interpret demanded PA2 literals,
and construct typed syntax. Do not retain a token forest for ordinary source,
call rustc, relex source slices, or recover facts from a rendered tree. Capture
PA3 token forests only when an actual macro invocation owns a delimited input.
Names and punctuation retain normalized byte spans. Reuse scratch capacity and
store children in contiguous ranges in an append-only arena. Nodes must not
recursively own other nodes.

The grammar includes:

- Literals, paths (including leading `::` and raw identifiers), grouping, tuples,
  arrays/repeats, struct expressions and update bases, calls, method calls,
  fields, tuple projections, indexing, `?` and `.await`.
- Unary negation, not, dereference, shared/mutable/raw borrows; casts; all binary
  arithmetic, shift, comparison, bitwise, lazy boolean and assignment operators;
  half-open, inclusive and open-ended ranges.
- Blocks, empty statements, semicolon statements, final tail expressions, `let`
  declarations with optional type/initializer, and `let ... else`.
- `if`/`else if`/`else`, condition let chains, loops, while/for, loop/block labels,
  return/break/continue, match arms and guards, closures (including move and async),
  and plain/unsafe/const/async blocks.
- Delimited macro invocations with opaque input. Invocation syntax is recorded;
  expansion is PA7/PA8. PA4 intentionally does not validate escapes or arbitrary
  suffixes inside opaque captures, even where rustc diagnoses them eagerly.

Calls through a named method and through a field have different syntax facts:
`x.f(1)` is a method call, `(x.f)(1)` is a call of a grouped field expression,
and `x.0(1)` is a call of a tuple field. Splitting float-shaped tokens in
`x.0.1`, or `&&` into two borrows, operates on typed tokens and source spans.
It never invokes the lexer again.

### Precedence and boundaries

From lowest to highest: assignment (right associative), ranges (nonassociative),
`||`, `&&`, comparisons (nonassociative), `|`, `^`, `&`, shifts, additive,
multiplicative, casts, unary, postfix. Other binary operators associate left.
Parentheses make explicit grouping nodes. Unparenthesized chained comparisons
and ranges, missing inclusive-range ends, and invalid tuple indices are errors.
Casts followed directly by postfix syntax, or ambiguous `<`/`<<` following a
path type, require parentheses.

A statement beginning with an expression-with-block ends at that expression
unless continued with `.` or `?`. Thus `{ if true {} [1,2] }` contains an if
statement followed by an array tail; `{ if true {} + 1 }` is rejected. Async
blocks are expressions-without-block for this purpose. Let-else initializers
ending in `}` or consisting of a lazy boolean expression require parentheses.
Only condition conjunction operands may be let expressions; parentheses and
ordinary expression positions do not introduce let chains. Match guards also
accept these conditional lets and conjunctions, with the same restrictions on
ungrouped `||` and range operands. Guard expressions and let scrutinees permit
unparenthesized struct expressions because `=>` delimits the guard; if/while
conditions retain their separate struct-expression ambiguity restriction.

### PA5 boundary

PA4 supports the type and pattern atoms its body grammar needs. Types include
nongeneric paths, grouped/tuple/slice/array types, references with optional
lifetimes and mutability, raw pointers, `_` and `!`. Patterns include bindings
with `ref`/`mut`/`@`, wildcards, literal/negative-numeric patterns, paths,
parenthesized/tuple/slice/tuple-variant patterns, reference patterns, rest
patterns in collections, and alternatives where allowed by the owner.

Full item, generic/qualified-path, type and pattern grammar belongs to PA5,
including items within blocks, struct/range patterns, function/trait-object
and impl-trait types. Attributes and docs attach to those syntax-owned
constructs in PA5, reusing PA3's configuration facts. PA4 does not accept an
attribute-bearing body or silently skip an unsupported construct. PA7/PA8 own
macro grammar/expansion; PA36 closes remaining edition and attribute semantics.

This is syntax, not type checking. Name lookup, binding validity, expression
places, exhaustiveness, lifetimes and valid return/break/await contexts are
later responsibilities. `1 + true` produces syntax here and is rejected later.
No LowIR or executable is produced by PA4.

## Text contract

Successful output is a flat, deterministic, postorder construction log. Each
node is printed as `node ID START END TAG FIELDS`. IDs are zero-based arena
indices. Spans are half-open normalized source byte offsets. Every child ID
precedes its parent; lists use `[0,1,2]`, an empty list `[]`, and missing optional
children `-`. Booleans use `true`/`false`. Names and labels use PA1's quoted byte
escaping. Construction includes path/name atoms used while forming bindings;
the `root` record identifies the reachable expression.

| Tag | Fields after the tag |
| --- | --- |
| `literal` | literal ID |
| `name` | quoted source spelling |
| `path` | absolute boolean, segment list |
| `group` | child |
| `tuple`, `array` | child list |
| `repeat` | value, count |
| `unary` | operator, operand |
| `binary` | source operator spelling, left, right |
| `range` | inclusive boolean, optional start, optional end |
| `cast` | value, type |
| `call` | callee, arguments |
| `method-call` | receiver, quoted method name, arguments |
| `index` | value, index |
| `field` | value, quoted name/index |
| `struct` | path, field list, optional update base |
| `field-init` | quoted name, value |
| `block` | flavor, statements, optional tail |
| `empty-statement` | no fields |
| `statement` | expression |
| `let` | pattern, optional type, optional value, optional else block |
| `let-condition` | pattern, value |
| `if` | condition, then block, optional else expression |
| `loop` | body |
| `while` | condition, body |
| `for` | pattern, iterator, body |
| `label` | quoted label, expression |
| `return` | optional value |
| `break` | optional quoted label, optional value |
| `continue` | optional quoted label |
| `match` | scrutinee, arms |
| `arm` | pattern, optional guard, value |
| `closure` | move boolean, async boolean, parameters, optional result type, body |
| `parameter` | pattern, optional type |
| `macro` | path, capture ID |
| `binding` | mutable boolean, by-ref boolean, quoted name, optional subpattern |
| `wildcard`, `rest-pattern`, `infer-type`, `never-type` | no fields |
| `pattern` | flavor, optional path, children |
| `reference-pattern` | mutable boolean, pattern |
| `type` | flavor, children |

Unary names are `negate`, `not`, `deref`, `borrow`, `borrow-mut`, `raw-const`,
`raw-mut`, `try`, `await`. Block flavors are `Plain`, `Unsafe`, `Const`, `Async`,
`AsyncMove`. Pattern flavors are `Tuple`, `Slice`, `Variant`, `Or`. Type flavors
are `Path`, `Tuple`, `Slice`, `Array`, `Ref`, `RefMut`, `PointerConst`, `PointerMut`.
Reference type children contain an optional lifetime name followed by the type.
An array type's second child is its length expression.

Immediately after a literal node, print `value LITERAL_ID ` followed by PA2's
literal record, with the same span. After all nodes, print each capture as
`capture ID START END TREE_COUNT "SOURCE"`; its typed in-process forest is
retained independently of this text view. The quoted source includes the
opening/closing delimiters and original comments/spacing. Finish with
`root ID` and `eof NORMALIZED_SOURCE_BYTES`.

For `1+2*3`, the final expression records are:

```text
node 3 2 5 binary * 1 2
node 4 0 5 binary + 0 3
root 4
eof 5
```

Parse fully before writing any output. A source error returns 1, emits no stdout
and prints `ERROR: OFFSET: MESSAGE` to stderr. Successful runs return 0 with
empty stderr. Exact messages and offsets are frozen by the fixtures. Panics,
signals, exit 86 and timeouts are implementation failures, never syntax rejection.
Ordinary recursive syntax has a 256-call nesting budget; reaching it diagnoses
an error. Flat left chains and captured delimiter trees do not consume that
budget. The cursor retains at most three lookahead tokens.

## Validation

`make test-pa4` compares the manifest's committed text sidecars. Independent
rustc checks wrap expressions in declared contexts; syntax-only differences
must carry an explicit explanation. Hand-checked structural tests cover
precedence, associativity, child-list ordering, calls, spans and statement
boundaries. Adversarial tests exercise long chains, deep captures and diagnosed
recursive nesting. `make lint audit-rust test-harness test-upstream test-rustc`
and all prior suites are required. `make perf-ast` retains scaling/A/A evidence;
changes to an existing hot path also require immutable A/B measurements.
