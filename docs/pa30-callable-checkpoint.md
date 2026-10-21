# PA30 callable convention checkpoint

PA30 remains planned. This checkpoint introduces callable ABI identity while
preserving ordinary Rust views and the separate safety and constant-permission
contracts. PA26's remaining hook families are still recorded in
`pa26-environment-checkpoint.md`; starting PA30 does not complete PA26.

`types::callable::Convention` contains an explicit Rust/C ABI and a variadic bit.
Canonical function types and inference constructors retain it through
substitution, binder normalization, specialization, body instantiation,
annotation elaboration, callable shape comparison and structural fingerprints.
Rust variadic canonical signatures are rejected. Source variadic signatures
remain beyond this first fixed-arity checkpoint.

Syntax distinguishes ordinary Rust from `extern` with an optional literal.
Visitors, transfers and rendering preserve that distinction; omission selects
C, while explicit Rust selects the ordinary convention. Unsupported ABI strings
remain errors. Safety and const eligibility stay independent of ABI.

Closures keep Rust signatures. A contextual C pointer expectation does not
change a closure's convention. Function-pointer coercions and call-trait
selection compare the exact convention. C callable items and pointers do not
receive implicit Fn implementations. Empty capture environments remain
necessary for closure reification.

Top-level safe-to-unsafe item, pointer and empty-closure coercions construct a
signature view with only the safety bit strengthened. The body-owned sparse map
retains that exact view for lifetime proof publication, including discarded
zero-length repeat handling and body reset. Nested callable shape equality
stays strict. Runtime values use ordinary reification or constraint lowering.

Fixed-arity C scalar and thin-pointer calls use the existing scalar backend
boundary. C aggregate parameters/results and variadic calls produce explicit
lowering errors until their real transport is implemented.

## Qualification

All 29 focused tests pass, including 17 declared source controls, five native
callable comparisons, 11 native contextual-closure comparisons and canonical
substitution/inference identity checks. Strict all-target/all-feature Clippy
passes. The syntax-aware audit reports 1,200 files and zero findings; layout,
24 pinned upstream fixtures and 126 Reference files pass. Four earlier PA13/18
callable views retain all reference bytes. Pinned rustc object generation agrees
with all 17 draft fixture outcomes. Draft fixture outcomes live in
`pa30/tests/convention-outcomes.json` (13 C ABI cases) and
`pa27/tests/callable-safety-outcomes.json` (four Rust-only safety cases);
no text references are published.

Commands, logs, fixture hashes and status receipts are retained under
`obj/parallel/pa30/conventions/` in `gates.json`, `gate-*.log`,
`qualification.json` and `earlier-text.json`. Initial development attempts caught
mechanical field omissions, a test-only default-construction lint and a test
helper without the ordinary constant execution provider; these were corrected
before this final gate. Broad cumulative and performance gates remain
integration/release responsibilities. The separate bounded check in
`pa30-callable-performance.md` retains release A/A and ABBA/RSS observations;
its short inputs do not establish a tight percentage bound.

The read-only C capability handshake independently passed integer and floating
exports, direct imports, indirect callbacks, and imported/exported globals.
It compiled hand-written LowIR to a host ELF object, linked that object with a
host C program, and ran it successfully. The immutable CPPGM driver pin has
SHA-256 `ded1b6800ce1bdaeaad45cd28c2903d7c96de7cdc29261f6834869a6eaf525fc`.
Its inputs, outputs and hashes are recorded under
`obj/parallel/pa30/handshake/receipt.json`; resource receipt
`6b9fe48895094370b9c1525b519be363` returned zero.

## Remaining PA30 work

- Retain foreign-block ownership and inherited ABI/implicit safety in typed
  declaration metadata, including macro-generated declarations.
- Publish the canonical foreign function/static registry and consume PA29's
  shared exact-name/linkage selector without fake bodies or initializers.
- Review and add the optional immutable CPPGM LowIR-to-object bridge, retaining
  the existing executable backend configuration and behavior.
- Implement repr(C) aggregate argument/result classification, stack/register
  transport, callbacks and global interoperability in both directions.
- Implement variadic signature/argument promotion and remaining callable cast
  coercions with their exact proof views.
- Qualify cumulative behavior, performance and reviewed assignment references.

Exact symbol spellings remain lossless metadata. The pinned backend accepts
only ASCII letters, digits, underscore, period and dollar in symbol tokens;
the shared selector must reject other exact spellings explicitly. It must not
normalize or silently substitute another external name.
