# First unchanged whole-core source census

This is an original-source failure census, not a core build or an assignment
release. No compiler source, upstream source, configuration or feature gate was
changed. The candidate is the exact `5fb7bb86` debug behavior pin, SHA-256
`11a4e5f26adfd7bddae113684c6f52a3573cc8a02ab90eb3f3ddc4bc878d49c6`.
There is no performance claim for this binary.

## Shared immutable sources

The single shared tree is
`/home/vishvananda/rustgm-sources/rust-1.98.0-88d9e12ae178fab0fb5cc050a94da85685d449ea`.
It contains the complete Rust 1.98.0 superproject at
`88d9e12ae178fab0fb5cc050a94da85685d449ea`. Reconstructing Git blob/tree objects
from every archive entry, executable mode and gitlink produced the exact commit
tree `b10ad5e3f8014a7d88a67cb98273459a0d77e3ce`. This verifies source identity,
not just a locally computed archive checksum. The archive SHA-256 is
`5a9993673ce40804e6b50300df59ecb3d13942c1c89cd11e76acf705b331f982`.

All 60,726 superproject files (220,775,833 content bytes), `COPYRIGHT`, both
licenses, manifests and lockfiles are retained. stdarch, portable-simd and
compiler-builtins are tracked directories at this revision. Core has no Cargo
dependencies. The sole library submodule, backtrace, was also acquired at exact
commit `d902726a1dcdc1e1c66f73d1162181b5423c645b` and verified against tree
`7f55c64f163ee354275bc92f21496620737401eb` (102 files, 358,116 bytes).
Other docs/tool/compiler-backend submodule gitlinks are recorded but their
contents are not materialized: none is a dependency of this core invocation.
This is not a recursive LLVM/GCC/compiler bootstrap checkout.

The tree is read-only (444 ordinary files,555 executable files/directories;
symlinks preserved). Allocated source disk use was 424,708 KiB; downloaded
archives/API receipts/per-file SHA-256 manifests were 48,884 KiB, about 463 MiB
together. Per-file hashes, URLs, exact gitlinks and source provenance are in
`/home/vishvananda/rustgm-sources/provenance/rust-1.98.0-88d9e12/manifest.json`
and its sibling JSONL files. No per-worker source copies were made.

## Actual reached result

The unchanged invocation was:

```sh
FINAL_PIN --emit-types --edition=2024 --crate-name core \
  --crate-disambiguator core_1_98_88d9 \
  --source-core core@core_1_98_88d9 SHARED_TREE/library/core/src/lib.rs
```

It terminates 1 with exactly:

```text
ERROR: file 0: 4062: unsupported active source-core feature
```

This is the first root request, `asm_experimental_arch`, at line 94. The
source-provider feature policy currently recognizes only `no_core`, `lang_items`,
`intrinsics` and `rustc_attrs`, each with the limited implemented contracts in
[source layout provider](pa31-source-layout-provider.md). No gate was added to
advance the census. This diagnostic is an admission boundary; it is not proof
that experimental-architecture assembly is required on the selected x86-64 target.

A file-access trace proves that the candidate opened only `library/core/src/lib.rs`
from the shared source tree. The original root independently passes lexing
(2,745 tokens), literal analysis (79), token trees (1,643 trees/204 attributes)
and configured crate syntax (225 nodes/85 items/all 225 active/208 attributes).
The name view stops at the same provider policy. No out-of-line module,
macro expansion, source binding reservation, completed names/types, body proof,
CTFE, layout, LowIR or native code was reached. The471 Rust files under core/src,
core_arch/src and core_simd/src have explicit root-only/unvisited records;
unvisited files have no acceptance or implementation judgment.

Pinned Rust 1.98 independently compiled the **unchanged whole crate** to metadata
successfully in 16.1 seconds, with empty stdout/stderr:

```sh
RUSTC_BOOTSTRAP=1 rustc --edition=2024 --crate-name core --crate-type=lib \
  --target x86_64-unknown-linux-gnu -Cpanic=abort -Cdebug-assertions=no \
  --emit=metadata -o core.rmeta SHARED_TREE/library/core/src/lib.rs
```

The generated metadata was pruned by the storage wrapper. This establishes a
source-valid reference outcome, not RustGM acceptance or native execution.
No source-invalid or ICE outcome occurred in this whole-crate attempt. The
separate `--count-expanded-crate --source-core ...` attempt rejected its CLI
combination with status 2; it is retained as a usage error, not a source blocker.

## Configuration and static inspection boundaries

