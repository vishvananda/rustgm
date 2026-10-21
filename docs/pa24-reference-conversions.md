# Reference conversions in captured places

PA24 remains planned. A selected reference coercion now records the exact
pointee path used inside a closure, together with its existing dereference
count and shared or mutable borrow type. This extends the same typed capture
consumer used by method and callable receivers. Value conversions such as
array-reference unsizing retain their original value use.

Body construction selects the physical captured path before trying to read an
ancestor that capture precision discarded. It validates the complete synthetic
reference prefix, checks the remaining selected dereferences and the source
type, then applies the ordinary borrow, unsizing or pointer conversion. These
records grant no new ownership or lifetime permission. A shortened forged
selection fails before it can skip an unpaid projection.

The declared matrix has twelve accepted programs and four rejected programs.
It covers shared and mutable reborrows, deep references, array-to-slice
conversions, disjoint fields, move and nested closures, returned references,
reference-slot reassignment and opposing live-loan/mutability cases. All sixteen
sources were independently compiled with pinned rustc and each accepted program
was executed before testing the candidate. The original hypothesis that reference
slot reassignment would reject was disproved: the closure retains the original
pointee. Its first declaration and independent result remain in the evidence;
`declared-v2.json` records the correction explicitly.

The first candidate matches all sixteen source outcomes. The focused maintainer
suite additionally executes all twelve positive programs through both rustc and
the pinned CPPGM backend, checks malformed selection, and verifies fixed path
storage and linear capture work at 8/32/128 repeated conversions. Existing direct,
adjusted and nested closure tests share that gate. All seventeen test groups pass,
including explicitly selected native groups; default CI skips only tests needing
the external backend.

Evidence is under `obj/parallel/pa24/reference-captures/`, including the immutable
baseline's rejection output, independent object/native results, source hashes and
`focused-v1/`. Its strict lint check caught an existing exhaustive test helper
missing the new access kind. After adding that arm, `focused-v2/` passes the
affected helper, strict all-target/all-feature Clippy, the file/layout audits,
upstream verification and all 7,787 active text comparisons. The earlier passing
seventeen functional groups were retained without repeating them for that
test-only correction. Combined performance/publication qualification remains
pending. No assignment has been activated and no text oracle has been regenerated.
