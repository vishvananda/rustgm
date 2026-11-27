# SIMD record transport packet

`../../simd-transport-outcomes.json` freezes one provider, two clients, their
hashes, edition, target, independent outcomes, the generated export controls
and the forged rows. They were declared at 2026-11-09T10:54:35Z, before any
oracle, development or candidate run of these files. One amendment is
recorded beside the outcome it changes: after rustc's first run, `store.rs`
stores into a `[u32; 4]` local instead of a local `V`, because the foundation
declares no `freeze` lang item.

The provider's lines 1-73 are `../asm-native/native.rs` through `__m128`'s
Clone and Copy impls, byte for byte: the lang-item prelude, the source-owned
`core::arch::asm` declaration, `concat!`, the genuine `#[repr(simd)] pub
struct __m128([f32;4]);` and the `[T; N]` impls. Their provenance and
attribution are recorded in `../asm-native/README.md`. Everything after the
line `// Authored SIMD transport section.` is authored:

- Clone and Copy for `u32`, since rustc requires every assembly operand
  type to be Copy;
- `#[repr(simd)] pub struct V(pub [u32; 4]);` with Clone and Copy;
- `put`, an `unsafe fn` whose explicit unsafe block stores `a: V` through
  `movups [{p}],{a}`, with one `reg` pointer input and one `xmm_reg` input,
  `nostack` and `preserves_flags`.

The foundation's crate root is unstable under `asm_native_fixture`, so each
client enables that library feature.

- `store` is accepted and returns 42. It calls `put` with
  `V([42, 1, 2, 3])` on a `[u32; 4]` local addressed as `*mut base::V`, then
  reads lane 0 through a raw pointer. Its fresh consumer re-proves `put`'s
  `xmm_reg` operand from its own SIMD import of `V`. `movups` does not need a
  16-byte-aligned destination.
- `mistyped` is rejected. It passes a `[u32; 4]` to `put` (E0308).

The export controls are generated under `obj/parallel/simd-transport`. Each
is the provider's first 73 lines followed by one SIMD declaration that
version 13 does not carry: packed, aligned, generic over a type or a const,
lane-limited, or combined with `C`. The older-profile gate is
`pa29/tests/spec/source-nominals/provider.rs` with `repr_simd` enabled and
one SIMD struct. The forged rows are byte patches of the provider's own
packet and of pa29's transparent wire control. The outcomes file's
`forged_row_levels` names the level each one is forged at.
