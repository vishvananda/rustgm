# CPPGM backend v0.9.1 checkpoint (c9b05c4c0)

The [current checkpoint](backend-current-checkpoint.md) superseded this pin on
2026-11-16. This record of the v0.9.1 qualification is otherwise unchanged.

All three tools and the LowIR library now pin release `v0.9.1` of
[cppgm](https://github.com/vishvananda/cppgm), commit
`c9b05c4c0f7bb1294c717551c61145940b21125e`, replacing the v0.9.0 pins at
`bb0980105` ([v0.9.0 checkpoint](backend-v0.9.0-checkpoint.md)). The user
asked for this update on 2026-11-15. The release raises the library's C
interface to version 3, which states the boundary facts, assembly templates
and debug records that version 2 could not. The LowIR text contract is
unchanged.

## Distribution

The lock binds the bytes of upstream's release package, as for v0.9.0, and
`backend.distribution.json` names the upstream asset:

- Asset: `cppgm-v0.9.1-linux-x86_64.tar.gz`, 13,534,556 bytes, SHA-256
  `90e45319e9d49c75726fe4859d9c59f79abdecb288d0fce5d9698f07defc3ea9`. Its one
  release directory is `cppgm-v0.9.1-linux-x86_64`, recorded as `prefix`.
- The published `SHA256SUMS` (`a9b07543…`) states the same archive digest.
  The package's provenance record
  (`cppgm-v0.9.1-linux-x86_64-provenance.json`, `e4e66af1…`) states the same
  digest, source `c9b05c4c0` and a hash for every file. Tag `v0.9.1` (tag
  object `5d1732bb4`) names that commit. The archive holds 20 plain files,
  all inside the release directory, and each one's hash equals the record.
  All eleven locked hashes equal it.
- The packaging changed. The provenance record, now schema 2, names the
  compiler as `cppgm c9b05c4c0`: the packaged tools are cppgm's second-stage
  build of itself, and cppgm compiled the library. The host compiler,
  g++ 13.3.0 on Ubuntu 24.04, builds the first stage, links and supplies the
  standard library. The v0.9.0 package was built by g++ alone.
- `scripts/setup_backend.py` is unchanged. It installs the same eleven files
  and nothing else from the package.

| File | Bytes | SHA-256 |
|---|---|---|
| `bin/lowir` | 638,448 | `434f5223c2a96d6099461df0ec229c732f347e7e1a6299704a805b76bf290bba` |
| `bin/lowir2native` | 6,174,304 | `c6068a0dcd669191313b98facb5f16247277a509fd939757d00cdf52b63d28cf` |
| `bin/cppgm` | 11,239,496 | `d59be152cc3d813a67208188602c2b9c78324d5dfbaf4a07ecb563a8489acf8f` |
| `lib/libcppgm_lowir.a` | 8,947,054 | `7ebd76316229ad7522d11d2538d7a6def6cf4a116ac5f84c427898131ff25f19` |
| `lib/libcppgm_lowir.so` | 4,858,952 | `cd6c387f23184a9758c9b77213023705f04e9c7bf0e2ef827728273e8508159d` |
| `include/lowir_interface.h` | 18,250 | `f94aea4cf86f7a2eef7a503c7106d72175bdf85375c5667fe96beb801eb08687` |
| `include/lowir_records.h` | 5,263 | `60f1d822476bd3b9bc38d9441ddb943e3f5bef2e97ab0891a5280154f0dc1910` |
| `include/lowir_c_vocabulary.h` | 15,627 | `f4a3a55849d369f7fafd8e0442a34b3d0fc9f6ace0389cb11513c8976aea2542` |
| `include/lowir_model.h` | 29,423 | `847a06baa6d92ca178f07698743e1b0a4f4080c30355640fdc3d3654193887bd` |
| `include/lowir_local_model.h` | 7,012 | `c6cbe9d4f485cf4fe7c567d07fda7c84d51dbde9581aad07b17218bf022743ae` |
| `include/ir_symbol_model.h` | 20,468 | `feda1b6df64e2263f327889b4684074fd91e46a36b78bccda48d5f73cb7ba93b` |

Four headers are byte-identical to v0.9.0: `lowir_records.h`,
`lowir_model.h`, `lowir_local_model.h` and `ir_symbol_model.h`. Install with
`python3 scripts/setup_backend.py`, or with `--bundle PATH` for a package
that is already downloaded. The v1, v2, v3 and v0.9.0 releases, locks and
pins stay unchanged for historical reproduction.

## Upstream CI at pin time

- The Release workflow for the tag, run 43752009450, succeeded in all four
  jobs: Linux x86-64 tools, library and source; Intel macOS tools and
  library; wasm; publish.
- On the tagged commit's push to main, Tests (run 43750859130), Upstream
  C++11 correctness (43750858858), Clang build (43750858924), Intel macOS
  port (43750858883), the Wasm32, simdjson, Godot, index-cpp, self-build,
  internal, public and Boost (43750858933) benchmark runs, Inception
  (43753429844) and Export Assignments (43753429631) succeeded. No run on
  the commit failed or is pending.
- The same commit passed the pull-request runs before the merge, including
  Tests (43748150242) and the Release workflow (43748150237).

## LowIR contract changes

There are 16 commits from `bb0980105` to the tag. None changes a form that
RustGM emits:

- The reviewed
  [LowIR contract](https://github.com/vishvananda/cppgm/blob/c9b05c4c0f7bb1294c717551c61145940b21125e/pa8/lowir.md)
  (`89233a22…`) and
  [assembly boundary](https://github.com/vishvananda/cppgm/blob/c9b05c4c0f7bb1294c717551c61145940b21125e/pa29/assembly.md)
  (`d8a49ac0…`) are byte-identical at the two commits.
- The text reader now holds the metadata keys it has seen in 64 bits,
  because `this_return` is the 33rd key. RustGM states `arity`, `return`,
  `result_classes`, `storage`, `role` and `classes`, which were all below
  the old limit. The reader's validation of debug records moved to a shared
  file. RustGM writes no debug records.
- Three optimizer changes do not apply at `-O0`, where RustGM compiles:
  split-form aggregate slots are no longer split again, a small callee is
  inlined at a cold site that passes a caller slot's address, and vectors of
  literal lanes load from read-only data. The six frozen inputs of the
  measurement below compile to the same bytes under both pins.
- The driver finds the headers of another GCC installation when the build's
  are absent. That concerns cppgm's C++ frontend only. `cppgm --version`
  prints `cppgm 0.9.1 (host configuration: g++)`.
- The remaining commits change upstream's workflows, packaging and README.

## The LowIR library

The release ships `libcppgm_lowir` (static and shared) with the C interface
`lowir_interface.h` at version 3. Version 3 adds to version 2 and changes no
function or record of it. The library accepts versions 2 and 3 from a
client; RustGM's lock, profile check and handshake accept only 3. Upstream's
instructions are
[Embedding the LowIR backend](https://github.com/vishvananda/cppgm/blob/c9b05c4c0f7bb1294c717551c61145940b21125e/docs/lowir-library.md).

What version 3 adds:

| Addition | Functions and records | LowIR text it states |
|---|---|---|
| Versions | `cppgm_lowir_abi_version`, `cppgm_lowir_version`, `CPPGM_LOWIR_ABI_VERSION_MIN` | none |
| Named numbers | an enumerator for every argument and record field: `CPPGM_TARGET_*`, `CPPGM_FUNCTION_*`, `CPPGM_GLOBAL_*` and `CPPGM_CODE_*` in `lowir_interface.h`; `CPPGM_FLAG_*`, `CPPGM_DATA_*`, `CPPGM_BINDING_*`, `CPPGM_STORAGE_*`, `CPPGM_ARITY_*`, `CPPGM_KEY_*`, `CPPGM_ROLE_*`, `CPPGM_ASM_*` and the other metadata values in `lowir_c_vocabulary.h` | none |
| Boundary metadata | `cppgm_lowir_boundary` with `CppgmParameter` (`object_bytes`, `pass`, `alias`, `classes`, `shape`) and `CppgmBoundary` (`arity`, `effects`, `unwind`, `returns`, `facts`, `result_classes`, `result_shape`); `CPPGM_CLASSES(first, second)` and `CPPGM_CLASSES_MEMORY` | a function's or a call boundary's `arity=`, `effects=`, `unwind=`, `return=`, `repeatable=`, `noalias_result=`, `this_return=`, `result_classes=`, `result_shape=`; a parameter's `pass=`, `alias=`, `object_bytes=`, `classes=`, `shape=` |
| Symbol metadata | `cppgm_lowir_symbol_fact`, `cppgm_lowir_symbol_name` | `role=`, `linkage=`, `binding=`, `storage=`, `tls_for=`, the yes-valued keys `keep_alias=` to `cold=`; `object=`, `section=` |
| Argument facts | `cppgm_lowir_body_argument_facts` with `CppgmArgumentFact` | a call's `classes=N:`, `argument_type=N:` and `argument_shape=N:`; a `va_arg`'s `classes=` and `shape=` |
| Assembly templates | `cppgm_lowir_assembly` with `CppgmAssembly` and its register, group, immediate, alternative, operand and piece records; the `asm` instruction `CPPGM_OP_ASM` | `assembly @name ... { ... }` and `asm` |
| Debug information | `cppgm_lowir_debug_file`, `cppgm_lowir_function_location`, `cppgm_lowir_body_locations`, `cppgm_lowir_debug_type`, `cppgm_lowir_debug_scope`, `cppgm_lowir_debug_local`; `CppgmDebugLocation`, `CppgmDebugType` | `!dbg(...)`, `debug_type`, `debug_scope`, `debug_local` |

A call states its boundary by setting `CPPGM_FLAG_SIGNATURE` and holding a
`cppgm_lowir_boundary` ID in `immediate`. That is the text's
`as (...) -> ...` form.

### Fit with RustGM's writer

The v0.9.0 checkpoint listed four forms of RustGM's writer that version 2
could not state. Version 3 states each one, and every other form the writer
emits has a counterpart:

| RustGM writes | Version 3 | Checked by |
|---|---|---|
| `[classes=...]` on an object parameter | `CppgmParameter::classes` | handshake |
| `[result_classes=...]` | `CppgmBoundary::result_classes` | handshake |
| `as (...) -> ... [...]` on a call through a value | `CPPGM_FLAG_SIGNATURE` and a boundary ID | handshake |
| `[classes=N:...]` on a call argument | `cppgm_lowir_body_argument_facts` | handshake |
| `[arity=variadic]` | `CppgmBoundary::arity` | handshake |
| `[return=noreturn]` | `CppgmBoundary::returns` | handshake, by text and object equality |
| `assembly` templates and `asm` | `cppgm_lowir_assembly`, `CPPGM_OP_ASM` | handshake |
| `[storage=readonly]` globals | `cppgm_lowir_global` | handshake |
| `[role=entry]` | `cppgm_lowir_symbol_fact` with `CPPGM_KEY_ROLE` | reading only |
| global declarations, zero runs and address items | `CPPGM_GLOBAL_DECLARATION`, `CPPGM_DATA_ZERO`, `CPPGM_DATA_ADDRESS` | reading only |
| `va_start`, `va_arg`, the atomic instructions, `unary`, `cmp`, `convert`, `copy`, `jump`, `branch`, `[volatile]`, `[exact]` | the `CPPGM_OP_*` and `CPPGM_FLAG_*` values of the same names | reading only |
| debug records | the debug functions | RustGM writes none |

What a bridge must still handle, because the library differs from the text
path:

- The library returns a relocatable object and nothing else. It has no
  counterpart of `lowir2native`'s linked executable, so the executable
  profile still needs a link step.
- A library function has C language linkage unless a symbol fact states
  another. RustGM's text states no linkage, so a bridge must state
  `CPPGM_LINKAGE_DEFAULT` for every function to build the module that its
  text describes.
- The header names every number, but it states how an instruction uses its
  record fields only for `convert`, `call` and `asm`. Upstream's text reader
  is the reference for the others. For example, `copyobj` holds its size in
  `immediate` and its alignment's logarithm in `sub`, and a compare-exchange
  holds its failure order in `sub`. The check is the one the handshake uses:
  write the module with `cppgm_lowir_write`, compile that text with the
  pinned driver and require equal object bytes.
- Temporaries are dense numbers with the parameters first, so a bridge
  renumbers RustGM's `%pN`, `%tN` and `%oN`. Float literals are bit
  patterns. An assembly template names a register by bank and number, not by
  its spelling.
- A module compiles once, a failure poisons it, and compilation is
  serialized across the process. The static library is C++ inside and links
  with the C++ runtime and pthread.
- Upstream states three exclusions: assembly-defined functions, ordered
  module assembly and Wasm objects. RustGM emits none of them.

### What this checkpoint does

- The lock's optional `library` profile records interface version 3, the
  commit and the hash of both library forms and the six headers.
  `backend.library_profile` accepts it only for the locked commit, target
  and interface version 3. `backend.library_release` derives the release
  that the library must report from the lock's tag.
- The handshake `backend.py --test-library`, part of `make test-backend`, is
  ported to version 3. The original C producer `tests/backend/library.c`
  fails to compile against any header whose `CPPGM_LOWIR_ABI_VERSION` is
  not 3, and refuses at run time a library that reports another interface
  version or another release than the lock's. It also requires the library
  to refuse a module created for version 4, for version 1 or for an unknown
  target.
- The producer builds one module in process: a read-only global, a function
  that loads it, and five functions that use the additions of version 3.
  `rg_library_swap` takes and returns 16-byte objects with
  `classes=integer_sse` and `result_classes=sse_integer`, and states default
  linkage through a symbol fact.
  `rg_library_apply` calls a function pointer through a stated boundary with
  both classes. `rg_library_tally` passes an object to a variadic host
  function with `classes=1:integer_sse`. `rg_library_stream` invokes an
  assembly template of the shape RustGM writes for `movntps`.
  `rg_library_stop` is `return=noreturn`.
- The handshake compiles the module twice, as fixed and as
  position-independent code. Each time it requires the library's text view
  to state every one of those forms, compiles that text with the pinned
  driver (`cppgm -O0 -c`, with `-fPIC` for the second) and requires the
  driver's object to equal the library's object byte for byte. It then
  links the object with an independent C observer. The host compiler places
  each argument and result by the psABI, so a class that did not reach the
  object fails a check.
- Six negative controls removed one fact each from the producer: the
  parameter classes and the result classes of `rg_library_swap`, the
  parameter classes of `rg_library_apply`, the parameter classes and the
  result classes of its call boundary, and the classes of the variadic call
  argument. The observer failed at the matching check every time.
  `return=noreturn` has no observable effect at `-O0`, so the text view and
  the equal objects are its evidence.

### The in-process bridge

RustGM can now hand its typed LowIR to the library directly. Nothing that
already exists changed: `rustgm`, every script and every test still reach
the backend through LowIR text and the subprocess bridge.

- `dev/backend-bridge/` is a separate crate, `rustgm-backend-bridge`, with
  its own lock file. The frontend does not depend on it and keeps
  `unsafe_code = "forbid"`. Its build script reads the verified pin from
  `scripts/backend.py --library-config`, which checks the hashes the lock
  records. It refuses an interface version other than 3 and links
  `libcppgm_lowir.a` with the C++ runtime, pthread and libm. With no
  installed library the crate does not build, and nothing else is affected.
- Every foreign declaration and unsafe block is in `src/ffi.rs`. Each block
  states why it is sound in a `// SAFETY:` comment. `scripts/rust_audit.py`
  accepts FFI in that one file only, requires the comment there and still
  refuses unsafe functions, impls and traits.
- At run time the bridge refuses a library that reports another interface
  version or another release than the lock's.
- The bridge states the module that the pinned text reader would build from
  RustGM's text for the same program. Symbols, signatures, blocks,
  temporaries, slots, literals and templates are created in the order the
  text first mentions them. Each literal carries the bits the reader gives
  its spelling in that context. The writer's typed checks run first
  (`lowir::validation::instruction` and `terminator`).
- It states every form the writer writes:
  - globals with scalar, zero and address items, including
    `storage=readonly`;
  - `classes=` and `result_classes=`;
  - call boundaries `as (...)`, through `FLAG_SIGNATURE` and a boundary;
  - call-site `classes=N:`, through argument facts;
  - `arity=variadic` and `return=noreturn`;
  - assembly templates and `asm`.

  Every function has default linkage through a symbol fact, and the entry
  has the entry role. The bridge states no debug records.
- A form it does not state sends the program down the text path, and says
  so. The forms and their counts go to standard error. The program's text is
  written beside the output and compiled by `backend.py --emit-object`, and
  that object is written. Most of these forms are refused by the text reader
  too: a literal it cannot read in its context, an object alignment that is
  not a power of two, a register outside the writer's tables, inconsistent
  identities, a call through a slot or a literal, and call-site classes of
  one eightbyte. The bridge alone also refuses an integer literal of more
  than 64 bits in a float context.
- `rustgm-inprocess` is `rustgm`'s command line with `--emit-object`, plus
  `--library-view PATH`, `--object-pic` and `--emission-timing`. Both
  binaries share `dev/bin/driver.rs`. `rustgm` emits through
  `dev/bin/emission.rs`, which is the writer exactly as before.
- `backend.py --test-bridge` joins `make test-backend`. It builds the crate,
  states `tests/backend/library.c`'s module through RustGM's own `Program`
  and checks it in fixed and position-independent form:
  - the library view must show every form;
  - the object must equal, byte for byte, the driver's objects from that view
    and from RustGM's own text;
  - `movntps` must be in `rg_library_stream`;
  - the object must link with `tests/backend/library-observer.c` and pass it.

  Without an installed library the handshake reports `SKIP`. With an
  installed library whose bytes changed, it fails.

What still does not use the bridge: linking, which the object bridge still
does; optimization, since the bridge compiles at level 0 only, matching the
driver's `-O0`; and the defaults. The text path cannot compile
position-independent code, so a fallback under `--object-pic` is an error.

The bridge's evidence, on the lane branch `lane/frontier-20261115` over main
`10995ebcd`, with the commands behind the same resource and storage prefixes
as below:

- `make test-backend`: all five handshakes pass, the fifth being
  `PASS: the in-process bridge states RustGM's LowIR to the library; its
  objects equal the driver's objects of the library view and of RustGM's
  text, and link`.
- Two temporary negative controls, reverted afterwards:
  - Swapping the handshake's call-site classes to `1:sse_integer` failed
    the view check. Bypassing that check, the observer still failed (exit 6)
    in both forms.
  - Forcing a fallback at every `branch` sent seven of eight sampled
    programs down the text path. Each reported the count of the form
    (`negative control: branch x4`, for example), left no temporary file,
    produced the driver's object of its text byte for byte and behaved the
    same.
- An equivalence corpus compiled each program twice with
  `rustgm-inprocess`: as text, and in process with the library view. The
  driver compiled the view and the text with `cppgm -O0 -c`. Both objects
  were linked, with the case's C providers, and run.

  | Corpus | Programs | Equal to view object | Equal to text object | Same behaviour | Fallbacks |
  |---|---|---|---|---|---|
  | PA11 to PA24 executed cases, every 25th | 131 | 131 | 131 | 131 | 0 |
  | PA30 `c-abi`, PA28 `atomic-u32`, PA27 `unsafe-places`, all | 103 | 103 | 103 | 103 | 0 |
  | PA25 `const-machine`, PA26 `operator-kernel` and `iteration-kernel`, PA28 `runtime-memory`, every 10th | 64 | 64 | 64 | 64 | 0 |
  | PA41 `asm-native/native.rs` with its C observer | 1 | 1 | 1 | 1 | 0 |
  | Linked core consumers (release build) | 7 | 7 | 7 | 7 | 0 |

  The texts of these programs state `arity=variadic` in 23, call-site
  `classes=N:` in 5, call boundaries `as (...)` in 29 and `result_classes=`
  in 17. The PA41 program states six assembly templates; its object has six
  `movntps` stores and prints `expected.stdout`. All seven linked consumers
  print their recorded standard output: `atomics`, `consts-statics`,
  `duration-net`, `integers`, `intrinsic-ops`, `fmt-floats` and
  `fmt-integers`.

  The in-process object equals the driver's object from RustGM's text
  because the bridge creates every identity in the reader's order. A bridge
  with its own numbering would link and behave the same, but its bytes
  would differ.

The measurement compared text plus a subprocess with in-process emission on
the largest of those modules. The module was `intrinsic-ops`, whose text is
3,443,718 bytes, built against core with the census's skip list. One
immutable release `rustgm-inprocess` served both arms (SHA-256 `6b48fc27`),
so the frontend was the same and only the transport differed:

- text: `--emit-lowir -o t.lowir`, then `cppgm -O0 -c t.lowir`;
- in process: `--emit-object`.

`--emission-timing` timed the emission step inside the binary. The input
tree was frozen read-only (tree hash `cb49e107`). The run was two A/A pairs
followed by three A B B A blocks. Every run produced the same LowIR and the
same object (`fe163f3f`) in both arms. The observations are in
`obj/bridge-measure/run-2.log`.

| Six A B B A runs per arm, median (range) | Text and subprocess | In process |
|---|---|---|
| Emission inside `rustgm` | 23.8 ms (23.2–25.6), the writer | 87.4 ms (80.5–145.9), statement and library compile |
| Driver process | 109.6 ms (105.0–125.0) | none |
| Transport | 134.3 ms (128.5–148.5) | 87.4 ms (80.5–145.9) |
| End to end | 25.7 s (23.4–33.8) | 27.6 s (24.7–32.6) |
| Peak RSS of `rustgm` | 901 MiB | 901 MiB |
| Peak RSS of the driver | 207 MiB | none |

Per A B B A block, in-process transport was 17.0, 44.0 and 55.4 ms faster.
The A/A emission differences were 0.5 ms (text) and 8.2 ms (in process).

End to end, the blocks differed by +1.66, −1.25 and −3.24 s, and the A/A
pairs by 0.44 and 1.86 s. That is the noise of a 24 to 34 s frontend on a
machine at load average 56, which is larger than the whole transport. The
bridge saves about a third of the transport, roughly 0.2 % of this build,
and adds no peak memory: the frontend's peak comes before emission. The
text arm called the driver directly, without `backend.py`'s Python start-up,
so in normal use the text path costs more than shown here.

## Decisions

- The library profile and the handshake accept exactly one interface
  version. Rejected: accepting the range 2 to 3 that the library itself
  accepts. The reviewed header set belongs to one version, and a client
  written for it must not run against another.
- The handshake compares the library's object with the pinned driver's
  object for the same text. That proves the library is the reviewed backend
  and gives a bridge a check for any form that only reading supports.
  Rejected: checking behaviour alone, which cannot see `return=noreturn` and
  missed a call boundary without classes in an earlier draft of the
  producer.
- The fixed code form is the one the object bridge's own command makes. The
  position-independent form is checked too, because the host C driver links
  position-independent executables by default.
- The bridge creates identities in the text reader's order, so that its
  object equals the driver's object from RustGM's text. Rejected: numbering
  in the bridge's own order, which links and behaves the same but cannot be
  checked byte for byte against the text path.
- A statement that meets a form the bridge does not state is skipped, not
  sent with a placeholder. One pass therefore counts every such form, and
  the library never holds a partial record.
- The fallback is automatic and printed, so a program the bridge cannot
  state still builds. Rejected: failing the build, which would make
  `--emit-object` less capable than the text path.
- The v0.9.0 lock, checkpoint and pins are kept. A worktree that still holds
  the v0.9.0 configuration must run `scripts/setup_backend.py` again.

## Qualification

Everything below ran on main `a576c7e96` plus this change, with the release
package installed by `setup_backend.py`. Each Makefile target ran as
`python3 scripts/resources.py run --wait-seconds 1800 --max-output-gib 24 -- make TARGET`.
Each other command ran behind that prefix and
`/usr/bin/python3 scripts/storage.py run --max-output-gib 24 --no-prune --`.

- `make test-backend`: all four handshakes pass.
  - `PASS: CPPGM parses, validates and executes LowIR (exit 42)`
  - `PASS: LowIR object interoperates with independent C calls, callbacks and globals`
  - `PASS: LowIR assembly emits movntps in objects and executables with full bits and guards`
  - `PASS: the LowIR library builds a module with boundary classes, call boundaries, noreturn and assembly in process; its objects equal the driver's and link`
- `make test-maintainer-native`: all 529 release maintainer native tests
  (`--ignored`) pass, in 16 `test result: ok` lines with no failure. Its
  released native units pass too, in eight `PASS` lines with 2,059
  independent checks and 729 executable comparisons.
- `cargo build --manifest-path dev/Cargo.toml --release --locked`, then
  `python3 scripts/check_native.py STAGE --app dev/rustgm` for PA11 to PA14
  and PA16 to PA24: all 13 stages pass. The totals equal the v0.9.0 sweep's.
  PA16, PA23 and PA24 were refused resource admission on the first pass and
  passed when rerun.

  | Stage | Independent checks | Executable | Codegen-only | Object |
  |---|---|---|---|---|
  | PA11 | 191 | 180 | 0 | 0 |
  | PA12 | 113 | 64 | 0 | 0 |
  | PA13 | 532 | 365 | 0 | 0 |
  | PA14 | 12 | 12 | 0 | 0 |
  | PA16 | 131 | 123 | 0 | 2 |
  | PA17 | 205 | 135 | 0 | 0 |
  | PA18 | 505 | 308 | 6 | 3 |
  | PA19 | 768 | 334 | 2 | 0 |
  | PA20 | 1,013 | 613 | 26 | 0 |
  | PA21 | 525 | 311 | 0 | 0 |
  | PA22 | 392 | 144 | 134 | 0 |
  | PA23 | 420 | 89 | 185 | 0 |
  | PA24 | 744 | 428 | 5 | 100 |
  | Total | 5,551 | 3,106 | 358 | 105 |

- A fresh root installed the published package from its URL with
  `python3 scripts/setup_backend.py`. Every pinned file matched the lock and
  all four handshakes passed there (`clean-install: PASS`).
- `python3 scripts/bringup_linked.py` with an observe build of this tree:
  the seven linked core consumers (`atomics`, `consts-statics`,
  `duration-net`, `integers`, `intrinsic-ops`, `fmt-floats`, `fmt-integers`)
  each report `native: exit 0; stdout matches`, and the run reports
  `passed 7 of 7`.
- `make test-upstream`: `Verified 128 pinned upstream fixtures` and
  `Verified 126 Rust 1.98.0 Reference files`.
- `make test-harness`: `Ran 375 tests` and `OK`, including the installer and
  bridge unit tests; `Maintainer suites: 500 roots in 13 binaries`.

Two contracts refuse any unreviewed lock, as before. Each now binds the
v0.9.1 lock and keeps the earlier bindings as history:

- the PA28 atomic capability contract
  (`pa28/tests/backend/atomics/contract.json`), with the seven reviewed CPPGM
  sources at the new commit. All seven are byte-identical to v0.9.0, so their
  hashes did not change.
- the PA31 source-attribute CLI contract
  (`pa31/tests/spec/source-attribute-transport/cli-contract.json`), with the
  new lock and driver hashes.

Both are declared contracts without a generator. A script replaced the lock,
the commit and the driver hash, recomputed the seven source hashes from
upstream's files at the new commit and moved the v0.9.0 binding to the
history list. No declared case, outcome, encoding count or diagnostic
changed. Their own tools then checked them:

- `python3 scripts/atomic_capabilities.py backend obj/parallel/pa28/backend-v0.9.1-pin`
  passes with all 65 commands.
- The maintainer native tests include both consumers:
  `intrinsic_backend::capabilities::atomic_width_pointer_bool_and_fence_backend_contracts`
  and
  `crate_linking::versioned_clients::cli_function_contracts_reach_fresh_native_consumers_returning_seven`.

No generated reference changed.

The measurement used immutable old and new tools on the frozen inputs of the
v0.9.0 checkpoint, under an exclusive resource lease. Each input ran in six
blocks of A A A B B A, with repeats so that each observation lasts roughly
0.04 to 0.2 s. For all six inputs the new tools produce the same object or
executable bytes as the old tools, and every observer and native behaviour
matched.

| Input | Kind | A/A range | Paired B/A median |
|---|---|---|---|
| `deref` (2,573 B) | object + observer | 0.923–1.040 | 1.016 |
| `receivers-128` (182,027 B) | object + observer | 0.885–1.111 | 1.004 |
| `f64-saturation` (20,999 B) | program | 0.948–1.129 | 1.005 |
| `text-bytes` (14,250 B) | program | 0.393–1.186 | 0.942 |
| `alias-callable` (1,312 B, control) | program | 0.959–1.100 | 1.014 |
| `core-atomics` (1,377,713 B) | object | 1.010–1.295 | 1.009 |

The medians show neither a speedup nor a regression from the larger
self-built tools. The run was noisy for two inputs: one A/A block of
`text-bytes` fell to 0.393, and every A/A block of `core-atomics` was above
1. This is not a qualification of generated-code quality.

Retained evidence, under `obj/backend-pin-v5/` unless a path says otherwise:

- `published/`: the downloaded asset, `SHA256SUMS` and provenance record
- `review/`: the contract documents, the library documentation and header
  differences between the two commits
- `logs/native-qualification.log` (release build and course sweep):
  `209adc6928fc51e51107531a88ba3b954915b96d8687117cbdde68793f0cd1d3`
- `logs/maintainer-native.log`:
  `cf275f071a94daeeb3e11336e49dea601ee98a8ee35a96ddb39c53fdaefcc4e9`
- `logs/clean-install.log`:
  `913fd3ded83434c2b9a233708195fce15647208e56b64d82b8f08b6fbab14836`
- `logs/handshake-negative-controls.log`:
  `b22aecbc7c1cc1e5843f46d16cd438dd06f703172ba8878776546d287b9e06a0`
- `logs/linked-consumers.log`:
  `517b055a5a30f15b28040037df643407ea93b53fcd022234ff0d398b9cb72496`
- `measure/run1/receipt.json`:
  `6a9adcd936c43e0ed6ab3beb6878dd560279f6ab3276bbb4745437c55ac1633c`
- `obj/parallel/pa28/backend-v0.9.1-pin/receipt.json`:
  `bc546b345c53a071ecdd66409ce1278e329cbbdaaa1cfb853fd5cfb36108cf73`
