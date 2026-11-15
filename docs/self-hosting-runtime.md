# Self-hosting runtime lane tracker

The runtime lane's only living document
([plan](self-hosting-plan.md)). It is updated in place. Generated evidence
stays under `obj/parallel/` and is not committed.

## Demand list

`scripts/demand_list.py` asks pinned rustc 1.98.0 which library function
instances a corpus needs. rustc is an oracle here and nothing it produces feeds
RustGM's frontend.

```sh
python3 scripts/resources.py run --wait-seconds 600 --max-output-gib 16 -- \
  /usr/bin/python3 scripts/storage.py run --max-output-gib 16 --no-prune -- \
  python3 scripts/demand_list.py rustgm m1 m2 m3 m4
```

Add `--no-build` to re-analyse an existing build. The first run downloads the
`rust-src` component of the pinned toolchain into
`obj/parallel/demand-work/rust-src` (its hash comes from the installed channel
manifest) and checks it against `scripts/original_core_manifest.json`: 718 of
722 pinned files are byte-identical and the other four are dotfiles the
component omits. The shared rustup toolchain is not modified.

### Method

- Cargo `-Zbuild-std=std,panic_abort -Zbuild-std-features=` for
  x86_64-unknown-linux-gnu under the course profile of
  `pa34/tests/stdlib-course-profile-outcomes.json`: opt-level 0, no debug
  assertions, overflow checks, panic=abort, default std features only.
- `-Cno-prepopulate-passes` and opt-level 0 keep every instance the
  monomorphization collector demanded as its own v0 symbol, including
  `#[inline(always)]` functions. The linked binary's symbol table (after the
  linker's section GC) is the reachable set.
- Unoptimized LLVM IR gives definition file and line, and the operations in
  reachable functions. `--print=link-args` gives the linked crate set, so
  `panic_unwind`, `proc_macro` and `test` are excluded.
- `scripts/rust_v0_symbols.py` parses v0 symbols structurally. The definition
  path keeps impl disambiguators and drops generic arguments, for example
  `core::option::{impl#0}::map`. It is rustc's own definition-path spelling.
- Each library crate's rustc invocation is replayed with
  `-Zdump-mir=LowerIntrinsics` (metadata only). That lists the bodies that
  exist and the intrinsic calls before and after rustc's intrinsic lowering.

`reach` is `linked` when the binary keeps the function and `unlinked` when only
dead blocks refer to it. rustc at opt-level 0 emits the false arm of a constant
`ub_checks()` branch; LLVM's code generator drops it. RustGM needs the unlinked
set too unless it folds those branches.

### Outputs

Each corpus has a directory `obj/parallel/demand/NAME/`. `.keep-artifacts` in
`obj/parallel/demand/` pins them.

| File | Content |
| --- | --- |
| `instances.tsv` | Every demanded instance, demangled: crate, kind, reach, instance |
| `source-functions.tsv` | Distinct library source functions: crate, kind, reach, definition, file, line, end line, instance counts, one example. The frontier matches census records against this |
| `summary.tsv` | Per crate and module: instances, source functions, closures |
| `files.tsv` | Per source file: demanded, linked and existing functions and closures |
| `statics.tsv` | Reachable statics and thread-locals |
| `runtime-surface.tsv` | Intrinsics, stdarch and portable-simd functions, atomics with widths and orderings, vector, i128 and float operations, `llvm.*` names, thread-locals, inline assembly, variadic calls, undefined (foreign) symbols |
| `receipt.json` | Toolchain, flags, source revision, crate versions and source hashes, binary hash, linked native libraries |
| `checks.json` | Counts and self-checks |

### RustGM corpus (`dev/` at 80229d28d, binary `rustgm`)

309,093 instances: 308,704 linked and 389 unlinked. By defining crate: core
174,441, rustgm 47,543, alloc 40,442, hashbrown 38,600, std 7,747,
compiler_builtins 113, rustc_demangle 94, unicode_normalization 51, tinyvec 43,
`__rustc` 14, libc 3, unicode_ident 2.

| Crate | Functions demanded (linked) | Functions that exist | Closures demanded | Closures that exist |
| --- | ---: | ---: | ---: | ---: |
| core | 2,471 (2,452) | 24,202 | 135 | 478 |
| alloc | 710 (707) | 2,607 | 54 | 244 |
| std | 737 (737) | 4,191 | 102 | 418 |
| hashbrown | 218 | 666 | 19 | 73 |
| compiler_builtins | 85 (83) | 1,658 | 4 | 17 |
| rustc_demangle | 49 | 61 | 31 | 38 |
| unicode_normalization | 36 | 92 | 9 | 12 |
| tinyvec | 23 | 251 | 1 | 5 |
| libc | 3 | 734 | 0 | 0 |
| unicode_ident | 2 | 2 | 0 | 0 |

RustGM needs 10.2% of core's functions, 27.2% of alloc's and 17.6% of std's.
Of the demanded core functions, 14 are in stdarch (SSE2: `_mm_movemask_epi8`,
`_mm_cmpeq_epi8`, `_mm_loadu_si128`, `_mm_set1_epi8`, `_mm_pause` and nine
more) and 12 are in portable-simd (`Mask<i8, 16>` and `Simd<u8, 16>` for
`core::str::pattern::simd_contains`).

"Exist" counts function and closure bodies rustc builds MIR for with this
target and feature set. Core also has 907 const and static bodies. The sum
(25,587) is 271 below the frontier census's 25,858 bodies; the difference is
not reconciled.

