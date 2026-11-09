# Inline assembly transport slice (B2 A)

This is Lane D's third B2 item, on `implement/asm-transport-20261109` from
`28e2b19fb`, under the integrator's approval of the A notice (amendments
A1-A11). Version 13 (`body_nodes()`) now exports the inline assembly forms of
B1's initial assembly profile as value code 0 and a cold `Original.assembly`
column. Fresh consumers rebuild each body's payload, re-prove its unsafe
context and lower one template per emitted instance through B1's own path.
Older profiles keep every rejection.

Before this slice, `cfg_rows/value_rows.rs` rejected `Rvalue::Assembly` with
"source rvalue requires a complete wider transport row". B1's pointer-only
transport control (`asm-native/opponents/transport.rs`) stopped there. That
control now exports, which `pa41/tests/spec/asm-native/contract.json` records
as an amendment with the README's new hash.

## Rows

Ids are 4-byte LE u32 and ranges are two ids. Every new row is admitted only
under `body_nodes()`.

| Row | Bytes |
| --- | --- |
| Value code 0 `Assembly` | `0 \| 2 \| Flag(instruction) \| Range(arguments)` |
| Instruction | `site \| templates \| operands \| options u16 \| safety u8` |
| Template | `text \| pieces` |
| Piece | `start \| end \| operand \| modifier u8` |
| Operand | `class u8 \| input option \| output u8 \| ty \| late u8` |

- `Original.assembly` follows `texts`. Both sides derive its presence from
  the values: it is present exactly when some value is code 0, and an
  asm-free body allocates none. A writer whose column and values disagree
  fails with "source assembly columns differ from the assembly values of its
  body".
- An older writer (the legacy value writer and the version-12 branch writer)
  and `supplied_profile` reject the value with "source inline assembly
  requires the version-thirteen body IR profile".
- Decode admits only the profile's bytes. An option bit outside the eight
  defined ones fails with "unknown source assembly option". A defined option
  other than `nostack` and `preserves_flags` fails with its B1 lowering text.
  Safety 0 fails with "source inline assembly has no unsafe context", 1 is an
  explicit unsafe block, 2 an unsafe function, and 3 and above fail with
  "unknown source inline assembly safety". Any modifier byte and any output
  byte fail with B1's modifier and output texts.
- Class 0 is `reg` and 3 is `xmm_reg`. Tags 1, 2 and 4 (`reg_abcd`,
  `reg_byte`, an explicit register) are reserved: each fails with its B1
  text before any payload, and the writer has no layout for them. Tags 5 and
  above fail with "unknown source assembly register class".
- B1's texts live in one table, `support::initial_assembly`, shared by the
  parser's option bits, lowering, the exporter's preflight and the decoder.
  A form outside the profile fails with the same text whichever of them
  meets it first.

## Exporter

- After a body validates, `cfg_rows/instruction_rows::preflight` admits the
  lowered payload with B1's profile check (`assembly::profile::admit`): a
  `reg` input must be a raw pointer to a sized pointee and an `xmm_reg` input
  a nominal. Every form outside the profile keeps its lowering text at
  export, before any body row is written.
- The value maps to code 0 with its instruction ordinal and argument range.
  The columns copy the lowered payload: each instruction's site is the body's
  own site, its operand types are type rows, and its safety is the lexical
  context sema retained for that invocation.
- Sema records each asm's context in `Declarations::assembly_contexts`, cold
  and allocated on first use, only in a source provider. A replay must find
  the same context. `DECLARATIONS_SIZE` records the new pointer.

## Consumer and validation

