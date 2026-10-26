# Source Assert: independent observations

LIB-029 owns source-declared `assert!` expansion and its ordinary runtime and
constant obligations. Execution is still unimplemented in RustGM. These results
prepare its required tests; they do not qualify a candidate or move the current
original-core frontier.

[CI 40230883578](https://github.com/vishvananda/rustgm/actions/runs/40230883578)
observes sixteen new, independently authored programs with Rust 1.98.0,
revision `88d9e12ae178fab0fb5cc050a94da85685d449ea`, edition 2024,
`x86_64-unknown-linux-gnu`. All sixteen match predictions frozen before execution;
there are no infrastructure failures or warnings. The observation loop takes
0.496 seconds. This is rustc observation time, not RustGM throughput.

| Control | Observed result |
| --- | --- |
| True constant assertion; trailing comma | Both compile. |
| False constant assertion | Rejected with E0080. |
| Side-effecting condition | Evaluated once. |
| Successful assertion with message-side effects | Message arguments are not evaluated. |
| False custom-message assertion | Message arguments run once and produce the expected payload. |
| Custom condition implementing `Not<Output = bool>` | Compiles and executes successfully. |
| Custom condition implementing `Not<Output = u8>` | Rejected with E0308. |
| Default messages from plain, block and forwarded opaque expressions | Exact expected expression messages observed. |
| Empty input, semicolon, missing comma or extra condition tokens | All rejected. |
| Invalid custom formatting with a true condition | Rejected; the unreachable branch is still checked. |

Nine controls compile metadata; seven compile and execute native programs.
Four native observers use rustc's standard-library `catch_unwind` only to inspect
panic payloads. Those commands explicitly select panic unwind; the remaining
commands select panic abort. This observation harness does not add an unwinding
requirement to PA31 or claim RustGM implements unwinding. Candidate fixtures must
use authentic source declarations and the appropriate existing runtime boundary.

The [pinned compiler implementation](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_builtin_macros/src/assert.rs)
parses one condition and constructs unary Not in an if expression. Its default
message calls defining-library `panicking::panic` directly; custom-message
tokens use an edition-sensitive panic macro. Enabled rich generic-assert capture
is a separate path. Reusing source panic-macro delegation for every form would
lose that distinction. Generated syntax must retain actual source ownership,
opaque fragments and ordinary operator/type/body/const checking.

The frozen sources are in the
[immutable CI tree](https://github.com/vishvananda/rustgm/tree/1f5833f47e47bd77145f1f41a78ccbf054170d3c/.github/assert-oracles).
Its [observer](https://github.com/vishvananda/rustgm/blob/1f5833f47e47bd77145f1f41a78ccbf054170d3c/.github/focused_refs.py)
records every command and raw stream.
Plan SHA-256:
`39ae5c72dbae101f73c6ae45ce66ad873a8bfe71a225b99a0b264017b7509293`.
Binding SHA-256:
`345d0c05334f67bdcccb2e2c48001a2535c13b04c5568a084b5482e0dac78db5`.
The integrator verifies the complete artifact digest, toolchain, every source,
raw compile/runtime stream and declared outcome. Nine disposable metadata/native
outputs are hashed and removed after observation. Raw source/diagnostics/receipts
remain under `obj/parallel/source-assert-design-v1/oracles-v1/`.

The earlier panic-delegation and unnamed-constant matrices were not repeated.
Keep the unchanged alignment reducer and both initializer obligations intact.
Actual expansion, source-owned panic execution, constant failure behavior,
edition/provenance opponents, charged transfer rollback and scaling still need
candidate qualification before LIB-029 can close.
