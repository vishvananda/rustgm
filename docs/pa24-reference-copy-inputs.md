# PA24 reference capture classification

Capture classification can inspect a reference's outer constructor without
requiring its pointee to have a complete canonical type. Shared references are
Copy for every pointee, including unsized and non-Copy types; mutable references
are not Copy. The classifier now returns that unconditional property directly.
It does not default numeric variables or skip referent formation, body inference,
ownership, call-kind selection or ordinary lifetime proof. All other type
families keep their existing selected Copy query.

Seven unchanged authored programs in `capture-copy-outcomes.json` independently
match pinned Rust 1.98.0 full-object outcomes and all three frontend views. Four
positive programs execute natively: unresolved numeric shared-reference result,
repeated move-closure shared-reference result, an unresolved mutable-reference
FnOnce result, and a shared reference to a non-Copy nominal. Opposing programs
reject a repeated moved mutable closure, a mutable-reference move required as Fn,
and a shared result escaping local backing. The type view intentionally defers
the moved-read and lifetime rejections to the executable proof pipeline.

The previous candidate rejected the first three positives and all three
negatives during Copy classification. The corrected negatives now reach the
actual move/call-kind/lifetime rules; the non-Copy-pointee control stays accepted.
The earlier object-input quantified-output positive advances from the Copy gate
to its separate inherited principal input-ownership boundary. Its expected
outcome remains an explicit supported-by-Rust implementation boundary, with the
precise diagnostic updated rather than promoted to a semantic negative. The later
[signature ownership repair](pa24-signature-slot-ownership.md) completes that
positive; these earlier observations remain historical evidence.

Evidence: `obj/parallel/pa24-copy-shape/` retains the predeclared hypotheses,
original source hashes, baseline observations, first candidate observations and
qualified logs. The qualified batch passes fifteen test functions: two new
source/native functions, four inherited object-input functions, five receiver
functions and four signature functions. Seven new source cases and four new
native programs pass; previously passing inherited object input/native/scaling
checks pass with unchanged recorded work counts. Strict release all-target and
all-feature Clippy, architecture/file audit (1451 files, zero findings), layout38
and upstream24+126 pass. An initial missing import compile failure is retained;
no failed batch is counted as qualification. No references, assignment activation,
full release or benchmark changed in this worker checkpoint.
