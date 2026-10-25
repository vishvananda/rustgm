# Trait-alias source requirements

These sixteen authored, frozen programs investigate LIB-031, the current
unchanged-core loading failure. They are planned PA31 controls, not active
course cases or passing RustGM tests. Their feature request is unstable; the
shared parsing and bound machinery belongs to PA5/PA21 respectively.

`contract.json` binds the exact source hashes, initial predictions, actual pinned
rustc commands and diagnostic/output hashes. Run40239493812 observes 32 metadata
compilations and six actual native executions across Rust 2021/2024. All native
executions print `7\n`. There are zero infrastructure failures and eight retained
prediction disagreements: grammar before `=`, private-bound warnings and the
inactive unstable-syntax warning. Do not rewrite those predictions or sources.

Acceptance requires real alias identity, complete substituted predicates and
associated equalities, lexical binders, empty/nested/multiple bounds, cycle
diagnostics and ordinary object restrictions after expansion. An alias cannot
be an implementation owner. Actual literal allowance and forwarded caller syntax
have different authority. Disabled syntax has a soft warning. Parsing never
supplies imported alias transport or specialization permissions.

See [the implementation plan](../../../../docs/trait-alias-design.md).
