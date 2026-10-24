# Source-client import consumers before descriptor publication

This is the consumer plan for the current unqualified producer draft, following
[the first transaction](pa29-source-client-first-transaction.md). It does not
enable a descriptor, wire tag, SourceClient success or source-role fallback.
The first unchanged provider and runtime client7 remain the executable target.

The producer now uses the existing completed source role/header adapter and
actual Checked/Proved originals. Complete source rows must first validate the
declaration/slot/binder/type graphs, actual provided-body coverage, all operation
requests and raw input ownership. Declaration lexical parents and associated
function parents are distinct. Required methods retain explicit body absence.
Owned-rvalue creation and transfer rows are part of the complete original and
must be checked against actual operations and storage coordinates on import.

## Authenticated resolver reservation

The source decoder must not call the legacy declaration path with invented empty
Rows. It reserves the raw source declaration kinds as real resolver declarations:
Module, Trait, SelfType, Function, Generic Type/Lifetime, Parameter and Local.
It builds member scopes only for actual owning kinds and installs bindings only
where the encoded declaration/visibility/parent requires them. Impl ownership is
its real SelfType declaration, not a newly invented Impl/body definition. Missing
implicit lifetime bindings remain absent. The existing cold origin catalog owns
the complete declaration/body/site remap and must check kind, artifact, parent,
ordinal, binding and bounds before its permanent freeze.

The transaction produces the same indexed coordinate queries as the existing
importer: `generic_definition(identity, DeclarationRef)` and
`generic_body(identity, BodyRef)`. Source header and body consumers use only
that completed map. A same-kind declaration from a different artifact/catalog
must fail, including coincident local DefIds. Failure exposes no resolver report,
source role catalog, public alias, executable row or descriptor replacement.

## Neutral associated owners and headers

The proposed minimum existing-record change is `Owner.source: Option<NodeKey>`.
Some is the current borrowed source fast path. None requires an exact authenticated
catalog declaration lookup through the existing actual definition/Self definition;
it never means a bootstrap declaration. The measured source baseline is Owner32
and OwnerKind20; adoption requires fresh measurements rather than assuming a niche.

A separate `OwnerKind::ImportedImpl` holds the canonical optional trait predicate
and existing unsafe/constant/negative flags. Its Self type, captures and bounds
remain in the actual associated Schema. This avoids fabricated target/trait AST
nodes while keeping ordinary source Impl untouched. Its enum size must be measured
before adoption. Trait owner fields remain actual DefId plus flags. Trait Self is
validated against its own real SelfType declaration and parameter slot, not inferred
from equal canonical types or familiar names.

The already borrowed `compiler_items::headers::Trait` and method/header views
are the validation boundary. A checked imported adapter borrows completed canonical
parameters, inputs, member rows, actual DeclarationOrigin and written-Self slot
metadata. Source predicate vectors and NodeKey Self-bound sets stay source-owned.
Imported predicates/sites and written-subject data belong in the cold imported
header sidecar, with indexed owner lookup, rather than widening every located
source predicate or copying a source vector. Canonical method conformance consumes
a neutral borrowed method header/receiver, not a fabricated sema::Function.

`FunctionHeader.parent` is the actual associated Trait/SelfType owner, or None;
its lexical Module/Function parent is already in the declaration row. Its original
and relocated public signatures, complete lifetime scheme and formal binding map
remain separate. Imported original Owner::Function is constructed with execution
None for the supported first domain. Actual later materialization fills execution
through the existing checked signature transport. A relocated original outside
that domain rejects rather than erasing the view.

## Source role/report authentication

The resolver's source `bindings::Declaration` must remain source-only: its
NodeKey is not made optional merely to fit imported roles. The semantic catalog
needs a neutral borrowed role view containing actual definition, DeclarationOrigin
and actual associated parent. Source validators may extract a real source
declaration only from the provider case. Semantic consumers must use the neutral
role query and distinguish absent/reserved/validated states; they may not interpret
a missing source-only accessor as permission to install legacy Copy/Sized/prelude.

Catalog binding storage should be one cold provider/imported variant, with the
existing provider declarations and a compact imported role-to-declaration index.
Measure its containing layout before changing it. The imported variant retains
the same permanently frozen metadata catalog Rc already used by Resolution and
Linkage, and the exact current/provider identities. It does not allocate a parallel
owner map or a proof graph. Role validation reads the actual decoded header flags,
members, hierarchy and deny-impl rules through their canonical remap. Role data
and declaration effects never constitute caller or const execution payment.

The existing `require_provider_report` boundary must explicitly match the actual
mode. Provider checks remain unchanged. A client check compares the exact current
and provider identities; `SourceClient::require_inventory` verifies the retained
Dependencies allocation, cached index and actual sealed source descriptor. It
also requires pointer identity with the report's completed immutable origin
catalog, binding the actual resolver remap as well as the dependency allocation.
This is O(1), with no per-use artifact search. A foreign report with the same local
DefIds and equal-shaped rows must fail. Catalog remains absent/allocation-free
on ordinary no-source imports.

## Imported originals and use effects

The existing single optional Rc<ImportedOriginals> owns immutable imported
Template/Contract rows. No additional source Template Arc layer is introduced.
Fresh body-owned allocations come from each exact raw body/input owner, retaining
disjoint normalization ranges, labelled equality/outlives/formation/coercion goals
and complete operation-derived requirements. Complete generic formals and actual
source Sized goals are rederived even for unused parameters. Original assumptions
and query outcomes cannot pay substituted instance obligations.

Declaration environment, const-trait default local Self and Contract-derived
ordinary Copy environment remain distinct. Const-trait Self is derived only from
the exact actual const-trait provided-body owner after B's defining-context repair;
it is not added to declaration predicates or executable const eligibility.
Per-Call effect is the defining request. Original proof consumes the actual
defining context and complete conditions. Reached runtime/const instantiation must
derive its actual use obligations; copying a defining Conditional request is not
proof of a const impl or permission to execute a method. Body.const_context remains
actual executable eligibility, independently of the defining check.

The actual imported original takes precedence over a foreign-runtime shortcut.
A required bodyless trait method is not an FFI function. A selected implementation
must supply its actual checked original/instance before executable publication.
Unused provided bodies, including clone_from and actual u8 clone, are re-proved.
Promotion, eligibility, complete lifetime/loan/cleanup proof and ordinary demanded
instance/CTFE proof remain mandatory. Stability-bearing source rows continue to
reject until authenticated stability transport and use consumers are implemented.

Only after the complete roundtrip, remap, original/instance proof and actual
runtime/native7 transaction pass should the source descriptor be published.
Const/noncopy7, unsized opposition and all remaining eighteen source scenarios
remain required; this profile is not PA29 or upstream bootstrap completion.
