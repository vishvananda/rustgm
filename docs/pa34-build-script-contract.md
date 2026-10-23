# Original Serde host build-script contract

PA34 remains planned. This checkpoint qualifies original upstream host scripts,
not target library compilation or a crate-build driver. It fixes a small,
observable prerequisite for the [upstream library plan](library-bootstrap-roadmap.md).

The local registry archives were checked against their exact cached registry
index checksums. Their `build.rs` and normalized `Cargo.toml` bytes match the
extracted source files used by the observation. Both manifests select Rust 2021.

| Package | Registry archive SHA-256 | build.rs SHA-256 |
| --- | --- | --- |
| serde_core 1.0.229 | `67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48` | `7bc6b1eec8913ff4cd8e4b9b92cd04a82a4f38beb30a0b73a64c1e6e49f07b5f` |
| serde_json 1.0.151 | `c841b55ecdae098c80dcae9cf767f6f8a0c2cdb3416bbef72181df4d0fe73f14` | `a681e754be844c7dbef957f5d2d00b01f37c5dca160ef1055a8d8f975697a881` |

## Independently declared observations

Each unchanged script was compiled once by the explicit host Rust 1.98.0
toolchain (`88d9e12ae178fab0fb5cc050a94da85685d449ea`), using edition 2021 and
host `x86_64-unknown-linux-gnu`. Each executable ran twice with separate output
directories and a recorded environment. All four executions returned zero,
produced no stderr and matched the complete stdout and generated-file bytes
declared before execution. No target Rust library or RustGM compiler was run.

`serde_core` reads `OUT_DIR`, `CARGO_PKG_VERSION_PATCH`, `RUSTC` and `TARGET`.
It writes `private.rs`, containing the `__private229` module reexport of
`crate::private::*`. It invokes the selected `RUSTC --version`. At the declared
1.98/x86-64 profile it emits `rerun-if-changed=build.rs` and ten
`rustc-check-cfg` directives, with no `rustc-cfg` compatibility fallback.
Generated `private.rs` SHA-256 is
`899563ce84b59b13cd7036e25f7897c1e730906f62a43aff14472d11c4229f34`.

`serde_json` reads `CARGO_CFG_TARGET_ARCH` and
`CARGO_CFG_TARGET_POINTER_WIDTH`. For x86-64/64-bit it emits
`rerun-if-changed=build.rs`, the `fast_arithmetic` check-cfg declaration and
`rustc-cfg=fast_arithmetic="64"`. It writes no files. The original script,
rather than a driver substitution, must select this configuration.

Full stdout SHA-256s are
`b05731ace603d330b9a7aaf76b8433df15c3e1b2a3a95f4c644d5f5ed0c82f84`
for serde_core and
`6bb524834a0c0657c682dbb81bc6eeb23ac92c48da502f17ce4455240e5d062b`
for serde_json. Repeated executions matched these hashes and every generated
file. These are observations of this exact profile, not evidence for other
targets, compiler versions or feature configurations.

## Driver responsibilities

Keep the build-script host toolchain separate from the target frontend. A host
rustc is explicitly permitted during bootstrap; the target crate and every
target Rust dependency must still pass through RustGM. Record a script's compiler
version queries as configuration inputs. Reporting an older compiler version to
select easier library paths would not qualify the declared upstream target.

Retain package identity, original manifest/script bytes, edition, selected
features, host and target facts, compiler identities, explicit environment,
ordered script directives and generated-file hashes. The script's current
directory is its original package root; generated files belong to a dedicated
output directory. `env!` and `include!` in the target crate must consume those
actual generated inputs with their provenance intact.

Parse `rustc-cfg`, `rustc-check-cfg`, environment, link and rerun directives into
separate typed records outside the frontend library. Preserve link argument
order. A check-cfg declaration does not enable a cfg or grant a language feature.
Reject unsupported build-affecting directives instead of silently ignoring them.
General directive parsing, dependency resolution, caching and replay remain
implementation work; these two scripts do not exercise every directive family.

Cache a completed script only under its exact host compiler, package/dependency
inputs, features, target facts, observed environment and declared file inputs.
Validate generated-file hashes on reuse. Retain small receipts and generated
source, and prune disposable host executables under the batch's scoped output
root. Do not repeat host builds per target source file or retain one executable
per historical observation.

The target compiler still needs genuine Rust 2021 semantics. Its current CLI
accepts only edition 2024; adding a 2021 flag alone is insufficient. Definition
edition must survive macro expansion and cross-crate transport. The pinned
Reference identifies differences in expression macro fragments, match binding
modes, temporary scopes, extern blocks and reserved `gen`, among other rules.
Declare paired 2021/2024 source and runtime controls before implementing these
consumers. Imported definitions must not inherit the consuming crate's edition.

## Evidence and remaining scope

Evidence is retained in the validation checkout under
`obj/parallel/pa34-host-build-scripts-v1/`: predeclared hypotheses, exact original
scripts, registry rows, command environments, outputs and generated source.
The first preparation attempt assumed an extracted `.cargo-checksum.json` file;
its missing-file error is preserved. No compiler ran in that attempt. Successful
preparation instead verified the original archives against the registry index.

Declaration SHA-256 is
`3e0c453cabd2bcc6d3fb9464b6b2dedd04a449661271d2b282998ce84a49dc3e`;
completed receipt SHA-256 is
`ddc282d1286c4518f49da679e9d09180bc850545a294e29a123c19a9128a57ab`.
The resource/storage guard completed successfully and removed both generated
executables while retaining evidence. The frontend source remained at validation
`9cc05f0326a527a0556610bbe7176bbea75b1f68` throughout.

Next work is a real deterministic build protocol using these original outputs,
followed by per-crate edition propagation and the required semantic differences.
The complete selected transitive graph must be frozen and built before claiming
upstream crate acceptance. No PA34 fixture, text oracle or activation changes in
this checkpoint.
