# CStr reference packet

This packet freezes source and native reference contracts only. RustGM has not
observed these inputs and no compiler implementation is supplied here.
`predictions-v4.json` lists every current compile input, its bytes, prediction and
profile. Earlier prediction sets and their retained original input snapshots
preserve setup failures and corrections; they are not manifests of current bytes.
The receipt distinguishes observations, mismatches and infrastructure.

`original-cstr.rs`, `original-slice.rs` and `original-primitives.rs` retain complete
pinned Rust 1.98.0 modules. `source-intervals.json` identifies exact byte intervals
for the real CStr declaration, CStr::as_ptr, CStr::to_bytes_with_nul and
slice::as_ptr. The no_core cutout retains their nominal representation, private
`[c_char]` field, attributes and reached bodies unchanged. Its authored support
supplies sizing/receiver/auto/destruct and ordinary Clone/Copy declarations, the
exact pinned compiler-owned synchronous drop_glue helper, and the pinned Linux
x86-64 `c_char = i8` target choice. Unused derives and unrelated module members are
omitted in this separate consumer cutout; the complete modules remain unchanged.

The installed-standard-library profile independently covers ordinary CStr API
legality, safe construction and const evaluation without pretending that this
cutout supplies every dependency. Raw-pointer C ABI observers cover empty,
ordinary, UTF-8, high-byte and raw literals, one trailing NUL, slice-pattern
lengths and a shared value's pointer/view consistency. Storage tests retain both
`b"A\0"` and `c"A"` in const and static subjects. They require actual byte/CStr
types and views; identical bytes neither require distinct addresses nor grant
interchangeable typed promotion. No CStr value or reference crosses the C ABI.

The final run accepts unused extra-field/generic role shapes and the generic
wildcard-literal metadata experiment. The initially uncertain contrary prediction
remains preserved. A panic, ICE, signal, timeout or missing setup is never a
semantic negative. No unsafe constructor
negative deliberately invokes undefined behavior. Privacy, wrong type,
mutability, lifetime, ordinary source-body errors and unsafe-call requirements
are independent opponents.

Complete derive generation, full method/module closure, from_ptr/strlen and all
constructor consumers, general CTFE and authenticated metadata transport remain
required. Independent standard-library execution is reference evidence, not a
RustGM implementation or a whole-PA claim. CPPGM will be used only when a future
candidate produces genuine LowIR; this reference packet uses Rust 1.98.0 directly.

Final qualification: all 24 source cases match, and all three C ABI observers
pass. The actual-source cutout prints `CStr 65 2 0 195 169 255 92 1`; the installed
standard-library companions print `storage 8 65 0` and `construct 111 107 0`.
Their static libraries retain the real standard runtime dependencies. Initial
standalone-object link failures are preserved, as are missing Copy/const-stability
and drop_glue setup errors. Harness and upstream verification pass. See
../../../../docs/parallel/source-cstr-reference.md for exact evidence bindings.

The local-source extension is frozen in `local-predictions-v1/v2.json` and
`view-predictions-v1/v2.json`; [local-reference.json](local-reference.json) binds
the independent observations and retained discrepancies. The v2 storage companion
preserves the actual CStr and slice method bodies and exercises byte/CStr constants,
statics, const `as_ptr`/`to_bytes_with_nul`, and signed high-byte views. Its real
Rust native output is `storage 8 65 0`. The raw fat-reference slot/aggregate
roundtrip prints `roundtrip 255 0`; raw literal writes and scalar overreads reject
for actual immutable-allocation and end-of-allocation causes. These companions
do not substitute for compiling the full original core or std graph.

Original failed setup and hypotheses remain intact: byte-array method unsizing
needs additional no_core support, the first array overread stops at missing Copy,
and both UTF8 view hypotheses are accepted through object generation in this
profile. UTF8 sources were never executed and are not candidate negative controls.
The independent literal keyword `bounds` was absent from the successful allocation
end diagnostic; the actual E0080 cause is retained. Maintainer candidate tests are
registered as `cstr_sources`; candidate implementation qualification is separate.
