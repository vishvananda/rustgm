# Diagnostic-only const and macro directives

This unpublished PA40 packet covers `diagnostic_on_const` and
`diagnostic_on_unmatched_args` in the pinned Rust 1.98.0 source. The active
prediction contract is `contract-v4.json`; `cases-v4.tsv` selects 59 original
fixtures in both the 2021 and 2024 editions. Primary implementation paths and
hashes, fixture hashes, required outcomes and the native C observer are bound
in the contract. These course-authored fixtures are not upstream test copies.

The requests grant no const, unsafe, trait, macro-matching or ABI capability.
Without the feature, Rust ignores these diagnostic directives with an optional
warning. Under the authenticated explicit Allow profile, RustGM admits their
metadata without allocating message-rendering tables. Syntax-owned attribute
forests supply the original literal tokens; escape/Unicode validity remains a
hard check, including disabled owners and nested `cfg_attr` input. Literal
suffix complaints and malformed/duplicate/unknown diagnostic options are soft
under this profile. Active unsafe wrappers, unknown/removed feature requests,
const or unsafe permission failures and failed macro matching remain errors.

The C observer calls actual trait and macro consumers and must print `7 9`.
Its attribute-free control must have the same behavior. Do not replace these
calls with a constant-output driver or use another Rust frontend to lower them.

Custom const-failure and macro-mismatch text, strict optional diagnostics,
feature/target-sensitive warnings and imported diagnostic metadata remain
required. RustGM currently rejects feature-enabled uncapped directives explicitly
rather than claim that rendering works. Without the actual crate feature it
ignores the metadata in strict compilation too; a macro allowance does not enable
either handler. The feature word records this diagnostic-only distinction without
granting any operation or growing ordinary records. A lint cap does not suppress rustc's custom
hard-error text: deferring that text is an explicitly unfinished compilation
profile, separate from preserving the actual hard semantic failure. This packet
is not PA40 completion or evidence of an original core/alloc/std artifact.

The first prediction packet is retained as failed investigation evidence. Its
independent oracle disproved hard suffix errors and exposed two incorrectly
scoped disabled fixtures. Version 2 corrected these predictions and wrapped the
whole removed owner in a disabled module. Version 3 adds active/inactive nested
configuration and an attribute-free native control. Version 4 adds required strict
successes for inactive directives, fixing the original offset_of provider regression.
Only version 4 is active; earlier predictions do not establish the current profile.
