# Scoped method lookup

Seven outcomes were declared and independently checked as Rust1.98 objects before
implementation. Six accepts are rejected by baseline77ddb733 at its scoped-clause
fence; static-unavailable is independently rejected. The scalar traits and real
function-pointer Split shape belong to PA21. The source-language versions remain
in PA31 without changing their original bytes.

Keep whole quantified goals through both qualified and receiver lookup. A static
implementation cannot satisfy a universal bound; a diagonal implementation cannot
satisfy two independent slots. Those impossible inherent candidates must permit
the ordinary trait fallback. The fallback's u16 result differs from the inherent
u8, so complete LowIR verifies the chosen callable's real signature. Caller
universal evidence remains valid, and the actual selected call proves all debts.

Outer captured binders and quantified outlives lookup clauses retain their named
fences. Caller-dependent or unresolved conditions remain possibility checks;
lookup publishes no proof authority. No generated references precede the frozen
outcomes or independent object checks.

Five later sources name `'static` inside the quantified condition,
`F: FnMut(&u8) -> &'static u8`, the shape of an impl whose closures return
`Result<_, &'static str>`. `'static` names no enclosing scope, so the binder is
as closed as one naming only its own regions. Their outcomes were declared on
2026-11-21 before the Rust 1.98 run and before any RustGM run of these bytes.
Receiver and `Self::` lookups accept, and so does the qualified path, which a
baseline without a receiver already accepted. A pointer returning its input's
lifetime does not satisfy the condition: alone it is rejected (E0599), beside a
trait method it falls back to that method's u16 result. An outer named lifetime
in the condition keeps its fence.
