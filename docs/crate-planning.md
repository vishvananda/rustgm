# PA34 crate planning protocol checkpoint

This is preparatory maintainer work. PA34 remains planned. Neither a successful
plan nor a successful host build script is evidence that RustGM compiled a
target crate. Every target Rust dependency must eventually use the RustGM
frontend, including the source sysroot.

The adapter uses pinned Cargo 1.98.0's
[unit graph](https://doc.rust-lang.org/cargo/reference/unstable.html#unit-graph)
with `--frozen --unit-graph -Z unstable-options`. Cargo is a package planner here.
`RUSTC_BOOTSTRAP=1` enables that introspection; the compiler wrapper rejects
compilation. The ordinary metadata graph cannot express the distinct feature
sets needed when the same package is both a build and a target dependency.

## Current interface and evidence

`scripts/cargo_plan.py` accepts a frozen `Job`: absolute pinned Cargo/query
compiler paths and hashes, the root manifest and adjacent lockfile, explicit
input file hashes, selected package, target triple, features and default-feature
policy. Run `execute(job, fresh_output_directory)` under `scripts/storage.py`.
The current selection is a library build. The adapter uses a fresh Cargo home,
rejects ambient Cargo configuration and accepts local path packages. Registry
resolution and acquisition are not implemented by this interface.

The wrapper accepts only the exact observed version and empty-input file-name,
sysroot and target-configuration queries. It retains raw query output, arguments
and status. Adding any query requires independent observation and protocol
review. The planner retains the command/environment, raw graph/diagnostics,
tool/implementation/input hashes and terminal status. Source pins are checked
again after execution. Unknown protocol fields, missing inputs, changed pins,
Cargo failure and invalid graphs fail; they never produce a successful plan.

`scripts/build_units.py` decodes schema 1 into immutable units, targets and
dependency edges. Units retain the complete profile, dependency edge flags,
renames, edition and feature set. The profile is retained as opaque canonical
JSON; a future executor must validate and implement every selected setting.
Host compilation, host execution and target compilation are distinct roles.
The run unit for a target's build script carries the target platform while its
compiled executable and build dependencies belong to the host. Ordinary library
edges cannot cross those domains. Procedural macros are represented as host
compile units; their execution and token protocol remain PA37 work.

An iterative dependency-first traversal visits each node and edge once and
rejects cycles, missing rows, unreachable rows and duplicates. Counts and input
bytes are bounded. No global package-name feature union is performed. Row
indices, absolute paths and Cargo package-ID strings are not reusable semantic
identities, build-cache keys or crate disambiguators.

The independent miniature workspace deliberately contains `compile_error!` in
every library and build script. The same renamed dependency is selected with
`host_side` for the build script and `target_side` for the target. A Windows-only
dependency is inactive. The declared expected graph has five units: target root,
target-context script execution, host script compilation and two separate
dependency units. The first exploratory probe incorrectly used different aliases
for the same dependency package and Cargo rejected it. That observation is
retained; the distinct corrected input uses one alias in both domains.

Harness controls cover domain separation, edge flags/profiles, deep iterative
ordering, malformed/empty/cyclic graphs, pin changes, failed planning and rejected
source input to compiler queries. The real pinned-tool probe checks that the
wrapper accepts Cargo's actual requests and no source compilation occurs.

## Remaining integration

The caller currently declares the input closure. A valid graph does not prove
that all transitive manifests, files, target facts, environment or build-script
rerun inputs are present. Persistent caching is disabled until the planner
authenticates that complete closure, including registry packages/checksums and
Cargo configuration. Preserve original upstream sources and packaged lockfiles;
an external acceptance workspace can own its own resolved lockfile.

Next, bind units to authenticated package snapshots, lower host tasks into the
existing `host_build_script.py` interface, and consume ordered generated cfg,
environment and source outputs in target jobs. Implement actual Rust 2021
semantics and per-crate macro edition provenance before claiming that profile.
Then execute dependency-first target jobs through RustGM's typed metadata and
CPPGM backend. Library acceptance requires the original selected dependency
graph and independently checked native consumers, not just this planning slice.
