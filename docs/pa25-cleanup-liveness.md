# Shared cleanup liveness requirements

The previously attributed quadratic path enumerated every live value at every
cleanup point, queried its destructor components and emitted duplicate region
or type-parameter seeds. Repeated values of one generic type therefore repeated
the same requirement throughout the body. See `pa25-associated-call-checkpoint.md`
for the retained original profiles and 8/32/128 source series.

Recording now uses immutable shared component sets and live multiplicities.
The fixed-point solver still tracks every value and cleanup path independently.
Its transfers use the same storage endpoints, whole-value definitions, moves,
branches, imaginary edges and ordered destructor effects. A separate recording
state observes those entity transitions and updates component counts. Each point
emits each currently required lifetime or opaque parameter once. A newly created
borrow temporarily contributes its result at that point; this does not extend
the result across earlier statements.

Ordinary type requirements, coerced operand requirements and destructor
requirements have distinct cache keys. Their immutable sets may share storage
when their contents coincide; their entity identities and kills never merge.
Each block and cleanup edge resets counts from the original solved entity state.
The existing region inference, permission checks and complete loan proof still
consume the resulting seeds. No new outlives assumption is introduced.

The recording cost follows component-set construction, memberships affected by
actual entity transitions, boundary-state initialization and distinct emitted
requirements. It no longer scales with unchanged live entities times points.
Programs with many distinct live parameters can still have genuinely quadratic
output; the implementation preserves that output. Construction/state storage has
a 64 MB logical budget, seed output retains the two-million-entry limit, and
construction, transitions and output remain charged to the liveness work budget.
CFG fixed-point complexity is unchanged.

Development evidence lives in `obj/parallel/cleanup-components-v3/` in the
`rustgm-pa29-work` checkout. Strict all-target/all-feature release Clippy and the
architecture/file audit pass. Three private invariants compare transitions,
independent successor resets and borrow births with a simple entity-expansion
model, bound shared-requirement work through 4,096 entities, and preserve the
distinct-parameter output relation. The value-liveness, template-cleanup,
storage-liveness and template-loan groups pass, including an independently
Rust-qualified 8/32/128 generic cleanup scaling control. Earlier lint findings
and their receipts are retained in the v1/v2 folders.

The frozen original performance inputs and declarations are retained in
`obj/parallel/cleanup-components-v1/`. Comparative instruction, wall-time and
memory measurements, combined release/native qualification and active text
comparison remain pending. These development invariants alone establish no
whole-compiler speedup or memory-neutrality claim. PA25 remains planned.
