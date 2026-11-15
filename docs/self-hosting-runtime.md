# Self-hosting runtime lane tracker

The runtime lane's only living document
([plan](self-hosting-plan.md)). It is updated in place. Generated evidence
stays under `obj/parallel/demand/` and is not committed.

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
| M1 | `pa32/tests/spec/core-consumers/` | `pa32/tests/core-consumer-outcomes.json` | 17 |
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
and hashes and refuses when a declaration is not met. All 47 programs pass in
check mode.

Conventions the frontier needs:

- **M1** programs are `#![no_std] #![no_main]`. They export an unmangled C
  `main`, format through `core::fmt::Write` into a 256-byte buffer and call
  only libc `write` and `abort`. The panic handler aborts without printing.
- **M2** programs add `extern crate alloc` and a `#[global_allocator]` over
  libc `malloc`, `posix_memalign` and `free`. It counts live blocks and bytes,
  and every program's last line is `heap: N blocks, M bytes live`, so a leak or
  double free changes the output.
- **M3** and **M4** programs are ordinary `fn main` programs. M4 links the
  compiler's own locked dependencies; the oracle fails if Cargo resolves any
  other version.
- An aborting program is declared `"status": "abort"`. rustc's binary dies of
  SIGABRT; RustGM's course runtime reports exit status 134.
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
native check rejects any output).