RustGM used its unchanged fixed Config: Unix; x86_64/Linux/GNU/unknown vendor;
little-endian 64-bit; empty target ABI; panic=abort; FXSR/SSE/SSE2;
`target_has_atomic` and `target_has_atomic_primitive_alignment` values 8/16/32/64/ptr.
No custom cfg, Cargo features, test/doc, debug-assertion or sanitizer flags were
supplied. Unknown cfg currently evaluates false. The actual config source hash
and full pinned-rustc `--print cfg` output are retained. These are not identical
cfg producers: rustc additionally reports x87, atomic load/store and bare atomic
flags, reliable f16/f128, ELF/thread-local and other compiler facts. Their precise
ownership/capability semantics remain a required compatibility contract; the
reference invocation was not edited to hide those differences.

The following inventories are **static source inspection** or explicitly
isolated file-configuration controls. They do not change the whole-core reached
set. The root has 112 explicit crate attributes: 90 feature requests, three doc,
four deny, six warn, five allow, and one each stable/no_core/
rustc_coherence_is_core/rustc_preserve_ub_checks. There are 72 module declarations:
69 out-of-line and three inline; no disabling root module cfg, consistent with
all root AST nodes being active. Three final include! invocations target
attribute_docs.rs, keyword_docs.rs and primitive_docs.rs and were not expanded.

Feature requests are not 90 missing implementations:

**Library (16).** `asm_experimental_arch`, `bstr_internals`, `cfg_target_has_reliable_f16_f128`, `const_carrying_mul_add`, `const_cmp`, `const_destruct`, `const_eval_select`, `const_select_unpredictable`, `core_intrinsics`, `coverage_attribute`, `disjoint_bitor`, `io_const_error`, `offset_of_enum`, `panic_internals`, `pattern_type_macro`, `ub_checks`.

**Language (59).** `abi_unadjusted`, `adt_const_params`, `allow_internal_unsafe`, `allow_internal_unstable`, `auto_traits`, `cfg_sanitize`, `cfg_target_has_atomic`, `cfg_ub_checks`, `const_closures`, `const_precise_live_drops`, `const_trait_impl`, `decl_macro`, `deprecated_suggestion`, `derive_const`, `diagnostic_on_const`, `diagnostic_on_unmatched_args`, `doc_cfg`, `doc_notable_trait`, `extern_types`, `f16`, `f128`, `field_projections`, `final_associated_functions`, `freeze_impls`, `fundamental`, `funnel_shifts`, `impl_restriction`, `intra_doc_pointers`, `intrinsics`, `lang_items`, `link_cfg`, `link_llvm_intrinsics`, `macro_metavar_expr`, `macro_metavar_expr_concat`, `marker_trait_attr`, `min_specialization`, `multiple_supertrait_upcastable`, `must_not_suspend`, `negative_impls`, `never_type`, `no_core`, `optimize_attribute`, `pattern_types`, `pin_macro_internals`, `prelude_import`, `repr_simd`, `rustc_attrs`, `rustdoc_internals`, `simd_ffi`, `staged_api`, `stmt_expr_attributes`, `strict_provenance_lints`, `trait_alias`, `transparent_unions`, `try_blocks`, `uint_carryless_mul`, `unboxed_closures`, `unsized_fn_params`, `with_negative_coherence`.

**Target (15).** `aarch64_unstable_target_feature`, `arm_target_feature`, `avx10_target_feature`, `clflushopt_target_feature`, `hexagon_target_feature`, `loongarch_target_feature`, `mips_target_feature`, `movrs_target_feature`, `nvptx_target_feature`, `powerpc_target_feature`, `riscv_target_feature`, `rtm_target_feature`, `s390x_target_feature`, `wasm_target_feature`, `x86_amx_intrinsics`.

`no_core`/`lang_items`/`intrinsics`/`rustc_attrs` have implemented source subsets.
Never types and retained statement/expression attributes already have ordinary
compiler mechanisms; their exact unstable source admission still needs scoped
qualification. Ordinary macro_rules repetitions do not establish `${...}`
metavariable expressions or concatenation. Bootstrap callable execution,
Copy/Clone properties, derives and builtins are existing mechanisms, but are not
permission to replace actual source trait/macro declarations. Const-trait
modifiers, const callable evidence, specialization/negative coherence, source
auto traits, external-type sizing, new ABIs and vector/assembly contracts require
specific compiler work; their presence is not waived by an unused public API.
Doc/diagnostic/stability and optimization attributes need explicit effect
classification rather than being conflated with those semantic obligations.

