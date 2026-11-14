# Initializer temporary storage checkpoint

Clean release24 reaches all 42,310 original-core declarations and 25,858 bodies
without a function failure, then fails completion at unicode_data.rs:1148:
`constant pointer refers to expired storage`. Its unchanged LOWERCASE_LUT
initializer borrows arrays containing actual const calls. Exact error-instruction
debugging finds frame return expiring those arrays before publishing the value.

The typed CFG already distinguishes initializer lifetime extension from promotion.
Execution snapshots now retain the unpromoted `Lifetime::Static` temporary IDs in
one optional operand prefix. Their evaluated backing survives frame return;
ordinary local and callee storage still expires. Validate the extended backing
after local expiration so an outer extended container cannot revive a dangling
nested reference. This grants no promotion, const-call or borrow permission.

Const eligibility separately rejects mutable borrows of the actual extended
temporary, including its field/index projections. Dereferencing an independently
owned pointer selects its referent. Projection answers are cached within that
check; ordinary bodies allocate no extension index unless a mutable borrow needs
one. Snapshots remain Code304/PreparedOperand32/Body456/Value24 bytes. The existing
code budget charges both the optional record and its shared ID storage before
snapshot allocation; malformed IDs cannot publish a catalog entry.

Fixtures and predeclared outcomes live in each owning `initializer-storage`
packet: PA25 has safe const/storage cases, PA27 actual unsafe dangling opponents,
PA28 C ABI observers, and PA31 source-role counterparts. All26 inputs remain:
24 qualified outcomes (16 accepting, eight rejecting) and two independently valid
named mutable-static positives that still hit existing compiler limitations.
Those two remain required: direct reference formation rejects mutable static
storage; the raw-pointer counterpart rejects constant mutable raw references to
static storage. Neither is reclassified as an expected language rejection.

The source packet retains the original Unicode definitions and LOWERCASE table
bytes. Its executable reduction preserves the nested borrowed shape and uses
the real unmarked `core::convert::identity` body for computation. It does not
claim full original Range constructor or Unicode lookup execution. Original
nominal-pointer library metadata-profile artifacts also remain required; native
program preparation is a separate passing surface.

Focused fast1794651862191683241 passes four exact tests, strict selected/CLI
Clippy, full formatting and the architecture audit. Native1794651989245247938
executes seven actual RustGM/CPPGM/C programs: three nested ordinary tables output
`7 11 13 14 2 2`, scalar/block controls output7/9, and the source table outputs
`192 22 32 256 1 1 304 775 2 0`. Compatibility1794652124030049912 passes four
existing promotion, effects/bounds and reached-storage-expiry regressions.

Hash-checked replay authenticates all3961 compiler/test sources, every packet
input and the exact frontend harness
`ca42ab30990178afe16b1c08127a4039ef8e78a5b8ebd6d5c5ee7ded418f1228`.
At widths8/32/128, inference/canonical/CTFE steps/code bytes are respectively
`154/123/318/2456`, `418/411/1184/6584`, `1474/1563/4624/23096`.
The ordinary shift matrix control remains passing. No timing speed claim is made.
All315 harness checks,493 integration roots in13 binaries,44 assignment layouts,
the separate component,128 pinned upstream fixtures and126 Reference files pass.

The first fast run executed its tests successfully but failed its outer cleanup
deadline; it is not a passing validation batch. A subsequent mutable guard test
exposed a real permission gap, and the first guard implementation used the wrong
IR mutability discriminator. The corrected typed eligibility check is covered
by the passing final runs. The initial mutated-temporary and mutable-static
acceptance predictions were false under pinned Rust (E0716/E0764); their original
bytes, predictions and diagnostics remain under `obj/parallel/static-temporary-storage/`.

The new private snapshot-budget invariant is pending full CI execution; no cold
local private build is claimed. The previous source823dc01df full43575779218
passes every stage, including3077 private tests in each profile. A fresh full CI
and unchanged original-core observation are required for this checkpoint.
No original library artifact or whole PA25–44 completion is claimed. The separate
released113-case const-machine component is unchanged.
