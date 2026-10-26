# PA34 crate planning protocol checkpoint

This is preparatory maintainer work. PA34 remains planned. Neither a successful
plan nor a successful host build script is evidence that RustGM compiled a
target crate. Every target Rust dependency must eventually use the RustGM
frontend, including the source sysroot.

The [original library observations](../pa34/tests/stdlib-graph-outcomes.json)
now retain the pinned source sysroot's own package graphs: one core unit, five
alloc units and nineteen std units. The std graph has thirteen target compile
units and three paired host compile/run scripts (`compiler_builtins`, `libc`,
`std`). All thirty registry archives match the complete original lockfile;
nineteen local packages and 2,375 original files are authenticated. The shared
source tree is reused without changing its manifests or lockfile.

`Job.panic` explicitly selects abort/unwind through Cargo's development profile;
`None` preserves the manifest default and does not inherit an ambient override.
Independent query-only observations show abort target profiles with unchanged
unwind host profiles. The graph reader retains all four Rust edition values and
the optional ordered per-package `rustflags`, including repetitions. The selected
panic_abort unit carries `-Cpanic=abort`; execution must interpret supported
settings or reject them. Recording a profile does not implement that execution.

The original std graph requires Rust 2015 rustc-demangle and Rust 2018 cfg-if.
The owning PA34 fixtures independently distinguish root imports and pattern
fragments from later editions. Frontend support remains required. Actual
compiler-builtins build-script input closure also includes
`library/compiler-builtins/libm/configure.rs` outside its Cargo package root;
the full input/configuration closure and target execution remain open.

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

The [host graph executor](pa34-build-script-contract.md#shared-host-dependency-execution)
now schedules ordinary host rlibs and scripts, retaining their own generated
configuration and immediate DEP links metadata. Next, close local
package/configuration provenance and consume ordered generated cfg, environment
and source outputs in target jobs. Target metadata inventory must remain separate
from source-visible direct aliases: `--dependency-metadata` supplies transitive
artifacts without inventing extern-prelude bindings. Implement actual Rust 2021
semantics and per-crate macro edition provenance before claiming that profile.
Then execute dependency-first target jobs through RustGM's typed metadata and
CPPGM backend. Library acceptance requires the original selected dependency
graph and independently checked native consumers, not just this planning slice.

## Complete local package snapshots

`scripts/local_package.py` freezes a caller-declared original local package tree
without copying, rewriting or filtering its files. It retains every regular file,
directory entry and mode, with the existing inventory size limits. Missing, added,
changed, symlinked or mode-changed inputs invalidate the snapshot. Manifests must
provide explicit matching names and versions; unresolved workspace inheritance
remains unsupported. Local origins are explicit portable relative identifiers;
they and inventory hashes form the retained identity. Absolute Cargo path IDs
serve only to bind this invocation's graph, including differently named paths.

`cargo_plan.Job.local_packages` opts into complete local unit ownership. Every
selected path unit must belong to its exact snapshot; registry units still need
authenticated vendor inputs. Roots and origins must be unique, package roots
cannot overlap, and evidence/generated output lives outside package trees.
Workspace manifest/lockfile and tool/configuration pins remain separate inputs.
The host executor accepts the same snapshots and revalidates them before
publishing results. This does not establish complete workspace inheritance,
arbitrary build-script rerun/environment closure or persistent caching.

Qualification in `obj/parallel/pa34-local-inputs/development-v1/` passes all 164
Python harness tests, registration/layout/upstream checks and both real host
graph comparisons. The local and registry variants each perform six host
compilations and five script runs and match independent Cargo's declared
generated source, with zero candidate target compilations. Their observed times
are 2.47 and 2.22 seconds; these are qualification observations, not a speedup
claim. At 8/32/128 shared units, each local package is verified twice rather than
once per unit. No RustGM rebuild or full frontend suite was needed for this
Python-only checkpoint. Scoped cleanup removed 44 generated files and preserved
original inputs and receipts. Completion SHA-256:
`50806abc65dc5b5948af492d3b0367491ad8ea8675bf3fbb1a3abcc6b7ce96d3`.
