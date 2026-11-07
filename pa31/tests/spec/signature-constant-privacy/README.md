# Type privacy of signature constants

Eleven no_core source-core libraries for type privacy in anonymous constants.
rustc 1.98.0's type-privacy pass visits bodies, including every anonymous
constant nested in one, but never an anonymous constant of an item signature.
A crate-root fn result length, struct field length, signature const argument,
enum discriminant, const item annotation, a local annotation nested inside a
signature length, and lengths that evaluate public const fns returning the
type are therefore accepted even though they reach a `pub(super)` type the
crate root cannot see.

The opponents keep the rule's other half: a local annotation length, a repeat
count and a const argument inside a fn body are rejected without an error code,
and associated function privacy (E0624) still holds in a signature length.

Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json`. The lang-item prefix is the exact pinned Rust core fragment
from the census batch 2 controls, with its attributes and comments. It keeps
Rust's MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/privacy_of_signature_constants.rs` checks every case. See
`docs/parallel/census-batch5.md`.
