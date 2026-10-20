# PA22 storage and reference coercions

The [PA22 release](pa22-release.md) supersedes the implementation-status notes
below. This document preserves its checkpoint evidence.

PA22 remains **planned**; PA21 is the latest released assignment. This
maintainer checkpoint extends the earlier metadata query with nominal sizedness,
unsized field addressing, reference coercions and constant pointer metadata.
It does not publish a PA22 grading suite.

## Typed boundaries

Nominal sizedness follows completed declaration schemas instead of the
properties of a reserved nominal identity. Struct prefixes, enum fields and
array elements require sized storage. Final fields may carry a `?Sized`
parameter, a nested storage tail or an associated projection. Caller
assumptions, normalization evidence and physical storage remain separate.
Representation validation includes packed-tail drop requirements.

Shape queries retain one frontier across hidden size conditions. Failed
queries release their pending work while preserving completed independent
results. An invalid resume request preserves the original suspension. The
full maintainer suite exposed the missing error cleanup; its existing foreign
parameter regression and an additional suspension test now cover recovery.

Unsizing retains an explicit source-side reference proof and the terminal
array identity. Lowering consumes that selection to construct metadata.
Mutable conversions reborrow the original sized source place; projections
forward the existing metadata and reuse prefix offsets and tail alignment.
No complete fixed size is invented for an unsized object.

Projected tails can change several eligible type or const arguments. Those
marker arguments need not themselves unsize: the independently normalized
last fields must support the conversion. This matches the detailed
[Rust 1.98 rule](https://github.com/rust-lang/rust/blob/1.98.0/library/core/src/marker.rs#L202-L208).
The selected physical field supplies the array length, which can differ from
a marker array's length. Field equality remains invariant even for a shared
outer reference; the source controls caught and corrected a false acceptance
of shortened projected element lifetimes.

The selector follows one physical tail instead of expanding marker and field
obligations as two trees. Declaration eligibility is cached. A forced shape
mismatch is checked once at the root, keeping ordinary lifetime conversions
out of speculative field normalization. Charged-work tests cover deep
declarations and nested projected tails.

Borrowed constructors retain the destination element context while inferring
the source array length. This supports empty literals and function-item
coercions inside arrays. Contextual `if` and `match` results choose the declared
DST destination before joining differently sized operands.

Constant pointer values retain a selected length separately from their backing
location. The backing keeps its original sized type, allocation identity and
physical subobject offsets. Nested tail borrows forward metadata. AST constant
evaluation and body promotion consume the same selection; generic promotion
can retain a deferred length until instantiation.

## Development evidence

The focused storage batch is recorded in
`obj/pa22-work/sizedness-v1/development-v36/`.
It passed the sizing/coercion tests, constant locations and storage, constant
patterns, static addresses, promotion and deferred promotion tests, and strict
release Clippy. The tail source matrix contains 30 independently checked
programs. The native matrix contains 43 rustc/CPPGM execution comparisons:
18 structural, 11 projected and 14 constant/promotion programs.

The first complete checkpoint run exposed the existing shape-query recovery
regression. After the fix, all 455 library tests and six bound-environment
tests passed. Complete requalification is recorded separately in
`obj/pa22-work/storage-v1/validation-v2/`; its final receipt, not these focused
results, determines publication readiness.

The only changed earlier byte oracle is PA12's `unsized-field.rs` diagnostic.
Its struct declaration is now legal, while its local binding still rejects.
Pinned rustc independently rejects the program. The harness regenerated the
diagnostic and compiler provenance; no source or expected exit status changed.

## Qualification and performance

The 49-gate checkpoint qualification passed for candidate SHA-256
`1b7571e7ecb099421bdd682d6fa501e12fbc44d2b0242338e2376431c3364be3`.

| Check | Result |
| --- | --- |
| Release maintainer tests | 1,756 passed |
| Explicit native maintainer groups | 107 passed |
| PA1–PA21 public byte oracles | 7,040 passed, one reviewed diagnostic update |
| PA12/13/16/17/18/19/20/21 independent source checks | 5,087 passed |
| Same assignments' independent native checks | 3,346 passed; 1,978 execution comparisons and three codegen-only comparisons |
| Formatting, strict release Clippy and architecture/file audit | Passed; 1,042 audited Rust files |
| Audit tests, harness, discovery, upstream, backend and layout checks | Passed |

Independent suites retain their existing documented acceptance and runtime
differences. Their successful contracts do not establish universal rustc parity.
The exact source snapshot and logs are in
`obj/pa22-work/storage-v1/validation-v2/`.

The immutable comparison uses the published metadata checkpoint as baseline:
`1f3195bf2a58331393ae050eb67aa7862930e462cf9c2675ff9cdd534fdea7e0`.
All 16 frozen inputs pass independent rustc object-generation checks. The ten
shared inputs use A/A calibration and five paired ABBA blocks, with three
executions per observation. Eight RSS samples and Callgrind profiles supplement
wall time. Plans, all observations and profiles remain in
`obj/pa22-work/storage-v1/performance-v1/`.

| Shared input | Candidate median | Instruction change | Paired wall median |
| --- | ---: | ---: | ---: |
| Ordinary 256 functions | 15.76 ms | +0.045% | −1.34% |
| 256 family arguments | 5.94 ms | +0.452% | −2.67% |
| 256 callable roots | 35.36 ms | +0.100% | +0.79% |
| 256 repeated opaque results | 6.20 ms | +0.222% | −1.06% |
| 256 shared inputs | 24.99 ms | +0.160% | −2.66% |
| Binder depth 16 | 3.27 ms | −0.033% | −4.58% |
| 256 ordinary equalities | 39.06 ms | +0.041% | +2.96% |
| 256 distinct captures | 81.27 ms | +0.053% | +0.22% |
| 256 slice-taking functions | 17.01 ms | +0.520% | −1.52% |
| 256 unsized-tuple layouts | 9.51 ms | +0.285% | −2.35% |

All shared outputs are byte-identical. Shared A/A spread is 1.59–8.15%; several
paired blocks change sign, so the wall results do not establish a general
speedup. Shared median RSS changes range from −132 to +518 KiB. GNU `size`
reports 139,252 additional bytes across text, data and BSS.

The baseline rejects the six new tail-coercion inputs. Candidate-only structural
chains at depths 8/32/64 take 3.09/3.64/4.40 ms; projected chains at depths
4/16/32 take 5.18/24.11/82.63 ms. The largest structural chain performs
10.06 million instructions, versus 436.68 million for the largest projected
chain. These measurements establish neither a baseline speedup nor linear
whole-frontend scaling. The projected profile spends roughly 95% of instructions
inside trait selection, including repeated nominal formation and size proofs.
The selector's charged single-tail walk is bounded, but the enclosing proof
pipeline still needs an algorithmic improvement before PA22 release.

The storage guard removed 7,223 generated files (7.85 GiB) after qualification,
then removed all 16 rustc benchmark object files. Inputs, diagnostics, hashes
and raw observations remain.

## Remaining PA22 release work

Raw-pointer construction, coercions and metadata observations still need their
source and execution matrix. Raw dereference remains PA27's responsibility.
Closed projected tails also need a stronger coherence proof: the retained
`coherence-v1` probe shows a false rejection of an unsized projected wrapper
beside a blanket `Sized` implementation. Sized and open-projection overlap
controls reject in both compilers.

Review metadata-only preparation of projected pointees, the remaining
provisional-proof consumers and repeated projected-tail proof work identified
above. Then publish the PA22 manifest, declared outcomes, generated text oracles,
complete handout and assignment release checks together. No PA23 implementation
is included in this checkpoint.
