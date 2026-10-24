# First executable source-client transaction

This is the concrete continuation of
[the source-client row and body review](pa29-source-client-executable-slice.md).
The proof-phase staging is qualified through7f1ce7ed; no source payload, descriptor
or SourceClient mode is enabled. This plan does not narrow the remaining eighteen
transport scenarios, PA25–30 requirements or upstream core/alloc/std goal.

## One unchanged executable family

The provider is the complete existing
`pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs`, SHA-256
`1eadd5e238a13743e96b14c62312b48cd4c13e67a9b521541e2aa66d70ec2eb4`.
It includes actual sizing, LegacyReceiver/reference impls, Destruct, Freeze,
Clone/Copy, required clone, provided clone_from, ordinary u8 clone/Copy impls and
unbounded const identity<T>. The export includes and import re-proves every
provided body, even though the first runtime client demands identity only.

Use the unchanged `runtime-client.rs` in that directory: it has no core, imports
the provider through the direct alias `base`, and exports C answer() returning
`base::identity(7u8)`. Its independent/native expected answer remains7. The client
selects the exact authenticated provider artifact through the new source-client
policy; it receives no bootstrap traits, prelude or library factories. Export,
encode/decode and actual source-free original/instance proof must precede native
execution. No target frontend delegation or native/bodyless substitute is used.

The same-family immediate successors are unchanged const-client and noncopy-client
(both7), plus unsized-client (E0277). Full-attribute/stability transport and all
other matrix scenarios remain required. The first version continues rejecting
stability-bearing profiles until their actual import/use semantics exist.

## Remaining shared representations and their consumers

| Boundary | Exact data/change | Consumers and authority |
| --- | --- | --- |
| Payload | Private Legacy(Rows) / Source(Box<source::Rows>) variant and borrowed view; checked legacy-only accessor | Codec, resolver inventory, semantic headers, body import dispatch once; old0–8 and generic0/1 bytes unchanged; never fake empty legacy rows |
| Source declarations | Existing actual kinds/lexical owners plus canonical public-name targets addressed by artifact/declaration coordinates | Resolver reserves genuine module/trait/SelfType/function/parameter/local definitions and scopes; no invented impl/body/node definitions or trait-as-record encoding |
| Parameter scopes | Exact self/member/formal identities, inherited prefixes and explicit absent lifetime/formal bindings | Authenticate complete real DefId remap and kinds before installing; application arguments must be in the actual caller's scope |
| Header binders | Add an explicit declaration-header binder kind for actual trait/impl captured regions, separate from function, clause and call-opening frames | Reference impl self types must not use a fabricated function/Clause binder; derive counts/slots from actual retained Schema/lifetime scheme |
| Body inputs | BodyRef-indexed raw normalization allocation/labelled relations, original formation/coercion inputs and source builtin queries | Fresh imported allocations and full reproof; exact owned intervals only; no serialization of solver success, local region IDs or Copy decisions |
| Owned rvalues | Optional cold created-temporary and returned-release rows, retaining exact value, scope, location and exits-before coordinates | Validate complete operation-derived creation/transfer coverage and re-run ordinary cleanup/loan proof; do not assume primitive factory reconstruction is equivalent |
| Environments | Preserve absent declaration input, actual declaration input, validated const-trait-default local derivation and contract-derived Copy environment separately | B's defining default Self:[const]ActualTrait remains local to that actual provided member; it never becomes a declaration/caller assumption or executable const flag |
| Associated owner | source:Option<NodeKey>; actual imported canonical impl form in existing OwnerKind payload | Borrow source directly; imported None requires owning current Catalog and exact kind/remap lookup; target Owner32/OwnerKind20, measure again before adoption |
| Trait contracts | Preserve source vectors and written-Self NodeKey set; add cold imported inputs and explicit written-subject slot data | Same role/header validators consume borrowed source/imported view; type equality cannot invent written Self, conditional effect or bounds |
| Imported originals | Extend the existing one immutable optional ImportedOriginals Rc with source rows/inputs, not another Template Arc layer | Actual template Environment, promotion, eligibility, complete original/group proof, demanded instance and CTFE proof remain mandatory |
| Call requests | Exact original/value/site, target/application, signature, actual opening map and use effect | Consume corrected common producer; on import derive/check against authenticated body/header context, then prove callee clauses; reached Const payment remains B's separate boundary |
| Root client policy | Root-owned cold client request/prepared state with exact current/provider identities and parsed Features moved once | Authenticate source metadata before bootstrap suppression; keep SourceCore own-provider-only and Package.library read-only; no fallback or duplicate library requests |
| Role/report validation | Current catalog-authenticated declaration identities and exact source-client report/dependency binding | Extend require_provider_report's ownership boundary honestly for clients; no pretending current==provider or accepting name-only roles |

