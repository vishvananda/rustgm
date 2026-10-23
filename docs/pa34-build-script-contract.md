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
Dependency resolution, persistent caching, target integration and replay remain
implementation work; these two scripts do not exercise every directive family.

Cache a completed script only under its exact host compiler, package/dependency
inputs, features, target facts, observed environment and declared file inputs.
Validate generated-file hashes on reuse. Retain small receipts and generated
source, and prune disposable host executables under the batch's scoped output
root. Do not repeat host builds per target source file or retain one executable
per historical observation.

The target compiler needs genuine Rust 2021 semantics. At the initial script
observation its CLI accepted only edition 2024; adding a 2021 flag alone would
be insufficient. The later [package edition checkpoint](pa34-package-editions.md)
implements explicit package edition propagation and paired semantic controls.
Definition
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

## Implemented host task and ordered output consumer

`scripts/host_build_script.py` now executes an explicitly configured host `Job`:
the caller supplies the package/script, edition, compiler pin, frozen inputs,
host dependencies, resolved features and separate compile/runtime environments.
There is no compiler discovery or target frontend fallback. It validates pins
before work and after execution, retains command/diagnostic receipts, and
publishes generated-file hashes and directives only after success. Every task
uses a fresh output root; persistent cache reuse is deliberately absent until
the crate planner closes the full source/dependency/environment/rerun graph.

