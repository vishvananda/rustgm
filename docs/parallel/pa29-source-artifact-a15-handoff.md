# Ordinary source artifact proof handoff

The A15 source snapshot `6fb6cee20003196736e002bda21dfc5c3df96a81`
passes the complete fresh private inventory (1,461/1,461), then a real provider,
public codec, fresh source-free consumer and native execution through both the
API and CLI. Both executables print exactly `7\n`. This closes the ordinary
Functions artifact slice, not the full original Clone/default/core family.
The source merge against `44a9a8a98e833c97c4ae952f860e13a4c9ae8b59`
contains the subsequent public, readiness and caller changes and remains
unqualified until root's combined gate passes.

## Proof and ownership

Raw imported bodies and full Contracts remain immutable. `OriginalInput` is the
shared producer: it first authenticates the exact raw original and Contract,
then prepares only an ephemeral Contract. Engine original proof and lower
context proof use this same producer. Cold `Contexts.imported` owns one boxed
`ImportedInputs` token, retaining the exact `Rc<ImportedOriginals>`, type arena
identity and dense prepared Contracts. Access checks the same arena and Rc;
region and loan phases borrow these rows. Raw Checked Contracts remain the
substitution source. No serialized row carries paid proof permission.

Preparation transfers ordinary Trait requests using the existing
`nominals::traits::State::body/prove/finish` facade. It retains all resulting
obligations in the region Proof and removes requests only after the whole
transfer succeeds. Errors, pending selection and budget exhaustion stay errors.
No Trait is converted into a legacy builtin. The no-Trait path returns before
allocating or changing shared contract ownership.

Materialized source originals with aliases or ordinary Trait debt capture the
full existing `InstanceScope` and a cold arena anchor. Transfer requires the
same full instance, key, source, public and execution signatures, universals,
actual environment and arena. A current builtin row is authoritative; its
mismatched or missing environment cannot fall back to a function header. Only
genuine row absence permits the exact current body-owner header environment.
Alias-bearing originals still require their full actual alias ledger. Each
substituted instance proves fresh; an unused `u8` parameter cannot be replaced
by `str` merely because the public signature stays unchanged.

All retained Contract vectors are charged before cloning; token publication is
atomic after all original proofs. Current whole-template checking is
O(imported-original-count) once per immutable proof context. Demand-based
imported proof remains an explicit optimization requirement. Prepared lookups
do not clone catalog data or reprove rows, including between region and loan
phases. The legacy/no-source and non-Trait, nonalias const-only provenance
paths were not silently extended.

## Exact qualification

A15 receipt:
`obj/parallel/pa29-source-client-staging/lib007/source-transaction-fifteenth/receipt.json`
SHA-256 `783311deee3be5ddf653e1b84159f8c23071af87ca8d18c241e091f6d095c8f0`.
Format, architecture/file audit and all-target/all-feature strict Clippy passed
before the build. The captured nonignored inventory ran exactly once, with
all required test names checked. Independent rustc oracle controls retained
10/10 declared outcomes. Their output is evidence about the source contracts,
not the target compiler implementation.

The resource token was `75e592ef926c4bc895dd83db023ac6af`, terminal zero and
released, under the unchanged 16 GiB absolute output cap and 150 GiB reserve.
Final accounted output was 7,556,628,480 bytes. Successful native objects and
executables were pruned with diagnostics, LowIR, metadata and hashes retained.

Trait-request counts 8/32/128 charged 131/514/2,050 canonical steps and one
owned universal. Actual selected lifetime debt fails without `a: b` and passes
with that exact assumption. Source-free immutable original counts 1/8/32 used
183/1,464/5,856 preparation work and 1,384/10,456/41,560 retained bytes; 128
borrowed repeated lookups added zero preparation work. Ordinary materialized
scope retained 208 logical bytes. Wrong Rc, arena, instance, owner, source,
signature, environment, missing scope, unsized subject and preallocation budget
controls pass.

The earlier A7 real failure, A11 bypass failure, A12 storage refusal, A13 audit
failure and A14 setup failures remain separate retained evidence. A14 also
completed a separately authenticated API/CLI native7 continuation; the later
Scope production change required and received the fresh A15 rerun above.
Failed runs were not relabeled as passing. The protected original full-provider
pin (`source-producer-ninth/pin/lib-tests`, SHA-256
`70eaa1eee10da37beb86a9faeadf08003266a7ab6a0a3b9db9e273b008eb9d91`)
remains intact.

## Integration boundaries

The handoff combines root canonical override matching, mapped projection views
and source scalar readiness with authenticated imported provenance. An imported
owner retains `source=None`; this never fabricates a source syntax node or
means Ready/paid/absent. Imported early-plan exclusion first authenticates its
actual owner, descriptor and complete predicate; it does not publish a source
unavailable marker. The normal later imported conformance consumer is preserved
and retains the current explicit receiver-only Functions profile limitation.
Source markers still require the actual NodeKey, predicate and arena. Pair
checking accepts optional source only to render honest unlocated imported
errors. Missing or old descriptor/proof rows deny.

Method conformance uses actual canonical trait member identity. Imported
expected methods use exact remapped headers, not fabricated source Functions;
source overrides still require actual syntax and complete signature/receiver
conformance. Declaration environments remain distinct from actual body-owner
environments. No wider default/const environment acceptance is implied.

The Engine merge retains root caller/reached-payment ordering and A's exact raw
preparation producer. Source imported originals use the producer; no-source
originals retain charged raw validation. The focused conflict review patch and
hashes are recorded under `lib007/a15-integration-conflict-review.json`.

## Frozen performance and next profile

No performance result is claimed for the Scope change yet. The frozen focused
A14-to-A15 plan is `lib007/scope-perf-after-a15-frozen.json`, SHA-256
`992bd6373a457d3421d3aabb9bec1bf539bd137be9b5efdbec91924a1b39e4cc`.
It declares four cold/repeated/distinct-instance/ordinary-control workloads,
A/A calibration and paired ABBA blocks, exact output equality and immutable
binary/input hashes. Run only in root's scheduled exclusive window after the
combined functional preflight. This comparison measures preintegration A14 and
A15; it does not qualify the newer combined readiness/caller source.

Current protected pins in `lib007/scope-perf-prep/pins` are:

| Pin | SHA-256 |
| --- | --- |
| a14/rustgm | `8cc3f9d8220ce65567d814bf76d0d8ac186aaae36704356e38cd1bf313a6558b` |
| a15/rustgm | `52559568d9a896bd73d261639258e7f84acb54a1a7b5b4079d70506245ef5372` |
| a15/lib | `c1099ce17b77bff991db4be062d5429fab7607971713a2da9846c9f39608e5ce` |
| a15/suite_linking | `49c55b25f540bba1825b23013ba05c8edff5b706d21e370ef6a77d6664c727e5` |

The ordinary Functions codec/profile remains unchanged. The next independent
artifact blocker is the full original family: provided/default bodies, actual
derived `ConstTraitDefault` body-owner environment, const/auto identity and
proof, nonempty lifetime/body allocation transport and all unused originals.
`lib007/full-source-successor-matrix.json` records that requirement/test matrix
(SHA-256 `4c61189d35c407b5c46423f470000edd7dc5dccec45de266b346dcf7c3b3c40a`).
Actual source attributes need complete transport; local imported map misses
must remain Unknown rather than absent. Future UB-check transport must bind
the actual containing BodyOrigin and call to Frozen producer versus Deferred
consumer policy; old or missing policy rows cannot infer either value.
