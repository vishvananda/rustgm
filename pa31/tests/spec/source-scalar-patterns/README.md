# Closed inclusive source scalar patterns

The frozen contract selects the original `Nanoseconds(u32 is 0..=999_999_999)`
declaration pattern and Clone/Copy derive producer from pinned
`library/core/src/num/niche_types.rs`. The local source foundation is reused from
the qualified union fixture; the actual `core::pat::pattern_type` declaration and
source Neg trait/i8 body are retained. Additional signed and char wrappers and C
observers are authored tests. Wider original comparison/hash/formatting methods
are not replaced by these helpers.

Before candidate observation, pinned Rust 1.98.0/2024 Linux metadata qualification
checks valid endpoints and rejects out-of-range values, invalid char base values,
uninitialized union reads, pointer-to-integer representations, reversed ranges
and a missing language gate. Separate object/C-link/native observation must emit
`0 7 999999999 -2 65` with status zero.

The first independent fixture omitted the actual Neg protocol; its positive
failure and all raw diagnostics remain retained. The corrected fixture has a
successful independent native observation. An observer ordering error that
initially skipped native execution is also retained separately.

Only closed inclusive scalar constraints belong to this checkpoint. Open,
exclusive and alternative patterns require the genuine RangeMin/RangeMax/RangeSub
source protocol. Generic endpoints, integer enum niches, refinement metadata
transport and complete original niche-type methods/runtime remain required.
