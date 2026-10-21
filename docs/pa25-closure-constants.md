# Constant closure reification checkpoint

Noncapturing closures stored in constants, statics, arrays and promotions now
retain their checked callable identity. Previously the constant machine and
promotion evaluator forwarded the empty environment as a product. Arrays and
const-function results could therefore serialize a null function pointer;
direct initializer coercions instead failed with a mismatched result type.

Both constant consumers and explicit function-pointer casts use one typed
reification operation. It admits only completed noncapturing closure types
with an empty environment, or already selected function values. The resulting
callable instance reaches ordinary checked body discovery and symbol demand.
Constant expression bodies use their selected coercion result type. Reification
does not authorize calling a closure or function pointer during constant evaluation.

The eighteen draft cases contain fourteen accepted programs and four Rust
language rejections. Eleven coercion cases belong to PA24
(`pa24/tests/pointer-outcomes.json`); seven constant-function, statement and
call-permission cases belong to PA25 (`pa25/tests/closure-pointer-outcomes.json`). Native controls
exercise named/static pointers, arrays, repeats, const factories, independently
specialized generic factories, borrowed signatures, casts and promotions. Both
runtime-only call negatives require a non-const-call diagnostic. Captured and
zero-sized captured closures remain invalid function-pointer conversions; these
negative controls do not claim that valid capture lowering is complete.

Qualification before integration: all 32 existing/new source outcomes match
pinned rustc, all 25 accepted programs execute independently, and 38 related
constant/closure regression tests pass. Strict all-target/all-feature Clippy
passes after extracting promotion's typed reification helper. The syntax-aware
file audit passes 1,276 files with zero findings. Full combined publication
validation remains required. Raw evidence is under
`obj/parallel/closure-constants/`; earlier failing lint/audit attempts remain
recorded. PA24 and PA25 remain planned, and no text references are generated.