- **Shared validation** (`body_ports/instruction_ports.rs`) runs in every
  consumer's staging, before any proof. Assembly values are dense in value
  order, one per instruction, and the column is exactly that long. The body
  must be a runtime body: a constant context fails with B1's "inline assembly
  is not permitted in constant contexts". Template, operand and piece ranges
  partition their columns. Each instruction's site, options and safety are
  checked, an unsafe-function context requires an unsafe header ("source
  inline assembly has no matching unsafe context"), and each placeholder must
  lie inside its text, in order, naming an operand. Each value is a unit
  effect whose arguments are its operands' inputs, in order and with their
  types; no input is late or an output. A `reg` input must be a raw pointer
  to a sized pointee and an `xmm_reg` input a nominal.
- **SSA.** `dependency()` lists explicit arms for every value kind, and an
  assembly value's arguments are its fields, so each input must be dominated
  by its evaluation (row 22).
- **Rebuild.** `encoded_assembly` rebuilds a `Payload` owned by the imported
  body's origin, with each site read through the original's own sites. A cached
  body must still match its rows ("source original changed its actual
  assembly payload"). The import pays each row and template byte once.
  `safety_requests` re-proves each recorded context against the imported
  header, as for unsafe calls. An `xmm_reg` operand re-proves its SIMD type
  and the target's SSE baseline, though no 128-bit vector has a type row
  before S and T.
- **Lowering.** An instance of an imported body copies the payload with the
  body, without a source-core charge (a client has none), and lowers through
  B1's `asm_invocations`. Each emitted instance gets its own template.
- The instruction identifies its invocation by `BodySite` and the payload
  by `BodyOrigin`, so local and imported bodies share one representation.

## Outcomes

Declared in `pa41/tests/asm-transport-outcomes.json` at
2026-11-09T04:20:08Z and amended at 04:35:02Z, before any run. After the
first oracle run, rustc rejected five controls because the foundation makes
only raw pointers and f32 Copy. Each generated control then gained u8 and u64
Clone/Copy impls in the foundation's own form (05:34:19Z), before any
development or candidate run. The baseline's local twins were recorded at
05:35Z.

Two predictions failed in development runs, before any candidate run. Each
is kept in the JSON with its amendment.

- **The envelope** (06:58:06Z). The provider exports, but its packet is
  Lane B's declarative macro envelope (tag 14, version 2) around a
  `[13, 0]` core, not a plain core. The frozen foundation's source-owned
  `pub macro asm` is an exported builtin macro row, which needs envelope
  version 2.
- **The comment template** (07:05:26Z). `stored` exited 42, but the pinned
  CPPGM backend rejected `tagged`'s LowIR with "assembly line 1: expected
  module assembly identifier". Its module assembly text strips `#` line
  comments but has no `/* */` block comments, which rustc's LLVM assembler
  accepts. The provider's `tagged` template changed from `/* {p} */` to
  `# {p}`, still with no effect and no options. The oracle was rerun on the
  amended provider (`d5b91a59…`) and every declared rustc outcome held.

The declared outcomes:

- The provider exports in v13. `stored` and `tagged` exit 42 natively.
  `stored` holds two templates with B1's `nostack, preserves_flags` and
  `nostack` headers. `tagged` holds one template per emitted instance, two,
  each with the no-option header.
- `unsafe-free` fails with "call to unsafe function requires an unsafe
  context" and `mistyped` with "incompatible types", their local twins'
  texts.
- v12 and v10 keep the baseline's "source type form requires an explicitly
  supported transport row" and write no packet.
- Each of the 15 generated export controls fails at export with its own B1
  text from `asm-native/opponents.tsv`, and writes no packet.
- The stream witness still fails with "source nominal representation
  requires its wider complete contract".

## Evidence

The baseline is the K candidate (SHA-256 `9fab7f87…`). The A candidate CLI
is `6552b73b…`, pinned under `obj/parallel/asm-transport/candidate`. Its
receipt is `obj/parallel/asm-transport/receipts/a-candidate.txt`, and the
amended oracle's is `oracle-amended-provider.txt` beside it.

- **Outcomes.** Every declared outcome holds, with the two amendments above.
  The provider packet is `420c8e74…` in all three of its publications.
  rustc 1.98.0 accepts the provider, both clients and all 15 controls, and
  rejects the opponents with E0133 and E0308.
- **Byte identity.** The baseline and the candidate publish identical
  packets for five providers: pa29's transparent-records, panic-attributes,
  macro-contexts and const-parameters, and the pa41 prelude-only crate.
- **Old readers.** The K candidate consuming the A packet fails with
  "encoded Rvalue::Assembly is not reconstructable by this consumer". The
  legacy and version-12 writers refuse the value by name
  (`register_wire`).
- **Forged rows.** At decode, `register_wire` rejects class tags 1, 2, 4 and
  5, an undefined option bit, `nomem`, an output flag, safety 0 and 3, and a
  modifier, each by name. The writer refuses the reserved classes too. In
  rows, `instruction_rows::tests` rejects rows 4, 8, 10-13 and 15-21 by name.
  A fresh consumer rejects a forged `nomem` and an undefined option bit in
  the packet (`register_transport`). Row 22 is the SSA domination control in
  `ssa_projection_ports`.
- **Width.** Compile-time assertions pin `Instruction` at 24 bytes (28
  before), `Payload` at 112 (104 before), `s::ValueKind` at 20, `s::Value`
  at 24 and `s::Original` at 408 (400 at `28e2b19fb`, plus one cold column
  pointer). `Declarations` gains one pointer, recorded in
  `DECLARATIONS_SIZE`.
- **Bounded work.** With N pointer inputs and N templates at N = 8, 16 and
  32, packet bytes are 8819, 10307 and 13283, and charged validation steps
  are 8705, 9873 and 12209. Both have zero second difference. Asm-free
  bodies allocate no column. K's follow-up, one application of N const
  formals, charges 47339, 47907 and 49043 steps, also with zero second
  difference.
- **Focused checks.** `make fastcheck` passed 31 exact private tests and 3
  linking tests (`obj/qualification/fast/1794208272949747970`):
  - the register wire, rows and scaling invariants, and the K follow-up;
  - every `ssa_projection_ports` case, including row 22;
  - both `term_codec` cases and `valued_slots`;
  - the ordinary and cast column round trips;
  - all eleven `asm_effect_contracts`, `asm_lowering` and `asm_vector_inputs`
    cases, and `asm_cpu_contexts`.

  With `--ignored`, the native client test passed
  (`obj/qualification/fast/1794208398530478168`). Two of the tests that
  check `DECLARATIONS_SIZE` pass on the same private binary,
  `collection_layout_preserves_scalar_owners_and_padded_derive_entries` and
  `iterator_clone_preserves_ordinary_record_sizes_and_lazy_owner_cost`.
  `make test-upstream` passes.

## Remaining

- Outputs, `inout`, clobbers, explicit registers, `reg_abcd`/`reg_byte`,
  integer and function `reg` inputs, modifiers, AT&T syntax and the other
  options stay outside the profile, at export and at lowering.
- `xmm_reg` inputs export only once S and T give 128-bit vectors type rows.
  The stream witness still stops at `__m128`'s `repr(simd)`.
- Closure and inline-const originals cannot reach the asm rows yet (A1).
