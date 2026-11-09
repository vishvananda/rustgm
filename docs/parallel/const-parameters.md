# Const-parameter slice (B2 K)

This is Lane D's second B2 item, on `implement/asm-stream-stage-b` after M
(`46fc6f14d`), under Lane C's K conditions in the B2 notice. Version 13
(`body_nodes()`) now exports const generic parameters of integer, bool and
char types, with evaluated and rigid const arguments in nominal and function
applications. Older profiles keep every rejection.

Before this slice, `authored_export::kind` rejected any `DefKind::Generic` in
the value namespace with "source const parameters are outside the initial
source profile". The real `_mm_stream_ps` crate stopped there, at `const N:
usize` in `impl<T: Copy, const N: usize> Clone for [T; N]`. That text stays
for older profiles.

## Rows

Ids are 4-byte LE u32. Every new tag is admitted only under `body_nodes()`.

| Row | Tag | Bytes |
| --- | --- | --- |
| Declaration `ConstParameter` | 12 | the existing declaration row |
| Parameter slot `Const { ty }` | 3 | `owner \| ordinal \| index \| definition \| 3 \| ty` |
| Generic argument `Const(ConstRef)` | 2 | `2 \| term` |
| Const term, value | 0 | `0 \| ty \| bits` (16 bytes LE) |
| Const term, rigid | 1 | `1 \| slot` |

- Declaration tag 13 is reserved for Lane C's 5b `Constant`; K leaves it free.
- A const slot's `index` is the space it shares with type slots, so
  `Counts.types` and `complete_type_slots` include const slots. The
  mixed-order fixture interns the same `Parameter::index` as a local
  definition with the const slot first (`struct Mixed<const N: usize, T>`)
  and last (`impl<T, const N: usize>`).
- Tag 3 has no field for a sized flag or Sized address, so neither can be
  forged. A default is rejected by name, "source const parameter slot has a
  default", both by the writer and by validation (`slot_bindings`).
- A slot's type must be an integer, bool or char scalar row
  (`const_parameter_type`); a float or any other row fails with "source const
  parameter type is outside the initial profile".
- Each term is written in place at its single argument reference. The reader
  appends it to the dense cold column `ColdColumns::consts` and names it by
  position (`ConstRef`), as `pattern_ranges` are read. The writer rejects
  terms out of argument-row order and terms with no argument row.
- Term tags 2 and above are rejected by name: "unknown source const term".
  An aggregate const argument is rejected at export, "source aggregate const
  arguments are outside the initial profile".

## Values and their canonical check

Bits are `types::arguments::Constant`'s masked two's-complement target-width
bits, so i8 −1 travels as 0xFF. `const_value` is a function of the type
alone. The term's `ty` must be a primitive row ("source const value type is
not a primitive row"), and it follows the back-reference rule of every type
operand: it is a child of the row or application that holds the argument and
must complete before it. The bits must then be canonical for that type
(`ScalarValue::new`): bool 0 or 1, a char scalar value (no surrogate), an
integer within its width, usize at the target's 64-bit width. Otherwise the
value fails with "non-canonical source const value".

For Lane C's 5b, the term codec (`recipe_columns::const_term_write` and
`const_term_read`) and the check (`source::const_value`) are callable
outside the parameter path. "Only tag 0" is a policy of each position, not of
the term, and `const_value` accepts any bit pattern of an f32 or f64 width.
K's own positions add `const_parameter_type`, which rejects float types. Any
change Lane C needs there goes through the integrator.

## Validation and consumers

- **Applications.** A nominal argument's kind must match its formal's: a
  const argument in a type slot, or the reverse, fails with "source nominal
  argument changed its formal kind". A function application reads its
  callee's complete type-space formals, inherited first and each scope in
  declared order, in one walk of the scope chain; a mismatch fails with
  "source application changed its actual formal kind". Before K every such
  formal was a type, so existing applications check as before. In both, a
  const argument's type must equal the formal's ("source const argument
  changed its parameter type"), and a rigid term must name a const slot
  ("source const term names a non-const slot"). Const arguments in
  projections and opaque captures are rejected by name.
- **Port plan.** A value is the node `ConstValue { scalar, bits }`, held as
  four words so the node keeps its 20 bytes (a compile-time assertion). A
  rigid term follows its slot's rewrite as a type parameter does: through the
  nominal's own argument chain, or to the caller's term under an application.
- **Body inputs.** `BodyInputs.parameters` lists every formal in complete
  order, const ones included, since they share the index space. The exporter
  interleaves const formals among the contract's type parameters and changes
  nothing for a crate without them. A const formal has no sizing or
  well-formedness goal, and a rebuilt imported contract keeps only type
  parameters, as a local contract does.
- **Consumer.** A const slot interns `Parameter { owner, index, kind: Const
  { ty } }` and the rigid `ConstKind::Parameter` term. Its declaration cell is
  its type, and `generics.constants` maps the binding to the term, as for a
  local const parameter. A value maps to `ConstKind::Value`.
- **Folded anonymous bodies.** `-1` in `Signed<-1>` is an anonymous constant
  whose body sema retains, since it calls `Neg`. The type carries the value,
  so under version 13 transport skips an anonymous body recorded in
  `Declarations::evaluated_arguments`, unless it is an array length. Every
  other anonymous body keeps "source transport currently requires an original
  function owner without a relocated execution view".

## Outcomes

Declared in `pa29/tests/const-parameter-outcomes.json` at
2026-11-09T01:47:16Z, before any oracle or candidate run. Four provider
amendments are recorded beside it, each before the run it governs: i8's
`Copy` impl and the oracle's `panic_const_neg_overflow` after rustc's first
two runs; after development probes, `neg` by subtraction, since version 13
transports no unary rvalue; and at 2026-11-09T03:22:33Z, body applications
with const arguments and two clients, since the provider had none and the
validator's application formals were unexercised. A failed older-profile
prediction is kept with its amendment.

- The provider exports as a plain `[13, 0]` core. `tag`, `mixed`, `scalars`
  and `applied` exit 42 natively. `wrong-value`, `wrong-sign` and
  `misapplied` fail with "incompatible types", and `wrong-order` with
  "unsupported value declaration", the texts of their local twins.
- v12 and v10 keep "named, associated or generic alias transport requires its
  wider profile".
- A body reading `N` keeps "source rvalue requires a complete wider transport
  row". A const default keeps "source parameter defaults are outside the
  initial profile".
- The stream witness moves from the const-parameter text to "source nominal
  representation requires its wider complete contract", at `__m128`'s
  `repr(simd)`. `pa41/tests/spec/asm-native/contract.json` records the
  amendment and the README's new hash.

## Evidence

The baseline is the M candidate (SHA-256 `e6d20708…`), and the K candidate
CLI is `9fab7f87…`, pinned under `obj/parallel/const-parameters/candidate`.
The receipt is `obj/parallel/const-parameters/receipts/k-candidate.txt`.

- **Outcomes.** Every declared outcome holds. The provider packet is
  `9b3d16cd…` in both of its publications. `tag`, `mixed`, `scalars` and
  `applied` exit 42 natively. The four opponents, the older profiles, the
  two controls and the stream witness give their declared texts. rustc
  1.98.0 accepts the provider, the four clients and both controls, and
  rejects the opponents with E0308 three times and E0747 once.
- **Byte identity.** The transparent-records, panic-attributes and
  macro-contexts providers publish byte-identical packets under the
  baseline and the candidate (`27fabdbc…`, `4afda652…` and `5689ff3d…`).
  A crate without const formals takes the exporter's unchanged path.
- **Old readers.** The M candidate consuming the K packet fails with
  "unknown source declaration kind". Under a profile without
  `body_nodes()`, tags 12, 3 and 2 fail with "unknown source declaration
  kind", "unknown source parameter kind" and "unknown source generic
  argument kind" (`term_codec`). A relabelled version-12 packet fails at
  declaration tag 12 (`valued_generics`).
- **Forged rows.** Term tags 2, 3, 0x80 and 0xFF are rejected by name at
  decode. On the wire, a term-tag forgery, a surrogate and a usize above 64
  bits are rejected (`valued_generics`). In the rows, `valued_slots`
  rejects thirteen forgeries by name, among them a sign-extended i8, bool 2,
  a float slot, a slot default, a kind swap in a nominal and in a function
  application, and a retyped value at an own and an inherited formal.
- **Width.** `s::Type` (20 bytes), `s::Argument` (8), `s::ParameterKind`
  (12) and `s::ParameterSlot` (40) are pinned by `term_codec`. A replica
  of M's and K's definitions, compiled with rustc 1.98.0, gives 12, 40 and 8
  bytes before and after; `Type` gains no variant. The port-plan `Node`
  keeps 20 bytes (a compile-time assertion). `Declarations` gains one
  pointer for the cold evaluated set, recorded in `DECLARATIONS_SIZE`.
- **Bounded work.** Terms are one dense column, one per const argument row.
  An application's formals are read in one charged walk of its scope chain,
  linear in its complete slots, and the exporter's const-formal walk is
  charged.
- **Focused checks.** `make fastcheck` passed 9 exact tests
  (`obj/qualification/fast/1794196857706632721`): both `valued_generics`
  controls, `hygienic_exports`, `lifetime_methods`, both `term_codec` cases,
  `valued_slots`, the ordinary column round trip and the asm module-body
  fence. `make test-upstream` passes.

## Remaining

- A body reading a const parameter as a value (`Rvalue::ConstArgument`) and
  const defaults stay fenced.
- Aggregate (`adt_const_params`) arguments and float const parameter types
  stay outside the profile.
- Const arguments in projections and opaque captures, and array lengths
  (T, tag 14), are later items.