Isolated parsing/configuration of the **original** core_arch module, x86 module
and core_simd module all succeeds under unchanged Config. In core_arch/mod.rs,
only external `macros`, `simd`, `x86` and `x86_64` module declarations are active;
test, ARM/AArch64, RISC-V, LoongArch, wasm, MIPS, PowerPC, NVPTX/AMDGPU,
S390x and Hexagon module declarations are disabled. This is configuration
inspection of those files, not loading them through the blocked core build.
The original rustc asm lowering explicitly lists X86_64 as stable assembly
(`compiler/rustc_ast_lowering/src/asm.rs:40–54`): the **experimental architecture**
gate has no semantic demand here. Target gates for those disabled architectures
can be classified inactive for this target with their source evidence retained;
that does not implement them globally. x86 target-feature declarations and
portable SIMD remain active source contracts. In particular core_arch's simd.rs
uses repr(simd), x86/sse2.rs has target_feature, and core_simd/vector.rs uses
repr(simd,packed). No vector/asm backend support is established by parsing the
module declarations, and no backend change is proposed in this census.

## Next dependency-ordered compiler work

1. Preserve this first failure while defining demand-aware source feature and
   attribute capability states. Separate a requested gate, an inactive configured
   consumer, a supported semantic construct and a still-unsupported active use.
   Begin with known mechanisms and source identity; never bulk-enable 90 names.
   Source stability, prelude_import, rustc_coherence_is_core and UB-preservation
   effects need explicit consumers. Keep current cfg facts/provenance independent.
2. Generalize actual Sized→MetaSized→PointeeSized formation and layout reflection
   through existing preparation/proof continuations. The current primitive-only
   kernel/nonempty-proof boundary must transport and pay real debts for arrays,
   tuples, records, references, generic callers and DST/extern distinctions.
   Use actual mem::SizedTypeProperties's SIZE/ALIGN defaults/blanket impl and
   original wrapper calls, not another library body factory.
3. Qualify runtime **public mem wrapper** acceptance via those associated constants.
   The original intrinsics/mod.rs:2781–2788 and2799–2807 document the size/align
   kernels as compile-time-only; mem/mod.rs:373/540 returns the associated constants.
   Runtime monomorphized public wrappers should contain evaluated constants.
   Do not invent backend kernel behavior or silently execute fallback 0. Preserve
   direct runtime/reification unsupported evidence and prior host ICEs separately.
4. Bind actual Copy/Clone, Tuple, Fn/FnMut/FnOnce and FnOnce::Output declarations
   to existing typed property/call selection, retaining exact Self/Args/binders,
   source clauses/competition, member contracts and rust-call ABI. The source
   `pub const trait`/`[const]`/`const FnOnce` protocol and Destruct/const permissions
   are prerequisites, not a synonym for ordinary callable evidence. This unlocks
   C's exact const_eval_select signature/UB-check consumer; it cannot proceed by
   spelling or by reusing a layout Selection as callback permission.
5. Register real builtin macro/derive declarations (macros/mod.rs, arch.rs) against
   existing typed expansion engines where they exist; keep source bodies, lexical
   shadowing, actual DefIds and expansion permissions. Add declarative-macro2
   syntax/hygiene and missing builtin semantics separately. Existing Copy/Clone
   marker/property validation is not executable derive coverage.
6. Track remaining active internal features and platform ABI deliberately:
   const traits/drop, auto/negative traits, specialization, primitive f16/f128,
   target_feature, SIMD layout/operations and stable x86 assembly. Imported unstable
   modules still require parsing/typing/layout; demand-only codegen is not blanket
   permission to omit active source semantics. Keep non-x86 work target-scoped.
   Generic crate transport remains independent and necessary for consumers.

The immediate successor is a bounded width/depth/deferred-annotation probe of
`closure_annotation`, requested during review, before expanding bindings. Its
finish scans only body_regions[first..] and filters actual owner identity.
Sequential closures skip old prefixes and pure memo repeats scan empty suffixes,
but nested dependencies can append foreign-owner regions inside every open
interval: the sum of visits may be depth×regions. The loop currently does not
charge individual visits. This is a static risk, not a measured regression.
The successor must declare/qualify real source outcomes, count visits including
errors, and replace repeated foreign scanning with owner-indexed accounting if
confirmed; adding a quota around quadratic work is not the proposed repair.

## Exact root module source inventory

Every entry below is enabled at the root declaration level. An out-of-line path
was not opened by the whole-core candidate; an inline entry was only parsed.

