# Ready call inputs before projected joins — PA20

Compiler c542453da advances original integer_decode past its mantissa conditional
at byte6203. Immutable dev-fast CLI
ae2e09ae4f02c69fc8f469bca88f1af94d849ddb20d330d288072e1424320f0a binds that
commit, an empty patch and3884 compiler/CLI/test Rust hashes. No original core,
alloc or std artifact exists; the120s original observation still times out.

An ordinary user Shift<R> method reproduces the problem. Float::Int promises
Shift<u32,Output=Self::Int>, but an inferred numeric call argument can leave the
selected output pending when a common-type join asks for its head. Typed RHS,
direct-parameter and no-join controls already pass. No source attribute, language
operator, GAT or closure is needed by the reduction, so PA20 owns the contract.

The existing body trait solver now supports a non-finishing poll before joins.
It advances ready call equations and projection work under the same immutable
caller, keeping its cursor, notifications, evidence, reservations and suspended
queries. It does not apply numeric fallback; completion still owns fallback and
the complete proof gate. One pending box is moved between the active poll and
body storage, preserving its allocation across joins. Declaration waits retain
the original Work::Exit before consuming a frame. Only a finishing solver wait
resumes finalization directly; intermediate waits preserve the remaining syntax
work. Work stays24 bytes, CallTypes728; no dense field, cache, quota or backend
contract changed.

The ordinary reference, pointer and function conversion paths still run after
input progress. A checked body's completed opaque root that lacks a demanded
equality receives a body equality diagnostic. Raw operations retain normalization
fences. General inference and coherence retain their conservative opaque-versus-
concrete result; strengthened controls check both raw and formed inference roots.
The body diagnostic supplies no equations, permission or reusable negative proof.

## Frozen reductions and independent checks

[The assignment packet](../pa20/tests/spec/pending-joins/README.md) declares29
outcomes before implementation/oracle generation:23 accept and6 reject. Every
source is registered in PA20. Rust1.98 matches every object outcome and executes
all23 positives, checking selected values and both conditional arms where present.
If, match, arrays, loop breaks, never, references, pointers, function pointers,
exact RHS/result contracts, missing bounds, mutability and lifetime escape are
covered. Six width8/32/128+128-noise cases keep their RHS inferred. Inference,
canonical work, actual trait selections, method steps and probes fit4x+1024 per
fourfold site count; unrelated types add none of those counters.

The initial pointer control used unsuffixed i32 locals for u8-pointer casts.
Independent Rust correctly rejected that setup (E0606). Its original source and
prediction are retained under obj/parallel/opaque-branches/pointer-setup-revision;
explicit u8 annotations repair it before any compiler implementation or oracle.
Unchanged completed independent observations were retained; changed/remaining
inputs were independently checked in the second batch.

## Qualification and reference review

Evidence is retained in B's obj/parallel/opaque-branches/:

- public4.log /1794335999956634852:7 exact production tests in109.55s, with
  strict production/CLI/selected Clippy, format and architecture. All29 complete
  LowIR outcomes, preceding26 receiver outcomes, raw property/formation controls,
  13 source lookup analysis boundaries and the earlier closure parameter/cast
  regressions pass. Scaling-final1794336419236152840 additionally checks the
  actual trait-selection counter (48.16s).
- native2.log /1794336221331675381:four groups,51 programs,88.97s. They are23
  new joins,6 canonical projections,15 parameter callbacks and7 deferred casts.
  The earlier57 count included six cast-scaling fixtures the native group does
  not execute; the authoritative count is51. All real conversions and values
  pass. The backend contract and bridge are unchanged.
- negative-controls-plan.json declares19 old rejection outcomes before any
  existing oracle update. Independent Rust confirms all19. Inventory48 checks
  every new case and those old controls. Source, manifest outcomes, and exit
  sidecars remain unchanged for all old cases.
- reviewed-controls.json identifies exactly seven changed negative diagnostics:
  four PA20 non-injective/wrong-member/trait-argument equalities, two PA21
  unrelated-caller/GAT-argument equalities, and one PA19 function used as a type
  prefix. Rust gives E0308 for the six equality cases and E0575 for the suffix.
  The other12 old raw-normalization controls remain byte-identical. This is a
  body diagnostic contract change; no failing outcome was changed to acceptance.
- final/steps.json generates only29 new triples and those seven reviewed old
  triples through exact course selections. All PA191199, PA201564, PA21867 and
  PA24527 text checks pass:4157 total. Existing positive references are unchanged.
  Harness315/468 roots/13 binaries/44 stages,128 upstream fixtures and126
  Reference files pass. PA20 now has1114 accepts/450 rejects,614 semantic/950
  LowIR cases and589 declared native executions.
- measure/passed.json retains30 A/A+twoABBA observations with complete output
  equality for ordinary typed/direct LowIR and74 frozen survey outcomes.
  Candidate/base ratios are0.989/1.266/1.015; A/A0.785/0.976/0.997. The tiny direct
  control is26.6% slower in this unoptimized sample. Bounded work passes; optimized
  and broad performance qualification remain pending. No speed improvement is
  claimed. Frozen inputs, binary hashes and every observation remain available.

New provenance batches are PA20's1794336769894894997-422fac766b88 (new) and
1794336772290826809-9cd44a8189a0 (reviewed), PA19's
1794336770959141544-e16164223fb5 and PA21's1794336773209085188-4722019e1f05.
Public1 failed a function-length lint; public2 exposed the two still-fenced
negative equalities. Their failures remain. The temporary general-inference
mismatch idea was reverted before compilation; final inference/coherence policy
is unchanged. Reuse of the pending box and the stronger counter were separately
validated after the initial passing checkpoint.

Full predecessor42867198200 on4446962e1 passed3063 private cases, every regular
integration group and all other active course cases, but FAILED one PA19 stderr
oracle at qualification-further-suffix. The intermediate f is an associated
function, not a type; independent E0575 supports the reviewed diagnostic above.
This resolves the earlier private cold-link uncertainty but does not qualify the
skipped native/debug/independent stages. A full semantic successor is due after
publication, now that this predecessor is terminal. One worker continues in the
same tree with fixed resource limits; optimized nested-proof checks stay pending.

## Original-core observation and next reduction

original/result.json is an infrastructure deadline FAILURE at121.18s under the
same120s window. Its partial stream retains262 failures/4434 item attempts through
function9025, file241 byte6527 node182 in core/src/time.rs. Partial SHA
7fcb8110537b458fcfcbdc0ecba116c90e14eef8edca6af0452133f38cd8d9b9.
All722 source hashes, six configuration bindings, semantic profile and quotas
are unchanged. Different incomplete prefixes are not count or speed comparisons.

The old integer_decode conditional fence is absent. First failure is now fmt,
num/error.rs file39 byte762: a string literal's fmt(f) inside a module-qualified
Display impl reports no applicable method. This is not a checked-body count,
complete census or original artifact.

The next seven exploratory controls freeze current-impl trait scope. Rust accepts
primitive and str receivers inside a module-qualified trait impl; current CLI
rejects both. Explicit imports and qualification pass under both. Outside and
different-impl visibility negatives reject under both. A second eligible trait
is ambiguous in Rust but currently accepted by RustGM. These are ordinary PA19
features. Next freeze their owning PA19 packet and preserve ambiguity/lexical
boundaries before implementation; no next scope fix has begun.
