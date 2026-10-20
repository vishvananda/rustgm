# PA22 implementation evidence

PA22 remains **planned**. PA21 is the latest released assignment. This record
tracks the first PA22 metadata checkpoint, not completion of unsized source
semantics or publication of a new grading suite.

The subsequent [storage and reference-coercion checkpoint](pa22-storage-checkpoint.md)
records nominal sizedness, tail addressing, structural/projected unsizing and
constant metadata, together with its newer qualification and remaining work.
The [projected-proof checkpoint](pa22-proof-checkpoint.md) adds coherence,
hidden `Sized` lifetime evidence and shared completed proofs for nested tails.
The measurements and implementation status below describe the first checkpoint.

## Storage-tail metadata

`abi::metadata::Queries` classifies prepared types as thin, length-bearing or
unknown with a retained tail identity. It follows only the final struct/tuple
field. Arrays, references and pointers stop the walk. Generic substitutions
remain distinct canonical instances; a representation query grants no
well-formedness, `Sized`, lifetime or coercion proof.

The iterative query memoizes completed suffixes and failed paths. Five
maintainer invariants cover nested tails, prepared generic instances, unresolved
parameters, cycle failures and exhausted work. The scaling control traverses
an 8,192-record chain once, then checks 8,192 additional wrappers sharing that
chain. Repeated completed roots add no charged work. A failed path leaves
independent completed results usable.

Pointer layout uses this query for nontrivial unsized pointees. Existing thin
and direct slice/`str` pointer paths retain their fast classification and do
not allocate the optional query. The independent width/alignment vector now
includes references, raw pointers and arrays of pointers to nested unsized
tuples. Pinned rustc executes those representation assertions. This does not
enable stable tuple unsizing, which Rust rejects.

## Independently declared source probes

The frozen probe plan contains 22 complete programs: 14 expected rustc
acceptances and eight expected rejections. All agree with Rust 1.98.0.
The PA21 binary accepts the direct slice control and an unused function with
an unsized-tuple argument. The latter is an acceptance check, not proof that
the function body can be emitted or executed.

The remaining accepted probes identify missing struct-tail declarations and
coercions, raw-pointer coercions, packed concrete tails, transparent wrappers
and associated-type tails. Negative probes cover non-final or enum DST fields,
missing sizedness, changed prefix parameters, pointer mutability, nested
reference coercions, stable tuple unsizing and an unconstrained packed tail.
Existing frontend rejections are not counted as proofs of the intended
negative rule while an earlier feature boundary still rejects the declaration.

The first two probe versions accidentally required `assert!` or slice/`str`
library methods. Their inputs and diagnostics are retained. The third version
uses indexing and plain expressions to reach language rules without PA26
library requirements. No oracle was generated from those exploratory failures.

Local evidence roots:

- `obj/pa22-work/baseline{,-v2,-v3}/`: predeclared source plans, compiler
  outcomes and diagnostics, including superseded probes.
- `obj/pa22-work/metadata-v1/validation/`: initial audit failure, caused by a
  compressed source string in a maintainer test. The string was expanded;
  no audit exception was added.
- `obj/pa22-work/metadata-v1/validation-v2/`: qualification of the corrected
  checkpoint, including the exact Rust source manifest.

## Checkpoint qualification

Candidate SHA-256:
`1f3195bf2a58331393ae050eb67aa7862930e462cf9c2675ff9cdd534fdea7e0`.
The 41-gate batch passed:

| Check | Result |
| --- | --- |
| Release maintainer tests | 1,722 passed |
| Explicit native maintainer groups | 102 passed |
| PA1–PA21 public byte oracles | 7,040 passed, no oracle changes |
| PA12/16/18/21 independent source checks | 1,610 passed |
| PA12/16/18/21 independent native checks | 1,120 passed; 709 execution comparisons and three codegen-only comparisons |
| Strict Clippy, formatting and architecture/file audit | Passed; 1,017 audited Rust files |
| Audit tests, harness, discovery, upstream, backend and layout checks | Passed |

The independent suites preserve their declared profile differences, including
PA12's three documented acceptance differences. This checkpoint does not
reclassify those cases or claim full rustc acceptance equivalence.

## Development performance

The immutable PA21 release binary is the baseline, with SHA-256
`0f43fe91623ecc405b6f3c34e5f3a426acd7b6d7c6d30e340cbabec513d5d647`.
The candidate is the exact binary qualified above. Six frozen inputs have
independent rustc object-generation checks. Measurements retain A/A calibration,
five paired ABBA blocks with three executions per observation, eight peak-RSS
samples, byte equality and Callgrind instruction profiles. Raw plans, inputs,
observations and profiles are in
`obj/pa22-work/metadata-v1/performance/`.

| Shared LowIR input | Candidate median | Instruction change | Paired wall median |
| --- | ---: | ---: | ---: |
| Ordinary 256-function control | 14.02 ms | +0.0847% | −0.48% |
| 64 slice-taking functions | 6.49 ms | +0.0006% | −5.23% |
| 256 slice-taking functions | 15.00 ms | −0.0064% | −1.29% |

Every shared output is byte-identical. A/A wall-time spread ranges from 3.35%
to 11.16%, and paired blocks change sign; these measurements do not establish
a stable wall-time speedup. Median peak RSS rises by 92–276 KiB on the shared
inputs. GNU `size` reports 12,320 additional bytes across text, data and BSS.

New unsized-tuple layout inspections at widths 16/64/256 take
**3.21/4.46/9.35 ms**, respectively. The largest performs 35.68 million
instructions. PA21 rejects these layout inspections, so they are candidate-only
measurements, not speedup comparisons. The independent charged-work invariants
cover much deeper chains and shared tails without relying on timing.

The storage guard removed 4,765 generated files, reclaiming 7.65 GiB after
qualification, and removed the six benchmark object files after measurement.
Source evidence, diagnostics and timing observations remain. Only the current
PA21 release and PA22 development comparison binaries are pinned.

## Release work identified at the first checkpoint

Nominal sizedness must stop treating all aggregate identities as unconditionally
sized, without introducing stale canonical properties or recursive template
expansion. Declaration and caller proofs must handle substituted and normalized
tails while preserving their original requirements. Physical metadata must not
be substituted for those proofs.

Unsizing selections still need complete semantic, lifetime and mutability
checks, followed by explicit metadata construction in body lowering. Struct
prefixes, nested tail projections, alignment, raw pointers, reborrows and
coercion propagation need source and native controls. Reified functions with
unsized-tuple arguments also need code-generation checks, even where stable
safe source cannot construct the argument.

The handout owns the intended matrix and phase boundaries. PA22 must not become
active until that implementation, independently qualified tests, generated byte
oracles and complete release measurements are published together.
