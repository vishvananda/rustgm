# Empty-binder inspection contracts

Full43154313816 on exact01e7013/sourcec388 passes every Rust regular test,
including3065 private tests with zero failures/three ignored and285 linking
tests. PA24 text passes642/642. PA21 text passes864/867: three inspection
oracles still show redundant pending scoped work for a zero-lifetime binder.
Later native/debug/independent CI stages are skipped.

The new inspection contract retains the declared Copy/trait/equality predicates
and zero-slot quantifiers. It omits the redundant proof/scope/block records.
The associated equality case also removes unused normalization intermediates
and renumbers private inference regions. This is inspection representation,
not permission to discard a caller's Copy, equality or lifetime obligations.

All three original inputs match the fully qualified5d source byte for byte.
Its immutable compiler reproduces their old references. Preceding67 and current
623 produce identical new inspection outputs, so the computed-match change did
not introduce these differences. The nonempty-binder Copy inspection remains
byte-identical to its existing reference.

Before generation, four intended positive programs actually execute through
independent pinned Rust and RustGM/CPPGM, including all three affected inputs'
identical LowIR companions. Six intended negative programs retain their complete
rejections: Clone without Copy, reverse Copy inference, a fixed lifetime used
as a universal, an escaping equality, a wrong associated result and swapped
correlated results. The selected independent suite passes10/10 with four
runtime comparisons. No input or acceptance/rejection outcome changes.

Normal scoped generation and exact text checking update only the three reviewed
inspection triples. New batch1794459158288820141-c8391c81a081.json has SHA
c8391c81a0816b0954be13f8275817c0667c41be5dbdb374328c3cb1db12b3d9.
All28,194 other reference hashes and the base reference.json remain unchanged.
No oracle is hand-edited and no unsupported positive is reclassified.

Production source08794/frozen623 is unchanged, so its existing core observation
and native/counter evidence remain applicable. Original core still first fails
FlattenCompat::iter_fold at flatten.rs:398; no core/alloc/std artifact exists.
One full successor is due after this reference repair publishes and a fresh
no-active-full check. Keep the stable fully qualified5d and optimized21f pins.

The [machine record](pa21-empty-binder-ci-review.json) binds the exact review.
