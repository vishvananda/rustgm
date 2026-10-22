# C variadic calls

The maintainer accepts direct and indirect calls to C variadic declarations on
the existing Linux x86-64 backend. Fixed parameters keep their ordinary expected
types and coercions. Extra arguments are inferred in source order without a
fixed-parameter expectation, then retain their actual types through ownership,
borrowing and lowering. Safe foreign declarations and unsafe function pointers
keep their independent call permissions. Stable Rust variadic function bodies
and variadic signatures with the Rust ABI remain rejected.

Concrete bool, i8, u8, i16, u16 and f32 extra arguments require the explicit C
promotion casts required by the pinned Rust compiler. A function item requires
an explicit function-pointer cast. Checking occurs after ordinary inference
fallback. Generic extra arguments retain the pinned compiler's source checking
behavior; specialization does not add a new promotion rejection. The generic
small-value native control uses a C provider that ignores the extra arguments,
so it makes no claim about reading a differently promoted type with va_arg.

The fixed prefix and variadic flag form one callable identity. Canonical ABI
plans retain only the fixed prefix; actual extra arguments use the existing
cached type classifier. Aligned nonempty repr(C) records, including the
qualified nested-record and array-field shapes, retain their complete object
classes. Function pointers, multiple record arguments, register spills, fixed
record prefixes and record results use the same path. Zero-sized values are
evaluated normally and contribute no physical argument. Other aggregate shapes
retain the existing explicit ABI boundaries.

LowIR declarations and indirect signatures emit the existing arity=variadic
attribute. Actual record arguments carry their per-argument classes. Scalar
literals first become typed temporaries so the backend cannot infer a different
integer width from an untyped literal. Validation checks each extra scalar
against its actual temporary, parameter or address type, and checks record
spans and classes before serialization. The optional object backend, bridge and
LowIR vocabulary are unchanged; its variadic interoperability handshake passes.

The optional object-call arena also serves scalar variadic calls. Ordinary
fixed scalar calls retain their prior path and bytes. Value, Operand, Operation,
Instruction, Signature and Function remain 12, 8, 32, 40, 12 and 200 bytes on
this build. BoundaryArgument remains 16 bytes. Repeated classification across
graph depths and use counts of 8, 32 and 128 charges exactly one cache lookup
per use, retains no extra classifier storage and creates no extra signatures.

## Qualification

`pa30/tests/variadic-outcomes.json` and `variadic-tail-outcomes.json` declare
42 independently qualified source outcomes: 24 accepted programs and 18
rejections. Twenty-three executable controls match pinned Rust and host C
through the CPPGM object backend; the remaining accepted bare-function-pointer
control is compile-only. They cover explicit promotions, unsafe permission,
arity and fixed types, inferred pointers, argument order, register spills,
record tails, moves, competing loans, expired borrows and constant eligibility.
Both C providers are checked in alongside their owning sources.

The initial moved-tail hypothesis is preserved in the preparation evidence:
`let _ = p.a` does not read the moved place. The corrected positive remains,
and a separate actual field read supplies the negative control. Original
independent observations are under `obj/parallel/pa30/variadics/prepared-v1`
and `tails-prepared-v2`; the discarded hypothesis remains in
`tails-prepared-v1`. These are independent source checks, not generated oracles.

The final focused gate in `obj/parallel/pa30/variadics/closing-v2` passes strict
all-target/all-feature release Clippy, the file/architecture audit (1,618 files,
zero findings), three private classifier/LowIR invariants, all four affected
source/native maintainer groups, 90 harness checks, registration, layout and
upstream verification. All 8,360 active PA1–PA24 text cases match exactly.
Eighteen frozen scaling inputs also pass independent Rust object compilation
and candidate LowIR emission. The immutable candidate SHA-256 is
`18cd04846191cda85b6561c79b943147d77d99a2dafa76305a740a26b6fbbe29`;
the complete source manifest and command receipts accompany that pin.

Select the native checks explicitly; an unavailable object profile fails:

```sh
python3 scripts/storage.py run -- python3 scripts/maintainer_suites.py run variadic_calls --release
python3 scripts/storage.py run -- python3 scripts/maintainer_suites.py run variadic_calls --release --ignored
```

## Performance

`obj/parallel/variadic-performance-v1` in the main checkout compares the published
`f46b8699...` compiler with the immutable candidate above. Seven shared controls
retain exact output bytes. Eighteen new inputs vary scalar call count, scalar
tail length, record call count, record tail length, unrelated declarations and
distinct record types at widths 8, 32 and 128. Independent source qualification,
preflight, six calibration/warmup observations, five ABBA blocks with three
executions per observation, eight RSS observations and instruction profiles are
retained. New-language inputs have no baseline speedup claim.

Shared instruction increases range from 0.159% to 0.380%; median RSS decreases
range from 18 to 906 KiB. The six new positive incremental instruction ratios
range from 3.761 to 4.047 for each fourfold width increase. Fixed-arity C aggregate
controls have an incremental ratio of 4.036. No predeclared instruction, RSS or
scaling attribution threshold is crossed. These results support the logical
classifier bounds without claiming whole-compiler linearity.

The original paired wall medians increased by 5.57–17.22%. One declared
follow-up records child CPU usage, minor/major faults and scheduling events
alongside an argument-parser startup control. With the same binaries and frozen
inputs, paired changes are +1.18% for ordinary512, +0.14% for ordinary-pointer
and +1.00% for the 128-root fixed aggregate; startup is -0.57%. Minor faults
increase by only one to three, with zero major faults and equal median context
switch counts. The original ordinary512 candidate median is 19.86 ms versus
11.52 ms in the follow-up. This variability prevents a stable wall-time
regression attribution; the original observations remain part of the review.
No favorable rerun or source optimization is substituted for them. The first
follow-up attempt used an incorrect usage-error exit status and failed before
any timing samples; its declaration and correction are retained separately.

The exclusive lease excludes registered RustGM jobs, not unrelated host work.
The combined compiler checkpoint still needs its own release and changed-path
qualification before publication.

This checkpoint leaves PA30 planned. Static/aggregate crate-metadata transport,
the remaining representation/calling-convention contracts, combined release
qualification and assignment manifest/oracle publication remain required.
