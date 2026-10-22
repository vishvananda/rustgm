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
`obj/parallel/cleanup-components-v1/`. The nine-input development comparison is
complete in the main checkout's `obj/parallel/cleanup-components-performance-v1/`.
Its relocation receipt binds those same inputs and immutable A/B binaries. All
outputs matched exactly. The exclusive measurement window retained six A/A
observations, five ABBA blocks with three executions per observation, eight RSS
observations and both Callgrind profiles for every input.

At width 128, the two shared-requirement families fell from 44,085,103 to
12,028,540 instructions (72.7%) and from 49,991,646 to 18,001,856 (64.0%). Their
paired median wall times fell 44.2% and 40.0%; A/A spreads were 22.4% and 28.7%,
so the instruction counts provide the clearer evidence. Median RSS fell 270 and
90 KiB. The three ordinary controls stayed within 0.15% in instructions and
within 96 KiB in median RSS. No declared attribution threshold was triggered.
These are measurements of the selected inputs, not a whole-compiler speedup.

The combined checkpoint subsequently passed 2,588 regular tests, 267 native
tests and all 8,360 active text cases. Its evidence is in
`obj/parallel/checked-memory-callers-checkpoint-v2/`, retaining the unchanged
integration groups from v1 after correcting one exact layout assertion.
Affected independent qualification and the broader combined performance matrix
remain pending before publication. PA25 remains planned.
