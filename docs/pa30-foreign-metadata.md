# Fixed C callable metadata

This maintainer checkpoint preserves fixed `extern "C"` definitions across
separately compiled RustGM libraries. It extends the existing scalar, reference
and closed-record metadata contracts; PA29 and PA30 remain planned and incomplete.
No assignment activation or generated text reference changes are included.

## Typed contract

`Signature` carries `Abi::{Rust,C}` independently of safety, exact linkage and
bound lifetime slots. Semantic export reads the completed callable convention;
import constructs the ordinary callable type with that convention. Function
pointer coercions, calls, result borrowing and lowering therefore consume the
same facts as source-defined functions.

Wire profile four is selected only when an artifact exports at least one C
function. One ownership tag selects signature-local typed graphs or the existing
shared nominal graph. The existing reserved function convention bytes encode
Rust or C and retain a zero variadic bit. Unknown tags, unsupported ABI/variadic
bits and a profile-four artifact containing only Rust functions are rejected.
Profiles zero through three keep their previous exact bytes. Shared records are
still remapped once per artifact and function signatures share their graph.

This is a fixed callable extension. Generic/const function exports, foreign
static declarations, variadic declarations, function-pointer type graph nodes,
raw-pointer type graph nodes and `repr(C)` aggregate graph transport remain
required follow-ups. Existing record limitations are unchanged. No const-call
permission, source bodies or invented declarations cross the wire.

## Independent qualification

`pa30/tests/foreign-metadata-outcomes.json` declares two library sources, six
accepted clients and six rejected clients before independent qualification.
Pinned Rust 1.98.0, edition 2024, Linux x86-64 builds the complete libraries and
client objects. Rejections retain the expected E0308, E0133, E0515 or E0499
category. The source bytes and their hashes remain unchanged after declaration.

The six accepted clients cover direct/transitive calls, safe-to-unsafe pointers,
unsafe calls, borrowed and mutable results, floating values, exact exported
symbols and references to closed records from another crate. Each runs through
independently compiled Rust and the shared CPPGM object/link backend. Both
external dependency orders emit identical LowIR. Negative clients fail ordinary
ABI, safety or borrow checking rather than an unsupported metadata profile.
Codec controls reject every truncated prefix and malformed convention/ownership
case while checking old-profile roundtrips and shared graph storage.

The complete affected linking group passes 72 regular tests and all 11 explicitly
selected native tests, including the six new native comparisons. Strict release
Clippy across all targets/features, formatting, the file/architecture audit and
suite registration pass. All 8,360 active PA1–24 text cases also match exactly,
and the 90-test harness, layout and upstream checks pass. These are focused
development checks; combined release
qualification and performance measurements are recorded by the later integration
checkpoint before publication.

## Retained failures and harness repair

The first independent attempt used generic `libfirst.rlib`/`libsecond.rlib`
filenames. Rust could not find a transitive crate when a client used the second
library's nominal type. The harness now uses `lib<crate-name>-<identity>.rlib`;
all earlier crate-native groups pass with this change. No language expectation
or fixture was changed to bypass the dependency check.

The preexisting literal `pub extern "C" fn f(n:u32)->u32{n}` moved from a
metadata-boundary assertion to an independently qualified positive assertion.
An initial test-only identifier `c-abi` violated the existing crate identifier
contract; `c_abi` fixes the test input. The original failed logs are retained.
Early file-audit failures were resolved by choosing module names describing
convention transport and codec rules, without splitting production logic.

Evidence lives in `obj/parallel/pa30/foreign-metadata/`: independent
`prepared-v2/`, earlier failures, and `check-v5/` for the passing affected group
and native checks. Source diagnostics and receipts survive automatic cleanup.
