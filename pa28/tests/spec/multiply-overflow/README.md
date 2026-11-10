# Multiplication with overflow

Six complete authored sources, observer.c and expected.stdout are frozen before
implementation. Pinned Rust1.98 accepts five and rejects the non-Copy call.
The old9af9f187 CLI reproduces the exact original-core unsupported intrinsic
constant error in direct, signed and widths. The ordinary same-name function
keeps its authored result/false flag. Native source type admission already
passes; it does not establish runtime execution.

The safe actual intrinsic returns the product modulo the target width and an
independent overflow flag, including all signed and unsigned 8/16/32/64/128-bit
and pointer-width types. No panic is allowed for overflow. The C observer checks
156 input pairs, including zeros, extrema, both signed MIN/-1 orders, exact
non-overflow boundaries and full-width products. Expected result bits and flags
are mathematically declared before reference execution; the independent Rust
object matches every row exactly. Every expected row is printed without doing
multiplication in the C observer. The source-width packet additionally forces
CTFE overflow for each of the12 integer types.

The first no_core setup omitted Freeze; a second tuple-pattern form caused a
reference compiler ICE. Both remain unqualified under obj. Frozen direct and
ordinary cases use tuple fields, with no pattern comparison. PA28 owns numeric intrinsic execution after the early source-declarations
prerequisites; an earlier ordinary same-name source is not a reproducer. No course text reference is regenerated or grading
activation changed. Imported provenance and indirect calls retain their guards.
