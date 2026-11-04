# Backend float-reliability configuration

PA41 owns backend capability truth, PA31 owns source feature and macro authority,
and PA34 owns the explicit target/configuration binding. This buildout packet
does not release an assignment or establish a core/alloc/std artifact.

`contract-v2.json` and `cases-v2.tsv` are the active 74-case source manifest.
Pinned Rust 1.98.0 independently agrees with all 148 predictions across editions
2021 and 2024. All four keys share `cfg_target_has_reliable_f16_f128`:

- `target_has_reliable_f16`
- `target_has_reliable_f16_math`
- `target_has_reliable_f128`
- `target_has_reliable_f128_math`

False predicates still require permission. Short-circuit operands, configured
removal order, inactive attributes/bodies, string values, malformed values,
captured metadata and macro invocations retain the pinned behavior. A genuine
definition-site allowance permits its introduced predicate; forwarding a caller's
predicate grants no permission. The query feature grants neither the `f16` nor
the `f128` language feature. Compiler-owned target facts cannot be overridden by
custom cfg inputs.

All four values are false for the current fixed CPPGM backend. Its frozen LowIR
contract supplies no f16/f128 scalar operations. Query authorization changes none
of these values or the existing target facts. The independent LLVM backend reports
`1 1 1 0`; its native observer is checked against its own `--print cfg` snapshot.
Different backend values are explicit, independently bound configuration facts.

The unchanged `configuration-snapshot.rs` uses a real macro allowance rather than
requesting the new crate feature. It checks fifteen target/profile predicates.
Old and new RustGM produce identical constant-view bytes and the declared values.
Because `expand/config.rs` changes, the original-core observation uses a new
versioned configuration binding. Original source files, all other configuration
source pins, CLI inputs and finite limits remain unchanged.

`native-execution-v2.json` separately binds the source executable, C reporter and
two independently accepted edition outcomes. The four observed function bodies
are an unchanged byte prefix of `native-target-values-v2.rs`. The executable adds
an ordinary main/foreign reporter and genuine source receiver/Clone/Copy roles
and bool/u8 implementations. Actual RustGM checking and lowering through the
locked backend produce `0 0 0 0`. SourceCore library publication still rejects
macro/name provenance that its current metadata profile cannot encode. Retain
that required transport boundary; executable success grants no metadata authority.

Failed setup versions remain explicit: the original native fixture placed an
inner feature attribute after declarations, corrected in v2 without changing an
expected outcome. The first source-executable foundation omitted the real Copy
roles required by body checking; its v2 successor appends those declarations.
Metadata-only acceptance of that earlier foundation was insufficient execution
evidence. The old manifests and inputs remain available for reproducing failures.

The compiler reuses existing demanded-use classification, configured owner
witnesses and memoized macro-frame queries. The packed feature word and ordinary
records do not grow. Repeated-query controls vary allowance width and call count;
they also retain the prior atomic, UB-check and sanitizer contracts. See the
[implementation checkpoint](../../../../docs/pa41-float-reliability-checkpoint.md)
for exact evidence and remaining original-library requirements.
