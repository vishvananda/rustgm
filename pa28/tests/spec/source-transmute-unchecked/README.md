# Source unchecked reinterpretation

The exact pinned unsafe const declaration retains its indirect-const marker.
Generic and unequal-size bodies type without the strict transmute size premise;
executed const mismatches are UB and reject. Same-size execution shares the
existing initialization, validity, provenance and owned-move kernels. Runtime
unequal/ZST and indirect intrinsic calls remain explicit finite requirements.

The non-const declaration with runtime-only callers is a Rust positive, retained
as an unsupported original-header profile. `const-use-nonconst.rs` separately
requires a const call and is a genuine Rust rejection. Wrong signature, missing
unsafe/intrinsics gate and invalid bool remain independent controls. All old
metadata profiles fence this operation; no old tag or payload changes.
