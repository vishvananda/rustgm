# PA26 owned primitive binary trait families

The existing value-returning descriptor and checked factory now cover `Sub`,
`Mul`, `Div`, `Rem`, `BitAnd`, `BitOr`, `BitXor`, `Shl` and `Shr` beside `Add`.
No new dispatch or proof authority is introduced. A shared supported-hook query
admits candidates, while complete selection still checks exact canonical trait
identity, scalar domains, RHS and output. Integer shifts retain their independently
typed integer RHS; arithmetic and bitwise operations retain their existing scalar
legality rules. Every descriptor validates its exact canonical Output schema and
method instance before the ordinary factory can receive its private permit.

The normal body arithmetic route preserves overflow and division checks. Boolean
bitwise methods and floating arithmetic use their existing operations; floating
bitwise and unsupported operand pairs receive no primitive evidence. User traits
with the same spelling retain their normal impls and lexical ambiguity rules.
Assignment methods and their shared-reference RHS lifetime abstraction are unchanged.

`pa26/tests/primitive-binary-outcomes.json` declares 44 independently qualified
controls: 32 Rust acceptances and twelve rejections. Thirty-one accepted programs
execute through the external CPPGM backend. The accepted shared-reference RHS
control remains an explicit frontend boundary. Runtime self-checks cover generic,
qualified, dot and reified calls, mixed-width shift RHS, signed shifts, wide
integers, float arithmetic and Boolean bitwise operations. Eight trap programs
cover division and remainder by zero, signed MIN/-1, oversized shifts, subtraction
underflow and multiplication overflow. The earlier unchanged `primitive-add`
Sub fixture now executes; its metadata retains the superseded `a8dcf79f` boundary.

Evidence under `obj/parallel/pa26/binary/` retains all 44 preimplementation Rust
qualifications in `qualification.json`. `first.json` records the focused complete
source/native matrix, four actually executed private tests, strict release
all-target/all-feature Clippy and architecture audit1381/0. Each new family's
private control requires exactly one retained selection, validates its factory
and rejects a forged descriptor for a different canonical method. The existing
8/32/128 caller controls retain one factory and one dispatch selection, with
LowIR definitions12/36/132 and instructions49/145/529.

`final.json` records all 21 shared operator source/native groups passing, with
layout38 and pinned upstream24+126 checks green. No wall-time performance claim,
activation or reference regeneration
is included. PA26 still requires primitive reference/unary forms, comparison/Deref,
iteration, question-mark, further derives and formatting. The next lane assessment
returns to PA30 aggregate callable ABI, variadic interfaces and metadata/release
wiring rather than extending isolated primitive cases indefinitely.

The later [shared RHS checkpoint](pa26-shared-value-operands.md) closes the
unchanged owned-left/shared-RHS Add/Sub boundaries after independent source and
native qualification. The earlier observations above remain historical evidence;
the outcome files retain their superseded boundary fields. Genuine source-language
Add/Sub ownership and shared-left forwarding remain separately required.
