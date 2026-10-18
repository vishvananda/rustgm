# RustGM Programming Assignment 20: Trait solving

Status: **planned**. These are implementation and release requirements, not a
released grading contract. Prerequisite: PA19. The bounded source profile is
Rust 1.98.0, edition 2024, Linux x86-64, panic abort, overflow checks enabled and
debug assertions disabled. PA20 must continue to fail as an unimplemented course
stage until its complete manifest and reviewed text oracles ship.

## Required implementation

PA19 supplies trait identities, methods, dispatch and basic bounds. PA20 completes
associated items, normalized trait obligations and coherence using that machinery.
An inventory entry or a matching impl header alone never proves an operation valid.

### Associated types and their contracts

Support required associated type declarations and their implementation definitions,
qualified projections such as `<T as Trait<Args>>::Item`, and unambiguous shorthand
paths. Validate member identity, visibility, arity and complete mixed arguments;
the same member name in another trait does not supply evidence. Check missing,
extra and conflicting definitions even when no executable root uses the impl.

Retain result bounds separately from assumptions about Self and the trait's inputs.
Support the profile's Sized, Copy, Clone, user-trait and outlives requirements,
including inherited contracts and associated equalities. An impl must establish
the promised result under its actual parameter environment. A caller can use that
promise only after proving the parent trait application. A requirement on an
associated result does not imply the same requirement on its inputs.

Normalize projections through aliases, nominal fields, callable signatures,
method contracts and demanded bodies. Preserve declaration-time formation and
lifetime obligations even if the resulting type no longer mentions their inputs.
Keep retained declaration field types available for variance while layout consumes
normalized storage. Associated equality is not injective: equal output types do
not establish equality of Self or any trait argument.