| Module | Original source location |
| --- | --- |
| prelude | `library/core/src/prelude/mod.rs` |
| macros | `library/core/src/macros/mod.rs` |
| from | inline in root |
| autodiff | inline in root |
| offload | `library/core/src/offload/mod.rs` |
| contracts | `library/core/src/contracts.rs` |
| internal_macros | `library/core/src/internal_macros.rs` |
| legacy_int_modules | `library/core/src/num/shells/legacy_int_modules.rs` |
| f128 | `library/core/src/num/f128.rs` |
| f16 | `library/core/src/num/f16.rs` |
| f32 | `library/core/src/num/f32.rs` |
| f64 | `library/core/src/num/f64.rs` |
| num | `library/core/src/num/mod.rs` |
| hint | `library/core/src/hint.rs` |
| intrinsics | `library/core/src/intrinsics/mod.rs` |
| mem | `library/core/src/mem/mod.rs` |
| profiling | `library/core/src/profiling.rs` |
| ptr | `library/core/src/ptr/mod.rs` |
| ub_checks | `library/core/src/ub_checks.rs` |
| borrow | `library/core/src/borrow.rs` |
| clone | `library/core/src/clone.rs` |
| cmp | `library/core/src/cmp.rs` |
| convert | `library/core/src/convert/mod.rs` |
| default | `library/core/src/default.rs` |
| error | `library/core/src/error.rs` |
| field | `library/core/src/field.rs` |
| index | `library/core/src/index.rs` |
| marker | `library/core/src/marker.rs` |
| ops | `library/core/src/ops/mod.rs` |
| any | `library/core/src/any.rs` |
| array | `library/core/src/array/mod.rs` |
| ascii | `library/core/src/ascii.rs` |
| asserting | `library/core/src/asserting.rs` |
| async_iter | `library/core/src/async_iter/mod.rs` |
| bstr | `library/core/src/bstr/mod.rs` |
| cell | `library/core/src/cell.rs` |
| char | `library/core/src/char/mod.rs` |
| ffi | `library/core/src/ffi/mod.rs` |
| io | `library/core/src/io/mod.rs` |
| iter | `library/core/src/iter/mod.rs` |
| net | `library/core/src/net/mod.rs` |
| option | `library/core/src/option.rs` |
| os | `library/core/src/os/mod.rs` |
| panic | `library/core/src/panic.rs` |
| panicking | `library/core/src/panicking.rs` |
| pat | `library/core/src/pat.rs` |
| pin | `library/core/src/pin.rs` |
| process | `library/core/src/process.rs` |
| random | `library/core/src/random.rs` |
| range | `library/core/src/range.rs` |
| result | `library/core/src/result.rs` |
| sync | `library/core/src/sync/mod.rs` |
| unsafe_binder | `library/core/src/unsafe_binder.rs` |
| fmt | `library/core/src/fmt/mod.rs` |
| hash | `library/core/src/hash/mod.rs` |
| slice | `library/core/src/slice/mod.rs` |
| str | `library/core/src/str/mod.rs` |
| time | `library/core/src/time.rs` |
| wtf8 | `library/core/src/wtf8.rs` |
| unicode | `library/core/src/unicode/mod.rs` |
| future | `library/core/src/future/mod.rs` |
| task | `library/core/src/task/mod.rs` |
| alloc | `library/core/src/alloc/mod.rs` |
| bool | `library/core/src/bool.rs` |
| escape | `library/core/src/escape.rs` |
| tuple | `library/core/src/tuple.rs` |
| unit | `library/core/src/unit.rs` |
| primitive | `library/core/src/primitive.rs` |
| core_arch | `library/stdarch/crates/core_arch/src/mod.rs` |
| arch | `library/core/src/arch.rs` |
| core_simd | `library/portable-simd/crates/core_simd/src/mod.rs` |
| simd | inline in root |

## Retained evidence

Worker `obj/parallel/pa31/whole-core-v1/` contains acquisition/probe/inventory
scripts, raw original metadata, inventory.json (all 90 gates, 112 attributes,
72 modules, phase states and source hashes), per-file visitation JSONL,
`census/{results.json,provenance.json,*.stdout,*.stderr,*.exit_status,whole-types.openat}`
and isolated `configured-inspection/` outputs. Every acquisition/compiler probe
used the existing storage/resource wrapper; both functional batches terminated0.
No full Cargo/active-text suite or performance measurement was repeated. Shared
source acquisition was bounded to 512 MiB per download/3 GiB expanded data and
200,000 archive members; actual source use is recorded above.
