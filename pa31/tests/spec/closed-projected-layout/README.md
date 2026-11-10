# Closed projected-field alignment

These authored PA31 reductions retain the source-intrinsic contract needed to
reproduce original core's AtomicPtr::from_mut alignment assertion. `Atomic<u8>`
stores `<u8 as Storage>::Out`, which is `Align8<u8>`; `Plain` stores that same
field directly. Both alignment assertions require 8. Rust 1.98.0 accepts both
in edition 2024 with RUSTC_BOOTSTRAP=1 and metadata emission. The exact frozen
inputs and expected acceptance were declared before the implementation change.

The qualified baseline f5b5db49 accepts plain.rs and rejects projection.rs with
"constant storage layout requires completed fields". Initial probe setup lacked
an explicit rustc_attrs source feature; that earlier rejection is retained in
obj/parallel/atomic-layout and is not the layout reproduction. All outcomes and
SHA-256s are in outcomes.json. These cases require PA31's actual source intrinsic
role; the projection syntax itself remains owned by PA20. No upstream bytes or
checksums change, and PA31 stays planned rather than becoming an empty graded suite.

The correction and retained proof/work requirements are documented in
[the layout checkpoint](../../../../docs/pa31-closed-projected-layout.md).
