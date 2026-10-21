# PA24 inherited object inputs

This checkpoint repairs inference for an already formed closure converted to an
object whose principal inherits a callable contract. The unchanged original
`inherited.rs` independently accepts with Rust 1.98.0, while the baseline rejects
its unannotated `|x| x+n` and accepts `|x:u8| x+n`. The original object conversion
registered Sized, the outer lifetime requirement and the principal trait, but
not the inherited callable input before numeric fallback. Debugger evidence
shows parent fallback selecting i32; child input export did not default it.

The conversion now elaborates the target's inherited language Fn/FnMut/FnOnce
and Output clauses as ordinary required goals. The principal header's existing
Self parameter is instantiated with the actual closure inference term. Every
quantifier, including an empty binder, remains explicit. Principal membership,
Sized, outer outlives, ordinary ownership and executable lifetime proofs still
apply. The new clauses are never assumptions or implementation evidence and
provide no call or const-execution permission. Numeric fallback is unchanged.
The principal-missing and wrong-output negatives independently establish that
inferred callable inputs do not substitute for either required proof.

`sema::traits::object_bounds::demands::Template` retains only that declared Self
parameter and an immutable Arc of clauses. Its key contains the complete object
shape, arguments and declared quantifier count; only the outer object storage
lifetime is excluded. That lifetime remains an original conversion requirement.
The complete trait graph supplies the inherited clauses without inventing an
owner or reparsing bounds. Only a completed build in the immutable contract
epoch is cached. Unresolved/normalization-pending target arguments do not enter
this cache. Each caller instantiates its own ordinary goals; no inference terms,
selection results, witnesses or proof authority are cached. Contract invalidation
also discards these templates.

The cache is lazy. Ordinary bodies and function-pointer object conversions do
not allocate it. Graph/candidate visits are charged; a cache hit costs one unit and a miss
runs charged preparation. Caller instantiation and lifetime folding are
additionally charged to inference work.
Graph preparation and aggregate retained work have one-million-step limits.
The 64 MiB retained policy charges conservative map slots plus each shared clause
payload once before publication. As with support::retained, allocator headers,
hash control bytes and Arc control blocks are excluded: this is a deterministic
logical storage bound, not a measured heap-size claim.

## Declared outcomes and evidence

`pa24/tests/object-input-outcomes.json` declares seventeen complete programs:
eight supported positives, six independently Rust-invalid executable negatives,
and three independently Rust-valid unsupported positives. All are checked with
pinned full-object compilation, types, LowIR and constants views. The static-only
quantified-input negative is accepted by the type view and rejected by the
ordinary executable lifetime proof. Eight positive programs additionally execute
through the pinned CPPGM backend; their Cargo test is explicitly ignored in the
ordinary backend-free lane.

The supported families cover inherited Fn/FnMut/FnOnce, mixed target arguments,
deep and diamond inheritance, an unused quantified binder, and opposing written
annotations/captured values. The explicit remaining positives are quantified
reference input, quantified external-reference output (currently stopped by
undetermined capture Copy input), and target arguments unresolved when the
conversion is first reached. These are implementation boundaries, not completed
language rejection checks. Broader quantified-object proof work remains separate.

Private invariants check payload sharing across outer lifetimes, distinct unused
binders, no failed/unfinished-epoch publication, cache-hit work exhaustion, and
ordinary/nonclosure allocation controls. At widths 8/32/128, repeated targets use
37/61/157 preparation steps and one 48-byte clause payload; distinct targets use
240/960/3840 steps and 8/32/128 entries. Whole source inference/canonical/projection
work is 9494/37838/151214 for repeated targets and 9851/39419/157691 for distinct
targets. These tested families have affine work; no wall-time, RSS, general
complexity or ordinary-code throughput conclusion follows from these counters.

Evidence is retained under `obj/parallel/pa24-object-inputs/`: hypotheses and
hashes were declared before independent probes; baseline/first/second preserve
all observations; qualified contains the exact focused/native/regression/lint
and audit logs. The earlier filename-filter private invocation ran zero tests
and is not counted; the corrected module-filter invocation ran four passing
tests. Original ordering/debugger evidence is under
`obj/parallel/pa24-inherited-inference/`.

The qualified batch passes sixteen test functions: four private invariants,
four new source/native/scaling functions, one existing erased-contract function,
two existing erased-dispatch functions, and five callable-inference functions.
Strict release all-target/all-feature Clippy, file/architecture audit (1450 files,
zero findings), layout and upstream checks pass. No references were generated,
no assignment was activated, and no duplicate full release or benchmark was run.
The integrator owns combined publication qualification and immutable timing.
