# Pinned upstream library compiler contracts

This selected-source investigation informs the
[implementation order](library-bootstrap-roadmap.md). It is not a successful
core/alloc build, a complete cfg expansion or a claim that every named feature
requires immediate machine-code support. The source pin is Rust 1.98.0 commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`; local 1.95 sources were not substituted.

The bounded collection contains 63 successful files, 3,058,732 bytes and three
retained failed path guesses. The [source receipt](library-bootstrap-sources.json)
records URLs and checksums; six critical files were additionally checked against
pinned Git blob identities. Selected-file indexing found 90 core and 119 alloc
unconditional root feature gates, 229 main and 75 SIMD intrinsic annotations,
54 attribute names and 71 lang-item occurrences. These are syntactic counts,
not the number of missing implementations or a complete target feature census.
Macro-generated and cfg-dependent requirements still need expansion analysis.

Detailed indexes, source files, failed requests, pinned rustc version/cfg output
and backend receipts remain under the inventory worker's ignored
`obj/parallel/library-bootstrap-inventory/`. No compiler/native build or
benchmark was run for this investigation. The frontend comparison used worker
checkpoint `13df7b2a`, whose Option layout work awaits combined qualification;
literal-name search alone does not establish a source rejection or its phase.

## Source contracts that change the implementation plan

| Contract | Source evidence and consequence |
| --- | --- |
| Internal language features | The [core root](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/lib.rs) and [alloc root](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/alloc/src/lib.rs) request const traits, specialization, negative/auto traits, internal attributes and other unstable features. Classify active requirements; accepting the attribute syntax does not implement their semantics. |
| Sized and layout properties | [Marker declarations](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/marker.rs) distinguish Sized, MetaSized and PointeeSized. [Memory properties](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/mem/mod.rs) use SizedTypeProperties and compiler-provided associated constants for size/alignment. Reusing one ordinary Sized identity or only old intrinsic wrappers would miss the actual protocol. |
| Intrinsic declarations | [Core intrinsics](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/intrinsics/mod.rs) attach compiler meaning to typed declarations, including bodyless forms. Preserve exact identity/signature, safety, const permissions and fallback rules. rustc_comptime adds a separate compile-time-only permission, distinct from const fn and runtime-only calls. const_eval_select selects checked function items; [UB checks](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ub_checks.rs) depend on that distinction. |
| Validity and niches | [UnsafeCell](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/cell.rs), [MaybeUninit](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/mem/maybe_uninit.rs), [NonNull](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ptr/non_null.rs) and [niche types](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/num/niche_types.rs) require source identity, initialization and validity semantics. This pin uses pattern types for valid-range storage; a raw-pointer stand-in loses guarantees. |
| Pointer metadata | [Pointee and DynMetadata](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ptr/metadata.rs) require typed metadata extraction/raw-parts construction, with separate data identity and access permissions. |
| Generic atomics | [Atomic declarations and aliases](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/sync/atomic.rs) use Atomic<T: AtomicPrimitive>, associated Storage, target alignment and internal const ordering parameters. AtomicBool/AtomicPtr/integer names are aliases; adding unrelated hardcoded nominal names would diverge from source. [Arc](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/alloc/src/sync.rs) needs Atomic<usize> and fences. |
| Allocation and ownership | [Allocator entry points](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/alloc/src/alloc.rs) use Rust ABI, typed Alignment/NonNull arguments, generated __rust_* symbols and OOM routing. [Box](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/alloc/src/boxed.rs) has compiler ownership/write hooks; [RawVec](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/alloc/src/raw_vec/mod.rs) relies on layout, overflow, ZST, raw move/drop and uninitialized spare capacity. C malloc aliases alone are insufficient. |
| Abort runtime | [Core panic routing](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/panicking.rs) still needs panic_impl identity, const panic and nounwind semantics. [Foreign memory declarations](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ffi/mod.rs) require exact symbols and zero-byte semantics. Full unwind cleanup can follow the initial abort profile. |

## Target and backend boundaries

The pinned [x86-64 target](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_target/src/spec/targets/x86_64_unknown_linux_gnu.rs)
has maximum atomic width 64. Saved target facts include 8/16/32/64/ptr atomics;
128-bit declarations are conditional. Internal
[cfg generation](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_session/src/config/cfg.rs)
also supplies target_has_atomic_load_store facts hidden by stable --print cfg.
Record those facts and the selected panic/overflow/debug/UB profile explicitly.
The saved toolchain default panic=unwind output does not select bootstrap policy.

Core directly imports stdarch/portable-simd modules. The selected
[SSE2 source](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/stdarch/crates/core_arch/src/x86/sse2.rs)
uses target_feature, and [vector storage](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/portable-simd/crates/core_simd/src/vector.rs)
uses repr(simd,packed). Unstable API status does not remove imported modules.
Parsing/typing/layout and lowering demanded instructions are distinct milestones;
neither every architecture's runtime support nor skipping all unused SIMD is
established by this investigation.

Alloc's [manifest](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/alloc/Cargo.toml)
includes compiler-builtins. Freeze its selected source/features and the runtime
symbol provider. All selected Rust target code must use RustGM. Native runtime
services must be explicit; prebuilt target Rust or silently omitted dependencies
cannot satisfy this roadmap. At this Rust commit the inspected stdarch,
portable-simd and compiler-builtins paths are ordinary directories, not gitlinks.

The frozen CPPGM LowIR vocabulary at commit
`905bd7e4412a2d77d88694fe6abc7f3e1c27bce1` has scalar/object types and F80, but
no F16/F128/vector type or arbitrary inline-assembly operation. That establishes
a direct vocabulary mismatch, not impossibility of scalar emulation or a future
typed extension. F80 is not Rust f128. Preserve the frozen backend boundary and
qualify any proposed route separately before making a workload claim.

Prior execution evidence covers scalar volatile 8–128, f32/f64, thin pointers
and U32 atomic load/store/exchange/CAS/add-fetch. Other widths, pointers and fences
are unqualified here, not demonstrated impossible. LowIR CAS result handling
differs from Rust intrinsics and needs the checked adapter. Bulk copy is not a
substitute for volatile semantics. No new backend qualification occurred during
this inventory; full std/platform and transitive crate analysis remain separate.

## Immediate implementation slices

1. Bind validated source intrinsic/lang-item declarations and the self-core/no_core
   namespace. Test spoofed/duplicate/wrong-signature declarations and replace
   synthetic identities when compiling actual sources.
2. Connect size_of/align_of/needs_drop and mem_size_const/mem_align_const to
   completed layout/destruction facts and selected associated constants. Use
   original pinned excerpts with generic, const, runtime and alias controls.
   This proposed slice needs no new backend opcode; it is not yet implemented.
3. Add const_eval_select/UB-check and ordinary pointer/memory paths, then actual
   UnsafeCell/MaybeUninit/NonNull and generic Atomic<T> contracts. Probe remaining
   backend widths independently. Generic crate-body transport proceeds in parallel.

These priorities retain all PA27/PA28 requirements, including the original
valid-but-unsupported default-tag/multi-variant enum cases. Option niche progress
does not establish general enum layout or successful core/alloc/std compilation.
