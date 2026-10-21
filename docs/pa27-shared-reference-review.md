# Shared references reconstructed through symbolic raw pointers

This bounded PA27 slice implements CTFE `&*raw` for exact-type shared references
to live, fully initialized ordinary symbolic storage. It introduces no public
Value/View/Origin or CFG enum. The private machine helper requires a retained raw
access anchor, validates canonical type, provenance, lifetime, bounds and the
actual leaf reference alignment, then returns the same pointer monotonically
restricted to shared-derived authority. A final `*mut` type grants no permission;
the original writable pointer and other aliases remain unchanged.

The helper uses existing checked memory reads as an initialization proof for
this supported profile. That internal proof is not a language rule that forming
a reference reads its contents. Immutable static contents use the existing
provider readiness/retry path. Reference formation to mutable/foreign statics,
cyclic static reference graphs, mutable raw reborrows, union-field references,
general reinterpretation and numeric nonnull ZST pointers remain explicit
accepted-Rust boundaries. Future deferred reference validity must close them
without treating legal address formation as an illegal contents read.

A packed raw base can permit a by-value field read while its leaf cannot support
a reference. Leaf alignment uses the allocation's actual retained physical
layout and offset, independently of the raw-base access anchor. Source packed
reference rules remain separate: the physically aligned packed-source control
succeeds during type inspection but rejects executable/constants checking for
its packed-field reference. A separately formed physically aligned field pointer
can reconstruct a reference. No unsafe, source initialization or final reference
escape obligation is bypassed.

Twenty-six independent full-object outcomes distinguish eleven supported
acceptances, eight real rejections, and seven accepted-Rust capability boundaries.
Every public frontend view is checked, with the explicit packed-source phase
distinction above. Two additional cases were independently declared after the
initial 24-case investigation; all original source bytes and observations remain.
Source E0381 rejection is separate from the private empty-cell initialization
invariant. Null, expired and escaping storage, unaligned leaves and shared-derived
writes retain their respective rejection reasons. Five ignored-by-default native
groups compare local products, calls, alias rights, packed/ZST cases and emitted
static reference values against pinned rustc.

The final private selector runs two tests, including a valid packed-base read
paired with rejection of its leaf reference, missing-anchor rejection,
uninitialized/expired cells, and per-pointer permission retention. The first
selector accidentally matched zero tests and is explicitly unqualified.
An initial test also assumed fixed retained slots; the ordinary reference loop
already grows linearly. Its failed observations are retained. At N=8/32/128,
ordinary checking uses 501/1749/6741 execution steps and 46/70/166 retained slots;
raw reconstruction uses 626/2210/8546 steps and 50/74/170 slots. The added storage
overhead is four slots at each size. Tests enforce bounded linear growth and
constant raw-versus-ordinary slot overhead, without a wall-time neutrality claim.

Strict all-target/all-feature Clippy and the 1,364-file audit pass. Seven affected
regression tests and all three new source/native/scaling tests pass. The old
raw-borrow fixture is supported without changing its bytes; the union-reference
fixture keeps its unsupported status with a more precise Image-reference reason.
The private visibility compile failure, zero-test selector, fixed-slot hypothesis
and initial diagnostic expectation are retained in the checkpoint receipt.
No active references, activation, backend changes or full-release claim occur.

At this checkpoint, PA27 requirements still needing closure included mutable/raw union-field
reference creation, the other reference profiles listed above, general raw
byte/type reinterpretation and pointer offsets, shared-derived interior writes,
pointer/reference/function/aggregate union Images, the default runtime
`invalid_reference_casting` lint, and complete packed/pattern/release qualification.
PA28 now covers scalar/thin volatile operations and U32 construction, load/store,
swap, add/sub, bitwise RMWs and strong/weak CAS. Memory/numeric intrinsic families,
aggregate/ZST/wide volatile, other atomic widths, fences/accessors/min/max/update
and concurrent release qualification remain. These partial checkpoints do not
complete either assignment.

The later [mutable reference review](pa27-mutable-reference-review.md) supersedes
the ordinary writable-local mutable reborrow boundary while retaining its other
explicit profiles. Historical outcomes above are unchanged.