Fourteen demanded definitions have no source body: enum and struct
constructors used as functions (`Option::Some`, `Result::Ok`, `Cow::Owned`,
`ControlFlow::Break`, ...) and compiler-built shims (`Clone::clone` for
built-in types, `FnOnce::call_once`, `Fn::call`, `FnMut::call_mut`). All
drop glue is one definition, `core::ptr::drop_glue`, with 12,149 instances.

Largest demanded modules by source functions: `core::num` 431, `core::iter`
403, `core::slice` 297, `alloc::collections` 251, `std::sys` 220, `core::fmt`
178, `core::str` 171, `alloc::vec` 156, `core::cmp` 135, `std::io` 132.

### Self-checks

- The demangler agrees with binutils `c++filt` on 10,391 sampled symbols and
  leaves none of the binary's 310,251 symbols undemangled.
- Per crate, `-Zprint-mono-items` function lines equal the functions defined in
  its IR (core 3,375; alloc 1,096; std 8,218; rustgm 297,403). Two units differ
  by one: the binary's C `main` and one compiler_builtins item.
- Every reference from a kept function resolves to a definition or a foreign
  symbol (`ir_references_unresolved` is empty).
- The only text symbols without reachable IR are the five allocator-shim
  symbols rustc generates at link time and the C runtime's start files.

### Milestone corpora

The same analysis over the consumer programs below, one directory per rung
(`obj/parallel/demand/m1` to `m4`). Counts are functions + closures.

| Corpus | Programs | Instances (linked) | core | alloc | std | Other crates |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| m1 | 17 | 8,454 (8,339) | 2,369 + 161 | 0 | 0 | compiler_builtins 38 |
| m2 | 12 | 23,021 (22,920) | 1,705 + 109 | 976 + 101 | 0 | none |
| m3 | 14 | 18,191 (18,110) | 2,028 + 112 | 457 + 26 | 1,332 + 218 | hashbrown 182 + 14, compiler_builtins 125 + 8 |
| m4 | 4 | 7,547 (7,443) | 1,333 + 76 | 357 + 21 | 376 + 55 | tinyvec 94 + 2, unicode_normalization 66 + 12, unicode_ident 2 |

The consumers reach 3,658 of the 4,209 core, alloc and std functions and
closures RustGM itself demands (87%): 1,690 after M1, 2,572 after M2, 3,645
after M3. By crate: core 2,204 of 2,606; alloc 698 of 764; std 756 of 839;
hashbrown 190 of 237. The uncovered remainder is mostly more instances of the
same families (`core::iter` 92, `core::num` 69, `core::cmp` 52,
`std::collections` 46).

## Consumer programs

Each rung has a directory of single-file programs, an outcomes file and one
`NAME.stdout` per program. The outcomes file declares arguments, environment,
standard input, exit status and standard error per program; it was written
before any output was recorded.

| Rung | Programs | Outcomes | Count |
| --- | --- | --- | ---: |
| M1 | `pa32/tests/spec/core-consumers/` | `pa32/tests/core-consumer-outcomes.json` | 18 |
| M2 | `pa33/tests/spec/alloc-consumers/` | `pa33/tests/alloc-consumer-outcomes.json` | 12 |
| M3 | `pa36/tests/spec/std-consumers/` | `pa36/tests/std-consumer-outcomes.json` | 14 |
| M4 | `pa35/tests/spec/unicode-consumers/` | `pa35/tests/unicode-consumer-outcomes.json` | 4 |

```sh
python3 scripts/resources.py run --wait-seconds 600 --max-output-gib 16 -- \
  /usr/bin/python3 scripts/storage.py run --max-output-gib 16 --no-prune -- \
  python3 scripts/consumer_oracle.py m1 m2 m3 m4
```

