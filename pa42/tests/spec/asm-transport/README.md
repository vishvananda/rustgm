# Inline assembly transport packet

`../../asm-transport-outcomes.json` freezes one provider, four clients, their
hashes, edition, target, independent outcomes, the generated export controls
and the forged rows. They were declared at 2026-11-09T04:20:08Z and amended at
2026-11-09T04:35:02Z, before any oracle, development or candidate run of these
files. Two later amendments are recorded beside the outcomes they change. The
generated controls gained u8 and u64 Copy impls after rustc's first run, and
the provider packet's macro envelope is a failed prediction from the first
development run.

The provider's lines 1-69 are `../asm-native/native.rs` before
`#[repr(simd)]`, byte for byte: the lang-item prelude, the source-owned
`core::arch::asm` declaration and `concat!`. Their provenance and attribution
are recorded in `../asm-native/README.md`. Everything after the line
`// Authored asm transport section.` is authored:

- `store`, an `unsafe fn` whose explicit unsafe block writes 40 through one
  `reg` pointer input with `nostack` and `preserves_flags`;
- `bump`, an `unsafe fn` with no unsafe block, whose one asm has two templates
  that each add 1, with `nostack`;
- `tagged<T>`, a safe generic function whose inner unsafe block holds a
  comment template with no options.

The foundation's crate root is unstable under `asm_native_fixture`, so each
client enables that library feature.

The two accepted clients each return 42:

- `stored` calls `store` and then `bump` on a local;
- `tagged` emits `tagged::<u8>` and `tagged::<u16>`, two instances of one
  imported asm body.

Two opponents are rejected:

- `unsafe-free` calls `store` outside an unsafe block (E0133);
- `mistyped` passes a `*mut u8` to `store` (E0308).

The export controls are generated under `obj/parallel/asm-transport`: each is
the foundation followed by one `../asm-native/opponents/` body, made generic so
that the provider lowers nothing. The forged rows are applied to the
provider's own packet, except the SSA row, which is built directly in rows by
`dev/tests/invariants/ssa_projection_ports.rs`. The outcomes file's
`forged_row_levels` names the level each row is forged at.

The comment fence (2026-11-09T08:12:41Z, declared after its probe and before
any of its runs) adds the `block-comment` and `line-comment` controls (17 in
all), the accepted `quoted` control and forged rows 23 and 24. It changes no
file in this directory.
