# Physical reference cells in constant execution

This bounded PA27 correction accepts direct reference cells whose physical
storage type differs only in its outer RegionId. The private machine predicate
requires identical outer reference mutability and identical referent TypeId.
It is used by raw projection, shared reference reconstruction, leaf raw-access
checking and raw-anchor checking. Identical IDs take the existing fast path;
otherwise two constant-time kind lookups suffice. No source type equality,
variance, lifetime proof, Value/View/Origin, layout or pointer permission changes.
No recursive type normalization or reachable-value walk is introduced.

The independent investigation disproved two initial rejection hypotheses. Pinned
Rust accepts an unused outer reference to a cell containing an expired inner
reference. It also accepts the retained nested shared-access write case with
`invalid_reference_casting` explicitly allowed. Both original hypotheses, source
bytes and observations remain in the declared contract and investigation records.
They do not justify eagerly traversing unused references or recursively poisoning
pointer authority. Actual access through an expired inner reference rejects.
Dangling raw-pointer fields remain terminal until dereferenced. Writes through a
shared-derived pointer to the reference cell still reject; a separate writable
alias remains usable.

Fourteen independently compiled object outcomes distinguish eight supported
acceptances, three real rejections, and three accepted-Rust reinterpretation
boundaries. Different referents, outer
mutability, and inner lifetime structure are explicit unsupported cases rather
than validity claims. All three frontend views check the same declared outcomes.
Four native groups compare live cells, terminal storage, alias rights and static
reference cells against pinned rustc. These programs perform the interesting
operations during constant evaluation; no invalid runtime access is executed.
Native checks remain ignored by default and run explicitly in qualification.

Three selected private invariants pass, including direct tests that the predicate
ignores only the outer lifetime and remains false for different nested reference
or pointer types. Seven affected source/scaling regressions pass. At N=8/32/128,
ordinary loops use 483/1611/6123 steps and raw loops use 517/1741/6637 steps;
both retain 44 slots. The loop keeps its reference binding outside the loop.
Tests bound linear work and constant raw-versus-ordinary slot overhead; these
observations do not establish wall-time neutrality.

Strict all-target/all-feature Clippy and the 1,365-file audit pass. The initial
Clippy failure was a test helper declared after statements; moving that helper
to module scope corrected it without production changes. Its failed log remains.
The final receipt records fresh build, upstream verification and exact artifact
hashes. No active oracle, assignment activation, backend or full-release change.

The other profiles in the shared-reference review remain boundaries: mutable
raw reborrows, raw union-field references, mutable/foreign/cyclic static reference
formation, numeric nonnull ZST pointers, wider type reinterpretation and pointer
offsets. Shared-derived interior writes and pointer-bearing union Images also
remain outside the current profile. Source packed reference rejection remains
independent from physical raw-leaf alignment. Neither PA27 nor PA28 is complete.
