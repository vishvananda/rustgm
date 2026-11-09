# Native inline assembly: the initial profile

Pinned revision: `88d9e12ae178fab0fb5cc050a94da85685d449ea`. `native.rs`
keeps two excerpts of the pinned stdarch sources byte for byte:

- `library/stdarch/crates/core_arch/src/x86/macros.rs` lines 102–107, the
  64-bit `vps!` macro
- `library/stdarch/crates/core_arch/src/x86/sse.rs` lines 2064–2077, the
  complete `_mm_stream_ps` declaration and body

Their hashes are in `contract.json`. Upstream licenses and attribution remain
those of the source repository. The lang-item prelude is the authored prefix of
`../source-simd/native-family-probe.rs`. The `core::arch::asm` declaration has
the form of `../asm-frontend/stream-ps.rs`.

Everything else is authored. Pinned core implements `Copy` and `Clone` for
`[T; N]` in `library/core/src/array/mod.rs`, and `__m128`'s `Copy` needs
them, so `native.rs` declares two impls of the same form.

The outcomes in `opponents.tsv`, `expected.stdout`, `stream-template.lowir`
and `contract.json` were declared before any observation. The one exception
is the stream body's publication error, which `contract.json` lists under
`observations_after_run`.

## Native consumer

`native.rs` is one source for both compilers. RustGM lowers it through the
real CLI as an executable. Its `main` calls `rg_observe` in `observer.c`, and
the pinned object backend links the two. Pinned Rust compiles the same source
as a library object, and `entry.c` supplies the C `main` that calls
`rg_observe`. The shared `observed_program` harness cannot be used: pinned
Rust needs a `start` lang item to build a `no_core` `fn main` executable, and
RustGM supports none.

The backend's `--native-object` path compiles with `cppgm++ -c` and links
with `cc`. This slice does not exercise standalone `lowir2native`; the
assembly handshake in `make test-backend` covers that path.

Both executables run the same observer:

- C passes `__m128` values by value to the exported Rust functions, so the
  Rust/C vector ABI is checked against both compilers.
- Each destination is 16 aligned bytes between two 16-byte sentinels. C
  checks the alignment, issues `_mm_sfence`, prints all 128 bits and stops if
  a sentinel changed.
- The lanes are a quiet NaN payload, -0, the least denormal, +inf, all ones, a
  negative signaling NaN, 1 and the least normal. Rust also builds one vector
  from bit-exact `f32` returns.
- `rg_reassigned` reassigns its `__m128` local after the asm, and
  `rg_operand_order` assigns it inside a later operand's expression. Both
  stores must keep the value the operand had when it was evaluated.
- `rg_stream_default` has no options, so its template may use the stack and
  red zone and clobbers the status flags.
- `tagged::<u8>` and `tagged::<u32>` are two instances of one generic body,
  and the `#[inline]` original is called from four sites. Each emitted body
  gets its own template: six in all.

Both objects must contain `movntps` with a memory destination and an xmm
source. The RustGM object must contain exactly six.

## Fences and controls

Each `opponents/*.rs` input is a form outside the initial profile that pinned
Rust accepts. `opponents.tsv` declares rustc's metadata status and RustGM's
lowering text for each one. Three inputs are positive controls instead:

- `joined.rs`: two template strings and `{{`/`}}` escapes
- `quoted.rs`: a quoted `.ascii` string holding `//` and `/*`, which is not a
  comment
- `transport.rs`: an `#[inline]` pointer-only asm body, appended to
  `native.rs`'s prelude (everything before `#[repr(simd)]`). The prelude
  alone publishes in version 13. With the body, it still lowers. Its
  module-body packet failed with "source rvalue requires a complete wider
  transport row" until B2's asm-transport slice
  (`docs/parallel/asm-transport.md`). It now exports the body's assembly
  value and its columns.

The real stream body's packet (`native.rs` before its authored observers)
was observed after the first run, not predicted. It stops earlier, at "source
macro/semantic-name provenance metadata is not implemented", because `vps!`
and `concat!` expand in it. The test pinned that observed text until B2's
macro-context slice (`docs/parallel/macro-contexts.md`) admitted those marks.
It then pinned the declared next stop, "source const parameters are outside
the initial source profile", until B2's const-parameter slice
(`docs/parallel/const-parameters.md`) admitted const formals. It then pinned
the declared stop after that, "source nominal representation requires its
wider complete contract", until B2's SIMD slice
(`docs/parallel/simd-header-rows.md`) carried `__m128`'s `repr(simd)` header.
It now pins the declared stop after that, "source named/foreign/target-feature
function metadata requires its wider profile", from `_mm_stream_ps`'s
`#[target_feature]`. `contract.json` keeps the B1 observation as recorded.

Amendment (B2 A's comment fence, `docs/parallel/asm-transport.md`): the
pinned backend's assembler reads only `#` comments. `block-comment.rs` and
`line-comment.rs` therefore fail with "inline assembly comments other than #
are outside the initial assembly profile", which is checked after the
modifiers. `joined.rs` held a `/* */` template, so it now uses `# {{ {p} }}`.
`quoted.rs` lowers and exports with its bytes intact. It does not run
natively, because the pinned backend rejects its `.ascii` directive with
"assembly line 2: expected assembly integer".

`stream-template.lowir` is the declared template for the original function.
The [assembly checkpoint](../../../../docs/source-asm-vector-join.md) records the
qualification. PA41 stays planned; this is not a graded suite.
