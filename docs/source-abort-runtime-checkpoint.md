# Checked source Abort reaches the existing native trap

Source `26ad813035931b0e7f9f156d13e44284ee19ec3f` qualifies the actual safe,
non-const source intrinsic `abort() -> !`. Hosted run `40280970420` passes the
corrected terminal-body control, the actual owned-scope native producer and the
candidate native observer. Four unaffected private controls and the independent
Rust native observations are retained from authenticated run `40280027733`.
Production Clippy and assignment metadata are retained from `40278943018`;
production bytes are unchanged across those runs. Current formatting,
architecture audit, strict private Clippy, build, harness and upstream checks
pass. All 2,736 source fingerprints match their actual frozen source before and
after execution. The final helper took 138.47 seconds, approximately 181.54
seconds including its preceding mandatory CLI build.

The pinned CLI is
`8de27f8df66b24c33cf3f07fca744a43c47b31d1841209ed6c5e0f184632e3ce`.
This is a local source runtime milestone. Assert expansion/execution, panic
construction, formatting, const panic and imported intrinsic execution remain
required. No original-core artifact or full standard-library claim follows.

## Actual contract and operation

Pinned Rust commit `88d9e12ae178fab0fb5cc050a94da85685d449ea`,
`library/core/src/intrinsics/mod.rs:351–371`, defines safe, non-const, no-argument
Abort returning never. Abnormal process termination is required; a particular
signal is not. No libc call is needed for this compiler operation.

The existing CPPGM object backend, commit
`3438f5aea24d8fcc9a0113316f7e27c890cd5ac6`, lowers LowIR `Unreachable` to MIR
`Trap` in `native/select/emit_control.cpp:504`, then `0f 0b` in
`native/x86/emitter.cpp:1147`. Backend executable SHA-256 is
`ded1b6800ce1bdaeaad45cd28c2903d7c96de7cdc29261f6834869a6eaf525fc`.
Independent capability run `40276927922` verified this contract and the pinned
Rust intrinsic, including a valid user-exported `abort` with observable effects.
The backend and ordinary lowering's existing abort-symbol path are unchanged.

The Catalog descriptor authenticates the actual attributed declaration, complete
callee instance, safe Rust signature, no parameters/predicates/regions, never
result and non-const bodyless header. Before omitting only that direct Call's
callable-body demand, the dependency consumer requires the current provider
Report and positive HeaderLedger source site. A same-name ordinary function,
changed signature/arguments, foreign Report or mutated live header cannot use
this path. Const calls, function-pointer coercion, raw callable demand and
unsupported imported origins remain denied.

Lowering borrows a non-clonable witness for the actual checked body, Call value,
evaluated callee, full instance, signature and existing statement location. The
caller must retain its ordinary original/instance, ownership and region proof.
The Call must be the actual final Evaluate of a block whose original terminator
is Unreachable. Consumption immediately terminates the builder and stops later
cleanup emission. It returns a private completion boolean, never a fabricated
Unit result. False branches keep their ordinary cleanup timeline.

No Body/Rvalue/Lower/Catalog field, cache, enum identity, factory, bootstrap slot,
wire/profile admission, library algorithm or backend operation was added. The
existing body-value and statement walks are reused; there is no additional AST
or whole-body scan. Ordinary calls incur a constant descriptor check without a
new allocation. Actual measurements are Catalog 704, Resolution 2056, Package
256 and IR Body 448 bytes, with unchanged field definitions. The transient
AbortSelection is 8 bytes. Retained slot/layout controls still observe 24
bootstrap intrinsics, Reservations 808, SourceCore 176 and Declaration 20 bytes.

## Genuine source and native outcomes

The independently compiled provider has a real source Drop trait, a Guard
implementation whose destructor calls the C observer, the actual attributed
Abort declaration and a user function exported as `abort`. Its DropGlue helper
uses the pinned source signature and empty body: `core/src/ptr/mod.rs:822–830`
explicitly requires the compiler to replace that body with actual glue.
The generated LowIR calls the Guard destructor on the normal branch.

Both pinned Rust and the candidate observe:

- Normal: status 0 and exactly `interposed\ndropped\n7 9 11\n`. The user symbol,
  ordinary owned Drop and lazy false branch all execute their proper paths.
- Reached Abort: only `before\n`, empty stderr and intentional synchronous
  abnormal termination. Neither the user symbol nor Drop runs and the call does
  not return. Both actual observations were signal 4; that signal is not an
  acceptance requirement. Compiler/linker failures, timeouts and unrelated
  termination signals cannot pass as this program outcome.

Rustc SHA-256 is
`3690cc576ede140504698405d5d8fa3826aaadbe71699c6c4ed0a565d6f493e2`.
The profile is edition 2024, x86_64-unknown-linux-gnu, library, panic=abort,
default optimization/overflow/debug/UB settings, and RUSTC_BOOTSTRAP=1 only for
actual internal declarations. No upstream source was patched. The provider
SHA-256 is `0faf395568cbb449d087be100df9df2bc21f1c5ebddf09d0bc36cf158b77ea8a`;
actual candidate LowIR is
`db478501cae9c3e378bf45922c131bd837c753c0ae26065464dc3f560c39ad06`.

The const and false-const controls reject through `non-const calls are not
allowed in const functions`; reification rejects through `source intrinsic
cannot be coerced or cast to a function pointer`. Terminal-edge, wrong-instance,
foreign-header, warm mutation, signature and argument opponents remain intact.

## Evidence and remaining work

Final receipt SHA-256:
`302a344d31d07d7e5f2b99053619a7f7908195610a83bda0b88f26a595a27021`.
The verified small evidence artifact is `12632089251`, ZIP SHA-256
`af843e941496b9603fdb43d6ca1f7474442ce93b77aa24654f1648fc6c9cb397`.
Source/LowIR/CLI artifact `12632520127` has published ZIP SHA-256
`d9645f9f4167f39487cd7066afe3fba5f9465d767f73bf47b04f94b89f675c13`.
Its source and LowIR members were read by bounded ranges and checked against the
qualified receipt; no compiler ELF was hydrated locally. Successful native
objects/executables were hashed and pruned. Raw statuses, outputs and source
bindings remain.

The failed test-error conversion (`40278943018`), private test line limit
(`40279558057`) and direct whole-generic body-builder setup (`40280027733`)
remain recorded. The latter now uses the actual prepared body/ownership proof;
no failed expectation was changed into success. The older capability probe's
incorrect handcrafted branch grammar (`40276514844`) is also retained.

PA26/PA31 own source identity and call proof, PA28 the intrinsic consumer, PA25
const restrictions and PA39 the supported native termination boundary. Real
Arguments/NonNull/PhantomData, caller Location/track_caller, PanicInfo, handlers,
formatting and the complete panic/Assert pipeline remain required. The ordinary
std Clone/size_of adapters and imported must_use/factory debt are unchanged.
