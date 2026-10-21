# PA25 inline constant bodies

Status: source integration under maintainer qualification. PA25 remains planned;
this document authorizes neither assignment activation nor generated references.

An inline `const { ... }` has an independent semantic and checked CFG body. Its
source key is its identity; it has no fabricated definition, closure identity or
runtime global. The enclosing expression supplies a result inference root through
the same two-way isolated `Refinement` transport used for nested bodies. Numeric
defaults wait for surrounding constraints, including later uses of a local.
Only the result interface crosses that boundary. Local bindings and loop/return
control remain in the child; enclosing value locals cannot be captured. Lexical
unsafe permission is inherited and remains independent of const eligibility.

The shared flat scheduler owns parked states without recursive ownership.
Closure-specific signatures, lifetime opening/closing and publication remain in
the closure module. Inline entries retain their own result term and semantic
registry identity. Parent body source and optional declaration owner are semantic
facts; the downstream body layer chooses `BodyKey::Anonymous(source)`. A const
block which is already an initializer's actual root keeps that initializer owner.

Result readiness and value evaluation are separate. The lazy append-only inline
registry indexes source and owner keys once. Closed values are required by retained
typed CFG operands, including operands in unused nongeneric functions and false
runtime branches. Operands omitted after an unconditional transfer are deferred
as unreachable. A zero-length inline-const repeat suppresses its value evaluation
and nested inline values; its bodies still require typing and const eligibility.
Parent/child links propagate suppression with bounded iterative work and stop at
previously suppressed subtrees. The value expression is omitted from the empty
repeat CFG, rather than evaluated and then discarded.

The outer execution provider supplies checked anonymous CFGs through typed
`InlineBody { source, result }` requirements. Interpreter frames prepare retained
inline dependencies before choosing runtime branches, then publish each required
closed value once. Missing dependencies suspend the existing iterative session;
the interpreter does not recursively invoke the semantic executor. Early and late
queries share retained bodies and the existing CFG lender under typed body keys.
Existing source parsing, inference, region/loan proof and const permission checks
remain the authority. Scalar metadata or an unsafe context cannot grant execution.

The first source slice targets closed nongeneric inline bodies and non-Copy
constant repeats. Generic anonymous-body specialization, including lifetime-parameter owners,
remains an explicit valid-Rust boundary. A direct closure body `|| const { ... }` also has an explicit
unsupported ownership diagnostic: its executable root currently aliases the
closure's semantic body identity. The wrapped form `|| { const { ... } }` is
supported; it retains a distinct inline child. Neither boundary is flattened
into runtime execution. Required, suppressed-zero-repeat, deferred-template and
deferred-unreachable states never substitute for one another. Only a completed
required value may become an inspection value root.

## Independent observations

Pinned Rust 1.98.0, edition 2024 object compilation independently establishes:

| Source shape | Object outcome |
| --- | --- |
| `take(const { 7 })` and a later `u16` constraint | accept |
| A transient inline local borrow, returning the copied scalar | accept |
| An enclosing local capture, return or escaping break | reject |
| `[const { NonCopyValue(7) }; 2]`, including a grouped operand | accept |
| A closed division-by-zero inline value in an unused function or false branch | reject |
| That value after an unconditional return | accept |
| That value, including nested inline values, in a zero repeat | accept |
| Non-const call, outer local capture or invalid return after return/in a zero repeat | reject |
| A generic function containing an invalid closed inline value, unused | accept |
| The same generic function instantiated | reject |

The unreachable value case initially had a predicted rejection. Its actual
acceptance is retained in `qualification-generics.json`; the implementation
contract was corrected from that observation. Metadata-only compilation is not a
substitute for these object checks: several evaluation failures pass metadata.
Exact sources, diagnostics and observations are retained under
`obj/parallel/pa25-inline-const/qualification*.json`.

## Inspection rows

The unreleased `constants 1` draft gains these alternatives:

```text
declaration source=FILE:NODE kind=inline type=T const=true status=STATUS parent=FILE:NODE
root inline source=FILE:NODE type=T value=vV
```

Inline declarations follow definition declarations in source-key order and count
as declarations. Status is `checked-value`, `deferred-template`,
`deferred-unreachable` or `suppressed-zero-repeat`. Only checked values add roots;
their roots follow named initializer roots and precede promotion roots. Existing
typed value/allocation sharing and FIFO graph numbering are preserved. Non-inline
output must remain byte-identical. No row treats declaration permission, result
readiness or an unevaluated template as an executed value.

## Qualified closed-body checkpoint

The frozen source hash is
`c0e1f539bb5d4fb751220077149b9cd87b702e80e037dfc1604891d200b671e0`;
the release executable hash is
`9996808ba86b87c5628284e777719d8c6adf2f8964354532761cb6ed53fa8ebe`.
All 34 declared fixture outcomes were independently object-qualified. Four are
valid-Rust unsupported boundaries: direct inline closure bodies, unused generic
owners, nested generic owners and lifetime-parameter owners. Direct-root negative
controls stop at the ownership frontier and do not qualify their underlying
invalid operation. Accepted CLI inventories are deterministic across repeated
invocations. The extra zero-repeat/type-length and inferred-reference cases also
agree with pinned rustc.

Validation passed strict release Clippy (all targets/features), 74 focused
source/inspection/constant/closure tests, eight executing native programs, the
1,295-file audit with zero findings, layout and upstream checks. Evidence lives
under `obj/parallel/pa25-inline-const/`: `inline-*.log`,
`final-qualification.json` and `extra-qualification.json`. No full matrix or
oracle generation was run in this lane.

Scaling at 8/32/128 distinct closed bodies proves one source index append, result
publication, preparation visit, checked CFG construction and value publication
per body. Lookup probes stay below 128 per body; this bounds the observed graph
visits, not sorting comparisons or total scheduler complexity. Registry logical
storage charges twice each body payload plus conservative index allowances before
append (one-million-body and 64 MiB caps). Retained anonymous bodies charge their
full owned collections and a body-key map allowance against the existing shared
64 MiB retained-template budget before execution snapshots are cloned. Snapshot
inline operand arrays and source indexes are charged separately to the existing
code budget. Shared immutable facts keep their existing ownership exclusions.
These are deterministic logical bounds, not exact allocator bytes or RSS.

The [bounded immutable performance follow-up](pa25-inline-performance.md)
retains all 427 observations and exact linear graph/index counters. Wall timing
is noisy; the earlier unexplained scalar RSS increase remains unresolved. This implementation slice does
not complete PA25's remaining dynamic pattern/enum continuations, slice execution,
raw/union validity, full promotions/statics or release fixture/oracle review.
