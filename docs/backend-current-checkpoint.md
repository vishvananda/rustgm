# CPPGM backend v0.9.3 checkpoint (41e06a23e)

All three tools and the LowIR library now pin release `v0.9.3` of
[cppgm](https://github.com/vishvananda/cppgm), commit
`41e06a23e91609dbf050866ef3a7de4848cf17ed`. This replaces the v0.9.1 pins at
`c9b05c4c0` ([v0.9.1 checkpoint](backend-v0.9.1-checkpoint.md)). The user
asked for the update on 2026-11-16 and pointed to the interface changes of
v0.9.2. Release v0.9.2 (`2a915cfbf`, tag object `70a79713e`) raised the
library's C interface to version 4, which adds record types. Release v0.9.3
contains v0.9.2 and adds two optimizer changes and Wasm local reuse. RustGM
never pinned v0.9.2. The LowIR text contract is unchanged.

## Distribution

The lock binds the bytes of upstream's release package, as for v0.9.1, and
`backend.distribution.json` names the upstream asset:

- Asset: `cppgm-v0.9.3-linux-x86_64.tar.gz`, 13,164,291 bytes, SHA-256
  `1c521c27b821d3aa565453695a537b0d5b2046e381addac2b6ba772412b9e6c6`. Its one
  release directory is `cppgm-v0.9.3-linux-x86_64`, recorded as `prefix`.
- The published `SHA256SUMS` (`17d84293…`) states the same archive digest.
- The package's provenance record
  (`cppgm-v0.9.3-linux-x86_64-provenance.json`, `12974cae…`, schema 2) states:
  - the same archive digest;
  - source `41e06a23e`;
  - compiler `cppgm 41e06a23e`, with host compiler g++ 13.3.0;
  - a hash for every file.
- `source-provenance.json` (`5a13dfda…`) names the same commit.
- Tag `v0.9.3` (tag object `e3d9ac59a`) names that commit.
- The archive holds 20 plain files, all inside the release directory. Each
  file's hash equals the record, and all eleven locked hashes equal it.
- `scripts/setup_backend.py` is unchanged. It installs the same eleven files
  and nothing else from the package.

| File | Bytes | SHA-256 |
|---|---|---|
| `bin/lowir` | 601,472 | `0d6aeb5482dc1402394ba907a7b3782dbd6396d6088f1ea66710815fefed71c1` |
| `bin/lowir2native` | 6,003,280 | `05af1fa6ff8bf961dde9f93d74453f7f1207b79df63fd9d51f526fec53745ed0` |
| `bin/cppgm` | 10,893,752 | `70e4168b0c631a0c996a2e0bc8fd27ab98aaac52c96b99eebdcdc7e0aad74f34` |
| `lib/libcppgm_lowir.a` | 7,517,638 | `84fca38d4e63f47312c83c53cdcfcfe29516a6c0b1b4afd3e6f21090cf722925` |
| `lib/libcppgm_lowir.so` | 4,746,472 | `09284004ac2c5255e262278088526b82fa8ffabbb9b9c26ad08315dcf4867666` |
| `include/lowir_interface.h` | 19,717 | `01347720f8b3b57066a3138aa005bfe886ad0c21f6d8ad9aa392e99048f1c92e` |
| `include/lowir_records.h` | 5,263 | `60f1d822476bd3b9bc38d9441ddb943e3f5bef2e97ab0891a5280154f0dc1910` |
| `include/lowir_c_vocabulary.h` | 15,627 | `f4a3a55849d369f7fafd8e0442a34b3d0fc9f6ace0389cb11513c8976aea2542` |
| `include/lowir_model.h` | 29,423 | `847a06baa6d92ca178f07698743e1b0a4f4080c30355640fdc3d3654193887bd` |
| `include/lowir_local_model.h` | 7,012 | `c6cbe9d4f485cf4fe7c567d07fda7c84d51dbde9581aad07b17218bf022743ae` |
| `include/ir_symbol_model.h` | 20,468 | `feda1b6df64e2263f327889b4684074fd91e46a36b78bccda48d5f73cb7ba93b` |

Five headers are byte-identical to v0.9.1; only `lowir_interface.h` changed.
Install with `python3 scripts/setup_backend.py`, or with `--bundle PATH` for
a package that is already downloaded. The v1, v2, v3, v0.9.0 and v0.9.1
releases, locks and pins stay unchanged for historical reproduction.

## Upstream CI at pin time

- The Release workflow for the tag, run 43785332096, succeeded in all four
  jobs:
  - Linux x86-64 tools, library and source;
  - Intel macOS tools and library;
  - wasm;
  - publish.
- These runs on the tagged commit's push to main succeeded:
  - Tests (43785224051);
  - Upstream C++11 correctness (43785224170);
  - Clang build (43785224344);
  - Intel macOS port (43785224111);
  - the Wasm32, Godot, index-cpp, simdjson, internal, self-build and public
    benchmark runs, and the benchmark badge freshness check;
  - Inception (43787627884) and Export Assignments (43787627955).
- The Boost benchmark run (43785224349) was still in progress. No run on the
  commit had failed.

## LowIR contract changes

Eight commits lead from `c9b05c4c0` to the tag. Only one changes code that
RustGM's programs compile to:

- The reviewed
  [LowIR contract](https://github.com/vishvananda/cppgm/blob/41e06a23e91609dbf050866ef3a7de4848cf17ed/pa8/lowir.md)
  (`89233a22…`) and
  [assembly boundary](https://github.com/vishvananda/cppgm/blob/41e06a23e91609dbf050866ef3a7de4848cf17ed/pa29/assembly.md)
  (`d8a49ac0…`) are byte-identical at the two commits.
- Constant division changed at `-O0` (`4a19895af`). The native selector now
  divides by a constant with shifts and multiplies at every level, as it did
  at `-O1` and above. It used `div` and `idiv` at `-O0` before, and it still
  does for `i1`. RustGM compiles at `-O0`, so its objects and executables
  change wherever a program divides by, or takes the remainder of, a
  constant. The maintainer native tests, the native course sweep and the
  linked consumers below run that code.
- Two optimizer changes do not apply at `-O0`, where neither the driver nor
  the library runs the optimizer (`optimize_unit` and `CppgmLowir::compile`
  return before it at level 0):
  - an `i1` field's byte is copied whole (`7e691717b`);
  - a function returns an object from its slot when the slot's load is the
    last step (`41e06a23e`).
- Weak constructor and destructor base entries are no longer rooted
  (`68e477c40`). That concerns cppgm's C++ frontend only, and it changed 159
  of cppgm's own C++ references.
- Wasm locals with disjoint live ranges share a slot (`70e5bcf4e`). RustGM
  emits no Wasm.
- Record types (`9258fd2e2`) are the library's interface version 4, below.
  The same commit moves the C++ frontend's psABI classifier into a shared
  header, `lowir/object_classes.h`, which the library uses too. `cppgm
  --version` prints `cppgm 0.9.3 (host configuration: g++)`.
- The other two commits raise the release number.

## The LowIR library

The release ships `libcppgm_lowir` (static and shared) with the C interface
`lowir_interface.h` at version 4. Version 4 adds to version 3 and changes no
function or record of it. The library accepts versions 2 to 4 from a client.
RustGM's lock, profile check, handshake and bridge accept only 4. Upstream's
instructions are
[Embedding the LowIR backend](https://github.com/vishvananda/cppgm/blob/41e06a23e91609dbf050866ef3a7de4848cf17ed/docs/lowir-library.md).

What version 4 adds:

| Addition | Functions and records | LowIR text it states |
|---|---|---|
| Record types | `cppgm_lowir_record` with `CppgmField` (`offset`, `type`, `count`); the type kind `CPPGM_TYPE_RECORD` | none of its own. The library writes `obj<bytes x align>`, plus the classes C gives an object with those fields: a parameter's `classes=`, `result_classes=`, a variadic call argument's `classes=N:` and a `va_arg`'s `classes=` |

A record is an object's size, alignment and fields. A field is an offset, a
scalar type or an earlier record type, and an element count. Fields may
overlap, as in a union, or be unaligned, as in a packed record. A record type
can be used wherever a type is taken. Classes that a boundary parameter, a
boundary result or an argument fact states are kept. An object of more than
16 bytes keeps the default classes, so it is passed in memory.

### What this checkpoint does

- The lock's optional `library` profile records interface version 4, the
  commit and the hash of both library forms and the six headers.
  `backend.library_profile` accepts it only for the locked commit, target
  and interface version 4.
- The handshake `backend.py --test-library` is ported to version 4. The
  producer `tests/backend/library.c`:
  - fails to compile against any header whose `CPPGM_LOWIR_ABI_VERSION` is
    not 4 or whose `CppgmField` is not 24 bytes;
  - at run time, refuses a library that reports another interface version
    or another release than the lock's;
  - requires the library to refuse a module created for version 5, for
    version 1 or for an unknown target.
- The producer gains one function, `rg_library_turn`, which states no
  classes. Its parameter is the record `{{f32, f32}, i64}`, a nested record.
  Its result is `{i64, f32[2]}`, an array field. Its two slots and its
  `return` name the records, and its signature comes from the version 2
  `cppgm_lowir_signature`.
  - The library's view must state the derived
    `[classes=sse_integer]) -> obj<16x8> [result_classes=integer_sse`, with
    the two floats sharing an SSE eightbyte.
  - The object must equal the driver's object from that view.
  - `tests/backend/library-observer.c` calls it with the matching C
    structs.
- The bridge is ported to version 4: `src/records.rs` and `build.rs` name
  it. The Rust handshake defines the same `rg_library_turn` through RustGM's
  model, with the classes that RustGM states for it (`Sse, Integer` and
  `Integer, Sse`). `backend.py --test-bridge` requires them in the bridge's
  view. The same observer links both objects, so the classes the library
  derives and the classes RustGM states meet one C caller.
- Three negative controls each changed one fact of `rg_library_turn` in a
  copy of the producer. No tracked file changed.

  | Control | Library view | Observer |
  |---|---|---|
  | Plain `obj<16x8>` instead of the records | no classes | exit 9 |
  | The second float an `i32` | `classes=integer_integer` | exit 9 |
  | The result's floats an `i64` | `result_classes=integer_integer` | exit 9 |

  In every control the object still equalled the driver's object of its
  view. The view check and the observer catch a wrong record; the equality
  check does not. The unchanged copy passed all three checks.

### Record types and RustGM

RustGM does not state record types: the bridge keeps stating the classes in
RustGM's model. The reasons are these:

- The text path is the default, and LowIR text has no record types.
  `dev/src/abi/classification.rs` must still give every C boundary object its
  classes for that text. Records would add a second classification, not
  replace the first.
- RustGM's typed LowIR keeps an object's span and classes
  (`lowir::boundary::Object`), not its fields. To state a record, the
  frontend would have to carry field layouts into LowIR for the bridge alone.
- The library keeps stated classes, so records stated beside RustGM's
  classes would change no byte.
- Typed as records, the objects of RustGM's own ABI would take C's classes,
  which would change that calling convention.

Records could give RustGM an independent oracle for its C classifier: state
each `repr(C)` boundary type of the PA30 `c-abi` corpus as a record, and
require the library's classes to equal the ones RustGM states. That is not
done here. The handshake's `rg_library_turn` makes that comparison for one
layout.

### The bridge's list under version 4

Version 4 resolves none of the items that the
[v0.9.1 checkpoint](backend-v0.9.1-checkpoint.md#fit-with-rustgms-writer)
lists as what a bridge must still handle:

- The library returns only a relocatable object, so executables still need
  a link step.
- A library function still has C linkage unless a symbol fact states
  another: `rg_library_turn`'s view says `linkage=c`.
- The header still states the use of record fields only for `convert`,
  `call` and `asm`.
- Temporaries are still dense with the parameters first. Float literals are
  still bit patterns, and registers are still named by bank and number.
- A module still compiles once, a failure still poisons it, and compilation
  is still serialized.
- Upstream's three exclusions remain.

Records add one implicit rule. Before a body is committed, its record types
become `obj` types, and before validation the library states the classes of
record objects past a variadic boundary's fixed parameters. Neither rule
applies to a module without records, such as every module the bridge states.

## Qualification

Everything below ran on main `952dfae84` plus this change, with the release
package installed by `setup_backend.py --bundle` from the verified download.
Each Makefile target ran as
`python3 scripts/resources.py run --wait-seconds 1800 --max-output-gib 24 -- make TARGET`.
Each other command ran behind that prefix and
`/usr/bin/python3 scripts/storage.py run --max-output-gib 24 --no-prune --`.

- `make test-backend`: all five handshakes pass.
  - `PASS: CPPGM parses, validates and executes LowIR (exit 42)`
  - `PASS: LowIR object interoperates with independent C calls, callbacks and globals`
  - `PASS: LowIR assembly emits movntps in objects and executables with full bits and guards`
  - `PASS: the LowIR library builds a module with boundary classes, call boundaries, noreturn, assembly and record types in process; its objects equal the driver's and link`
  - `PASS: the in-process bridge states RustGM's LowIR to the library; its objects equal the driver's objects of the library view and of RustGM's text, and link`
- The bridge crate passes strict Clippy (`cargo clippy --manifest-path
  dev/backend-bridge/Cargo.toml --locked --profile dev-fast --all-targets --
  -D warnings`) and `rustfmt --check`. `scripts/rust_audit.py --root .`
  reports `4017 files; 0 findings`.
- `make test-maintainer-native`: all 531 release maintainer native tests
  (`--ignored`) pass, in 16 `test result: ok` lines with no failure. Its
  released native units pass too: eight `PASS` lines with 2,059 independent
  checks and 729 executable comparisons, the same totals as under v0.9.1.
  - It ran with `CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=2`, after a separate
    `cargo build --release --locked` with two jobs.
  - From a cold cache, two jobs exceeded the 12 GiB memory limit of the
    job's scope while compiling the crate's library and its unit-test
    harness side by side.
- `cargo build --manifest-path dev/Cargo.toml --release --locked`, then
  `python3 scripts/check_native.py STAGE --app dev/rustgm` for PA11 to PA14
  and PA16 to PA24: all 13 stages pass. Every count equals the v0.9.1 sweep's,
  so the totals do too: 5,551 independent checks, 3,106 executable, 358
  codegen-only and 105 object comparisons.
- A fresh root installed the published package from its URL with
  `python3 scripts/setup_backend.py`. The root held only `scripts/`,
  `tests/backend/` and the lock files. Every pinned file matched the lock,
  and the four handshakes that need no Rust build passed there
  (`clean-install: PASS`).
- `python3 scripts/bringup_linked.py` compiled eight linked core consumers
  with this tree's release compiler (`c68388f7`) and the census's skip list
  (1,063 rows). The eight were `atomics`, `consts-statics`, `duration-net`,
  `integers`, `intrinsic-ops`, `fmt-floats`, `fmt-integers` and `cells-mem`.
  Each reports `native: exit 0; stdout matches`, and the run reports
  `passed 8 of 8`.
- `make test-upstream`: `Verified 128 pinned upstream fixtures` and
  `Verified 126 Rust 1.98.0 Reference files`.
- `make test-harness`: `Ran 376 tests` and `OK`, including the installer and
  bridge unit tests; `Maintainer suites: 501 roots in 13 binaries`.

Two contracts refuse any unreviewed lock, as before. Each now binds the
v0.9.3 lock and keeps the earlier bindings as history:

- the PA28 atomic capability contract
  (`pa28/tests/backend/atomics/contract.json`), with the seven reviewed CPPGM
  sources at the new commit. All seven are byte-identical to v0.9.1, so their
  hashes did not change.
- the PA31 source-attribute CLI contract
  (`pa31/tests/spec/source-attribute-transport/cli-contract.json`), with the
  new lock and driver hashes.

Both are declared contracts without a generator. A script did four things:
- replaced the lock, the commit and the driver hash;
- checked the seven source hashes against upstream's files at the new
  commit;
- moved the v0.9.1 bindings to the history lists;
- changed no declared case, outcome, encoding count or diagnostic.

Their own tools then checked them:

- `python3 scripts/atomic_capabilities.py backend obj/parallel/pa28/backend-v0.9.3-pin`
  passes with all 65 commands.
- The maintainer native tests include both consumers:
  `intrinsic_backend::capabilities::atomic_width_pointer_bool_and_fence_backend_contracts`
  and
  `crate_linking::versioned_clients::cli_function_contracts_reach_fresh_native_consumers_returning_seven`.

No generated reference changed. No timing measurement was taken: the
optimizer changes do not run at `-O0`, and the qualification checks the
behaviour of the one `-O0` change, not its speed.

Retained evidence, under `obj/backend-pin-v6/` unless a path says otherwise:

- `published/`: the downloaded asset, `SHA256SUMS` and both provenance
  records
- `review/`: the reviewed sources and contract documents at both commits,
  with the differences of `lowir_interface.h` and the library documentation
- `logs/test-backend.log`:
  `b3d5f137a9097eced05c459ca05324de123af06c3363f5c9c138419ae3047e89`
- `logs/maintainer-native.log`:
  `3e956f1d23765e9fe921d67c3d18ca5d72057ef03baa6f5d6b26008b5f406490`
- `logs/native-sweep.log`:
  `d6b7176539b80ae60d5d15d533045fb3f75a86c1663176ab3e450ad75cba21d9`
- `logs/clean-install.log`:
  `6df9b8bda631b354d36d62672f72daea9a9994aa1d940da2b05b381714976337`
- `logs/record-negative-controls.log`:
  `4746f408fc380d091e8664e6e5a62d04042ade88ed54c56f29ac857811479ee4`
- `logs/linked-consumers.log`:
  `c1cb2319753c742776bbc8003cdf10836e123294063368751a8487e50b9fb3db`
- `obj/parallel/pa28/backend-v0.9.3-pin/receipt.json`:
  `ae5b546da09439899f89c42989c2f2755b1d5cb56ea3aea9c27490284fc2ee56`

## Decisions

- The lock, profile, handshake and bridge accept exactly interface version
  4. Rejected: keeping the bridge at version 3, which the library still
  accepts. The reviewed header set belongs to one version, and a client
  written for version 3 against a version 4 header would not be checked
  against the interface it builds with.
- The pin moves straight from v0.9.1 to v0.9.3. v0.9.3 contains v0.9.2's
  interface and is the release the user named.
- The bridge states no record types (see above). The handshake states one
  layout both ways and checks that the library's classes and RustGM's agree.
- The v0.9.1 lock, checkpoint and pins are kept. A worktree that still holds
  the v0.9.1 configuration must run `scripts/setup_backend.py` again.
