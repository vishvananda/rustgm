# Iterator public-signature storage readiness

The shared-slice iterator prototype exposed an existing selected-method storage
readiness gap on combined base `b870a3ee`. Its generic `next` returns
`Option<&'a T>` after matching a stored slice into its first element and rest.
The unchanged source independently compiles and executes with Rust 1.98.0.
The baseline accepts its semantic view, then rejects LowIR construction with
`generic aggregate requires instantiated fields`.

This is not specific to `for`: the direct qualified `Iterator::next` control
fails identically, while equivalent inherent `next` and ordinary generic
function controls already lower successfully. The original sources, hashes,
independent native results and baseline observations are retained under
`obj/parallel/pa26/collection-iteration/{declared-v2,direct-controls-v1}`.

## Owning correction

The retained debugger trace follows the failing exact aggregate through
`Context::demand` and `Context::signature` to the selected function owner's
public result. In that trace, the public result is canonical Option type 129,
while the already prepared opened body/call result is type 107. Both have the
same canonical Option declaration; their argument lists differ. The public
callable retains its binder, while the body's proof interface uses its own
opened lifetime identities. Preparing one canonical type cannot establish the
other's physical field readiness.

At the existing materialization boundary, after substitution and required
projection normalization, the compiler now prepares the exact public function
parameter and result roots through the existing typed aggregate cache. It does
not resubstitute, rewrite or equate those roots. The complete selected Instance,
callable binder, source template and original/specialized proof requirements
are unchanged. Immutable lowering gains no mutation or semantic query path.
No registry, IR record, enum, field layout or new persistent cache is added.
The added work visits the selected signature's parameter list once and reuses
charged aggregate preparation/cache hits; it never scans other declarations.

The private invariant keeps distinct body/public region-bearing aggregate IDs,
checks that preparing the former leaves the latter unprepared, and checks
successful exact-root preparation without signature rewriting. Repeated
preparation adds no aggregate instantiation steps. A malformed owner signature
still fails. Existing instance-owner/original-proof and shared-cleanup controls
remain part of the focused closing selection.

## Declared controls and retained boundaries

Ten new draft controls live in `tests/spec/iteration/generic-storage` and
`tests/iteration-outcomes.json`, with explicit LowIR modes, source hashes and
native metadata. Eight positives cover shared splitting, no-Copy elements,
empty/ZST input, generic array unsizing, qualified and reified trait methods,
and the existing inherent/function routes. Two negatives retain real result
escape and pointee-loan failures through ordinary LowIR verification.

The first probe's final assertion incorrectly required borrow rejection in the
semantic-only `--emit-types` view. Its full observations and failed assertion
remain in `correction-probe-v2`; `stage-review.json` records the correction:
all semantic views accept, and the two negative LowIR views reject with live
reference-loan conflicts. No compiler behavior or fixture was changed to mask
that probe error.

The direct `s.next()` version is independently Rust-positive but still rejects
with `no applicable method for this receiver` on the baseline. It is retained
as a separate canonical iteration method-visibility correction, not counted as
passing coverage here. Shared/mutable/owned collection adapters, ranges and the
other PA26 hooks remain required after this bounded correction.

## Closing evidence

`close-correction-v1` is terminal zero: all 58 declared iterator source outcomes,
eight generic-storage native pairs, the binder/readiness invariant, original
instance-owner proof rejection, and existing shared-cleanup scaling pass.
Strict release Clippy across all targets/features, architecture audit (1,715
files, zero findings), layout, discovery and upstream checks pass. The two
ordinary inherent/function controls retain exact baseline/candidate LowIR,
stderr and status bytes. No generated references were changed. The immutable
candidate SHA-256 is
`e88d1b58fd484feeb96f3950345c7d6d583692ff1de02f26607cad85a07da989`;
the matching b870 baseline is
`b27d846fbb0f4b97304bd13ec6f8bc8c081ea67ecbbdefb81b9697ad0bc1a4d5`.
The combined release and active-text integration remain the integrator's gate.
