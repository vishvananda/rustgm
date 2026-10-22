# Raw slice selection and wildcard place duties

This development checkpoint follows `636b5c03`. PA27 and PA28 remain planned.
The source profile is Rust 1.98.0, edition 2024, Linux x86-64. Existing relocation
sources are preserved byte-for-byte; this checkpoint supersedes their explicit
raw-slice-indexing boundaries, rather than their historical evidence.

## Typed contract

An unsafe raw slice dereference retains its actual pointer value and length.
Constant indexing and checked start/end pattern indices first check the logical
length, then select a sized typed element within the original physical backing.
Oversized raw metadata is not clamped to the allocation: selecting an actual
in-bounds element can succeed, while selecting beyond backing storage rejects.
The incoming per-pointer write restriction survives every selection. Neither a
`*mut` result nor an array parent restores shared-derived write authority.

The private `indexed_element` consumer retains the ordinary sized projection
path and dispatches raw Slice places to `raw_slice_element`. Its existing
`Resolved`/`RawAccess` result anchors alignment at the selected T; packed fields
subsequently retain that anchor. A separately formed unaligned element pointer
still requires the canonical unaligned operation. Actual reads, stores and
references use the existing initialization, lifetime, exact physical type,
static-context and permission checks.

Raw length observation consumes only already-evaluated pointer metadata. It does
not read pointee bytes, validate a reference, demand static contents or perform
pointer arithmetic. Selecting index zero or a zero-stride element retains the
address-only traversal; actual accesses remain live-storage operations. Nonzero
address arithmetic retains the existing expiry checks. Logical index bounds
apply to zero-sized elements too.

Raw slice word decoding attaches independently transported length without
pretending that the value is a reference. Reference words still normalize a
bounded actual range and retain their full deferred reference-validity duties.
No Value, Origin, Pointer, WordTarget, Machine or public layout is widened.

## Wildcards

The three body pattern producers omit `selected_place` only for exact Wildcard.
Constructor testing still jumps to its yes successor, and slice length decisions
remain. Bindings, literals, constructors and refutable descendants retain their
selected accesses. Scrutinee evaluation and ordinary move/drop cleanup remain.
This removes unused element address arithmetic that otherwise rejects all-
wildcard length patterns over expired or deliberately oversized raw metadata.
It also removes unused Project rows from earlier body views; those exact text
deltas require integrator review and are not blessed by this checkpoint.

## Independent evidence and corrected hypotheses

The initial 27 opposing object outcomes matched. The next six disproved the
uninitialized-length acceptance hypothesis: source E0381 rejects forming a raw
address of the uninitialized local. Both original inputs and diagnostics remain;
this is not evidence of byte-mask validity. Three further controls distinguish
expired nonzero arithmetic, zero-stride arithmetic and logical bounds.

Twenty exact literal programs independently compile and execute, including both
constant and runtime values, packed raw address plus unaligned read, shared and
mutable selected references, relocation roundtrips, scrutinee evaluation once,
and ordinary pattern move/drop counts. Numeric/null/expired and invalid-reference
forms have no native-validity claims.

A transient oversized shared-reference union field followed only by an in-backing
selection accepts pinned CTFE, contrary to the original rejection hypothesis.
The original source/receipt is retained separately. Publishing that oversized
reference rejects. The compiler keeps its complete reference duties; transient
oracle acceptance is not permission to weaken them.

The first private test assumed shared-derived and writable aliases share an
arena LocationId. Retained projections may have distinct IDs while naming the
same root and field path; corrected invariants compare physical paths and check
pointer permissions independently. The failed test receipt remains available.

## Open scope

The new selection profile covers original array storage, normalized array/rest
views and exact single typed data cells. It does not provide arbitrary numeric
pointer reconstruction, whole unsized raw values, general raw range reconstruction,
unsized record-tail metadata production, broader reinterpretation or relaxed
reference publication. Those remain required course work. PA27/28 activation,
reviewed text vocabulary and full release qualification remain integrator-owned.

## Closing receipts

`obj/parallel/pa27/raw-slice-index/closing-second/qualification.json` records
43 source cases / 129 checked views, 20 exact literal Rust/CPPGM native pairs,
three private invariants, 40 independently qualified work inputs and affected
relocation, fill, slice-pattern and metadata regressions. Strict all-target,
all-feature Clippy, audit (1,619 files, zero findings), layout, upstream and suite
registration pass. Native tests remain explicitly ignored in ordinary CI.

The original declarations and independent receipts are retained under the same
root as `declared-{first,second,third,fourth,fifth,native}.json`,
`independent-{first,second,third}/`, `candidate-second/independent.json` and
`candidate-third/published.rustc.json`. `candidate-fifth/` records the three
private tests and all 129 views. Its predecessor preserves the failed arena-ID
hypothesis. The first placed-test lint failure (unnecessary raw-string hashes)
and earlier project-dispatch function-length failure are retained too.

The work grid varies array width and repetitions over 8/32/128 and path depth
0/8, with ordinary-reference controls and 64 unrelated-type controls. All array
widths have identical measured repetition growth. At raw depth eight, the three
repeat counts cost 845/3005/11645 steps and 68/164/548 slots, independently of
width. Unrelated types leave both counters unchanged. These are logical work
and storage observations, not wall-time benchmark claims.

