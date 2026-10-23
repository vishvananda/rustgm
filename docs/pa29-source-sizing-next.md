# Source sizing headers after direct generic calls

This is a retained next-family proposal, not v1 policy or execution permission.
The direct-call version keeps defining Rust 2024 / legacy-role policy and still
rejects an active source-language provider.

The pinned original `library/core/src/convert/mod.rs:106` defines
`pub const fn identity<T>(x: T) -> T { x }`. Its operation shape already fits
Read/Return. Actual source-core mode adds the real MetaSized/PointeeSized default
clauses in `sema/declarations/generics/bounds/sizing_defaults.rs`; dropping them
or changing source mode to legacy would change the contract.

The next minimum header family should retain the actual three source sizing
traits (`marker.rs:158,175,190`), their real Self/declaration owners, complete
supertrait applications, empty member sets, source constness/safety/structural
attributes and exact role availability. Import authenticates the selected core
artifact and declaration remaps, then derives registration from those typed
headers using the same semantic validators as source. It never imports a
validated flag, registry permission, synthetic trait or fabricated NodeKey.
Declaration sites stay distinct from body sites, and bodyless trait declarations
do not invent originals. Source and imported header-validation inputs need one
neutral view with honest Source/Imported diagnostics.

Use the exact upstream identity body with a reduced source provider containing
only those genuine declarations to isolate this seam. Copy/Clone or other roles
may be absent only if actually absent in that provider. Full core's actual roles,
required/provided members and default bodies cannot be omitted to imitate that
reduced package. Every original and demanded instance still pays all symbolic
formation, sizing, lifetime and caller obligations; role observation supplies no
proof. Include foreign/same-shaped roles, missing supertrait/default clauses,
wrong Self owner, absent-vs-unvalidated availability, unpaid instance and warm
foreign-catalog opposites.

Direct generic calls are independently necessary for actual wrappers such as
`core/src/hint.rs:490` black_box forwarding to its source intrinsic. That target
also needs authenticated intrinsic contracts; no bodyless generic fallback or
serialized intrinsic permit can substitute. Traits/impls/qualified effects,
references/lifetimes, const arguments, conditional originals and every remaining
case in the original 18-scenario manifest remain required.