The raw row draft is not the frozen encoding. In particular the real reference
impls have header lifetime capture independent of body ownership, and u8 clone's
normalization-owned slot is independent of its operation rows. These domains must
be explicit before any descriptor can be emitted. Source/global Predicate12,
Value24 and no-import allocation behavior stay fixed; measure cold role/header,
payload and containing catalog sizes and combined retained/scratch bounds.

## Typed installation and publication order

1. A source exporter stage reserves the complete authenticated declaration/header
   closure and deterministic BodyRefs. It reads canonical declaration inputs and
   the separate actual body environment, preserving B's corrected defining context.
   Generic originals come from actual Checked; ordinary originals come from the
   successful borrowed collector. No provided body is omitted or demanded merely
   to make it appear in a template index.
2. Complete region/type classification and one memoized contextual graph export.
   TypeId alone is a key only for proven context-independent subgraphs. Region and
   binder-dependent roots retain exact body/header/frame/depth identity. Build call
   opening and owned-allocation indexes once, not by rescanning logs per port.
3. The source Payload validator checks complete typed tables, bodies, required
   versus provided members, roles, ownership, scopes, effects and operation-derived
   mandatory inputs. It rejects unsupported rows, omitted obligations or incomplete
   coverage. Source flags/claims confer no proof authority. The exporter completes
   its private payload only after body coverage and successful assembly/lowering.
4. The canonical codec dispatches a distinct source row version, with fixed typed
   table namespaces/order, LE integers and combined old/new count/byte/work limits.
   The provisional envelope9/version2 remains unpublished until fields and the
   complete roundtrip are frozen. Old versions reject new forms byte-exactly.
5. Before resolving a source client, validate its exact dependency/provider source
   descriptor and complete structural catalog. Reserve actual resolver declarations
   and private scopes, authenticate the complete mapping, then prepare the cold
   SourceClient policy with no bootstrap coexistence. No aliases escape a failure.
6. Ingest canonical trait/impl/function headers and argument graphs once. Validate
   exact role headers through the neutral adapter, complete impl conformance and
   retain every qualified/lifetime condition. No header validity pays caller debt.
7. Build source-free operations/inputs using fresh body-owned allocations and
   actual catalog identities. Validate operation-derived goals, local environment
   derivations and exact call requests. Freeze the one imported-original allocation
   only after promotion/eligibility; ordinary original/group proofs and all demanded
   instances/CTFE follow their existing gates before executable publication.

Failures unwind the staged payload, remap, aliases and executable installation.
Previously valid unrelated catalog rows remain unchanged. Runtime candidate,
context-only proof row and header-only required member remain distinct.

## Closing evidence for this family

Keep the staged collector's six qualified failure/legacy cases. Add actual provider
roundtrip and source-free runtime/native7, const/noncopy7 and unsized rejection;
missing Copy policy, invalid unused body, omitted provided body, omitted raw
normalization relation, wrong Self/owner/frame, foreign scope arguments and local
Self premise leaking to an ordinary impl must reject. Check stale/foreign catalog,
cold/warm unpaid caller and a late transaction failure without partial aliases or
executable rows. Existing legacy byte goldens remain exact.

Use independent declaration/body/call and shared-type-DAG scaling, counting actual
visits and aggregate retained/transient capacity before growth. No per-use full
header/ancestor scan, flattened scope closure, copied proof graph or widened work
budget substitutes for reuse. Worker checks remain focused/debug/static/native
for this vertical family; integrator owns the next full release/performance gate.
