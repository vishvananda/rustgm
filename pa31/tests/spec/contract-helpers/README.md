# Actual contract helpers and capture marker proofs

This PA31 source-declaration packet uses authored reducers following pinned
Rust1.98 core/contracts.rs and its callable traits. PA25 owns const execution,
PA24 owns captures, PA19/20 own trait proof, PA26 owns call hooks, and PA29
retains imported body/value authority. PA31 remains planned as a whole assignment.

The six contract files declare source hashes and required independent outcomes
before generation. Their union has38 distinct programs. The original three
contract language roles target free function definitions and have no fixed
generic arity. Their tags grant no constness, safety, implementation or body.
Actual generic bodies, function items, function pointers and copied captures
use ordinary call/trait/const machinery. Provided/inherent/foreign targets,
unsafe calls, nonconst constant calls, missing Copy and escaping captures reject.

Original staged API/const annotations are preserved in the header cases.
Pinned Rust accepts these same-crate const calls without requesting contracts;
that corrected initial prediction is disclosed in supplemental-contract.json.
The control contract also records the no_core Destruct prerequisite exposed by
initial const controls; ordinary functions isolate Copy from const-drop policy.
Failed original inputs/diagnostics are retained locally. No required native
output is weakened.

Seven native observers require7. A typed-wildcard reborrow also executes a real
foreign callback and requires7 9. This distinguishes genuine adjusted capture
use from an unadjusted non-reading wildcard. Captured bools and source Copy
nominals pass; non-Copy captures, mutable references and lifetime escape reject.
Generic and associated-type captures retain their actual caller assumptions;
missing generic Copy/static bounds reject independently.

These declarations do not implement the two rustc_intrinsic check functions,
contract attribute transformations, cross-crate const API transport, executable
Clone shims or original core/alloc/std artifacts. Those remain explicit work.
See [the checkpoint](../../../../docs/pa31-contract-builders-checkpoint.md).