`scripts/build_script_output.py` implements the ordered host output protocol.
Enabled cfgs and check-cfg expectations have distinct typed kinds. Environment
assignments, rerun inputs, metadata and scoped link directives remain distinct;
native link order and embedded equals signs survive parsing. Legacy metadata
keys and the modern prefix follow the
[Cargo build-script protocol](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
Only `-l`/`-L` are accepted in `rustc-flags`; error directives, nonzero statuses,
malformed/unsupported modern directives and resource limits fail explicitly.
Ordinary stdout remains raw evidence rather than frontend input.

This task primitive does not resolve Cargo manifests or select target inputs.
It does not yet consume link/check-cfg records in the target driver or establish
host/target graph closure. Its caller must provide the complete frozen source
and host dependency set before claiming reproducible crate builds. A successful
host task alone remains insufficient for PA34 or an upstream crate milestone.

Validation under `obj/parallel/pa34-host-runner-v2/` passes 12 focused protocol
and task tests plus all 110 harness tests, suite registration and assignment
layout checks. Both unchanged original Serde scripts executed through this new
task primitive and matched the previously declared stdout/generated-source
bytes. No RustGM rebuild was needed. Scoped cleanup removed the two generated
host executables and retained sources, outputs and receipts. The first focused
attempt's standalone test import-path error is retained under the v1 root;
the fix makes this test root runnable independently of test discovery order.

## Graph-bound host execution

`scripts/planned_build_script.py` now pairs a selected host-execution unit with
its exact host-compilation unit and authenticated registry package snapshot.
It validates package identity, source membership, edition, features, platform
and both profiles before launching. Selecting a target compilation, another
package's source, the wrong target context or an unbuilt host dependency fails.
The initial executor handles dependency-free script pairs; host dependency
scheduling and DEP links metadata are explicit remaining requirements.

`scripts/host_profile.py` translates the supported pinned profile fields into
explicit host rustc flags. It preserves debug assertions, overflow checks, panic,
optimization, debug information, codegen units, split debuginfo, rpath and LTO
semantics. Cargo's `lto=false` retains local thin LTO; it is distinct from `off`.
Unknown settings, custom profile inheritance, alternate codegen backends,
incremental caching and unimplemented strip representations fail rather than
disappear. The host task's optional profile is retained in its receipt.

`scripts/build_environment.py` derives package variables from the original
normalized manifest and keeps explicitly pinned host/target configuration
separate. Host compilation receives package/feature inputs; script execution
receives its selected target context and run profile. `DEBUG` describes generated
debug information, not `cfg(debug_assertions)`. Unimplemented manifest lint and
unstable policy is diagnosed. Arbitrary ambient environment is never inherited.
The declared cfg context and its input pins are retained in the outer receipt;
they are not evidence of RustGM backend capabilities or a complete source sysroot.

The script's `RUSTC` is the restricted query wrapper, while actual host compilation
uses the explicit pinned host compiler. A source-compiling probe is rejected.
Even if a script swallows that subprocess failure and exits successfully, the
outer task rejects its result; it cannot select an apparent missing capability
from an unsupported compiler probe. All query arguments/output/status are kept.
Sources, configuration inputs and implementation pins are rechecked before
publishing generated-file hashes and ordered directives. Cache reuse remains
disabled. General compiler-probe support, full toolchain/runtime closure and
target consumption still need implementation.

The original serde_core 1.0.229 archive now passes from the frozen four-unit graph
through this executor. Its host script emits the same complete stdout hash and
`private.rs` hash declared above, and its only compiler query is `--version`.
No target source is compiled. The final guarded batch under
`obj/parallel/pa34-planned-host-v2/` passes all 152 harness tests, registration,
assignment layout and upstream checks. Its outer host receipt SHA-256 is
`b5f0dfa4789af71197f0537721395fea7b62e7e3a0e3ec9da8abcb183fb6ecc7`.
Scoped cleanup removed the host executable and retained its generated source.

An independent Cargo probe under `obj/parallel/pa34-host-context-v1/` deliberately
stops target compilation with `compile_error!` after running its host script.
It falsified the initial assumption that host debug assertions stay enabled:
the selected false setting is inherited, while host panic remains unwind when
the target uses abort. The exact source, initial failed hypothesis and diagnostics
remain. `pa34-host-profile-v1/` plans that unchanged workspace and compiles its
script through the host task; actual cfg behavior and nine selected environment
observations match the independent Cargo execution.

The same study exposed bootstrap environment leakage into planning queries.
The query wrapper now uses only explicit PATH/LANG. Its complete target-query
stdout matches an independent pinned-tool invocation byte-for-byte. Cargo's
query includes the proc-macro crate type, so its `proc_macro` cfg is expected;
the earlier hypothesis that it would disappear is retained and corrected.
Unstable-only target facts disappear with bootstrap removed. These raw package
planning queries must not become the RustGM target compiler's capability table.

## Shared host dependency execution

`scripts/host_graph.py` now consumes the full validated unit order, authenticated
registry snapshots and explicitly pinned host/target contexts. It compiles each
selected ordinary host rlib or build script once. A shared build-script executable
can run separately for host and target consumers, with distinct cfg facts even
when both use the same target triple. Target compilation units remain untouched.
The source manifest is parsed once per package; a source-path index binds its
units without repeatedly scanning the package inventory.

Each host library consumes its exact own script's ordered cfg, check-cfg, native
library/search flags, environment and generated directory. Direct renamed host
dependencies use their built artifacts. All host rlibs occupy one invocation's
library inventory, avoiding per-consumer copies of transitive dependency lists.
Only immediate links metadata edges supply `DEP_*` variables to a script;
metadata cannot enable cfg or leak transitively. Repeated assignments retain the
last value, including empty values, and ambiguous normalized names fail.

Each process keeps its own bounded command receipt. Completed nodes append one
journal record; the full receipt is written only at initialization and completion.
The executor bounds aggregate generated files/bytes, artifact bytes and parsed
script/query output. Before returning any graph result it revalidates packages,
configuration inputs, artifacts, generated inputs and implementation pins.
Failed, timed-out, mutated or unsupported jobs never publish successful results.
This shares completed work within one invocation; it does not introduce a
persistent cache or claim complete system toolchain/runtime provenance.

`make test-host-builds` runs the maintained independent control. Three authored
registry archives exercise renamed dependencies, separate host/target feature
sets, a host library's own generated source and cfg/environment, immediate links
metadata and a shared script. Independent offline Cargo compiles the host graph
and deliberately fails on the target crate's declared `compile_error!`. The
executor then uses the frozen graph and produces identical generated source
with six host compilations and five script executions, with no target compilation.
The first qualified answer SHA-256 remains
`9823372a0e335dc9ac0b0759c214de0fb4d7bb0917cb288a6136b9993590e4b0`.

The Python harness checks changed inputs/artifacts, failed and caught queries,
metadata collisions, unsupported units/profiles, resource budgets and sharing.
At 8, 32 and 128 additional consumers, each package is verified exactly twice,
each host node launches once, and the full graph receipt is written three times.
Process-specific receipts and completion records grow with nodes/edges; they do
not repeatedly serialize the accumulated graph. Real host compilation is checked
separately rather than repeated for every harness failure control.

Original discovery failures remain under `obj/parallel/pa34-host-graph-v1/`:
Cargo rejected differing dependency aliases for the same package across domains;
the corrected fixture uses one renamed alias in both domains. The first candidate
then rejected Cargo's observed inferred `strip=debuginfo` representation before
launch. The supported profile translation now preserves that exact representation.
The maintained runner retains each new declaration, raw command outputs and
generated-source hashes under `obj/qualification/host-graph/`; scoped cleanup
removes disposable executables and objects after success.

The later [local input checkpoint](crate-planning.md#complete-local-package-snapshots)
adds complete local package inventories and the same independent host graph
comparison for path dependencies. `make test-host-builds` runs both variants.

Procedural macro execution, general compiler probes, persistent rerun/cache
closure, complete workspace configuration, toolchain/runtime provenance and
actual target consumption remain required. Link-argument directives stay ordered
in script outputs but do not apply to the ordinary host rlib targets supported
here. This checkpoint neither activates PA34 nor establishes upstream crate or
source-sysroot compilation.
