# Source arithmetic needed by range helpers

This is a compilation prerequisite identified from unchanged pinned source,
not a claim about the latest whole-core diagnostic. The current original-core
frontier remains LIB-082's open/exclusive pattern profile. The source revision
is88d9e12ae178fab0fb5cc050a94da85685d449ea.

`library/core/src/pat.rs` implements actual RangeSub by calling the primitive
`checked_sub`, or `char::from_u32` for char. The unsigned implementation at
`num/uint_macros.rs:1046` checks its operands and calls the genuine
`intrinsics::unchecked_sub`. The signed implementation at
`num/int_macros.rs:734` calls `overflowing_sub`; that method at2658 calls
`intrinsics::sub_with_overflow`. The ordinary `unlikely` body in
`intrinsics/mod.rs:458` calls the `cold_path` intrinsic on its true branch.
None of these three intrinsic identities is currently in SourceIntrinsic.

PA28 owns numeric intrinsic binding and LowIR lowering; PA25 owns actual const
execution and PA31 owns authenticated source declaration/API policy. Use the
existing registered source intrinsic, typed signature, instance, body-call and
CTFE dispatch interfaces. Preserve the real unsafe requirement of unchecked_sub
and the separate const eligibility of all selected calls. The two bodyless
arithmetic declarations must match the pinned signatures and actual Copy premise.
ColdPath has a genuine empty source fallback and `miri::intrinsic_fallback_is_spec`;
preserve that declaration instead of requiring an invented bodyless form. An
ordinary same-name function grants no compiler contract. Preserve old bootstrap
identities and wire contracts. No frontend delegation or helper-body substitution
is permitted.

The next executable exit should cover unchecked subtraction and its overflow
pair across actual signed/unsigned target widths, plus the genuine effect-free
cold_path hint. Reuse target-width arithmetic and the frozen LowIR backend.
Checked source methods and RangeSub stay ordinary source bodies. Compile-time
unchecked overflow is a fault; the overflow-pair intrinsic returns wrapping bits
and the actual overflow flag. Declare and independently observe boundary, unsafe,
const, wrong-signature and lookalike cases before implementation observations,
placing them in PA28's source-intrinsic test cluster. Native consumers must match
pinned Rust; hot records and paid cold/failed work retain their existing checks.

The connected source-range consumer proceeds first with genuine MIN/MAX/open
bounds. An actual Sub request must retain its missing intrinsic/body failure
until this prerequisite is implemented. Char conversion/panic bodies, imported
transport, other numeric intrinsics and all original crate artifacts remain
required. This plan does not release PA25, PA28 or PA31.
