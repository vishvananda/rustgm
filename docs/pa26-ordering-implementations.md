# Canonical Ordering comparison checkpoint

This isolated PA26 checkpoint supplies canonical `core::cmp::Ordering` and
`std::cmp::Ordering` with PartialEq, Eq, PartialOrd and Ord evidence. It remains
planned and is not an assignment release.

The demand-created comparison registry retains its exact nominal identity.
Requesting that type registers its real comparison prerequisite contracts;
recursive Ord/result preparation reuses completed identities. Merely declaring
an unrelated enum does not create comparison evidence. Selected method bodies
remain demand-loaded and use the existing private Selected/Permit route and
ordinary region, ownership, loan and drop checks.

Operation::Ordering carries the comparison family without increasing the
operation/descriptor/selection representation. Its I8 fields describe validated
signed discriminant operations, never the source operand types. Both candidate
production and factory validation require the exact registered nominal, empty
arguments, completed fieldless enum schema and -1/0/1 discriminants. The factory
reads actual nominal operands, then uses ordinary EnumToInteger casts and the
existing scalar comparison/Ordering-result CFG. Primitive paths remain intact.

Validated completion also publishes the canonical type's actual derived
PartialEq/Eq structural property. Named constants enter the ordinary constant
pattern worklist; aliases retain identity and user lookalikes receive no derived
property. Comparison implementations and structural pattern permission remain
separate facts coupled to that same exact library declaration.

Thirty predeclared source programs distinguish all nine variant pairs,
qualified/dot/reified methods, generic traits/helpers, aliases, local nominal RHS
implementations, user lookalikes, no-prelude use, atomic Ordering mismatch and
invalid callable/const/orphan controls. Reference and array comparisons are
Rust-valid remaining boundaries. Independently qualified sources and every
preparation/failure observation are retained under
`obj/parallel/pa26/ordering-impls/`. All 30 source outcomes and 20 selected
Rust/CPPGM native programs pass; the no-prelude program also passes direct
LowIR/backend execution. Eleven factory invariants and four provided-original
invariants pass, including both original contract collectors. Existing equality,
ordering and helper source/native groups remain green.

Strict all-target/all-feature release Clippy, architecture audit (1,465 files,
zero findings), layout and pinned upstream checks pass. All 7,787 committed
PA1–23 text comparisons agree exactly. Operation, Descriptor, Selected and
selection Source retain their prior 2/6/12/16-byte sizes. The production source
manifest covers 864 files, SHA-256
`7443c097ce0e1c66ef3a3f4e95010649289d7bb1c8bba1ac7bff731de427ba58`.
`qualified-checkpoint.json` records the exact evidence paths. No full Cargo
publication suite or exclusive performance measurement is repeated here.

Remaining PA26 work includes broader structural/reference comparison impls,
Ordering convenience methods, primitive reference/unary and Deref hooks,
unsuffixed assignment-method RHS inference, iteration, question-mark, required
derives and formatting. PA30 remains separately queued. No references are
regenerated and no assignment is activated.

The first PartialOrd-first control compared `a < b` for `a,b: &T`, which needs
the outstanding reference blanket implementation. Its original Rust acceptance
and source are retained. The independently qualified v2 `a.lt(b)` checks the
intended prerequisite registration order without adding that requirement.
The reference-positive remains required work, alongside the explicit reference
and array boundary fixtures. No accepted language program is reclassified as a
Rust rejection to close this checkpoint.

The provided-original regression now expects exactly eight originals when
Ordering registers Ord: ne, four partial relations and three owned helpers.
Those owned originals undergo ordinary promotion analysis before proof; the
separate missing-analysis negative remains intact. Earlier failed preparation
and audit observations are retained with the successful qualification.
