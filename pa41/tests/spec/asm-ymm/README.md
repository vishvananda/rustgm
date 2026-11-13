# Assembly ymm operands

Sixteen std-form libraries check `ymm_reg` operand typing against rustc 1.98.
`stream-ps`, `stream-pd` and `stream-si256` reduce pinned core's three
`core_arch` `x86/avx.rs` stream stores, `_mm256_stream_ps`, `_mm256_stream_pd`
and `_mm256_stream_si256`, with the 64-bit `vps` macro of `x86/macros.rs`. The
vector in each is a local `repr(simd)` declaration with the original lanes.
Each reduction's first comment names its origin. The other cases are authored.

rustc's x86 register table admits `ymm_reg` under `avx` for 32- and 64-bit
integer and float scalars and for 128- and 256-bit vectors. Its template
modifiers are `x`, `y` and `z`.

- Positives:
  - `stream-ps`, `stream-pd` and `stream-si256` are core's forms;
  - `vector-i32x4` and `vector-i16x16` pass 128- and 256-bit vectors;
  - `scalar-f64` and `scalar-u32` pass scalars;
  - `avx2-implies` enables only `avx2`, which implies `avx`;
  - `modifier-x` uses the `x` modifier;
  - `output` is a 256-bit `out(ymm_reg)`, the form of core's
    `x86/avx2.rs` `_mm256_stream_load_si256`.
- Negatives:
  - `scalar-u8` has no `ymm_reg` type;
  - `no-avx` enables only `sse`;
  - `wide` (512 bits) and `short` (64 bits) do not fit;
  - `modifier-e` uses a modifier the class lacks;
  - `xmm-256` passes a 256-bit vector in `xmm_reg`.

Every positive types in RustGM and then stops at the initial assembly profile
before native lowering ("inline assembly ymm_reg operand is outside the
initial assembly profile"). `output` stops first at the profile's output
check. Module-body export applies the same profile, so no published body
carries a ymm operand.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run of each file, and both are recorded in `outcomes.json`. All 16
declarations agree. `obj/parallel/asm-ymm/core/declared.json` records the
batch and an invocation error. The reduced cases keep Rust's MIT/Apache-2.0
attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/ymm_operands.rs` checks every case's typing and lowers
every positive. It also exports `stream-ps` on the native prelude. See
`docs/pa41-ymm-operands.md`.
