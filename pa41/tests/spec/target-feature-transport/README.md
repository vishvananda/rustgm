# Target-feature function transport packet

`../../target-feature-transport-outcomes.json` freezes two providers, eight
clients, their hashes, edition, target, independent outcomes, the generated
export controls, the local twins and the forged rows. They were declared at
2026-11-09T12:30:08Z, before any oracle, development or candidate run of these
files, with two wording corrections at 12:31Z. Its two amendments are
recorded under `obj/parallel/target-feature-transport/f-amendments` and
listed in
[target-feature-rows.md](../../../../docs/parallel/target-feature-rows.md).
Amendment 2 withdrew the declared inherent method `Lane::get` and its
`inherent` client: unsafe inherent methods are fenced at export under every
profile, so the declared provider could not be exported.

## Provider

`provider.rs`'s lines 1-73 are `../asm-native/native.rs` through `__m128`'s
Clone and Copy impls, byte for byte, as in `../simd-transport/provider.rs`.
Their provenance and attribution are recorded in `../asm-native/README.md`.
Everything after the line `// Authored target-feature transport section.` is
authored:

- Clone and Copy for `u8` and `u32`;
- `second(a, b) -> b` with `enable = "sse2"`. It replaces an approved `add`:
  the foundation declares no `add` lang item, so `no_core` `+` would not
  type-check;
- the generic `keep<T: Copy>` with `enable = "sse2,sse"`, which is written as
  the declared set `sse, sse2`;
- `wide` with `enable = "avx"`, which is metadata and LowIR only: no control
  ever executes AVX;
- the forgery targets `plain` (safe), `bare` (unsafe) and `zero` (const).

## Clients

Each client enables the foundation's `asm_native_fixture` library feature and
returns 42 from `entry`.

- `call`, `generic` and `pointer` are accepted and run natively.
  `pointer` coerces the imported `unsafe` target-feature function item to an
  `unsafe fn` pointer.
- `wide` is accepted, LowIR only.
- `gated` calls `legacy` from the generated provider whose `x87` feature needs
  `x87_target_feature`. The client declares no gate: rustc checks the gate only
  in the defining crate.
- `unsafe-free` calls `second` outside an unsafe block (E0133).
- `safe-pointer` coerces `second` to a safe `fn` pointer (E0308).

## Stream witness

`stream-provider.rs`'s lines 1-95 are `../asm-native/native.rs` before its
authored observers, byte for byte: the real `_mm_stream_ps` body. After the
line `// Authored stream witness section.` are `rg_vector`, an unsafe,
non-const stand-in for `_mm_setr_ps` with `#[inline]` and `enable = "sse"`,
and native.rs's observer bodies moved into Rust-ABI functions.
`stream-client.rs` defines the six `rg_*` symbols that
`../asm-native/observer.c` calls, through the provider's functions.

## Generated controls

The export opponents, the older-profile gate, the gated provider and the local
twins are generated under `obj/parallel/target-feature-transport`. Each export
opponent is the provider's first 73 lines, the section comment and one line;
`opponent-inherent` is the withdrawn `Lane::get` on one line. The private
inherent forgery is the provider followed by a safe `Lane::get`.
The older-profile gate is `pa29/tests/spec/source-nominals/provider.rs`
followed by `second`'s line. The local twins append the client's body to the
foundation, the Copy impls and the one function it calls.
