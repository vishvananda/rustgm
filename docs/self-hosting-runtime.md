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

Pending the consumer programs below.
