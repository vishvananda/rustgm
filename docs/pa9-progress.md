# PA9 types and inference buildout

PA9 remains planned. Its canonical type and constraint core is implemented and
validated, but source-level checking, the `--emit-types` view and course fixtures
are not yet implemented. Passing PA8 does not mean Rust has been type checked.

## Implemented core

`types::Store` owns immutable type identities and one shared child-list arena.
Its hash indexes compare typed identities; rendered type names are not keys.
Primitive types, unit/never, tuples, arrays/slices, references/raw pointers,
function signatures and function items have distinct representations. Function
items retain declaration identity; references retain region identity. The initial
integer width table explicitly targets x86-64. Child IDs must belong to their
owning store; range checks cannot detect coincident IDs from another store.

`types::infer::Context` owns flat terms and shared child ranges. Variable unions
use rank and path compression. General, integer and float variables have distinct
domains. Occurs checks use reusable iterative work storage and epoch marks;
structural equality memoizes compared pairs within a constraint. Known canonical
types materialize child terms only when structural comparison requires them,
reusing scratch capacity. Numeric fallback occurs during finalization, after
constraints: i32 for integer variables and f64 for floats. General unresolved
variables diagnose an error. Canonical finalization and destruction are iterative.
A cumulative million-step budget bounds each body context.

Equality is separate from coercion. Function items do not become pointers, never
does not become another type, and differing numeric primitives do not convert
merely because an equality is requested. Reference comparisons retain explicit
region equalities for later checking. A failed constraint may retain earlier
progress; callers discard the body context rather than using it for speculative
candidate selection. Transactional trait/method probing remains a later owner.

The Rust compiler development guide describes separate general/integer/float
inference variables and deferred region constraints; it is architectural guidance,
not a library invoked by this implementation:
<https://rustc-dev-guide.rust-lang.org/type-inference.html>.
The pinned local Reference defines function-item identity, coercion sites and
numeric types. CPPGM's specification and `fundamental_type.h` were consulted for
compact identities and explicit target facts, without copying C++ conversion rules.

## Next implementation steps

1. Build declaration/type-annotation facts from PA8's resolved definition and
   reference identities. Resolve aliases with explicit pending/in-progress/complete
   states, diagnose cycles, and retain function signatures before checking bodies.
   Item signatures reject inference placeholders; local annotations may infer.
2. Check scalar function bodies using dense per-node type facts and one inference
   context per body. Consume decoded literal values, binding identities, selected
   call targets and parsed operators directly. Keep expected types and coercions
   explicit; record selected scalar operations, numeric conversions and call facts
   for PA10 instead of asking lowering to infer them again.
3. Handle numeric fallback/range checks, unit/never, returns, joins, assignments,
   short-circuit operands and scalar casts. Keep nominal aggregate semantics,
   pattern usefulness, ownership, lifetime solving, generics, traits and library
   hooks with their existing later owners. No source rewriting or rustc fallback.
4. Define the PA9 text view and nonempty fixture manifest with independently
   declared outcomes. Compare all supported whole programs with pinned rustc;
   document any intentionally deferred semantic checks before generating oracles.
5. Validate earlier stages, strict Clippy/audits, full text equivalence and scaling,
   then activate PA9. PA10 builds typed control-flow bodies from these facts; PA11
   must lower them to actual LowIR and execute via CPPGM against rustc results.

## Current validation

Six new typed invariants cover canonical list sharing, function/region identity,
invalid arena inputs, numeric constraints/fallback, unresolved variables, indirect
occurs checks, shared graphs, 10,000-level structural unification/finalization and
retained region obligations. They pass in debug and release. The full debug suite
has 189 tests; all 1,106 active course fixtures and 16 harness checks pass.
Strict Clippy and the 110-file architecture audit report zero findings.
This core is not yet invoked by an existing compiler mode, so its addition does
not change existing source compilation paths or oracle bytes.

An isolated public-API benchmark constructs constrained numeric variables and
shared tuple shapes, then finalizes every term. It is a core measurement, not a
Rust source frontend benchmark. Frozen A/A binaries and input counts, complete
output hashes, warmups, A/A calibration and three ABBA blocks are retained at
`obj/perf/1791795117842783744/samples.json`. At 4,000/16,000/64,000 constraints,
paired A/B medians were 4.06/3.99, 7.24/7.16 and 19.57/19.56 ms. Largest peak
RSS was 7,872 KiB; largest A/A samples span 19.48–19.78 ms. The ignored benchmark
source and runner are under `obj/pa9-work/`, alongside build/test/audit evidence.
