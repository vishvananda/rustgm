# Source-owned iteration checkpoint

The maintainer now compiles genuine source-owned for-loops without activating
bootstrap iterator metadata. PA26 owns desugaring, PA31 owns source language
claims and PA29 retains authentic imported declaration/body transport. This is
a local source-provider checkpoint, not a whole assignment or library release.

## Implementation

Append Iterator, IteratorNext and IntoIter at language ordinals72–74; old
ordinals remain unchanged. Validate actual source trait/required-method owners.
The pinned compiler allows arbitrary method names, an absent Iterator marker,
and a next method on another trait. Desugaring therefore consumes the actual
next and into_iter tags rather than assuming spellings or a shared trait parent.

Instantiate both real methods using ordinary generic call inference, obligations,
projection normalization and late-bound lifetime opening. Reuse the current
body's source-call context and existing const/unsafe permission checks. Source
Option and Some/None retain real source witnesses, formal payload types and
variant identities/ordinals. The common protocol-plan owner now serves both
Try and for-loops. CFG construction reauthenticates the current body, actual
For AST children, source declarations and completed calls before using the
existing loop/call/borrow/drop builders. The no-bootstrap guard stays intact.

Source-mode name walking no longer creates synthetic iteration/range demands.
Bootstrap iteration continues through its existing registry and proofs. No new
global For map, cache, environment read or source spelling dispatch is added.

## Qualification

- Private23 passes in31.82s after the coherent rebuild; the earlier8-case
  lint/link/test gate took189.84s. Strict all/pedantic Clippy, formatting and
  syntax-aware architecture audit pass.
- Production6, including existing iterator dispatch/clone/visibility, operators
  and capped source controls, plus PA19 1193/1193 pass in205.13s.
- Thirteen independent pinned Rust metadata outcomes and both genuine native
  observers match. Exported-root candidate compilation through the pinned
  CPPGM backend prints7, and prints7 8 9 for both normal exit and break,
  preserving item-before-iterator destruction.
- Twenty-six CLI outcomes pass uncapped/capAllow, along with upstream checks
  and the complete harness. Missing roles/implementations and wrong results
  reject; moved iterators and escaping items fail ordinary ownership/regions.
- The extra generic DropGlue probe used the concrete-only --emit-loans view
  and failed its pre-existing require_concrete_bodies gate. That failed receipt
  remains recorded. Its source declarations use --emit-types; the unchanged
  full generic source executes through real exported-root lowering. No generic
  inspection, metadata artifact or source-client success is inferred.

The [source packet](../pa31/tests/spec/source-iteration/README.md) records hashes,
predeclared outcomes, primary sources and explicit reference prerequisites.
Private source tampering, changed calls, wrong current owners/nodes and failed
payments remain rejected; all six existing source Try controls still pass.

Measured source-for widths8/32/128 use3421/12901/50821 inference steps, exactly
261 +395 per loop, and512/2048/8192 plan bytes. Each canonical plan is64 bytes,
OpenPlan56 and the current context owner88. Syntax Node36, Operation16,
NodeFacts80 and Body456 remain unchanged. The three explicit cold role slots
change Catalog1632→1696 and Reservations2328→2392; provider1500/imported1272
stay comparable fixed arrays. Existing cold assertions now name these measured
values while preserving all ordinary layout and closed-wire guards.

Evidence under obj/parallel/source-iteration-v1:

| Observation | Receipt SHA-256 |
| --- | --- |
| Independent method/variant/native7 | 5ca216a3c10553cfa77a7cb66304a6355024c8543bba9b571e32b25044ce87f5 |
| Independent move/region/native drop | 1aa2509c6d626c079b82cb3c7d7a8ae265be47615cd6326b82fc4b7d4bb8d6f4 |
| Joined26CLI/native/scaling/harness/core | 4ae77b9e1f21dba5706fad2e5c5ebf547f11962b3262116c7be859d97d04acfb |

Private receipt1793648633531146943 hashes to
cb234635c3aceca6c265b31113d3d7233c5a234dde1a211503ad0eb216882ed9;
production1793648683202841872 to
2a012cdba997bfc92dad4042f02236c5dd11a00176e091ca3f51199df059313f.

## Original source frontier and remaining work

Immutable CLI6abca8a0dc03108d178f57902339fecb09a57fbde5fa71a944752ed49f77d8d7
clears LIB-118's source-mode bootstrap reservation. The bound original core
attempt takes25.28s,375 loading opens, all722 hashes unchanged, and stops at
file5:1005 unsupported source language item. The opening map identifies actual
contracts.rs18 contract_build_check_ensures. LIB-119 owns that real generic const
identity helper and its related contract-check functions/feature transformations.
Target hardware, CLI arguments, capAllow, finite source-contract limits and the
prior explicit atomic-config binding remain unchanged; no ad-hoc cfg is added.

Authentic source-client/metadata protocol transport, complete iteration families,
all remaining assignment requirements and original core/alloc/std artifacts
remain required. Local native reducers do not replace that acceptance ladder.
Root continues alone on btrfs with the existing warm caches,150GiB reserve,
16GiB output cap,two threads and bounded scoped artifact cleanup. Prior public
97861a4eb's full CI41503512103 is terminal success; this successor's full CI is
tracked independently after publication.
