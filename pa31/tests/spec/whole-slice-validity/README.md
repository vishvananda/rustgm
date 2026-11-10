# Whole-array slice result validity

Five complete authored sources have their hashes and outcomes in outcomes.json.
Pinned Rust1.98 independently accepts all five before implementation. Baseline
a73fa135 rejects byte, array and empty with the exact original-core missing
normalized-range diagnostic. The string and ordinary-wrapper controls accept.
No generated text references or graded PA activation are involved.

Registering the actual MaybeDangling language role enables strict result-reference
revalidation. The valid named slice points at original array storage with actual
length metadata, without an explicit sequence projection. A function-body array
annotation forces the same late constant-evaluation path as original fmt/num.rs;
a top-level annotation instead meets a separate early implementation limitation.
The unregistered ordinary lookalike does not activate that source policy.
PA31 owns the actual source role; ordinary CTFE alone does not reproduce the bug.

The repair must retain actual backing bounds, element type, initialization,
liveness, static-read policy and per-pointer write authority. A missing range
must not grant access outside retained array storage. All assignment requirements
and original core/alloc/std artifact gates remain open.
