# Assembly zmm, kreg and f16-lane operands

Release candidate6's core census (`obj/parallel/capture-match/candidate6-release`,
54039f5d, clean 0d5530cca) fails thirteen `core_arch` AVX-512 bodies while
typing:

- `x86/avx512f.rs`, "assembly register class is not implemented for this
  target":
  - `_mm512_stream_ps` 12882, `_mm512_stream_pd` 12883,
    `_mm512_stream_si512` 12884 and `_mm512_stream_load_si512` 12885
    (`zmm_reg`);
  - `_mm_mask_load_ss` 13314, `_mm_maskz_load_ss` 13315, `_mm_mask_load_sd`
    13316, `_mm_maskz_load_sd` 13317, `_mm_mask_store_ss` 13342 and
    `_mm_mask_store_sd` 13343 (`kreg`).
- `x86/avx512fp16.rs`:
  - `_mm_mask_store_sh` 16255 (`kreg`), with the same failure;
  - `_mm_mask_load_sh` 16243 and `_mm_maskz_load_sh` 16244, with "assembly xmm
    vector requires exactly 128 bits of supported numeric lanes". Their
    `inout(xmm_reg)` operand comes first, and it is an `__m128h` of eight
    `f16` lanes.

```rust
#[inline]
#[target_feature(enable = "avx512f")]
pub unsafe fn _mm_mask_load_ss(src: __m128, k: __mmask8, mem_addr: *const f32) -> __m128 {
    let mut dst: __m128 = src;
    asm!(
        vpl!("vmovss {dst}{{{k}}}"),
        p = in(reg) mem_addr,
        k = in(kreg) k,
        dst = inout(xmm_reg) dst,
        options(pure, readonly, nostack, preserves_flags),
    );
    dst
}
```

The parser knew no `zmm_reg` or `kreg` class, and lane widths had no `f16`.

## Rule

rustc's x86 register table (`rustc_target/src/asm/x86.rs` at the pinned
revision) admits:

- `zmm_reg` under `avx512f`: 32- and 64-bit integer and float scalars and
  128-, 256- and 512-bit vectors, with the modifiers `x`, `y` and `z`;
- `kreg` for `i8` and `i16` under `avx512f`, and `i32` and `i64` under
  `avx512bw`, with no modifiers;
- `f16` lanes (`VecF16`) in `xmm_reg`, `ymm_reg` and `zmm_reg` vectors.

The declared batch confirmed each point against rustc 1.98.0.

## Change

- **Classes.** `zmm_reg` parses to `Register::Zmm` and `kreg` to
  `Register::Kreg`. Explicit zmm and mask registers are still rejected by
  name.