The pinned stable profile does not support default associated type definitions
or inherent associated types; see the
[Reference](../doc/rust-reference/src/items/associated-items.md#associated-types).
Generic associated type families and higher-ranked trait bounds belong to PA21.

### Associated constants

Support required trait constants, default trait values, impl overrides and
inherent associated constants using the existing constant-expression language.
Resolve qualified and unambiguous shorthand uses through typed declaration and
selection facts. Default values and overrides retain distinct selected identities.
Do not implement constant lookup by rendering an associated path and reparsing it.

Validate declaration types and unused definitions; instantiate generic values in
the selected owner's complete argument environment. Share evaluation for repeated
demands of the same identity, diagnose dependency cycles and retain the existing
overflow and invalid-value rules. Preserve parameter bounds and lifetimes when a
selected value reaches an expression, initializer or supported type-level use.
General constant statements and const functions remain PA25 responsibilities.

Use selected values in constant patterns and range endpoints. Constrain their
types during body inference, then evaluate and check structural equality and
coverage after initializer typing. Values must be available before function
specialization: unused owner parameters are permitted, but parameter-dependent
initializers and opaque caller assumptions cannot supply a pattern value. Keep
the requesting body's complete bound and lifetime proof when reusing a value.

### Selection, inference and normalization

Use typed goals, explicit parameter environments and isolated candidate equations.
Caller parameters are rigid; inference variables may settle from ordinary call
arguments or result context. An unresolved candidate can remain possible during
lookup, but cannot authorize the completed body. Retrying after inference must
use the completed goal without retaining assignments from failed probes.

Check every compatible assumption. A blocked opaque equation must not hide an
independent complete match; two complete matches remain ambiguous. Unresolved
inputs cannot be chosen by ignoring an alternative. Method-name ambiguity remains
a lookup error: later call arguments or expected return types cannot choose between
different applicable methods. Selected lifetime equations remain obligations of
the caller's ordinary region proof.

Normalize complete subject and argument graphs with bounded worklists. Retain
the selected impl, member, substitutions and proof obligations for later phases.
Keep immutable environments fixed for each query; do not publish an unfinished
declaration environment to break a dependency cycle. Structural type caches must
not become caches of permission learned from a particular caller.

### Coherence and recursive obligations

Validate all impls, including unused ones, for orphan legality and overlap within
the supported crate and builtin-trait profile. Keep local trait/type identity and
covered parameter rules explicit. Compare full type and const argument equations;
lifetimes or a different associated value do not make overlapping impls disjoint.

Coherence asks whether headers can coexist, separately from whether a particular
caller proves their predicates. Missing implementations in the current registry
are not general negative evidence about other compatible implementations. Preserve
the existing special rules for builtin traits, derives and Drop conformance.

Traverse recursive obligations iteratively, with explicit pending, active and
completed states and bounded work. An inductive cycle is not a proof. A cycle
cutoff is also not a reusable negative answer for its ancestors. Cache only
completed results under the full environment, query mode and binder scope.
Reused evidence with fresh lifetime identities must be rebased before use.
Repeated subgoals should share work without sharing permissions across callers.

## Tool surface

Extend `dev/rustgm --emit-semantics -o OUTPUT SOURCE` and `--count-semantics`.
Preserve PA19's inspection boundary: pending lifetime facts are not proof that
the whole program can execute. The semantic view must expose the retained
associated declarations, result contracts, selected definitions and normalized
uses that later phases consume. Count mode must read the same typed facts.

The exact added row vocabulary, ordering, version/count changes and example
outputs are still to be implemented and frozen. Existing PA19 rows alone do not
fulfil this requirement. Keep deterministic output ordering separate from hash
lookup order, and do not serialize work counters as language decisions.

The supplemental `--emit-lowir` lane is required. It runs complete body, lifetime,
ownership, destruction and layout checks before emitting the existing LowIR
contract. Selected associated values and methods must reach executable code through
the ordinary specialization pipeline. CPPGM remains the external LowIR-to-native
backend; neither CPPGM nor rustc may implement a Rust frontend phase.

Success exits 0 with newline-terminated stdout and empty stderr. Diagnosed
rejection exits 1 with empty stdout. Signals, timeouts, panics and exit 86 are
implementation failures, never acceptable negative language outcomes. Earlier
stage views and their committed byte contracts remain available.

## Tests to develop with the reference

Develop the families listed in [tests/README.md](tests/README.md). Each family
needs positive, negative and cross-feature controls; a successful isolated type
query cannot stand in for complete source-to-native validation.

Add spec fixtures, positive/negative controls and cross-feature regressions in
the earliest owning assignment. Use the course reference for text oracles and
pinned rustc for independent language checks. From PA11, execute emitted LowIR
with the configured CPPGM backend. Do not require new future-stage behavior
implicitly. An empty suite or a not-implemented result cannot pass.

## Completion and review

Implement the responsibilities in `dev/src/` using typed phase results, safe Rust,
Clippy all/pedantic with warnings denied and the file/architecture audits. Follow
`spec.md` and the CPPGM architecture guidance: parse once, consume retained facts,
index compatible candidates and avoid speculative work on ordinary paths.

Measure environment width, candidate count, repeated subgoals, shared projection
graphs and recursive depth separately. Work counters must establish sharing and
bounded algorithms. Wall-time comparisons require immutable binaries and inputs,
A/A calibration, paired ABBA blocks, retained observations and output equality.

Before changing the assignment's status, publish its semantic and lowering
contracts, complete `tests/manifest.json`, all reference sidecars and a validation
review together. Independently qualify declared source outcomes with pinned rustc
before generating oracles. Execute every accepted native control with rustc and
CPPGM, and explain any deliberate profile differences in the manifest.

Run every active earlier suite, strict lint, file/architecture audits, harness,
upstream and backend checks, plus affected scaling measurements. Review every
generated reference diff and validate the root report's stage discovery. A planned
stage must not acquire an unregistered executable manifest or empty-suite success.
Student export is a later operation: this repository owns the full solution.

The current maintainer implementation has substantial coverage of associated types,
associated constants and the solver. Trait constants in implementation headers can
wait for compatible headers while preserving inference and lifetime evidence.
Remaining work includes early inherent-constant lookup, associated-type readiness
inside implementation headers, further normalization/coherence work, the extended
semantic view and the public grading suite. The maintainer journal in
`docs/pa18-pa21-buildout.md` records evidence and known gaps; individual checkpoints
do not reduce the requirements above.
