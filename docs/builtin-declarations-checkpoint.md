# Complete pinned builtin declaration identities

The [isolated focused gate](https://github.com/vishvananda/rustgm/actions/runs/40187981859)
qualifies `b862dddc`: seven exact private controls, formatting, assignment
metadata, architecture audit, strict private Clippy and an unoptimized build
pass in 165.33 seconds. All 212 harness tests pass. The CLI SHA-256 is
`af667d74e8629936ad34761c97162f32e858a1d1e3173d42b66f0af8d9ab547c`.
The independent empty-function transport work is excluded from this checkpoint.

The catalog models all 61 roles from the pinned Rust compiler registry:
30 bang, 19 attribute and 12 derive roles. The original 24 ordinals are stable.
All 37 newly represented roles remain unavailable for execution. Declarations
retain actual source owners, kinds, feature checks and ordinary lookup identity;
unknown claims, wrong-kind calls and unavailable execution still fail. Allowance
frames check both their actual body and retained header. Source-provider derive
applications cannot bypass availability through the literal attribute consumer.
Ordinary existing derives keep their existing behavior.

The ordinary bootstrap table and allocated slot capacity remain 24, with 384
bytes of slot payload. Actual measurements preserve Report 3040, Engine 4088,
Body 96, Definition 136 and MacroTarget 16 bytes. Growing declaration widths
8/32/128 use 783/3039/12063 work units and 12032/48128/192512 retained bytes.
These controls establish bounded growth and layout costs, not a whole-compiler
wall-time comparison. The finite role/kind golden is checked against the pinned
primary registry; new internal-attribute fixtures do not claim independent
rustc language parity or new macro execution support.

The [fresh unchanged-core observation](https://github.com/vishvananda/rustgm/actions/runs/40188472605)
uses that exact CLI and verifies all 722 original inputs before the traced attempt.
It passes `offload_kernel` and opens nine original files, up from four:
core's root, the two prelude files, macros, offload, contracts, internal_macros,
num/shells/legacy_int_modules and num/f128. It stops at:

```text
ERROR: file 8: 921: invalid floating suffix
```

The actual input is the valid `_f128` PI literal at `core/src/num/f128.rs:27`.
Core explicitly enables f16/f128 and loads these modules unconditionally.
The failure occurs while parsing/configuring that newly loaded physical file;
its FileId is authenticated by the loader path, not inferred from expansion IDs.
The next issue is LIB-024, owned by PA2 literals with its real-source fixture in
PA31. Exact value/type/feature behavior and later arithmetic, metadata and
runtime obligations remain required. No core artifact or additional PA is complete.

Download, verification and observation take 6.731 seconds. The result SHA-256 is
`7764cc78c57c6dd2e3a4c2a98b0392f7c0e674f9c8416d04bd98cc1fe23b8f36`;
its binding is
`d9623514cf9c5db621cca4a6a4dbda703857e8eb3793fc7cc189b841d4604fbf`.
All raw trace/stream hashes are verified locally. Evidence is retained in the
integrator worktree under `obj/parallel/builtin-isolated-passed-40187981859/`,
`builtin-isolated-cli-40187981859/` and `whole-core-lib022-40188472605/`.

The observer now accepts an explicit caller-declared qualified CLI digest while
preserving its old pinned default. It verifies both the binary and producer
binding against that digest; it never derives its own expected digest from an
unqualified executable. Original source, target and deadline checks are unchanged.
