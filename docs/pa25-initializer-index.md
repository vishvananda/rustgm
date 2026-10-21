# Shared initializer lookup checkpoint

Declarations owns one append-time DefId-to-initializer index. Its optional boxed
table is allocated only at the first initializer; each slot is a four-byte
`Option<NonZeroU32>` containing the validated vector index plus one. Initializers
retain declaration order. Missing lookups never allocate. Duplicate owners,
invalid owners and the existing one-million-entry bound fail before publication.

The declaration scheduler, checked-body provider and constant queries use this
same index. Query scratch no longer owns or reconstructs a duplicate table.
`initializer_indexed` counts publications; `initializer_lookups` counts every
lookup, including cached values and missing definitions. Source-budget
preparation remains separately counted by `value_preparation`.

`initializer_indexing` covers 8/32/128 declarations, exact append counts, four
bytes per definition, preserved order, three cached read rounds, and one missing
read. Their lookup deltas are exactly `3*N+1`. An ordinary-function-only control
retains zero index bytes, including after a missing demand.

Focused check and thirteen integration tests (including native controls), strict
all-target/all-feature Clippy, formatting and the architecture/file audit passed.
Evidence is in `obj/pa25-initializer-index/{0..5}.log`; the final lint/audit rerun
includes the separately committed aggregate helper refactor. The source manifest
is `obj/pa25-initializer-index/source-sha256.json`.

This checkpoint changes lookup complexity, not unused-initializer policy. The
independent exploratory outcomes in `unused/outcomes.json` under that evidence
directory demonstrate seven Rust-invalid unused ordinary constants/statics still
accepted by the compiler. Eager ordinary initializer validation is the next
correction; associated/generic laziness and unused function bodies must retain
their independently qualified behavior. A combined immutable performance study
will follow that correction rather than measuring two transient candidates.
