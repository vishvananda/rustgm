# Nominal tail reference casts — PA22 and PA23

Original core's `as_request` explicitly casts a mutable reference to a nominal
wrapper into a reference to the same wrapper with a dynamic tail. A slice-tail
reduction reproduces the scalar-operation rejection without objects or source
compiler roles. Its implicit-coercion twin passes. PA22 therefore owns the
ordinary cast correction; PA23 owns object counterparts and PA31 owns the actual
source Unsize protocol continuation.

## Typed change

Compiler `ba4cc6555` routes same-owner nominal reference cast operands through the
existing coercion selection, as direct array-to-slice casts already do. Final
validation requires the exact selected reference target, an allowed adjustment,
no mutability strengthening, distinct nominal referents with the same owner and
the retained terminal unsizing conversion at the operand. That final lookup is
constant time. No new eligibility walk or proof is reconstructed from layout.

The existing typed selection owns element/prefix compatibility, actual tail
normalization, source-place reborrowing, lifetimes and loans. Lowering consumes
its original slice/object conversion after evaluating the operand once. No hot
record, global scan, cache, compiler limit, LowIR vocabulary or backend change.

## Owning reductions and independent review

[PA22's packet](../pa22/tests/spec/casts/nominal-reference/README.md) declares36
cases:24 executable positives and12 negatives. They cover real prefix/tail reads,
mutable writes, nested wrappers, aliases, generics, associated projection markers,
covariance, reborrows, constants, padding, empty/ZST/non-Copy tails, raw/implicit
controls and single evaluation. Negative controls preserve element/prefix/owner
identity, nested-reference restrictions, mutable invariance, lifetimes, active
loans and packed-field alignment.

[PA23's packet](../pa23/tests/spec/nominal-reference-casts/README.md) adds three
executable positives and two negatives: shared/mutable virtual dispatch, borrowed
object extents, missing principal evidence and lifetime escape. Every input is in
its owning assignment manifest with its predeclared outcome and native policy.

B's `obj/parallel/dst-casts/packet-plan.json` predates the implementation. Pinned
Rust1.98 object-checks all41 final inputs; all27 positive programs execute with
assertions and match the candidate's native observations. `independent-final.json`
binds the current source hashes. An initial empty-array spelling was refined to
an explicitly typed zero-length nominal value to avoid an unrelated array cast;
both versions and independent results remain retained.

The14 rejection diagnostics were reviewed before generating references. They
report incompatible types, mutability, lifetime, loan, packed alignment or missing
trait evidence. None accepts a budget, normalization or not-implemented fence.
Native positives observe actual conversion effects rather than empty mains.

## Focused qualification

All evidence is under B's `obj/parallel/dst-casts/`:

- `public2.log`, receipt1794352619515397280,50.84s: five exact tests pass, including
  all41 complete LowIR/status outcomes, eight-counter width8/32/128 growth with
  unrelated-type invariance,28 old array-cast outcomes, explicit pointer casts and
  independent pointer region/ownership proofs. Strict selected production/CLI/test
  Clippy, full formatting and architecture checks pass. The earlier public1 run
  passed static checks but executed no tests because its exact selector used the
  file name instead of the existing module name; that failure remains retained.
- `native.log`, receipt1794352683577490572,66.11s:27 new programs and18 old array
  conversion programs execute through the unchanged CPPGM backend. They check
  selected lengths, prefix/payload data, writes, reborrows, dispatch and effects.
- `text/` generates only the36 new PA22 triples and passes all392 PA22 texts.
  A stale expected PA23 total then stops the script before that stage. `text2/`
  generates only the five PA23 triples and passes all420 PA23 texts. All2313
  pre-existing reference files retain their original hashes. There is no broad
  blessing or historical oracle change;812 current text outcomes pass.
- `harness.log`, receipt1794353223207592137:315 harness checks,468 roots in13
  binaries,44 assignment layouts,128 pinned upstream fixtures and126 pinned
  Reference files pass.

The exact selected-terminal private guard is added but its cold library harness
is not rebuilt locally: the known link exceeds the five-minute budget. Full
successor CI must execute it, the existing private guards and broader release,
native, debug and independent checks. This checkpoint is focused development
evidence; the earlier assignment release records retain their original scope.

## Immutable comparison and original core

Production CLI SHA-256
`2cab63208593afcab05565227edfa4db3906e1754e58ba4c5fd5b3ad0778f2ef`
binds committed compiler `ba4cc655567ffa4e91a215cceae525a69511ded9`, an empty source
patch and3889 Rust source hashes. Baseline is the preceding `e06cd0d9` CLI.
`measure3/` retains frozen inputs, A/A calibration and two paired ABBA cycles:
40 observations preserve complete ordinary import/array-cast LowIR, a source
known-representation output and74 existing source outcomes. Candidate/baseline
ratios are1.034/1.022/1.078/1.052; A/A ratios0.777/1.032/0.887/1.025 are noisy.
These observations establish output equality, with no speed or neutrality claim.
Two earlier measurement setups stopped before observations: missing executable
main, then incomplete library identity. Their plans/failures remain retained;
final controls use existing independently qualified programs and CLI requests.

`probe-current/` passes all seven earliest-owner controls and the complete ordinary
request-shaped source. Its actual source-role counterpart reaches the explicit
`source Unsize nominal-tail goals are not implemented` boundary.

The unchanged120s original observer also finds that boundary first at
`as_request`, error.rs file78 byte34589. It still FAILS its deadline at121.2293s,
retaining249 failure records and5638 item attempts through function10921,
file288 byte2132 node112 (stdarch x86/sse4a.rs). All722 original hashes, six code
bindings, profile and work/storage limits remain fixed. Partial progress SHA-256:
`6b17984da8f3ecd7b126d5a924acad1bf91677738c48d7f5e9c6323b0176a4bb`.
The earlier scalar-cast error is absent. Four reached records have the source
nominal-tail fence. Different prefixes are not timing or completion evidence;
there is no complete census or original core/alloc/std artifact.

Full42918510671 on216364b72 remains active for the preceding projection-region
checkpoint, with release/course and native maintainer stages passed. Wait for
that full run to terminate before dispatching this checkpoint's successor.
