# Source Neg and real enum discriminants

Source unary minus now resolves the genuine Neg trait and owned method before
primitive evaluation. It reuses the source Not header walk, ordinary trait and
const-effect obligations, existing pending projection normalization and selected
method/body machinery. Primitive syntax emits the checked scalar operation only
after those proofs; nonprimitive overloads retain real source calls. Negative
literal bookkeeping still admits the genuine minimum signed literal.

The transient descriptor remains12 bytes. No role ordinal, body/IR field or
cache is added. The deferred normalizer connects the actual operand before
requiring its associated Output, preserving ordinary numeric inference. Real
enum discriminants use the existing anonymous-constant producer.

The integrator authenticated the isolated compiler component af5ba23cc and native
test component05d12bb27 against the worker's complete committed source maps,
passing receipts and raw streams. No unfinished handler implementation is merged.
The original six controls and ten lexer tests pass on qualified source2b094e731;
the new producer has its separately authenticated focused and fixture-replay
evidence. Earlier projection, stale-Assert and native setup failures remain
retained, rather than being relabeled as successes.

The combined root has21 passing exact controls: eleven private cases cover
Neg/Not, actual const/discriminant/instance lowering, cfg behavior, closed wire
profiles and existing transmute/admission/restriction guards; ten are ordinary
lexer controls. Metadata, formatting, architecture, both strict Clippy modes and
build passed at `obj/qualification/fast/1793102426356085047/receipt.json`.
That gate then failed because one selector used a filename instead of the actual
Rust module alias. A source-identical continuation verified the built binaries
and reran only inventory/execution with the corrected selector. All21 tests pass
in `obj/parallel/captured-cfg-integration-v1/neg-continuation-v1/receipt.json`
(SHA-256 `954600fb35e55904f57381e40326d1d7dd88ed0eacd602fbded5e947f2766209`).
An earlier cold build exceeded its five-minute gate and remains recorded as a
failure. No passing test is attributed to that timeout.

Four stale cold-layout assertions now match the actual combined types:
Features8, Catalog1376; SourceCore176, Resolution2056, Package256, Source184,
Body448 and the ordinary syntax records retain their checked widths.
The changes keep the original behavioral tests and all narrow hot-layout checks.
Harness registration, all225 script controls and24+126 upstream pins pass.

## Adapted native evidence

Pinned Rust and the locked backend both execute to `-1 -7` with exit0 and empty
stderr. Their literal source trait/impl/repr(isize) enum/constant prefix is equal.
The adapters differ explicitly: Rust returns the enum through its C ABI, while
the candidate uses its actual typed enum-to-isize cast. The original Rust cast
adapter requires a real panic helper, and the candidate enum-return adapter
exposes an unsupported C enum ABI. Both failures remain explicit requirements;
this comparison does not claim exact-source crate parity or completed enum ABI.

The integrator authenticated both frozen plans, all sixteen candidate inputs,
three independent source inputs and raw observations. Its new combined producer
emits LowIR with SHA-256
`149004c000aca89c133d09178df221abda3dba3a05a3fc993fccde4413597efd`, identical
to the actual worker native input. This preserves that native observation and
does not claim another native execution. Bindings are under
`obj/parallel/captured-cfg-integration-v1/neg-native-*.json`.

Full original Neg diagnostic/overflow attributes, varying unchecked/foreign
overflow policy, the complete Formatter graph, panic handler and broader
transport remain required. PA26/PA31 and the original libraries remain unfinished.
