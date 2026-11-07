# Source header storage admission

The first unchanged original-core probe after cold closure admission reached the
local source-header byte limit. The exact diagnostic run against compiler base
5775efbef proves this was not position-cache work: cold preparations, nodes and
visits were all zero. The original root requests stmt_expr_attributes and uses
the authorized fast path.

At failure, observe_owner had charged 16,000,370 bytes and 128,510 steps, with
23,219 declaration rows, no bound definition rows, and exactly one owner per row.
The declaration map capacity was 28,672; owner-vector capacity totaled 92,876
(four slots per owner). The located site is portable-simd/core_simd/src/swizzle.rs
line 526, byte 18932, source SHA 9803145df0f1d4e6d2f4196691b0ecda3e79428fc4d9a172a4da9f5e8ccc41be.
The unchanged local limits are 16,000,000 bytes and 1,000,000 steps.

The common owner vector now reserves exactly one initial slot, then uses normal
geometric growth when another actual owner appears. Map entry admission applies
only to a new key; a Header value already lives inside the map's charged slots.
Binding an existing row pays its new definition-map entry rather than another
fictional Header and declaration map. New rows still retain the same source
image, owner syntax and effect ranges. No record, permission or lookup result is
omitted. The existing four-slot-per-new-key conservative map envelope remains;
owner-vector growth pays its actual typed buffer bound and copy work. Charges
precede fallible reservation, remain consumed on errors and use the same local
producer. There are no new fields, caches or registries, and no budget increase.

Qualified production checks cover format/audit, strict library/frontend/CLI,
three cold tests, three inherited-overflow tests, four foreign-header/call tests
and the real cold capture native observer. Header 80, Owner 56, Headers 120,
Declarations 7120 and Code 304 layouts remain unchanged. The private 28,000-row,
duplicate-binding/no-phantom-storage and exhausted-owner-growth controls compile
under all-target metadata; their execution remains for CI.

The new immutable production CLI 5bd46ec7 reaches the actual next source fence in
18.1196s with all 722 input hashes, six configuration pins, 375 source opens, fifteen
cfg values, cap/profile and finite limits unchanged. The error is at
core/src/intrinsics/mod.rs:2297, file 47 byte 88680, ptr_guaranteed_cmp: source
do-not-const-check currently requires a supported panic role and indirect marker.
This is a named later requirement, not an artifact or complete-core pass.

Diagnostic evidence (including the temporary patch) remains under
obj/parallel/source-transport/header-budget/diagnostic1. The temporary atomics,
located failure formatting and counters were removed before qualifying the
production fix. Production checks are in fix1, the unchanged-core receipt in
core-fix1, and final metadata/harness evidence in final1. The separate no-body
cold commit 44f0 and its immutable binary remain preserved on their own branch.

Final strict all-target/all-feature metadata passes in 90.74s, plus 282 harness checks,
457 maintainer roots/layout and pinned upstream 127/Reference 126 verification.
The adjacent JSON binds all qualified sources, six pins, immutable CLI and both
original-core observations. No general performance or artifact claim is made.
