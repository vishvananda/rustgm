# Source-owned primitive layout provider

This follows the declaration-only foundation in
[pa31-source-binding-foundation.md](pa31-source-binding-foundation.md). The first
consumer uses actual source `Sized`, `MetaSized`, `PointeeSized`, `size_of`,
`align_of`, `mem_size_const` and `mem_align_const` declarations. It does not
complete PA31 or compile upstream core. The source pin is Rust 1.98.0 commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`, edition 2024.

## Identity, declarations and evidence

The separate semantic catalog is allocated only for the explicitly configured
source provider. Real language-item DefIds remain singletons; intrinsic DefIds
remain distinct even when they implement the same operation. Reservation is not
validation. Complete sizing headers and intrinsic signatures are validated
before the corresponding queries can produce a result. Actual
`rustc_deny_explicit_impl` attributes restrict source implementations; merely
naming a trait `Sized` does not imply that restriction.

Source `MetaSized` and `PointeeSized` remain distinct actual trait applications.
Explicit written sizing bounds control implicit Sized/default Meta clauses.
PointeeSized alone does not imply MetaSized, and an explicit PointeeSized+Sized
parameter recovers MetaSized through the proved Sized/hierarchy route. Existing
scoped assumptions and source candidates precede the limited builtin formation
fallback. Deferred source candidates remain pending. Nonempty returned formation
proofs are diagnosed as an unsupported continuation in this first profile; they
are never discarded to publish an answer. Extern types and richer headers outside
the profile remain required source-valid work where pinned rustc accepts them.

The first layout signature is a safe Rust `fn<T: Sized>() -> usize`, with no late
lifetimes or occurrences of T in its value signature. The full selected Instance
must retain the validated original signature and exactly one closed type argument.
That signature equality is a property of this limited profile, not a general
intrinsic substitution rule. Registered layout language items must be actual
associated usize constants, validated against the actual member/parent identities
before completed declaration publication. Their trait selection, defaults and initializer execution
use the ordinary source provider; neither the trait nor member spelling selects
an evaluator.

## Reached execution and function preparation

The existing FunctionBody dependency reaches the layout consumer. A fresh witness
checks the current retained Code, frame, statement, evaluated callee, complete
Instance, signature, result, empty value arguments and actual const eligibility.
It is checked before every result-cache read. The optional Instance-to-u64 map
stores only pure layout data. It stores no caller proof or permission. A fallback
body still requires ordinary original and specialized body, region and loan proof,
even when the layout kernel substitutes its result.

The first kernel accepts closed primitive types other than str and uses the
existing layout queries. Rich-layout readiness, proof-tail continuation and actual
runtime intrinsic lowering remain required. Reification rejects by actual identity.
Reached runtime calls, including bodyless declarations and legal fallback bodies,
report that source layout intrinsic runtime lowering is not implemented. A normal
source function named size_of retains ordinary behavior. Catalog presence only
keeps runtime dependency selection active; it grants no execution authority.

Demanded const functions prepare their annotations through the existing declaration
worklist. Begin/Prepared/Resume replaces an overloaded boolean without another
readiness table. The dependency scanner applies each node's actual local context
and stops at nested item owners, whose existing declaration demands own their
binders. A shared whole-closure continuation finishes input annotations before
result annotations. Newly created input regions must belong to that exact closure;
unrelated dependency regions are excluded, and repeated completion preserves
previously recorded inputs. Completed visits are memoized once. Missing suspended state remains an
error. Ordinary-context closure, nested generic function, nested constant and
lifetime-bearing inline-constant controls retain these ownership boundaries.
Source closures currently diagnose the missing actual Copy/callable bindings when
those roles are demanded, rather than borrowing bootstrap identities.

## Qualification and retained limits

The evidence root is `obj/parallel/pa31/source-bindings/` in the PA26 worktree.
`qualified-v3` retains the original 45-case independent source matrix. Later v4-v9
receipts preserve corrections to parser, sizing and closure hypotheses. In
particular, direct runtime intrinsic object emission can ICE in pinned rustc while
metadata compilation accepts the same source; this is a host-codegen boundary,
not a Rust language rejection. A fallback with an invalid local-to-static borrow
has an independently verified E0597 control. An earlier uninitialized fallback
instead failed for missing Copy and is not claimed as loan coverage.

`layout-v11` passes strict all-target/all-feature Clippy, fourteen source catalog
and provider groups, the warmed/wrong-current-call invariant, three existing
execution-storage groups, four execution/closure-readiness invariants, two Code
storage groups and three measured ordinary-layout controls. The explicitly ignored
native group was run: both selected source SIZE and a generic const wrapper execute
through pinned rustc objects and CPPGM objects linked to C clients. Audit: 1,849
files, zero findings; layout and 353-root/13-binary inventory pass. The optimized
pin's CLI also evaluates the original early generic wrapper to the declared value2.

`qualified-v11` confirms that unused u32/bool language-item members are accepted by
pinned rustc. They are unsupported shapes for this initial usize provider profile,
not language-invalid negatives. Ordinary unregistered members of those types pass.
Pending member types and a forged implicit-Self parent cannot validate a contract.

`qualified-v12` independently settles three source-mode primitive MAX missing-
definition rejections plus nominal/qualified-trait MAX positives. The original
v11 optimized pin wrongly supplied builtin values for all three missing definitions;
its unchanged output/diagnostics are retained. The final gate makes
`declarations/primitive_constants::primitive_constant_path` return None in explicit
source-core mode, so both scheduling and publication use ordinary associated
resolution. The existing constants/body/pattern/match readers cannot acquire a
builtin Uses entry through that path. `layout-v12` passes all five source CLI
outcomes, the owning source invariant, all four existing bootstrap MAX invariant
groups (including linear 8/32/128 work), strict Clippy, audit/layout/inventory.
No primitive inherent constant implementation was added.

Measured storage is Package272, Resolution2160, Declarations8576 and execution
Catalog208 bytes: one cold8-byte pointer added to each of their respective prior
owners. The semantic source catalog is200 bytes and absent on ordinary inputs;
source reservations are240 (the foundation's192 plus a source-only48-byte denied-
implementation set). Code336, Body464, Value24, Kind24, inference Context/Store448,
declaration Work80, Checker12952 and Phase1 remain compact. The old Range8552-byte
Declarations receipt remains historical: the public Option policy/layout caches
added16 bytes, then this source catalog added8. No Type/Body/Call/Value schema grew.
Full combined release, active-view comparison and performance remain integrator
work; no assignment was activated and no references were regenerated.

## Immutable pins and original source census

Final exact source-behavior pin (debug profile, not a performance binary):
`obj/parallel/pa31/source-bindings/layout-v12/candidate/rustgm`, SHA-256
`11a4e5f26adfd7bddae113684c6f52a3573cc8a02ab90eb3f3ddc4bc878d49c6`.
Its source-map.json and source.tar sit beside the candidate directory. The earlier
optimized v11 pin, before the MAX correction, is retained separately with SHA-256
`505d7fa9ac614792267b609482843f108524a89495a514051aa6b7e6eb2b8311`.
It must not be represented as the final source-mode MAX behavior. The integrator
will build the combined optimized candidate once.

No whole-core census was run here. `source-pin-v1/manifest.json` records the original
core root/marker/mem/intrinsics bytes, URLs and hashes, not a complete module tree.
With a complete unchanged checkout of the pinned revision, the minimal semantic
invocation is:

```sh
rustgm --emit-types --crate-name core --crate-disambiguator core_1_98_88d9 \
  --source-core core@core_1_98_88d9 /FULL_TREE/library/core/src/lib.rs
```

The current target cfg remains x86_64, Linux, GNU, little-endian,64-bit, panic-abort,
FXSR/SSE/SSE2 (`expand/config.rs`); retain the actual cfg in the census receipt.
Do not use the four-file inventory as a substitute module tree or switch back to
bootstrap identities to get past the first unsupported source contract.

Remaining source-binding work includes actual Copy/callable/Tuple roles, general
sizing/formation continuations, richer layouts, runtime layout lowering, additional
intrinsic kinds, source-sysroot metadata and upstream core/alloc/std acceptance.
This checkpoint adds no ordinary library method factories or metadata wire format.
