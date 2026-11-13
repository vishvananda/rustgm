# Assembly zmm, kreg and f16-lane operands

Twenty-six std-form libraries check `zmm_reg`, `kreg` and `f16`-lane vector
operand typing against rustc 1.98. Four reduce pinned core's `core_arch`
AVX-512 forms, each with the 64-bit `vps` or `vpl` macro of `x86/macros.rs`:

- `stream-ps512`: `x86/avx512f.rs` `_mm512_stream_ps`, a 512-bit vector in
  `zmm_reg`;
- `mask-load-ss`: `x86/avx512f.rs` `_mm_mask_load_ss`, a `kreg` mask and an
  `inout(xmm_reg)` vector;
- `mask-load-sh` and `mask-store-sh`: `x86/avx512fp16.rs` `_mm_mask_load_sh`
  and `_mm_mask_store_sh`, a `kreg` mask and an `__m128h` of eight `f16`
  lanes in `xmm_reg`.

Their vectors are local `repr(simd)` declarations with the original lanes,
and `__mmask8` is core's `u8` alias. Each reduction's first comment names its
origin. The other cases are authored.

rustc's x86 register table admits `zmm_reg` under `avx512f` for 32- and 64-bit
integer and float scalars and for 128-, 256- and 512-bit vectors, with the
`x`, `y` and `z` modifiers. It admits `kreg` for 8- and 16-bit integers under
`avx512f` and for 32- and 64-bit ones under `avx512bw`, with no modifiers.
Its `xmm_reg`, `ymm_reg` and `zmm_reg` vector types include `f16` lanes.

- Positives:
  - the four core forms;
  - `zmm-i64x8`, `zmm-i32x4` and `zmm-f32x8` pass 512-, 128- and 256-bit
    vectors, and `zmm-scalar-f64` a scalar;
  - `zmm-modifier-y` uses the `y` modifier, and `zmm-output` is an
    `out(zmm_reg)`;
  - `kreg-u16`, `kreg-u64-bw` under `avx512bw`, and `kreg-fp16-implies`,
    whose `avx512fp16` implies `avx512f`;
  - `xmm-f16x8` and `ymm-f16x16` pass `f16` lanes.
- Negatives:
  - `zmm-no-avx512` and `kreg-no-avx512` enable only `avx2`;
  - `kreg-u32-no-bw` passes a 32-bit mask without `avx512bw`;
  - `kreg-f32`, `kreg-vector` and `zmm-scalar-u8` have no type in the class;
  - `kreg-modifier` and `zmm-modifier-e` use modifiers the class lacks;
  - `zmm-wide` (1024 bits), `zmm-short` (64 bits) and `xmm-f16x16` (256 bits
    in `xmm_reg`) do not fit.

Every positive types in RustGM. Native lowering then stops at the initial
assembly profile, at its first fence in profile order: options, then each
operand's output and class, then comments.

- The `zmm_reg` cases stop at "inline assembly zmm_reg operand is outside the
  initial assembly profile", and the `kreg` ones at its `kreg` text.
- `zmm-output` stops at the output check.
- `mask-load-ss` and `mask-load-sh` stop at their `pure` option.
- `ymm-f16x16` stops at the `ymm_reg` text.
- `xmm-f16x8` passes the class and stops at its `/* */` comment.

Module-body export applies the same profile, so no published body carries a
zmm or kreg operand.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. All 26 Rust
declarations agree. `obj/parallel/asm-avx512/core/declared.json` records the
batch, and three profile stops that the declaration predicted wrongly. The
reduced cases keep Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/avx512_operands.rs` checks every case's typing and
lowers every positive. It also exports `stream-ps512` on the native prelude.
See `docs/pa41-avx512-operands.md`.
