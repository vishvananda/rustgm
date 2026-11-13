# Zero-sized niche companions

These eleven authored source-core reductions preserve the shapes and layout
assertions used by pinned core's `io/error/repr_bitpacked.rs:350` and
`slice/cmp.rs:52`. The final inputs are exact copies from declared batch5 under
`obj/parallel/zst-niche/`; `outcomes.json` binds their SHA-256s, prior declarations,
Rust 1.98.0 results and candidate10/current outcomes. `cases.tsv` declares every
Rust input and the intended rejection site. The preceding malformed-prefix
batches remain recorded separately.

Ten cases accept, including both Result orientations, PhantomData and aligned
zero-sized companions, references, const matching, zero-byte transmutation and
the unchanged Option/tagged controls. The incorrect sixteen-byte assertion
rejects with incompatible types, agreeing with Rust's E0308.

The registered `zero_sized_niche_companions` integration tests check CLI typing,
prepared lowering and actual scalar/out-pointer signatures. See the
[checkpoint](../../../../docs/pa31-zero-sized-niche-companions.md) for qualification,
recorded failures and remaining contracts. No generated course oracle or PA31
activation is introduced.
