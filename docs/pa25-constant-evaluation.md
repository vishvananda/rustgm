# PA25 implementation handoff

PA25 remains planned. This document records integration requirements; it is not
an implemented grading contract. PA23's frozen candidate has completed release
qualification, and PA24 closures remain a prerequisite.

The existing compiler already has typed constant arenas, selected associated
initializers, structural constant patterns, array/discriminant queries, static
address identity, promotions and const-function eligibility checks. Ordinary
runtime calls to a `const fn` work. These pieces do not constitute a general
constant interpreter: required function calls, local mutation/control flow and
constant-body readiness still need implementation.

## Integration requirements

- Execute typed CFGs with an explicit frame/continuation stack, initialized local
  storage, branch/block positions and bounded work. Never evaluate recursive
  calls on the Rust host stack or reparse source expressions. Reuse the existing
  arithmetic, value and allocation semantics through a coherent typed interface.
- Keep eligibility, body/instance checking and actual execution separate. Reject
  invalid unused const bodies where the language requires it; do not execute a
  dead branch merely to discover its value. Required constant failures remain
  errors, while optional promotion failure preserves runtime behavior.
- Retain complete instance arguments, caller proof obligations and selected
  associated declarations. Cache values only with the inputs and permissions
  they require. Completed structural values cannot skip another caller's proof.
- A declaration can demand a value before the defining body, nominal layout or
  selected member is ready. Return an exact typed dependency to orchestration
  and retain the evaluator's frames and value graph. Resume once the dependency
  is proved; do not repeatedly infer a body or scan the whole crate for each wait.
- Preserve symbolic allocations, offsets, bounds, mutability, liveness and static
  identity independently of host addresses. Address formation is not a value
  read. Reject dangling references and unsupported pointer observations; do not
  manufacture portable integer addresses from host pointers.
- Reuse one report-owned definition index and cleared scratch storage across
  demands. Bound call depth, instructions, allocations and retained values;
  distinguish a true value dependency cycle from ordinary terminating recursion.
- Feed completed values to array lengths, discriminants, const arguments,
  patterns, statics, promotions and read-only LowIR data emission. Keep the
  semantic facts and checked CFGs reusable by later runtime compilation.
- Publish `--emit-constants` only with a specified deterministic typed view,
  stable ordering, diagnosed failure contract and nonempty manifest-listed text
  oracles. Do not treat a planned-stage error as a successful negative semantic
  test. Preserve earlier scalar/aggregate/pointer constant behavior.

`doc/rust-reference/src/const_eval.md` is the pinned language reference. The
phase audit currently gives `const_eval` access to support/types/body and keeps
sema independent of CFG/borrow execution. New orchestration must preserve that
direction. Shared values should have an owning typed interface rather than
creating a dependency cycle between the interpreter and semantic analysis.

## Qualification matrix

Declare positive and negative outcomes before implementation: ordinary and
recursive const calls; local bindings and mutation; branches, matches and loops;
aggregate values and nested calls; generic/associated initializers; static and
promoted references; allocation and alias identity; uninitialized, dangling,
out-of-bounds and arithmetic failures; rejected nonconst/trait/function-pointer
calls; forbidden destructor effects; and resource exhaustion. Distinguish stable
language support from later unsafe, library and async assignments.

Use pinned rustc for source outcomes and observable values, then run accepted
programs through the unchanged CPPGM backend. Measure repeated identical demands,
long dependency chains, wide unrelated declarations, call depth and symbolic
allocation size independently. Keep every frozen-binary observation and prove
shared outputs equal. Cleanup belongs in the runners from the beginning.

A remaining preparation path to include in that integration is
`lower::instances::demand`: concrete instantiation currently creates a fresh
promotion evaluator for each materialized body, and `body::constants::instance`
opens a separate required-value batch. PA23's reusable index covers early defining
proofs and the final template pass, not these later concrete batches. Preserve
the same report-owned index across those consumers when introducing the general
interpreter, while keeping instance-specific queries and permissions separate.
The PA23 work counters deliberately do not claim to measure those later passes.
