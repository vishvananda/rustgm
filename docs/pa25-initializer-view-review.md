# Initializer lifetime inspection review

The initializer lifetime correction gives each source borrow its owning body
region and ordinary coercion obligations. Previously, some initializer borrows
were stamped `static` before the storage and lifetime proof ran. The proof still
establishes static backing for eligible promoted values; an intermediate borrow
is not itself evidence of that backing.

This changes 74 successful inspection outputs: 31 PA13 pattern views, four PA16
loan views, and 39 PA20 trait-semantic views. Source fixtures, declared outcomes,
diagnostics and status sidecars are unchanged. No successful LowIR sidecar changes.
The views retain the additional reference types, region variables and coercions;
dependent type/region identities consequently change. Pattern roots and coverage
decisions remain unchanged. PA16's additional coercion also changes statement
and point numbering.

All three complete source suites were checked independently against the pinned
rustc before regeneration: PA13 has 796 checks, PA16 has 307, and PA20 has 1,381.
The suites retain their eight PA13 and one PA20 documented acceptance differences;
every one of the 74 changed inspections is accepted by both compilers. The
independent evidence, strict all-target/all-feature Clippy and clean file audit
are retained in `obj/parallel/initializer-view-review/complete.json` in the
validation worktree.

A separate regression initially duplicated opaque-result obligations during
contextual callable inference. Commit `4a449961` retains contextual callable
signatures while leaving the final opaque proof authoritative. PA21 and PA23
return to their existing exact text contracts without regenerated references.
The initially failed comparison and complete diff remain available for review.

The normal full-stage reference generator refreshed PA13, PA16 and PA20 only
after a fresh comparison verified the exact reviewed 74-path set and byte-for-byte
agreement with the retained reviewed outputs. A pre/post snapshot verifies that
all other sidecars remain unchanged. The full receipts now also record effective
case contracts, which allow later independently qualified scoped batches to
preserve the earlier base and unselected oracles. No fixture or expected outcome
was changed to make this comparison pass.

The generation evidence is in
`obj/parallel/publication-397/references/`: `approved-changes.json`, the fresh
comparison and generation receipts, `generated.diff`, and `complete.json`.
The compiler source is identical between validation commit `397177f2` and its
main-line cherry-pick `4a449961`; the later receipt-harness commits do not change
the compiler. This is a development checkpoint; PA24 through PA30 remain planned.
