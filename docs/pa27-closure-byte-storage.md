# Retained closure capture byte storage

This slice consumes the compiler's existing closure environment tuple and ABI
layout. It does not choose a new closure layout or recover callable identity from
bytes. The exact demanded Closure TypeId continues to select the original checked
callable and its invocation permissions. Product values contain only captures.

## Contract and consumers

`Reader::storage_shape` recognizes a closure as the existing Product shape only
after its exact tuple environment and completed copied ABI layout agree. It
checks the canonical capture ListId, physical variant range, size, alignment,
inhabited/reference/alignment flags and field/offset counts. These are borrowed
retained facts; selection does not rescan the capture list. The existing iterative
family query, scalar/aggregate/word codec, child projection and reference duties
then handle the captures. No Value/View/Origin/Pointer/Frame shape changes.

The encode-product check requires the exact capture count and no nominal variant.
The zero-byte path also walks structural products so nested zero-sized closures
cannot hide malformed capture products. A nonempty Repeat uses one element
recipe; an empty array visits none. Union Images remain nonstructural, and byte
transport does not manufacture initialization, provenance or write authority.

Actual captured reference values keep their original symbolic allocation and
write restriction. Decoder cache hits still recheck selected reference expiry
and validity. Source capture ownership, lifetime proofs and ordinary typed field
paths are unchanged. Opaque, function-item and other unsupported capture-child
backing families remain required work; this is not a claim that every closure can
already be transported through arbitrary constant bytes.

## Frozen outcomes and disagreements

The original explicit enum source is copied without alteration from the earlier
default-enum investigation: SHA-256
02c62180d32e7283b36cc5cddc4f9c25eca7410146540de24b05be935fb39709.
The companion PA27/PA28 `closure-byte-outcomes.json` records exact source hashes
and per-view outcomes before execution. Controls cover owned/shared/mutable
captures, mixed fields, zero-sized captures, counted copies, two distinct closure
bodies with equal captured bytes, local escape and invalid capture bytes.

Pinned Rust independently accepts and executes the valid inputs. The Non-Copy
ownership control uses a ManuallyDrop source and ptr::read, then observes exactly
one destructor at runtime. Its original const-enum-match hypotheses instead
failed with E0493: one captured only a Copy field, leaving Token to drop in const;
the other moved Token but hit Rust's const enum destruction restriction. Those
original sources, declarations and diagnostics remain separate evidence, not
positive native claims or new rejection contracts.

The initial uniform-per-view negative hypothesis was also disproved. Type
inspection may defer inline closure result materialization, while constants and
LowIR emission reject invalid bool, uninitialized and null-reference captures.
An equivalent named initializer rejects in all views; an escaping local reference
fails earlier during lifetime checking. A's owning-phase review confirmed this
inspection/checked distinction. No scheduling rule was weakened to match it.

Private tests additionally reject malformed environment layout, capture counts
and variants, nested zero-byte products, current uninitialized bytes and expired
selected references; copied raw pointer restrictions cannot be upgraded.

## Work and evidence

Capture widths 8/32/128 and reuse counts 8/32/128 vary independently, with 0/1,024
unrelated canonical types. A warm selected capture write/read pair costs exactly
36 logical steps at every width; totals are 288/1,152/4,608. Cold conversion and
first selection cost 140/477/1,827 steps, with retained storage 19/22/35 slots.
Neither unrelated types nor repeated selected access changes retained storage.
The zero-stride Repeat control includes extent 2^30 and performs the same recipe
work as extents 8/32/128. These are logical-work bounds, not wall-time/RSS claims.

The final focused batch passed 17 independently checked source cases across all
three views, 12 literal-source native positives, the six capture-memory private
tests and three generated scaling sources at widths 8/32/128 (independent Rust,
all three candidate views and native execution). The earlier strict batch passed
all-target/all-feature release Clippy, the 1,826-file architecture audit with zero
findings, the exact Reader environment test and 33 enum private regressions.
Memory-transfer, tagless-enum, ordinary Rust-enum and Option regressions passed.
The final text pass compared all 8,362 PA1–24 cases exactly; 98 harness tests,
assignment layout and all 24 upstream/126 Reference pins passed. No full release
or performance-neutrality claim is made for this isolated worker checkpoint.

Exact commands, independently declared inputs, failed hypotheses and immutable
source/tool/output receipts live under
`obj/parallel/pa27/closure-byte-backing/`. The successful closing receipt is
`closing-first/qualification.json`; `handoff.json` records final source/binary
hashes and the retained evidence index. Scoped terminal cleanup is recorded in
`obj/cleanup/1792702150841312789.jsonl`. The integrator owns combined
release/performance and generated reference publication.
