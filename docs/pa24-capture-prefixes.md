# PA24 capture-prefix correction

PA24 remains planned. The capture graph is a typed analysis component; this
checkpoint does not enable source closures with stored captures.

Merging a referent mutation into a required ancestor now preserves the mutable
reference edge. For `let x = &mut value`, a closure that both reads `&x` and
writes `*x` requires a unique immutable capture of `x`. Replacing that mode with
a mutable borrow would incorrectly require the binding `x` itself to be mutable.
An explicit mutation or move of the ancestor still wins, and a precise capture
of just the referent retains its own mutable permission.

Finalization records the selected covering capture for every interned path in
the same forward pass that selects environment fields. Consumers can retrieve
that capture in constant time without copying or walking the ancestor chain.
The completed capture list is immutable through the public interface, so the
covering index cannot be invalidated by consumer edits. Paths with only narrower
captures have no covering capture of their own.

Evidence is retained in `obj/parallel/capture-prefixes/`. Eight declared pinned
rustc object outcomes passed: five accepts and three rejects covering unique
borrows, conflicting aliases, Copy, Fn/FnMut, moved references and raw writes.
These independently establish the language controls, not source-frontend support.
Eight graph invariants passed, including widths/depths 64, 256, 1024 and 16384.
They check stable capture selection, distinct modes, failure invalidation and
linear construction/finalization work. Strict release all-target/all-feature
Clippy, the file audit and layout check also passed. Temporary qualification
objects were removed by the storage guard; sources and diagnostics remain.

The next integration must transport lexical local types through the nested
inference interface without publishing them as child-owned declarations, derive
actual place uses separately from lexical mentions, and construct checked
environment receivers. Capture modes do not themselves authorize calls or skip
the ordinary initialization, region, loan and destruction checks.
