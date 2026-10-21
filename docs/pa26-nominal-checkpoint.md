# PA26 nominal library metadata checkpoint

This checkpoint adds demanded `Option`, `Result` and `ManuallyDrop` metadata.
PA26 stays planned. Draft source outcomes are declared before qualification in
`pa26/tests/library-type-outcomes.json`; no generated text references are changed.

## Typed contracts

`support::library_types::LibraryType` selects explicit metadata entries in
`Resolution.library_types`. `get(kind)` returns the nominal declaration,
parameters and distinct variant/constructor DefIds. `identify(definition)` tests
the canonical nominal declaration; a user declaration with the same name never
acquires library behavior. Type and value namespace shadowing remain independent.
Demanded Option/Result types and variants join the enabled implicit prelude;
qualified paths and aliases use normal resolver scopes and access checks.

The semantic checker publishes ordinary parameter environments, nominal shapes,
field types, constructor signatures and callable templates directly from typed
metadata. Every diagnostic points at the actual demanding source node. Metadata
member publication deliberately omits source-node ownership indexes, so several
variants may share diagnostic provenance while retaining distinct identities.
No generated source, rendered syntax or synthetic source nodes carry contracts.

`ManuallyDrop<T>` has one `T: ?Sized` argument, transparent representation and a
private field. `Access::ExternalPrivate` prevents caller field reads, aggregate
literals and destructuring. `Definitions::destruction(item)` reports explicit
`FieldDestruction::{Fields,Suppressed}` policy; only validated ManuallyDrop
metadata registers Suppressed. Ordinary field formation, layout, validity and
conditional Copy/Clone obligations are unchanged. NeedsDrop, drop-region duties
and cleanup recipes respect suppression, including nested ordinary wrappers.
An explicitly selected destructor remains independent of field suppression.

## Proof ownership

`body::ir::BodyKey` separates declaration, closure and anonymous-body identities
from diagnostic `NodeKey` provenance. Body region contracts and solved analyses
retain this key; template, concrete-body, instance and loan proof pairing uses
it. Explicit pairing also validates provenance and instance identity. Constructor
contracts use the actual constructor DefId, while generic assumptions still come
from their nominal owner. This prevents Result's Ok and Err constructors from
colliding when metadata was demanded at the same source node.

PA24 integration must map its `Owner::Closure { identity, .. }` to
`BodyKey::Closure(identity)` and retain the same key on the closure contract.
The PA25 expression collector should keep anonymous expression keys while
selected declaration constants retain their actual declaration owner.

## PA29 visibility integration

`ExternalPrivate` deliberately carries no invented local ModuleId. Cross-crate
metadata remapping must preserve external ownership and private accessibility;
ordinary importing or reexporting must never turn an external private field into
a local-visible field. Dedicated metadata consumers can retain its field type
for layout and proof without granting caller field access.

## Qualification

The final guarded batch passed formatting, strict all-target/all-feature Clippy,
five sealed instance proof tests, ten projection debt tests, 22 caller-outlives
tests, eight quantified proof tests, three proof-view tests, three operator tests
and three nominal tests (54 tests total, including ignored native controls).
The nominal checks cover 17 source outcomes independently qualified with pinned
rustc object generation, plus four rustc-versus-CPPGM native programs: Option,
Result, nested cleanup suppression and a same-spelled ordinary wrapper.
Privacy tests require the actual aggregate privacy diagnostic for field reads,
literals and destructuring; an ignored-field rest pattern remains valid.

The Rust syntax/architecture/file audit reports 1,161 files and zero findings;
assignment layout reports 38 stages. `make test-upstream` verifies all 24 pinned
upstream fixtures and 126 Rust Reference files. Commands, logs, expected outcomes, source
hashes and object qualification results are retained under
`obj/parallel/pa26/nominals/`. The final resource receipt is
`609af1f22f2843f6a82418cc80f05c9b.json`. No generated references were changed.
Combined release/native suites and all-active text comparisons remain integration
work; this worker ran only the affected bounded gates.

## Boundaries

Question-mark control flow, option_env expansion, iteration, richer derive and
formatting hooks, and ManuallyDrop dereference/extraction/destruction methods
remain separate implementation work. This is not a PA26 release or PA30 ABI
claim. The integrator owns combined release validation and reference publication.