- **Typing.** Both classes need `avx512f` in the body's available features
  ("assembly register class requires the avx512f target feature"). The
  existing closure supplies it from `avx512bw` and `avx512fp16`.
  - `zmm_reg` scalars follow the `xmm_reg` rule. A vector must be a completed
    genuine vector of 128, 256 or 512 lane bits ("assembly zmm vector requires
    128, 256 or 512 bits of supported numeric lanes"). Modifiers follow
    `xmm_reg`.
  - A `kreg` operand is an 8-, 16-, 32- or 64-bit non-float scalar, pointer
    included. 32 and 64 bits need `avx512bw` ("assembly kreg operand of 32 or
    64 bits requires the avx512bw target feature"). A float or vector is
    "incompatible with register class", and every modifier is rejected.
  - The shared lane width computation counts `f16` lanes as 16 bits, so an
    `f16x8` fits `xmm_reg`, as rustc's `VecF16(8)` does.
- **Profile.** The initial assembly profile rejects both classes before native
  lowering and module-body export ("inline assembly zmm_reg operand is outside
  the initial assembly profile", and the same for `kreg`). No `LowIR` or
  published body carries one.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- Core's thirteen bodies now type, so the typing census passes them. The core
  build still stops at the profile until a backend contract for 512-bit
  vectors and mask registers exists. The backend is pinned at v3.
- An `f16x8` vector in `xmm_reg` is now inside the profile. Its lowering
  passes the vector's 16-byte snapshot address, as for any 128-bit vector,
  without reading its lanes. Module-body transport re-proves it with the
  same lane check. The packet's `xmm-f16x8` stops at its `/* */` comment
  fence, so no `f16` vector reaches `LowIR` in the packet.
- `f16` and `f128` scalars, which rustc admits in `reg`, `xmm_reg`, `ymm_reg`
  and `zmm_reg`, stay "not supported as an assembly register operand". So do
  `f128` lanes.
- Admitting typing does not weaken the forged target-feature rule. A forged
  `avx512f` on an imported function could widen only the CPU check of a body
  with a zmm or kreg operand, and export rejects every such body. `spec.md`
  still requires a revisit before the profile admits any class that needs a
  non-baseline feature.
- RustGM's messages differ from rustc's. Every rejection is at typing, where
  rustc rejects.

## Contract

**PA41** (`pa41/tests/spec/asm-avx512/`, 26 cases). All Rust declarations agree
with rustc 1.98.0 (`outcomes.json`). The cases run through the library
invariant on the `no_core` foundation, as the `asm-ymm` cases do. The base,
candidate6, rejects every case at typing.

| Case | Form | Rust | RustGM |
| --- | --- | --- | --- |
| `stream-ps512` | core's `_mm512_stream_ps` | accept | profile (zmm) |
| `mask-load-ss` | core's `_mm_mask_load_ss` | accept | profile (`pure`) |
| `mask-load-sh` | core's `_mm_mask_load_sh` | accept | profile (`pure`) |
| `mask-store-sh` | core's `_mm_mask_store_sh` | accept | profile (kreg) |
| `zmm-i64x8` | a 512-bit vector | accept | profile (zmm) |
| `zmm-i32x4` | a 128-bit vector | accept | profile (zmm) |
| `zmm-f32x8` | a 256-bit vector | accept | profile (zmm) |
| `zmm-scalar-f64` | an `f64` scalar | accept | profile (zmm) |
| `zmm-modifier-y` | the `y` modifier | accept | profile (zmm) |
| `zmm-output` | an `out(zmm_reg)` | accept | profile (output) |
| `kreg-u16` | a `u16` mask | accept | profile (kreg) |
| `kreg-u64-bw` | a `u64` mask under `avx512bw` | accept | profile (kreg) |
| `kreg-fp16-implies` | a `u8` mask under `avx512fp16` | accept | profile (kreg) |
| `xmm-f16x8` | eight `f16` lanes in `xmm_reg` | accept | profile (comment) |
| `ymm-f16x16` | sixteen `f16` lanes in `ymm_reg` | accept | profile (ymm) |
| `zmm-no-avx512` | `zmm_reg` under `avx2` | reject | typing |
| `kreg-no-avx512` | `kreg` under `avx2` | reject | typing |
| `kreg-u32-no-bw` | a `u32` mask without `avx512bw` | reject | typing |
| `kreg-f32` | an `f32` mask | reject | typing |
| `kreg-vector` | a vector mask | reject | typing |
| `kreg-modifier` | the `x` modifier on `kreg` | reject | typing |
| `zmm-wide` | a 1024-bit vector | reject | typing |
| `zmm-short` | a 64-bit vector | reject | typing |
| `zmm-modifier-e` | the `e` modifier on `zmm_reg` | reject | typing |
| `zmm-scalar-u8` | a `u8` scalar | reject | typing |
| `xmm-f16x16` | sixteen `f16` lanes in `xmm_reg` | reject | typing |

"Profile" means the case types and native lowering stops at the named
initial assembly profile fence. The declaration predicted the `kreg` and output
fences for `mask-load-ss` and `mask-load-sh`, and lowering for `xmm-f16x8`.
The profile checks options first and comments last, so they stop at their
`pure` option and `/* */` comment. The record keeps both.

## Qualification

One focused receipt passes with exact selections on the final source:
1794547593986105559 (181.0s), shared with the
[blanket candidate winnowing scan bound](pa31-blanket-candidate-winnowing.md).
It covers:

- assignment metadata, formatting and the architecture audit;
- strict Clippy for the traits, projections and linking suites and the CLI,
  and strict private Clippy;
- `avx512_operands_type_like_rustc_and_stop_at_the_assembly_profile`. It
  requires each rejection's reason, lowers every positive to its profile
  fence and exports `stream-ps512` to its zmm fence;
- the existing ymm, vector, lowering, scalar and place assembly invariants
  (eight), and `asm_storage_and_width_growth_remain_cold_and_body_owned`;
- `enabled_callables`' two target-feature export tests;
- PA20's text oracles. PA41 is planned, and its text stage refuses to run
  (1794547199809374980).

Earlier attempts, in `obj/parallel/asm-avx512/fastcheck1.log` to
`fastcheck4.log`, failed formatting twice, the `f16` feature of the
foundation, and the three mispredicted profile stops with the `mask-load-ss`
export alias. 1794546806234981648 failed private Clippy's single-element loop.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated. Work is
constant per operand, and the existing storage and width growth invariant
passes. No separate scaling run is added.

## Core progress

Release candidate8 (`obj/parallel/capture-match/candidate8-release`, clean
53667da5b, CLI ae92eca9) carries the bounded scan, the derived field path
bounds and the AVX-512 operands. The unchanged 120s observer ran it
exclusively from load 9.5. It reached the end of the body pass in 23.0s, with
all 35,120 item attempts, 8 failed and 13 blocked bodies, against
candidate6's 27 and 14. Every candidate8 record is a candidate6 record, so no
previously passing body failed. `_mm512_movm_epi16` 14975 passes. All thirteen AVX-512 bodies, 12882-12885, 13314-13317, 13342,
13343, 16243, 16244 and 16255, pass typing. A core build would still stop
at the initial assembly profile for them.
