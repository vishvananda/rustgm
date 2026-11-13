# Assembly ymm operands

The census-progress observation (2026-11-10,
`obj/parallel/census-progress/original/`) recorded four pinned core bodies that
failed with "assembly register class is not implemented for this target":

- `_mm256_stream_si256` 10511, `_mm256_stream_pd` 10512 and
  `_mm256_stream_ps` 10513, in `core_arch` `x86/avx.rs`;
- `_mm256_stream_load_si256` 10805, in `x86/avx2.rs`.

```rust
#[inline]
#[target_feature(enable = "avx")]
pub unsafe fn _mm256_stream_ps(mem_addr: *mut f32, a: __m256) {
    crate::arch::asm!(
        vps!("vmovntps", ",{a}"),
        p = in(reg) mem_addr,
        a = in(ymm_reg) a,
        options(nostack, preserves_flags),
    );
}
```

The parser knew no `ymm_reg` class, so the body failed while typing.

## Rule

rustc's x86 register table (`rustc_target/src/asm/x86.rs` at the pinned
revision) admits `ymm_reg` under `avx`. Its types are 32- and 64-bit integer
and float scalars and 128- and 256-bit vectors. Its template modifiers are `x`,
`y` and `z`.

## Change

- **Class.** `ymm_reg` parses to `Register::Ymm`. Explicit ymm registers
  (`"ymm0"`) are still rejected by name.
- **Typing.** A body needs `avx` in its available features ("assembly
  register class requires the avx target feature"). `avx2` implies it through
  the existing closure. Scalars follow the `xmm_reg` rule. A vector must be a
  completed genuine vector of 128 or 256 supported lane bits ("assembly ymm
  vector requires 128 or 256 bits of supported numeric lanes"). The `xmm_reg`
  check and this one share one lane width computation, and `xmm_reg` still
  requires exactly 128 bits. Modifiers follow `xmm_reg`.
- **Profile.** The initial assembly profile rejects the class before native
  lowering and module-body export ("inline assembly ymm_reg operand is outside
  the initial assembly profile"). No `LowIR` or published body carries one.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- Core's four bodies now type, so the typing census passes them. The core
  build still stops at the profile until a backend contract for 256-bit
  vector operands exists. The backend is frozen at v3.
- Admitting typing does not weaken the forged target-feature rule. A forged
  `avx` on an imported function could only widen the CPU check of a body that
  contains a ymm operand, and export rejects every such body. `spec.md` still
  requires a revisit before the profile admits any class that needs a
  non-baseline feature.
- RustGM's messages differ from rustc's. Every rejection is at typing, where
  rustc rejects.
- `zmm_reg` and `kreg`, core's thirteen AVX-512 sites, stay "not implemented
  for this target".

## Contract

**PA41** (`pa41/tests/spec/asm-ymm/`, sixteen cases). All declarations agree
with rustc 1.98.0 (`outcomes.json`). The base's ordinary-mode CLI cannot
resolve `core::arch::asm`, so the cases run through the library invariant on
the `no_core` foundation.

| Case | Form | Rust | RustGM |
| --- | --- | --- | --- |
| `stream-ps` | core's `_mm256_stream_ps` | accept | profile |
| `stream-pd` | core's `_mm256_stream_pd` | accept | profile |
| `stream-si256` | core's `_mm256_stream_si256` | accept | profile |
| `vector-i32x4` | a 128-bit vector | accept | profile |
| `vector-i16x16` | a 256-bit `i16` vector | accept | profile |
| `scalar-f64` | an `f64` scalar | accept | profile |
| `scalar-u32` | a `u32` scalar | accept | profile |
| `avx2-implies` | only `avx2` enabled | accept | profile |
| `modifier-x` | the `x` modifier | accept | profile |
| `output` | a 256-bit `out(ymm_reg)` | accept | profile (output) |
| `scalar-u8` | a `u8` scalar | reject | typing |
| `no-avx` | only `sse` enabled | reject | typing |
| `wide` | a 512-bit vector | reject | typing |
| `short` | a 64-bit vector | reject | typing |
| `modifier-e` | the `e` modifier | reject | typing |
| `xmm-256` | a 256-bit vector in `xmm_reg` | reject | typing |

"Profile" means the case types and native lowering stops at the initial
assembly profile. `output` stops at the profile's earlier output check, the
form of core's `_mm256_stream_load_si256`. The `asm-vectors` packet's
`ymm-no-feature` now fails at the `avx` check, and its `ymm-body-feature`
types and stops at the profile. Both failed at the parser before.

## Qualification

Two focused receipts pass with exact selections. Both cover assignment
metadata, formatting and the architecture audit:

- 1794530651927726953 (45.1s), on the final tree: strict private Clippy and
  27 library tests:
  - the new `ymm_operands_type_like_rustc_and_stop_at_the_assembly_profile`.
    It requires each rejection's reason and lowers every positive with its
    asm function as the only export. It also exports `stream-ps` on the
    native prelude through the module-body publication, which must stop at
    the profile;
  - the seven vector, template, scalar and storage invariants, including the
    real `_mm_stream_ps` body;
  - the six lowering invariants, among them every declared profile fence and
    the version-thirteen asm body transport;
  - the asm CPU context invariant and the twelve target-feature policy tests.
- 1794530736486298382 (35.7s), with strict Clippy for the lowering suite and
  the CLI: the ignored native test
  `stream_assembly::original_stream_matches_independent_rust_bits_sentinels_and_movntps`.
  Core's `_mm_stream_ps` compiles through the pinned CPPGM backend and matches
  rustc's bits.

Earlier attempts:

- An attempt was refused at admission while the exclusive core observation
  held the lock.
- 1794530132517910561: the positives lowered, since no root demanded their
  bodies. The vector test's `ymm-body-feature` therefore passed too. Both tests
  now export the asm function.
- 1794530261984719161: export on the `no_core` foundation stopped first at
  "conditional member requires its actual From declaration". It now uses the
  publishable native prelude.
- 1794530451495245252 failed formatting.
- 1794530485796346944 (121.0s) passed the new test alone.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated. Work is
constant per operand, and the existing storage and width growth invariant
passes. No separate scaling run is added.

## Core progress

No core observation of this change has run. The unchanged 120s observer last
stopped at item 10246 ([clean candidate](pa31-gat-region-premises.md#core-progress)),
before 10511.
