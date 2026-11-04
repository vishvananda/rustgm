# Owned unsized function parameters

Active source predictions are in `cases-v2.tsv` and `contract-v3.json`, with
file hashes frozen before the final independent qualification. The32 authored
sources target Rust1.98.0, editions2021/2024, Linux x86-64 and cap-lints=allow.
Earlier contracts and source bytes remain available.

The crate's genuine unsized_fn_params request permits actual function parameter
bindings to own a well-formed unsized type. The feature does not relax ordinary
locals, function results, closure environments, nested pattern bindings or
explicit Sized requirements. Shared/mutable references cannot supply owned
moves, and moves invalidate their source. Forwarding direct parameter places is
valid; parentheses preserve place identity, while blocks/if/match/loop results
cannot create unsized temporaries. An internal macro allowance alone is
insufficient: the pinned compiler consults the actual crate feature here.

The private source check includes the existing complete original-body,
ownership and loan pipeline.31 cases agree in both editions. The unchanged
Rust-positive `reference-control.rs` retains an explicitly required PA26
str::len library-method boundary; its existing lookup failure is asserted
separately and is not reported as Rust language rejection or a passing positive.
All32 predictions remain required for the final assignment.

The declared generic native example with println remains a Rust-positive
standard-library/macro requirement. The [PA30 observer](../../../../pa30/tests/spec/unsized-parameters/native.rs)
qualifies the same ?Sized original, an actual sized specialization and automatic
destruction using the external C observer supported by the backend. Both
independent Rust programs must print7 then9 on separate lines. RustGM's native
checkpoint executes the C-observed version. No original fixture is rewritten to
hide unavailable library lookup.

Concrete unsized data/metadata ABI, actual owned allocation handoff, full
metadata-aware destruction, reified pointers, capture/receiver profiles and
fresh imported originals remain required. Imported contracts retain their
existing mandatory Sized checks until actual permission and ownership transport
is implemented. This packet does not release PA26 or establish a core artifact.

Primary pinned source evidence: compiler/rustc_hir_typeck/src/gather_locals.rs,
expr.rs and fn_ctxt/checks.rs; compiler/rustc_borrowck/src/type_check/mod.rs;
tests/ui/unsized-locals and tests/ui/feature-gates/feature-gate-unsized_fn_params.rs.
Original core consumes the permission in mem::forget_unsized and FnOnce;
alloc uses it for owned dynamic call shims. Removed unsized_locals stays invalid.
