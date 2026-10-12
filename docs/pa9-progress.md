# PA9 implementation checkpoint

PA9 remains **planned**. These library interfaces are under construction; the
course CLI still rejects `--emit-types` and there is no PA9 grading suite yet.
PA1–PA8 remain the active assignments. This record does not change their
contracts or claim full Rust type checking.

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
types materialize child terms only when comparison or shape inspection requires them,
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

## Declaration and body interfaces

`sema::declarations::analyze` consumes PA8's resolved report. It elaborates
primitive/alias/constant/static annotations and function signatures into the
canonical `types::Store`. Alias dependencies use an explicit work stack and
cycle states; a 10,000-alias chain does not recurse. The work limit is
proportional to the syntax graph, so an otherwise ordinary large crate does
not hit a fixed declaration-count ceiling. Inactive syntax is excluded and
expanded declarations retain their resolved identities.

Function items remain distinct from function-pointer types. Signature types
retain binder counts and bound lifetimes encode binder depth and index, so
alpha-renamed binders canonicalize without capturing outer lifetimes. Named
lifetime references come from resolution. Elision candidates come from source
annotations, separately for each parameter; alias internals and nested
function-pointer inputs do not become outer candidates. This follows the
pinned rustc resolver as well as independent acceptance probes, rather than
inferring elision from a rendered/canonical type.

`sema::bodies::analyze` currently checks scalar function bodies. It uses dense
node/definition slots, iterative syntax traversal, and one inference context
per body. Canonical types are shared across the crate, but a context memoizes
only the canonical types it touches; it does not initialize a crate-sized
lookup table for every function. Numeric fallback runs after constraints.
General variables at never-coercion sites receive edition-2024 never fallback
only after the remaining constraints have had their opportunity to bind them.

Current body operations cover scalar literals, simple binding/wildcard locals,
resolved value paths, plain blocks, arithmetic/bitwise/comparison/short-circuit
operators, scalar casts, known function calls, returns, and `if` expressions.
Scalar/alias local annotations and `_` work. Function-item reification at an
explicit function-pointer coercion site is recorded. Integer range checking
uses the inferred type and grouped unary-minus context; float literals round
directly to their inferred format. Unary signed-minimum literal exceptions
remain distinguishable from ordinary checked negation.

Dense node facts retain solved types, scalar bits, selected operations, call
signatures and coercion adjustments for later body/LowIR construction. No
frontend phase calls rustc, reparses source, or uses rendered types as keys.
The audit now permits `sema` to consume PA2's typed `literal` values and
registers the semantic fact model under the model-file size limit.

## Independent checks and limits

The declaration tests include 22 pinned-rustc signature/elision acceptance
cases, nested-binder identity assertions, configured/generated declarations,
forward aliases and cycles. The body tests include 60 pinned-rustc scalar
acceptance cases plus assertions on solved types and retained lowering facts.
Body probes use default rustc lints so overflowing literals cannot be accepted
by accidentally capping those lints. Raw probe source, rustc stderr and metadata
are retained under `obj/pa9-work/`; they are not course oracles.

This is not yet the PA9 release boundary. Remaining work includes general local
annotation constraints, delayed callable constraints, full coercion-site
propagation and control-flow joins, assignment/place checks, declaration
well-formedness, and constant/static initializer type checking. The current
body pass does not establish initialization, ownership, outlives, constant
validity or whole-program acceptance. Those later-phase contracts must remain
explicit when the PA9 handout is released. Return-only function lifetime
parameters, generic aliases, foreign/async signatures, and lifetime bounds are
currently diagnosed as unsupported. Lifetime-polymorphic calls need proper
binder instantiation before they can be enabled. General lint attributes are
not yet interpreted by the body checker.

Next, finish and freeze the scalar source profile and deterministic `--emit-types`
view, then add declared-outcome PA9 fixtures and independently verify them before
activation. PA10 will construct typed control-flow bodies; PA11 must lower them
to real LowIR and execute it through the CPPGM bridge. No empty or provisional
suite may count toward that goal.

## Validation

The checkpoint passes 199 Rust tests, all 1,106 active course fixtures,
Clippy all/pedantic with warnings denied, the 118-file architecture audit, and
16 harness tests. The same Rust tests also run in release mode. Earlier oracle
stdout/stderr/status bytes remain unchanged; `make ref-test` refreshes only
compiler-source provenance in the receipts.

The six core invariants continue to cover shared canonical lists, function/region
identity, invalid inputs, numeric constraints and fallback, unresolved variables,
indirect occurs checks, shared graphs and 10,000-level iterative unification and
finalization. The original core-only A/A baseline is retained at
`obj/perf/1791795117842783744/samples.json`; its 4k/16k/64k paired A/B medians
were 4.06/3.99, 7.24/7.16 and 19.57/19.56 ms, with maximum RSS 7,872 KiB.

The signature-elision implementation was also checked against the pinned
[rustc resolver](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_resolve/src/late.rs),
especially its per-parameter elision-candidate accounting.

## Current measurements

All measurements use immutable binaries and frozen inputs, full output hashes,
warmups, A/A calibration and three ABBA blocks, pinned to CPU 10. No repository
builds or tests run during the measurement blocks. Scripts, drivers and every raw
sample remain under `obj/pa9-work/` and `obj/perf/`.

The core-only before/after comparison is
`obj/perf/1791797902878857968/samples.json`. At 4k/16k/64k constraints the paired
baseline/candidate medians are 4.40/4.53, 7.18/6.95 and 16.78/17.63 ms. The
largest candidate median is about 5% higher in this run, but the largest A/A
samples span 17.20–18.10 ms. These observations do not establish a stable
regression or improvement. Complete core output agrees byte-for-byte.

A new API driver exercises source loading, parsing, expansion, name resolution,
declarations and scalar body constraints on separate procedures containing
locals, arithmetic, calls and conditional returns. This is a source frontend
measurement, but the driver is temporary and is not the unreleased course mode.
The completed A/A scaling run is
`obj/perf/1791798092890572365/samples.json`. At 4k/16k/64k generated procedures
(plus one shared callee), paired A/B medians are 112.24/106.33, 505.28/494.41
and 2130.18/2056.16 ms. Largest A/A samples span 1967.84–2313.84 ms. Full typed
facts agree between both snapshots. Peak RSS at the three scales is 30,880,
116,460 and 458,260 KiB. This is scaling evidence, not a comparison
against another source type checker.

The earlier attempt at `obj/perf/1791797903660372027/` retains its successful
4k/16k observations and failed 64k input. It diagnosed the fixed declaration-work
limit; the final run uses the input-proportional limit and completes all scales.