The branch-active PA1–24 comparison is **8,233/8,283 byte-exact**. Fifty stdout
outputs differ; every expected status and stderr is unchanged. Forty-nine LowIR
outputs are identical after an offline review removes transitively unused
`addr`/`index`/`binary` address computations and renumbers surviving SSA values.
The remaining PA16 loan view removes the unused middle-element place and two
Project points; its retained two loans keep their physical selections and region
identities, with point/bitset renumbering. This artifact comparison is review
evidence, not a replacement for combined independent/native qualification.
No reference or active manifest was changed. The integrator owns scoped oracle
review and regeneration after combined qualification.

The exact delta inventory, hashes and complete `.review.diff` files are under
`closing-second/text-deltas.json` and `course/`. Affected declared paths:

- `pa13/tests/spec/bindings/array-copy-subfields.rs` (--emit-lowir).
- `pa13/tests/spec/bindings/array-disjoint-middle.rs` (--emit-lowir).
- `pa13/tests/spec/bindings/array-disjoint.rs` (--emit-lowir).
- `pa13/tests/spec/bindings/array-nested-moves.rs` (--emit-lowir).
- `pa13/tests/spec/bindings/array-reinitialize.rs` (--emit-lowir).
- `pa13/tests/spec/bindings/array-rest-alternatives.rs` (--emit-lowir).
- `pa13/tests/spec/bindings/array-wildcard-uninitialized.rs` (--emit-lowir).
- `pa13/tests/spec/bindings/partial-move.rs` (--emit-lowir).
- `pa13/tests/spec/bindings/wildcard-uninitialized.rs` (--emit-lowir).
- `pa13/tests/spec/conditions/chain-failure-disjoint.rs` (--emit-lowir).
- `pa13/tests/spec/constructors/delayed-move.rs` (--emit-lowir).
- `pa13/tests/spec/constructors/first-alternative.rs` (--emit-lowir).
- `pa13/tests/spec/constructors/move-or-captures.rs` (--emit-lowir).
- `pa13/tests/spec/constructors/partial-product.rs` (--emit-lowir).
- `pa13/tests/spec/constructors/tuple-uninitialized.rs` (--emit-lowir).
- `pa13/tests/spec/floats/f32-0-enum.rs` (--emit-lowir).
- `pa13/tests/spec/floats/f32-0-product.rs` (--emit-lowir).
- `pa13/tests/spec/floats/f64-0-enum.rs` (--emit-lowir).
- `pa13/tests/spec/floats/f64-0-product.rs` (--emit-lowir).
- `pa13/tests/spec/guards/contract-disjoint-write.rs` (--emit-lowir).
- `pa13/tests/spec/guards/dynamic-input-once.rs` (--emit-lowir).
- `pa13/tests/spec/guards/empty-field-after-disjoint-move.rs` (--emit-lowir).
- `pa13/tests/spec/guards/empty-rest-after-disjoint-move.rs` (--emit-lowir).
- `pa13/tests/spec/guards/guard-let-retry-storage.rs` (--emit-lowir).
- `pa13/tests/spec/guards/nested-constructor-retry.rs` (--emit-lowir).
- `pa13/tests/spec/guards/nested-retries.rs` (--emit-lowir).
- `pa13/tests/spec/guards/order-nested-order.rs` (--emit-lowir).
- `pa13/tests/spec/guards/order-parent-skips-inactive-choices.rs` (--emit-lowir).
- `pa13/tests/spec/guards/order-record-order.rs` (--emit-lowir).
- `pa13/tests/spec/guards/order-record-single-tests.rs` (--emit-lowir).
- `pa13/tests/spec/guards/order-rejected-prefixes.rs` (--emit-lowir).
- `pa13/tests/spec/guards/order-success-prefix.rs` (--emit-lowir).
- `pa13/tests/spec/guards/order-tuple-order.rs` (--emit-lowir).
- `pa13/tests/spec/guards/places-disjoint-move.rs` (--emit-lowir).
- `pa13/tests/spec/scalar-constants/scalar-in-array.rs` (--emit-lowir).
- `pa13/tests/spec/scalar-constants/scalar-in-enum.rs` (--emit-lowir).
- `pa14/tests/spec/execution/single-variant.rs` (--emit-lowir).
- `pa16/tests/spec/pattern-loans/cases-005.rs` (--emit-loans).
- `pa16/tests/spec/pattern-native/rest-return.rs` (--emit-lowir).
- `pa16/tests/spec/pattern-native/slice-rest-guard.rs` (--emit-lowir).
- `pa16/tests/spec/thin-native/guarded-alternatives.rs` (--emit-lowir).
- `pa17/tests/spec/order/parameters.rs` (--emit-lowir).
- `pa22/tests/spec/casts/reference-unsizing/const-generic.rs` (--emit-lowir).
- `pa22/tests/spec/casts/reference-unsizing/generic.rs` (--emit-lowir).
- `pa22/tests/spec/casts/reference-unsizing/grouped.rs` (--emit-lowir).
- `pa22/tests/spec/casts/reference-unsizing/mutable-shared.rs` (--emit-lowir).
- `pa22/tests/spec/casts/reference-unsizing/mutable.rs` (--emit-lowir).
- `pa22/tests/spec/casts/reference-unsizing/named-array.rs` (--emit-lowir).
- `pa22/tests/spec/casts/reference-unsizing/shared.rs` (--emit-lowir).
- `pa22/tests/spec/constants/persistent/constant-tails-keep-lengths-locations-and-promotions/tail-reference.rs` (--emit-lowir).

Compiler production SHA-256: `f23350c2aa15b13a09dc7a77a9b82efbdaa606fc460536f34992b2dbae64fc77`.
Candidate binary SHA-256: `954794376484a0914849e2326d3ecc86c7fefe4d3ed3a50723fe4240c421875e`.
Pinned backend tool hashes remain exactly those in `backend.lock.json`.