`scripts/consumer_oracle.py` builds each rung twice with rustc 1.98.0: the
course profile with std rebuilt from the pinned sources, and opt-level 2 (the
toolchain's prebuilt std for M3 and M4, rebuilt core and alloc for M1 and M2).
It runs every program three times per build in a fresh empty directory with
only the declared environment. All six runs must meet the declared status and
standard error and print identical bytes. `--record` writes the `.stdout` files
and hashes and refuses when a declaration is not met. All 48 programs pass in
check mode. The demand table above predates `intrinsic-ops.rs` and counts 17.

Conventions the frontier needs:

- **M1** programs are `#![no_std] #![no_main]`. They export an unmangled C
  `main`, format through `core::fmt::Write` into a 256-byte buffer and call
  only libc `write` and `abort`. The panic handler aborts without printing.
  `intrinsic-ops.rs` alone avoids `core::fmt`: it writes hexadecimal lines
  itself and exits with the number of values that differ from its own
  predictions, so it runs before formatting does.
- **M2** programs add `extern crate alloc` and a `#[global_allocator]` over
  libc `malloc`, `posix_memalign` and `free`. It counts live blocks and bytes,
  and every program's last line is `heap: N blocks, M bytes live`, so a leak or
  double free changes the output.
- **M3** and **M4** programs are ordinary `fn main` programs. M4 links the
  compiler's own locked dependencies; the oracle fails if Cargo resolves any
  other version.
- An aborting program is declared `"status": "abort"`. rustc's binary dies of
  SIGABRT. A RustGM program does the same when linked through the object
  bridge, and exits with status 134 under the backend's own runtime (see
  [Link paths](#link-paths)).
- Atomics have their own M1 program (`atomics.rs`). 128-bit arithmetic is in
  `integers.rs`, `fmt-integers.rs` and `fmt-floats.rs`. `chars-strs.rs`
  reaches the SSE2 substring search. `threads.rs` is the only program that
  spawns threads.
- Output never depends on hash iteration order, addresses, time or thread
  scheduling. `float-math.rs` prints libm results to twelve significant digits.

The stages stay planned: there is no `manifest.json`, `scripts/course.py` still
fails for PA32, PA33, PA35 and PA36, and `make test-harness` passes.
`scripts/tests/test_consumer_oracle.py` keeps declarations, sources and recorded
output consistent without building anything. Nothing runs these programs
through RustGM yet; that needs a runner that compares standard output (today's
native check rejects any output) and links through the object bridge.

## Runtime and backend gaps

What the corpora demand, against what RustGM lowers today and what the pinned
backend accepts. RustGM references are `dev/` at 80229d28d. Backend references
start with `cppgm:` and are files of `~/cppgm-extended` at the pinned commit
17b024f69, read with `git show`; that checkout's HEAD is 9d520c101, 512 commits
later, with local edits, and was not used. "Observed" names a probe of
`scripts/runtime_gap_probes.py`, which declares each expected observation and
fails when one changes:

```sh
python3 scripts/resources.py run --wait-seconds 600 --max-output-gib 16 -- \
  /usr/bin/python3 scripts/storage.py run --max-output-gib 16 --no-prune -- \
  python3 scripts/runtime_gap_probes.py --app PATH_TO_RUSTGM
```

All 28 observations agree (`obj/parallel/runtime-gaps/observations.tsv`). The
frontend probes ran under the ordinary library profile; they test attributes
and entry rules, which do not depend on the library. "No handler" means no
code was found; it was not confirmed by running.

### Link paths

The backend makes an executable in three ways, and they differ in what a
program may call.

| Path | Used today | Start and runtime | libc, libm, pthread |
| --- | --- | --- | --- |
| Standalone `lowir2native` | One unit: `scripts/check_native.py:316`, `scripts/backend.py:177-183` | Own start code. argc and argv only when the entry has parameters, no envp, no `.init_array` (`cppgm:dev/src/native/runtime/runtime.cpp:397-417`) | No: "undefined symbol: write" (observed `libc-write`) |
| Course link, the automatic fallback of `lowir2native` | Same | `main(argc, argv, envp)`; `malloc`, `free`, `calloc`, `realloc`, `atexit`, the i128 helpers; `abort` is `exit_group(134)` (`cppgm:dev/src/native/link/runtime_support.cpp:24-76`, `runtime_library.cpp:466-481`) | No (`cppgm:dev/src/native/link/linker.cpp:361`) |
| Object bridge: `cppgm -c` per unit, then `cc ... -lm` | Several units or native libraries: `scripts/backend.py:172-175,186-206` | glibc start files; the entry is exported as `main` (`cppgm:dev/src/native/object/object_compiler.cpp:1101-1110`) | Yes (observed `libc-write`, `variadic-open`) |

Every consumer rung calls libc (`write` at least), so every rung must be built
through the object bridge, also for a single unit. There `abort` is glibc's and
the program dies of SIGABRT like rustc's; exit status 134 belongs to the other
two paths.

### Gaps

Ordered by the first rung that meets them. "Owner" is where the work is; no
row needs a backend change.

| # | Area | First rung | Owner | State |
| ---: | --- | --- | --- | --- |
| 1 | Link through the object bridge | M1 | Runner | Bridge exists; the consumer runner must use it |
| 2 | Program entry: `#![no_main]`, C `main` with arguments, `lang_start`, `Termination` | M1 | Frontend | Fenced |
| 3 | Intrinsics executed at run time | M1 | Frontend | A fixed list is lowered; the rest stop at one fence |
| 3a | Bring-up lowering, integer arithmetic: `saturating_add/sub`, `exact_div`; `add_with_overflow` takes the checked contract of `sub_with_overflow` and `mul_with_overflow`, in constants too | M1 | Runtime lane | Lowered in place and waived by name; `intrinsic-ops.rs` matches rustc |
| 3b | Bring-up lowering, bits: `ctpop`, `ctlz`, `cttz`, their `_nonzero` forms, `bswap`, `bitreverse` | M1 | Runtime lane | Same; shift-and-mask sequences at 8 to 128 bits |
| 3c | Bring-up lowering, pointers: `offset`, `slice_get_unchecked`, `ptr_offset_from`, `ptr_offset_from_unsigned` | M1 | Runtime lane | Same; a zero-sized `ptr_offset_from` keeps its trap |
| 3d | Bring-up lowering, moves: `read_via_copy`, `write_via_move`, `black_box`, `forget`, `unreachable` | M1 | Runtime lane | Same; `unreachable` aborts |
| 3e | Bring-up lowering, memory: `copy_nonoverlapping`, `copy`, `write_bytes`, `compare_bytes`, `raw_eq`, `volatile_load/store` | M1 | Runtime lane | One shim per instance for transfers and comparisons, scalar volatile accesses in place; an aggregate volatile access keeps its trap |
| 3f | Bring-up lowering, layouts and floats: `size_of_val`, `align_of_val`, `assert_zero_valid`, `fabs`, `copysignf32/64` | M1 | Runtime lane | Sized values, slices and strings; the assertion lowers only when it holds, so a failing or undecided one, a trait object and an unsized tail keep their traps |
| 3g | Bring-up lowering, spin-wait hint: the LLVM operation `llvm.x86.sse2.pause` behind `core::hint::spin_loop` | M1 `atomics.rs` | Runtime lane | A shim that returns at once, waived by name; any other LLVM operation or shape stays fenced |
| 4 | Atomics other than `u32` | M1 `atomics.rs`, M2 `arc.rs` | Frontend | Fenced; backend already qualified for 8 to 64 bits and pointers |
| 4a | Bring-up lowering, atomics: the fifteen `atomic_*` accesses and `atomic_fence` | M1 | Runtime lane | Lowered in place for 8 to 64 bits and pointers, always sequentially consistent; `intrinsic-ops.rs` matches rustc |
| 5 | SIMD: 16-byte vectors for hashbrown and `str` search | M1 `chars-strs.rs`, M3 every `HashMap` | Frontend | No handler; backend has `v128` |
| 6 | `compiler_builtins` | M1 | Integrator decision | Its symbols are provided at link time; its sources need assembly RustGM fences |
| 7 | Allocator shims and `#[global_allocator]` | M2 | Frontend | Fenced |
| 8 | Panic runtime under panic=abort | M1 handler, M3 std | Frontend | Abort works; handler shapes, `catch_unwind` and messages do not |
| 9 | Compiler-inserted checks when core installs its panic hooks | M1 | Frontier | Fenced; the bring-up profile is reported to waive the fence and keep the abort |
| 10 | Thread-locals | M3 | Frontend | Unknown attribute; backend has thread-local globals |
| 11 | Weak and placed statics: `#[linkage]`, `#[link_section]`, `#[used]`, link modifiers | M3 | Frontend | Unknown attributes; backend has weak references and sections |
| 12 | Float math intrinsics | M3 `float-math.rs` | Frontend | No handler; libm is on the bridge's link line |
| 13 | Memory builtins | M1 for `compare_bytes`, otherwise speed | Frontend | Loops instead of `memcpy`; compare has no handler |
| 14 | Scale of one program | Self-hosting | Integrator decision | Backend measured to 4.2 million functions; RustGM's own budgets are lower |

**1. Link path.** Demand: M1 needs `write abort memcmp memcpy memmove memset`;
M2 adds `malloc free posix_memalign`; M3 has 117 foreign symbols (libm,
pthread, file system); the rustgm corpus 57. See the table above.

**2. Program entry.**
- Demand: M1 and M2 are `#![no_main]` with an exported C
  `main(argc, argv)`. M3 and later enter through C `main`, `std::rt::lang_start`,
  `lang_start_internal` and `std::rt::init`, need `Termination` for
  `main-result.rs` and `exit-code.rs`, and read argv
  from `std::sys::args::unix::imp::ARGV_INIT_ARRAY`, a
  `#[used] #[link_section = ".init_array.00099"]` static that glibc calls with
  argc, argv and envp.
- RustGM: an executable needs a root `fn main` (`dev/src/sema/entry.rs:40`)
  that returns `()` or `!` (`entry.rs:83`, observed `main-returns-result`).
  `#![no_main]` is an unknown attribute (observed `no-main`). The generated
  LowIR `main` takes nothing and returns 0 (`dev/src/lower/context.rs:1072-1089`).
  No `start` lang item handling was found.
- Backend: `role=entry`; on the bridge glibc passes argc, argv and envp. A
  pointer global with `section=.init_array` runs before `main` and receives
  argc (observed `init-array-section`; exit 255 on the standalone path).

**3. Intrinsics.**
- Demand: 80 distinct intrinsics are called in the rustgm corpus and 117
  across all corpora (`runtime-surface.tsv`, section `rust-intrinsic-call`,
  with the number of calls that reach code generation). Core declares 311; 51
  have fallback bodies.
- RustGM lowers `wrapping_add/sub/mul`, `unchecked_add/sub/mul/div/rem/shl/shr`,
  `sub_with_overflow`, `mul_with_overflow`, `three_way_compare`, `ub_checks`,
  `abort`, `transmute`, `transmute_unchecked`, `caller_location`,
  `ptr_metadata`, `aggregate_raw_ptr`, `arith_offset`, `discriminant_value`,
  the layout queries, `const_eval_select` and `cold_path`
  (`dev/src/support/bindings.rs:45-64`, `dev/src/lower/function.rs:913-957`,
  `dev/src/sema/compiler_items/subtraction_contract.rs:17-66`). `ctpop`,
  `ctlz` and `cttz` are evaluated in constants
  (`dev/src/sema/compiler_items/bit_counts.rs:25-38`); no run-time lowering
  was found.
- Any other intrinsic without a usable fallback body stops at "source
  intrinsic executable lowering is not implemented"
  (`dev/src/lower/instances/demand.rs:66-91`). Demanded names outside the
  lowered list, by absence of a handler: `add_with_overflow`,
  `saturating_add`, `saturating_sub`, `exact_div`, `ctlz`, `ctlz_nonzero`,
  `cttz`, `cttz_nonzero`, `ctpop`, `bswap`, `bitreverse`, `black_box`,
  `size_of_val`, `align_of_val`, `ptr_offset_from`, `ptr_offset_from_unsigned`,
  `offset`, `read_via_copy`, `slice_get_unchecked`, `assert_inhabited`,
  `assert_zero_valid`, `unreachable`, and the families of rows 4, 5, 8, 12
  and 13. The frontier's census decides each one.
- Backend: `ctz`, `clz` and `popcount` exist for i32, u32 and i64 only
  (`cppgm:dev/src/lowir/validator.cpp:925-931`, observed `count-zeros-i8`),
  `bswap` for 16 to 64 bits, and there is no rotate, overflow-reporting or
  saturating operation (`cppgm:dev/src/lowir_model.h:287-316`). RustGM already
  composes checked arithmetic and builds `llvm.ctpop`, `llvm.bitreverse` and
  `llvm.bswap` from shifts and masks (`dev/src/support/llvm_symbols.rs:21-45`,
  `dev/src/lower/bit_kernels.rs`), so these stay frontend work. The demand
  includes the 128-bit forms `llvm.ctlz.i128`, `llvm.cttz.i128` and
  `llvm.ctpop.i128`.

**4. Atomics.**
- Demand, rustgm corpus: i8, i32, i64 and pointer; load, store, exchange, add,
  sub, strong and weak compare-exchange, fence; all five orderings, because
  core's `atomic_*` helpers switch on a run-time `Ordering`. RustGM's own code
  uses `Atomic<u32>::fetch_update`. M1 `atomics.rs` adds i16 and and, or, xor,
  nand, max, min and umin.
- RustGM binds only `AtomicU32` (`dev/src/resolve/builtins/library.rs:241-243`).
  The LowIR builder rejects every other width: "atomic load currently requires
  u32" and four more (`dev/src/lowir/builder.rs:232,252,273,294,314`). LowIR has
  no fence operation (`dev/src/lowir/program.rs`, `Operation`). The u32 and, or,
  xor, nand, min and max are compare-exchange loops
  (`dev/src/lower/intrinsics/atomic.rs`).
- Backend: `atomic_load`, `atomic_store`, `atomic_exchange`,
  `atomic_compare_exchange` (strong only), `atomic_add_fetch`, thread and
  signal fences, on i1, i8, i16, i32, i64 and ptr
  (`cppgm:pa24/README.md:354`, `cppgm:dev/src/lowir_model.h:193-216`).
  `pa28/tests/backend/atomics/contract.json` already runs 8, 16, 32 and 64
  bits, pointers, booleans and fences on the pinned backend. The strong
  compare-exchange is a valid weak one.

**5. SIMD.**
- Demand: 14 stdarch and 12 portable-simd functions; nine intrinsics
  (`simd_eq`, `simd_gt`, `simd_lt`, `simd_and`, `simd_or`, `simd_bitmask`,
  `simd_reduce_all`, `simd_shuffle`, `simd_splat`) on `<16 x i8>`,
  `<2 x i64>` and `<16 x i1>`; `llvm.x86.sse2.pause`. The users are
  hashbrown's SSE2 group probe (every `HashMap`), `core::str::pattern`'s
  `simd_contains` and `spin_loop`. The first two are selected by
  `cfg(target_feature = "sse2")`, which RustGM's fixed target sets as rustc
  does (`dev/src/expand/config.rs:25-27`).
- RustGM has `repr(simd)` layout (`dev/src/abi/layout/simd.rs`) and the C
  class for 128-bit vectors, but its LowIR has no vector type
  (`dev/src/lowir/vocabulary.rs:3-17`), `simd_*` have no handler, and
  `llvm.x86.*` names reach "selected LLVM intrinsic operation is not
  implemented" (`dev/src/lower/llvm_calls.rs:23`).
- Backend: `v128` and about 200 SSE operations, among them `pcmpeqb`,
  `pcmpgtb`, `pmovmskb`, `por`, `pand` and `pshufb`
  (`cppgm:pa8/lowir.md:1384-1435`; observed `sse2-byte-compare` on both
  paths). There is no `pause` operation.
- Three ways, none needs the backend to change: add `v128` and `vector` to
  RustGM's LowIR and map the nine intrinsics for 16-byte vectors; lower them
  lane by lane on memory; or stop advertising `sse2` to `cfg`, which selects
  hashbrown's generic group and core's scalar search but departs from rustc's
  target facts and from this demand list. `spin_loop` needs
  `llvm.x86.sse2.pause` in every case; a no-op is a correct lowering.

**6. `compiler_builtins`.**
- Demand: 85 of its functions in the rustgm corpus (`__udivti3`, `__umodti3`,
  `__divti3`, `__modti3`, `__fixdfti`, `__fixunsdfti`, `__floattidf`,
  `__floattisf`, `__floatuntidf`, `__floatuntisf`, `fmod`, `fmodf` and their
  helpers), 38 in M1, 133 in M3. rustc reaches them through LLVM libcalls.
  Their sources hold the only inline assembly in the corpora: `div {2:q}`,
  `cpuid` and `vfmadd*`, with outputs, explicit registers and template
  modifiers.
- RustGM never calls them: i128 division and i128 to float are LowIR
  operations (`dev/src/lower/context.rs:1091-1124`, `dev/src/lower/casts.rs:99-181`)
  and float `%` is its own helper `rg.rem.f32`/`rg.rem.f64`
  (`dev/src/lower/remainder.rs`). A `no_std` crate still needs the crate to
  exist (`dev/src/resolve/standard_imports.rs:53-82`). RustGM's assembly
  profile fences outputs, explicit registers and modifiers
  (`dev/src/support/initial_assembly.rs:45-57`).
- Backend: turns i128 division and conversions into calls to `__divti3` and
  its family, which the course runtime and libgcc define (observed
  `i128-division` on both paths). It has no float remainder (observed
  `float-remainder`), so RustGM's helper is needed.
- Decision: compile `compiler_builtins` from source, or satisfy the loader
  with an empty crate and take the helpers from the link.

**7. Allocator.**
- Demand: `__rust_alloc`, `__rust_dealloc`, `__rust_realloc`,
  `__rust_alloc_zeroed`, `__rust_no_alloc_shim_is_unstable_v2` and
  `__rust_alloc_error_handler` in the synthetic `__rustc` crate. rustc
  generates them at link time; they are the only text symbols without IR. Std
  programs get `__rdl_alloc` and its family behind them. Below: `malloc`,
  `calloc`, `realloc`, `free`, `posix_memalign`. M2 declares its own
  `#[global_allocator]`.
- RustGM: `#[global_allocator]` and `#[alloc_error_handler]` are catalogued
  but not executable (`dev/src/expand/builtin/catalog.rs:175-180,224-264`);
  observed as an unresolved attribute (`global-allocator`). No shim
  generation and no reader of `rustc_std_internal_symbol` were found.
- Backend: nothing special is needed.

**8. Panic runtime.**
- Demand under panic=abort: `__rust_start_panic`, `rust_begin_unwind`,
  `rust_panic` and `__rust_abort`; the `catch_unwind` intrinsic in
  `lang_start`; the `abort` intrinsic (14 uses); `_Unwind_Backtrace` and
  `_Unwind_GetIP` from libgcc_s, although std was built without the backtrace
  feature.
- RustGM: every compiler-inserted check (overflow, bounds, division) jumps to
  one block that calls libc `abort` (`dev/src/lower/function.rs:281-309`). It
  does not call the panic lang items, so such a panic prints nothing and runs
  no hook. The `abort` intrinsic becomes `unreachable`
  (`dev/src/lower/function/termination_call.rs:81-107`), which the backend
  emits as `ud2` (`cppgm:dev/src/native/select/emit_control.cpp:665-667`).
  `#[panic_handler]` is exported as `rust_begin_unwind`
  (`dev/src/sema/foreign/handler_linkage.rs:10`); other handler shapes are
  fenced (`handler_linkage.rs:112`). `catch_unwind` has no handler.
- Consumers: `panic-index.rs` and `panic-overflow.rs` only need the abort.
  `panic-default.rs` panics with `panic!` and `panic-hook.rs` with `expect`;
  both go through the library.

**9. Compiler-inserted checks with real core.**
- Demand: rustc sends each failed overflow, division and bounds check to a
  lang item of core: `panic_const_add_overflow` and nine siblings, and
  `panic_bounds_check`. All eleven are linked in M1, M2 and M3.
- RustGM: when the source installs one of the 28 guard lang items, a reached
  check stops at "compiler assertion requires checked source panic signature,
  caller and runtime routing"
  (`dev/src/sema/compiler_items/guard_panics.rs:57-67`, the same on main at
  1bbd69b83). Row 8's abort is what a check does when no hook is installed.
  With real core as the provider every program with a checked operation
  meets the fence. The frontier lane reported this; this lane read the code
  and did not run it.
- The frontier lane also reports that its bring-up profile now waives the
  fence and keeps the abort. That change is not on main at 1bbd69b83 and was
  not checked here.
- Cost: route each guard to the installed hook with its checked signature and
  caller location. Until then a failed check aborts without a message, which
  is what `panic-index.rs` and `panic-overflow.rs` declare.

**10. Thread-locals.**
- Demand: 6 in the rustgm corpus (`RandomState::new::KEYS`, used by every
  `HashMap::new`; `OUTPUT_CAPTURE`; `LOCAL_PANIC_COUNT`; `DTORS`;
  `thread::current::CURRENT`; `thread::current::id::ID`) and 13 in M3;
  `__tls_get_addr`; weak `__cxa_thread_atexit_impl`. The rustgm corpus has no
  `thread::spawn` instance. M3 `threads.rs` is the only consumer with threads.
- RustGM: `#[thread_local]` is an unknown attribute (observed `thread-local`).
  A LowIR global carries only read-only and declaration flags
  (`dev/src/lowir/data.rs:28-38`, `dev/src/lowir/writer.rs:111-118`).
- Backend: `storage=thread_local` globals, local-exec or initial-exec in host
  objects (`cppgm:dev/src/native/select/operands.cpp:262-300`; observed
  `thread-local-global`, one thread only). The standalone and course images
  address them as ordinary data.
- A single-threaded bring-up can treat thread-locals as plain statics.

**11. Weak and placed statics.**
- Demand: weak `#[linkage = "extern_weak"]` statics for
  `__cxa_thread_atexit_impl`, `__dso_handle`, `getrandom`, `gettid` and
  `statx`; the `.init_array` static of row 2; `#[link(...)]` with `cfg(...)`
  and `modifiers = "-bundle"` in the `unwind` and `libc` crates.
- RustGM: `#[linkage]`, `#[link_section]` and `#[used]` are unknown attributes
  (observed `extern-weak`, `link-section`, `used`). Linkage is import, export
  or internal (`dev/src/support/linkage.rs`), and the writer emits no binding
  or section metadata (`dev/src/lowir/writer.rs:57-83,111-118`). Link
  modifiers are fenced (`dev/src/sema/link_dependencies.rs:107`); std's uses
  sit under a false `cfg`, and whether the fence comes first was not checked.
- Backend: a `binding=weak` declaration is a weak reference that resolves to
  null (observed `weak-undefined`); `section=` places a global (observed
  `init-array-section`).

**12. Float math.**
- Demand: `fabs` in the rustgm corpus. M3 `float-math.rs` adds `sqrt`,
  `floor`, `ceil`, `trunc`, `round`, `round_ties_even`, `fma`, `copysign`,
  `powi`, `exp`, `exp2`, `log`, `log2`, `log10`, `pow`, `sin` and `cos`.
- RustGM: no handler. Its LowIR has no `sqrt`. f16 and f128 are fenced
  (`dev/src/lower/arithmetic.rs:49,76`) and no corpus demands them.
- Backend: `unary sqrt` on f32 and f64
  (`cppgm:dev/src/lowir/validator.cpp:932-934`) and nothing else; libm is on
  the bridge's link line (`scripts/backend.py:206`).

**13. Memory builtins.**
- Demand: `llvm.memcpy`, `llvm.memmove`, `llvm.memset`, `memcmp`, `strlen`;
  the intrinsics `copy`, `copy_nonoverlapping`, `write_bytes`,
  `compare_bytes`, `raw_eq`, `typed_swap_nonoverlapping`.
- RustGM copies and fills with per-type LowIR loops
  (`dev/src/lower/intrinsics/counted.rs`, `overlapping.rs`, `fill.rs`) and
  `copyobj`. Nothing in `dev/src` calls `memcpy`, `memset` or `memcmp`.
  `compare_bytes` and `raw_eq` have no handler.
- Backend: `copyobj`, `zeroinit`, and `cppgm_builtin_memcpy`, `memmove`,
  `memset`, `memcmp` and `strlen` on every path (`cppgm:pa8/lowir.md:480-486`).

**14. Scale.** The self-hosting program is one whole-program compilation of
309,093 instances (rustgm corpus), far beyond any fixture. The pinned backend
was measured on programs of that size and larger; see
[Backend scale probe](#backend-scale-probe). RustGM's own budgets and the job
memory limit are met before any backend limit.

### Checked, no gap

- 128-bit integers: RustGM lowers them as LowIR `i128`; the backend has the
  arithmetic, division included (observed `i128-division`). RustGM emits no
  `phi` or `select`, which the backend refuses for `i128`.
- f32 and f64 arithmetic, comparisons and conversions. RustGM composes Rust's
  saturating float-to-integer casts (`dev/src/lower/casts.rs:99-181`); the
  backend only truncates.
- Variadic C calls (`open64`, `fcntl`, `syscall`): RustGM
  `dev/src/lower/function/c_boundary.rs:126-184`, backend `arity=variadic`
  (observed `variadic-open`).
- By-value structs across the C boundary (`dev/src/abi/classification.rs`),
  `#[no_mangle]`, `#[export_name]`, `#[link_name]`, extern statics.
- Symbol names: RustGM uses its own scheme (`dev/src/lower/linkage.rs:52-91`)
  inside the backend's alphabet; nothing links against rustc objects.

## Backend scale probe

`scripts/lowir_scale_probe.py` composes one large valid LowIR program from
RustGM's own output and measures the pinned tools on it. It changes nothing in
the backend or the frontend.

```sh
python3 scripts/resources.py run --wait-seconds 600 --max-output-gib 16 -- \
  /usr/bin/python3 scripts/storage.py run --max-output-gib 16 --no-prune -- \
  python3 scripts/lowir_scale_probe.py --app PATH_TO_RUSTGM --density 16 64 128 256
```

Outputs, in `obj/parallel/lowir-scale/`: `observations.tsv` (one row per
process), `receipt.json` (backend pin, compiler hash, every unit's path and
hash, every composed program's counts and hash), `density.tsv`.

### Method

- A unit is the LowIR RustGM emits for one executable native fixture that is
  declared to exit 0 and needs no auxiliary crate or environment. Each unit is
  first validated, built by `lowir2native -O0` and run by itself. 2,965 of
  2,966 qualify. `pa16/tests/spec/persistent-native/zero-repeat.rs` is left
  out: its 16 MB zero constant made the first attempts measure data.
- A size of N rounds holds N copies of every unit with the unit's symbols
  renamed, each foreign declaration once, and a generated `main` that calls
  every copy's former entry through a tree of functions and adds the results.
  Exit status 0 means every copy ran and passed.
- Each size is built and run on both link paths: `lowir`, then
  `lowir2native -O0`; and `cppgm -O0 -c`, then `cc ... -lm`. Every process
  runs alone under a 10 GiB address-space limit and is measured with `wait4`.
  Three repeats.
- RustGM is the rustc-built opt-level 0 binary of `dev/` at 80229d28d. The
  machine was shared with the other lanes' builds, so wall times vary.

### Results

Every process of every size exited 0 (72 rows), and each composed program ran
and exited 0 on both paths. Times are wall seconds, median (minimum to
maximum) of three; memory is the largest peak resident size in MiB.

| Rounds | Functions | Globals | LowIR MB |
| ---: | ---: | ---: | ---: |
| 16 | 264,123 | 19,280 | 100.4 |
| 64 | 1,056,490 | 77,120 | 403.2 |
| 128 | 2,112,978 | 154,240 | 808.0 |
| 256 | 4,225,954 | 308,480 | 1,617.7 |

Standalone path:

| Rounds | `lowir` s | MiB | `lowir2native -O0` s | MiB | Image MB |
| ---: | --- | ---: | --- | ---: | ---: |
| 16 | 3.5 (3.2 to 4.0) | 490 | 8.9 (6.9 to 11.0) | 490 | 14.6 |
| 64 | 17.8 (15.8 to 19.2) | 1,964 | 36.1 (24.0 to 40.5) | 1,966 | 58.4 |
| 128 | 21.0 (20.6 to 23.2) | 3,911 | 45.2 (43.7 to 45.9) | 3,914 | 116.9 |
| 256 | 52.0 (38.9 to 53.4) | 7,808 | 127.4 (88.8 to 175.2) | 7,813 | 233.7 |

Object bridge:

| Rounds | `cppgm -O0 -c` s | MiB | Object MB | `cc` s | MiB | Executable MB |
| ---: | --- | ---: | ---: | --- | ---: | ---: |
| 16 | 7.7 (5.9 to 8.8) | 491 | 51.6 | 2.6 (2.5 to 3.0) | 150 | 36.3 |
| 64 | 20.5 (17.7 to 29.5) | 1,964 | 207.1 | 10.8 (8.7 to 13.2) | 573 | 145.7 |
| 128 | 32.6 (31.0 to 34.5) | 3,912 | 414.9 | 17.3 (15.1 to 18.5) | 1,094 | 292.1 |
| 256 | 70.2 (66.0 to 128.2) | 7,808 | 830.4 | 35.6 (33.4 to 50.5) | 2,177 | 584.8 |

- Nothing is worse than proportional. From 16 to 256 rounds the size grows
  16 times and the fastest runs 11 to 13 times (`lowir` 12.2, `lowir2native`
  12.9, `cppgm` 11.2, `cc` 13.4).
- `lowir`, `lowir2native` and `cppgm` each hold the whole program: 4.8 to
  4.9 MiB of resident memory per MB of LowIR. The link takes about 1.4.
- The bridge is not the slower path: 106 s against 179 s at 256 rounds (sums
  of medians).
- The spread is the shared machine. CPU time is within 3% of wall time in
  every tool row, so a busy machine slowed the processes themselves.
- The peak of a `run` row (41 to 226 MiB) is the measuring script's own, which
  a child created by `vfork` inherits as its floor. Tool rows are far above it.

### Code density

With `--density`, each unit is also compiled to an object by `cppgm -O0 -c`
and, from its Rust source, by rustc 1.98.0 at opt-level 0 under the course
profile; the bytes of executable sections are compared (`density.tsv`).

- 2,965 units: 5,936,627 bytes of LowIR, 16,496 functions. Backend 868,447
  bytes of code, rustc 882,097 (ratio 0.98; per-unit median 0.84).
- On the 20 largest units the backend emits 132,356 bytes against 89,860
  (1.47).
- That is 53 bytes of code per function and 6.8 bytes of LowIR per byte of
  code.

### Projection

rustc's opt-level 0 `rustgm` has a 41.3 MB executable segment for 309,093
instances. At the fixtures' ratios that is 280 to 415 MB of LowIR, about the
64-round size: half a minute and 2 GiB through the bridge. Three times the
upper figure is still 6 GiB. This is an estimate. Fixture code is small scalar
functions; RustGM has emitted no library-sized program yet, and its own time
and memory at this size were not measured.

### Limits

- **Standalone image, 2 GiB.** Code refers to data by 32-bit PC-relative
  offsets, and `lowir2native` stops with "PC-relative reference out of range"
  when an image outgrows them (`cppgm:dev/src/native/image/image.cpp:166`;
  observed `image-over-2gib`, two 1.2 GB zero objects). The object bridge
  builds and runs that program. The first attempts met the limit through
  data: with a 16 MB constant in every round, 128 rounds failed with
  "relocation value out of range" and 256 rounds with `std::bad_alloc` under
  the 10 GiB limit. Those two messages were read from the terminal and are
  not kept. Code alone is far from the limit: 4.2 million functions make a
  234 MB image.
- **Job memory, 12 GiB.** The resource controller runs each guarded job in a
  scope with `MemoryMax=12G` and no swap. At 4.8 MiB per MB that is about
  2.5 GB of LowIR in one unit. The first run set the per-process limit above
  the job's and the job was killed at 256 rounds
  (`obj/parallel/lowir-scale/attempt-256-rounds.log`).
- **RustGM's own budgets** are lower than anything measured here and were not
  exercised: 64 MB of generic instance bodies and of instance work
  (`dev/src/lower/instances/demand.rs:215-216,238`), 100,000 export roots
  (`dev/src/lower/roots.rs:42-60`), 100,000 intrinsic shims
  (`dev/src/lower/intrinsics.rs:133-134`), 1,000,000 persistent data steps
  (`dev/src/lower/data.rs:62-68`) and 1,025 LowIR units per link
  (`scripts/backend.py:170-171`). What each counts against the 309,093
  instances of the self-hosting program is for the frontier's census to
  report.

Not measured: backend optimisation levels above `-O0`; a program above 256
rounds, so the backend's own failure point is unknown; a link of many units;
initialised data of that size on the bridge; an idle machine.

## Not verified

- Nothing runs a consumer program through RustGM yet.
- Gap rows that say "no handler" rest on the absence of code in `dev/src`,
  not on a failing run: the ordinary profile cannot express those programs.
- Frontend probes ran under the ordinary library profile only.
- Row 9's fence was read, not run, and the bring-up waiver is the frontier
  lane's report.
- The thread-local probe has one thread. Thread-locals with `pthread_create`
  through the object bridge were not run.
- Whether RustGM's link-modifier fence is reached before `cfg` removes std's
  uses (row 11).
- The backend was read at the pinned commit only. The local checkout is 512
  commits ahead and may already differ.
- The census discrepancy of 271 bodies in core (Demand list).

## Open decisions

For the integrator. The lane has not acted on any of them.

1. **Consumer runner.** It must link through the object bridge also for one
   unit, compare standard output and accept SIGABRT for `"status": "abort"`.
   Who writes it, and where it is registered.
2. **Entry for M1 and M2.** The programs are `#![no_main]` with an exported C
   `main`; rustc gives a `no_std` program no other stable entry. RustGM must
   accept that form before any M1 program compiles (row 2).
3. **`compiler_builtins`.** Compile it from source, which needs the assembly
   forms RustGM fences, or give the loader an empty crate and take
   `__udivti3` and its family from the link.
4. **SIMD.** `v128` in RustGM's LowIR, lane-by-lane lowering, or no `sse2`
   in `cfg`. The last one changes which library functions are demanded.
5. **Thread-locals.** Plain statics for a single-threaded bring-up, or real
   TLS at once. With plain statics `threads.rs` cannot be in M3's exit test.
6. **std features.** `_Unwind_Backtrace` and `_Unwind_GetIP` are demanded with
   an empty feature list. Link libgcc_s, or cut the backtrace path.
7. **`ub_checks()` branches.** If RustGM does not fold the constant branch, it
   needs the 389 unlinked instances too.
8. **Scale.** Raise or re-base RustGM's 64 MB instance budgets for the
   self-hosting unit, and decide whether the 12 GiB job limit stands for it.
9. **Compiler-inserted checks.** Keep the silent abort (rows 8 and 9) through
   the rungs, or route the guards to core's hooks. No consumer's declared
   output depends on the message.
