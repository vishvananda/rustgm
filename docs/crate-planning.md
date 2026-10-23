# PA34 crate planning protocol checkpoint

This is preparatory maintainer work. PA34 remains planned. Neither a successful
plan nor a successful host build script is evidence that RustGM compiled a
target crate. Every target Rust dependency must eventually use the RustGM
frontend, including the source sysroot.

The adapter uses pinned Cargo 1.98.0's
[unit graph](https://doc.rust-lang.org/cargo/reference/unstable.html#unit-graph)
with `--frozen --unit-graph -Z unstable-options`. Cargo is a package planner here.
`RUSTC_BOOTSTRAP=1` enables that introspection; the compiler wrapper rejects
compilation and forwards queries with only explicit PATH/LANG, without inheriting
Cargo's bootstrap switch, Rust flags or loader configuration. Enabling the
planner must not change the query compiler's reported target capabilities.
The ordinary metadata graph cannot express the distinct feature
sets needed when the same package is both a build and a target dependency.

## Current interface and evidence

`scripts/cargo_plan.py` accepts a frozen `Job`: absolute pinned Cargo/query
compiler paths and hashes, the root manifest and adjacent lockfile, explicit
input file hashes, selected package, target triple, features and default-feature
policy and an optional frozen `Vendor`. Run `execute(job, fresh_output_directory)`
under `scripts/storage.py`. The current selection is a library build. The adapter
uses a fresh Cargo home and rejects ambient Cargo configuration. Local path
packages use the caller's input pins; crates.io packages use the authenticated
directory source described below. Network acquisition and other registries are
outside this interface.

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

## Original registry package snapshots

`scripts/package_snapshot.py` extracts an explicitly pinned local `.crate`
archive into a fresh vendor package directory. The caller supplies the trusted
archive checksum, name and version. Extraction verifies the archive and original
manifest identity, rejects links, special files, escaping/duplicate paths and
preexisting Cargo checksum metadata, and bounds entries and bytes. Original
source, manifests, packaged lockfiles and attribution remain byte-for-byte
unchanged. Permissions are normalized to ordinary files or executable files;
set-id permissions are never imported. Cargo's generated checksum metadata is
derived from those bytes, not trusted as the source of their provenance.

Each immutable snapshot retains its archive pin, all file hashes and modes, and
directory inventory. Validation rejects changed, missing or added inputs,
including empty directories. Its portable package identity contains registry,
name, version and archive hash; filesystem paths remain invocation-local. This
is not yet a semantic crate identity or build-cache key. Keep snapshots under a
dedicated source parent with `.keep-artifacts` when upstream packages contain
files the cleanup policy would otherwise classify as generated objects.

`scripts/crate_vendor.py` validates the exact registry package set against the
external workspace's frozen version-4 lockfile, including packages resolved but
inactive in this build. It accepts only crates.io with explicit matching archive
checksums. The planner supplies only the two reviewed
[directory-source replacement settings](https://doc.rust-lang.org/cargo/reference/source-replacement.html#directory-sources);
it never permits arbitrary ambient Cargo configuration. It verifies every
package before and after planning, builds one source-ownership index, and binds
each registry unit to both the expected package ID and one of that package's
actual pinned source files. Multiple host/target units do not rehash a package
per unit. Receipts retain complete inventories and unit-to-package bindings.

The original serde_core 1.0.229 probe uses an external Rust 2024/resolver-2
workspace with default dependency features disabled. Its unmodified Rust 2021
manifest resolves six registry packages: serde_core 1.0.229, serde_derive 1.0.229,
proc-macro2 1.0.107, quote 1.0.47, syn 3.0.7 and unicode-ident 1.0.26. Cargo plans
only four units: the consumer library, serde_core library, its host build-script
compilation and target-context execution. The cfg-inactive derive dependency
remains in the lockfile but has no selected compilation unit. The consumer uses
`compile_error!` and the strict query wrapper prevents all source compilation.
The original packaged lockfiles are preserved; only the external workspace owns
the acceptance lockfile. All six archive pins also match their exact cached
crates.io index rows; those rows are retained with the replay evidence.

Evidence lives in `obj/parallel/pa34-serde-planning-v1/` (independent original
graph) and `obj/parallel/pa34-vendor-adapter-v1/` (implemented adapter replay).
The latter passed all 141 harness tests, suite registration, assignment layout
and upstream-byte checks, then reproduced the four-unit graph offline using a
fresh Cargo home. Receipt SHA-256 is
`e20e4182193d8dc82f3dd16b01d87cc66b2ddd9cf0e6b3bac119a0daa1dc3146`.
This checkpoint changes no frontend code, assignment activation or text oracle.

## Remaining integration

The caller still declares local path-package inputs. A valid graph does not prove
that all target facts, environment or build-script rerun inputs are present.
Persistent caching is disabled until the planner authenticates that complete
closure. Cargo's implicit sysroot is absent from this package graph: RustGM must
add and compile the original source sysroot rather than consume rustc's target
libraries. Four planned package units are not the complete frontend build graph.

The [host task binding](pa34-build-script-contract.md#graph-bound-host-execution)
now executes authenticated dependency-free registry script pairs, retaining
their actual generated inputs. Next, schedule host dependency artifacts and
DEP metadata, close local package/configuration provenance, and consume ordered
generated cfg, environment and source outputs in target jobs. Implement actual Rust 2021
semantics and per-crate macro edition provenance before claiming that profile.
Then execute dependency-first target jobs through RustGM's typed metadata and
CPPGM backend. Library acceptance requires the original selected dependency
graph and independently checked native consumers, not just this planning slice.
