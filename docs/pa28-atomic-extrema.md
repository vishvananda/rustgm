# PA28 U32 atomic extrema

CAS and weak CAS were already implemented on the accepted `3c1a9ce0` cut.
Their exact canonical Result identities, hidden-result ABI, fifteen ordering
pairs and backend-updated expected scratch are retained; the existing CAS
source/native/scaling suite passes again. Weak continues to use strong CAS.
No CAS implementation was duplicated.

The new family appends `AtomicFetchMin=15` and `AtomicFetchMax=16`, preserving
indices 0–14. The existing canonical AtomicU32 factory now prepares thirteen
atomic methods; its ordinary shared-receiver signature path supplies
`(&AtomicU32,u32,Ordering)->u32`. Both new methods are safe and RuntimeOnly.
No new body/value/provenance/Result schema or backend vocabulary is introduced.
The library identity, shared mutation policy and natural U32 layout remain exact.

Both extrema share the existing bitwise CAS retry scaffold. Each selected order
executes one Relaxed initial load, stores it in one expected scratch slot and
retries CAS with the requested success order and Relaxed failure order. Failure
reloads only updated local scratch. Success returns the captured old value, with
no extra atomic-object read or source operand evaluation. Equal/unchanged desired
values still execute CAS, preserving RMW effects and ordering.

Pure updates compare U32 operands with explicit Ugt for minimum or Ult for
maximum. LowIR comparisons return canonical I64 0/1; truncating to U32 preserves
that boolean. Wrapping U32 arithmetic then computes mask = 0-selected and
`old ^ ((old ^ operand) & mask)`. No signed predicate, widened arithmetic,
conditional early return or ordinary object-copy substitute is used.

The initial independent declaration qualified all 28 source outcomes and six
Rust native programs without hypothesis disagreements. The first frontend
implementation passed source checks but failed the high-bit native matrix:
signed Gt/Lt predicates were incorrect even with U32 operands. The failure and
artifacts are retained. Explicit unsigned predicates fix it, and typed invariants
now bind that distinction and the compare-result truncation. A test-helper borrow
error and a doc-markdown lint were also retained and corrected; neither was a
language rejection or backend failure.

The final 28 object/three-view controls cover all five orders, core/std/explicit
core alias/no-prelude identity, local shadowing, variable/aliased/reified calls,
static storage, const rejection and type/arity/generic-argument diagnostics.
Packed implicit receiver borrowing and unused const-function eligibility defer
from type inspection to checked lowering, as explicitly declared. No new
Rust-positive unsupported profile is introduced by this family.

Six ordinary rustc/CPPGM native comparisons pass: unsigned high-bit/zero/MAX and
all-order behavior, equality/ignored effects, once-only operands, reification,
static nested storage and aliased orders. These are uncontended comparisons.
Separate controlled typed-LowIR tests preserve and validate the original shim,
then inject one ordinary U32 store after each initial seed in its one-shot
preheader. They assert exactly one selected injection per operation and none
inside the retry loop. Observations below and above 0x80000000 and equality
check returned old bits and final storage. All three positive executions pass;
three deliberately stale-seed return mutants abort with the expected status134.
This is deterministic backend retry evidence, not an unmodified-source oracle or
concurrency stress claim. Original and instrumented LowIR plus exact statuses are
retained separately.

At 8/32/128 callers, body work is 267/1035/4107. The registry remains thirteen
methods and only two demanded extrema shims. Each has 62 instructions, 22 blocks,
one scratch slot and five static seed/CAS branches. The existing success codes
0/2/3/4/5 and Relaxed failure code0 remain unchanged. Source arguments are captured
outside the retry, and original-shim dataflow checks return the current scratch
observation. These are work/structure invariants, not timing or RSS benchmarks.

Strict all-target/all-feature Clippy, syntax audit (1,454 files, zero findings),
28 source checks, six ordinary native comparisons, controlled positive/negative
executions, scaling and all affected atomic/volatile/storage suites pass. Existing
bitwise source/native/dataflow/scaling pass with the shared retry helper. The
24 upstream fixtures and 126 pinned Reference files verify. All active PA1–PA23
text comparisons pass with zero differences; ordinary scalar views remain byte
identical to the earlier qualified checkpoint. No reference or assignment status
was changed. Full combined qualification/performance and release layout belong
to integration.

Source fingerprint (sorted dev/**/*.rs path, NUL, bytes, NUL):
`bc75502e461e503f25e836f21cf8b0a26a22ba27d612779c6a7f8f0741ebfcde`.
Compiler SHA256: `76690c60980bbab062674c386eb2adadd745a565425ddab13286e80d04c13e50`.
Contract SHA256: `f20c633e603b4c0fc4843b7635b43aa93bf878780defd1debf1687916e5bb867`.
Final evidence is `obj/parallel/pa28/atomic-extrema-strict2` and
`obj/parallel/pa28/atomic-extrema-finish`; exact tracked outcomes bind fixture,
source, output and original/instrumented LowIR hashes. Other atomic widths,
fetch-update/control methods and remaining memory/numeric intrinsic families
remain open; this is no PA28 completion claim.
