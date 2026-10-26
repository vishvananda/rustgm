# Source non-null pointer patterns

This planned PA31 source provider keeps the actual NonNull field and selected
from_ref/as_ptr/as_ref/cast bodies, with actual source sizing/Clone/Copy and
transmute contracts. The raw-pointer cast_const body is source-owned too.
Stability declarations/full NonNull file, new_unchecked/track_caller, and complete
range/or/nonpointer pattern families remain required. The non-null documentation
and lint attributes never establish scalar validity or layout; the real Pattern
field does. No factory, backend or foreign wire profile is added.

Candidate results are pending. Independent pinned evidence is frozen under the
maintainer source-nonnull-pattern-v1 receipt; no native claim follows from object
acceptance alone.
