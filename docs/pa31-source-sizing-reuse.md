# Closed source-sizing proof reuse

The frozen whole-caller alias family exposed a boundary the isolated Freeze
query did not exercise. `need::<Layer128>()` is independently Rust-valid and
returns native 7, but the frozen candidate exhausted the unchanged type-inference
budget. Root attribution found repeated `nominal_requirements` → `prove_bound`
→ `prove_canonical_trait` source-sizing prefixes. The earlier direct query had
already shared formation nodes inside one query; it did not cover these many
canonical roots of the complete typed caller. The failed nested-syntax source,
alias-depth budget failure and original instruction observations remain intact.

This correction uses the existing fixed-caller completed-proof owner for exact
ordinary source MetaSized/PointeeSized goals. The existing `sizing_predicate`
facade validates their actual declarations and completed sizing hierarchy.
Legacy Bound::Sized remains unchanged. A checked completed producer can mark its
node eligible for sharing; only an eligible Builtin child with the identical
canonical goal can carry that eligibility through a proof wrapper. Every sibling
and formation prefix must still be complete and debt-free. This bounded child
summary pass is charged only when reuse exists. The ordinary reuse-None path
keeps its prior accounting.

Only concrete positive results without private regions, retained Sized duties,
provisional sizing, cycles or caller-assumption evidence enter the map. Every
publication, direct hit and iterative hit validates retained provenance and the
current actual sizing role. Pending, missing or changed role state errors; it
cannot become absence or permission. Tuple, Destruct, Freeze and Clone Builtin
results do not originate this eligibility. Formation-only nodes never export as
trait evidence. Reusing a sizing trait answer does not discharge separate body
or nominal lifetime-well-formedness duties.

The existing canonical proof owner now activates for nominal-containing actual
source-sizing roots as well as its prior projected-size roots. No representation
field was added: Query 656, Node 160 and Frame 1312 bytes remain exact. The ordinary
scalar body leaves that optional owner absent. Retained map values share complete
proof outcomes; they do not store flattened prerequisite vectors or caller
execution permission.

The full caller was measured independently of the direct-query controls:

| Semantic depth | Inference work | Canonical work | Map entries | Logical retained map bytes |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 2,691 | 1,100 | 23 | 816 |
| 32 | 10,587 | 4,652 | 95 | 3,168 |
| 128 | 42,171 | 18,860 | 383 | 12,576 |

The corresponding 128 unrelated source declarations change none of these four
observations. Map bytes include the owner and entry capacity, excluding allocator
metadata and immutable shared arenas. These are deterministic work/storage
observations, not wall-time or heap measurements. The
[combined checkpoint](source-calls-sizing-checkpoint.md) now records optimized
CLI measurements and fresh release qualification. These worker counters remain
separate from those wall-time observations; neither establishes whole-core acceptance.

The six owning PA31 development fixtures preserve the exact frozen v2 bytes and
Rust 1.98/x86-64/edition2024 profile. Their hashes and original evidence paths are
in `pa31/tests/spec/source-freeze/formation/typed-depth-contract.json`. They run
through complete typed source checks and normal original/instance, ownership,
loan, LowIR, backend and linked native consumers, returning 7. The native helper
uses its existing self-owned minicore/source_traits_v1 identity; the full typed
work control uses minicore/freeze_perf_v1. Neither exports source-trait metadata
or claims public crate-artifact readiness.

Evidence is under `obj/parallel/pa31/source-sizing-reuse-v1`. Development-v3
passed strict/audit and six focused controls, including full typed 8/32/128,
ordinary no-allocation, unchanged direct-query, projected and retained-debt
controls. Development-v4 corrected only private sizing-role setup and passed the
debt/scope and sibling controls; its empty-provider hypothesis remained an
explicit setup failure at the mandatory sizing-profile gate. Development-v5
uses real reserved and shifted completed provider catalogs, passes the remaining
catalog controls and existing source Freeze family, and executes all six native
programs. The v3 immutable private binary emits the table above. All passed and
failed receipts, before/after source manifests and pin hashes remain available.

The unrelated reached-const prototype is preserved unqualified at 90cc5998 on
its own branch. None of its const capability, header, Machine, State, Engine or
per-use payment changes are part of this correction.
