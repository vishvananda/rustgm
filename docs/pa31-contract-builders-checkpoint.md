# Contract builder and source capture checkpoint

The maintainer compiles the actual source contract builder through ordinary
generic calls and constant evaluation, including real function items, pointers
and copied closure captures. It also fixes callable Copy proof consumption and
typed-wildcard reborrow capture. This is a source-provider checkpoint across
PA31/25/24/19/20/26, not a release of those assignments or an original library.

## Declaration and execution authority

Append ContractBuildCheckEnsures/ContractCheckRequires/ContractCheckEnsures at
language ordinals75/76/77. The pinned table specifies function targets and no
fixed generic count. Reject associated/foreign/nonfunction claims while allowing
ordinary nonconst and unsafe function definitions where Rust allows them. Calls
still require their written constness, safety and complete source body proof.
The [source packet](../pa31/tests/spec/contract-helpers/README.md) preserves
original staged const/API headers and independent owner/gate opponents.

build_check_ensures<Ret,C> returns its actual C argument under the real
Fn(&Ret)->bool, Copy and static obligations. It has no replacement body/kernel.
The two tagged check functions also declare actual function identities, but
their genuine rustc_intrinsic fallback/execution and contract attribute lowering
remain required. No unknown intrinsic or builtin expansion is ignored.

## Shared callable marker proofs

The old generic call validator accepted canonical source sizing/tuple/auto
proofs but rejected already selected Copy evidence for function items. An
immutable baseline reproduces this even without any contract tag: the same
ordinary generic function fails with Copy and passes without it. Its Rust
counterparts all pass. The call validator now uses the same authenticated
ordinary Copy/Clone family classifier as open inference goals. It supplies
neither a synthetic implementation nor an executable Clone method.

Source closure markers descend through their actual typed capture tuple and
prove the requested real trait separately for every field in the same caller
and query. This supports concrete, generic and projected fields; it never uses
physical layout as Copy proof. A cursor and retained child nodes avoid prefix
replay. Pending/deferred/partial children, source header/normalization waits,
numeric wakeups, region scopes and proof publication use the existing search
machinery. The existing optional language-condition box owns the cold state;
empty captures allocate no field plan and ordinary search records do not grow.

A contextual reference conversion evaluates its place even when a typed
wildcard discards the result. The capture collector now consumes that actual
conversion as shared/mutable use; unadjusted wildcards remain non-reading
mentions. The previous guard rejected a valid program. Pinned Rust executes
its callback, the immutable old candidate rejects it, and the corrected native
program prints7 9. Copy of a closure storing a mutable reference stays rejected.

## Qualification and complexity

- Private29 passes171.86s with strict all/pedantic Clippy, formatting and the
  syntax-aware architecture audit. Nine prior callable marker/formation/debt
  controls, old source For/Try/budget checks and eight cold layout owners pass.
- Production27 closure/safety tests and PA24 482/482 pass125.66s.
- Thirty-eight packet programs pass76 uncapped/capAllow CLI checks;
  eight genuine exported-root LowIR programs execute through the locked CPPGM
  backend and C observers. All expected native outputs match pinned Rust.
- Generic/projected capture and missing-bound controls also match
  independent Rust using the exact same immutable CLI. Their semantic checks
  retain Copy and static assumptions without introducing a new build/cache.
- Full harness/upstream checks pass; no backend implementation/contract changes.

On the frozen same-type capture corpus, widths8/32/128 take1772/3740/11612
inference steps and537/1089/3297 canonical steps:1116+82 per field and353+23
per field respectively. Selections stay12. Both dimensions pass the paired
bounded-growth check. These are counter/scaling observations, not a wall-time
speedup claim. Search Frame1312 and optional language payload96 are measured;
Node36, NodeFacts80, Operation16 and Body456 remain unchanged. Three cold slots
change Catalog1696→1752 and Reservations2392→2456; provider1560/imported1320
remain comparable fixed tables. Every owning cold assertion names actual values.

Evidence under obj/parallel/source-contract-helpers-v1:

| Observation | Receipt SHA-256 |
| --- | --- |
| Original generic item/closure/const native7 | 5895cbf90718c98ba9cb4e217a4b31fbe4c272cd2693398cd73e1e93297f33e8 |
| Staged headers/captured fields/native7 | 3218a6ea68108ad91c301fa2724367643f2bd513648071e732f7a3d0df69be8e |
| Actual typed reborrow/callback/native7 9 | 35958bb1dc24955c02c37371208f58b9bfcca76f82308f26e3918ec97f67ea58 |
| Function target/safety outcomes | 0da8a3bfadc51829913be0d9c705f8448e5a49cdf73024b10d4c8abb65e85574 |
| Symbolic capture candidate outcomes | 951beb6955e5482f2820fdfa1360642380680f247330c3588e733382e9a5bd9a |
| Joined CLI/native/scaling/harness/original core | 573e94f746180f4071000ed30dfdcb10ad5088296dd4cf4cda7c0cc5499e9dee |

Private1793653527336350347 hashes to
10994942b810f1179b93ac2a3a7ec05d9cbe4e39cb7af22bea0972154777a296;
production1793653881692713481 to
9bb477b853a43cf0ffccbdbe47ee106fc54f1396c60d6c4b8f9bf9eabfbc40fc.
Raw failed hypotheses, diagnostics, complete input/reference bindings and current
immutable CLI are retained; diagnosed generated objects/metadata are pruned.

## Original source and next blocker

CLI035e3b48c8da9ca39f64f62bb69d0dec4da486285b067a472ebb4384a9b9fbe1
clears the original contract builder binding. The new attempt takes16.89s with
375 loading opens and all722 hashes unchanged, then reports file7:357 diagnostic
item requires one string literal. The opening map identifies
num/shells/legacy_int_modules.rs11:
rustc_diagnostic_item=concat!(stringify!($T),"_legacy_mod"). LIB-120 owns proper
attribute-expression macro expansion and actual diagnostic-string authority.
Target/config bindings, arguments, capAllow and finite work/storage stay fixed.

Contract check intrinsic execution, contract macro transformations, imported
bodies/values/Clone/API authority and the complete original core/alloc/std plus
independent consumer ladder remain required. No original library artifact or
additional whole assignment is declared complete. Root continues alone in the
same btrfs tree with bounded warm caches,2threads,16GiB output caps,150GiB reserve
and five-minute local checks. Prior public9950910f0 full CI41514507772 is terminal
success; this successor's full CI is tracked independently after publication.
