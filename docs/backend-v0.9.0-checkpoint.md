# CPPGM backend v0.9.0 checkpoint (bb0980105)

The [v0.9.1 checkpoint](backend-v0.9.1-checkpoint.md) superseded this pin on
2026-11-15. This record of the v0.9.0 qualification is otherwise unchanged.

All three tools now pin release `v0.9.0` of
[cppgm](https://github.com/vishvananda/cppgm), commit
`bb09801053e48486beebe014071a71353ce9be0b`, replacing the v3 pins at
`17b024f69` ([v3 checkpoint](backend-v3-checkpoint.md)). The user asked for
this update on 2026-11-15. Two upstream names changed: the repository was
`cppgm-extended`, and the compiler driver was `cppgm++` and is now `cppgm`.
The release also ships the LowIR backend as a library, which the lock records
as a new optional capability.

## Distribution

Earlier pins were rebuilt here and published as a RustGM archive. Upstream now
publishes versioned release packages, so the lock binds the package's own
bytes and `backend.distribution.json` names the upstream asset:

- Asset: `cppgm-v0.9.0-linux-x86_64.tar.gz`, 12,775,959 bytes, SHA-256
  `8448da3fbb55c13e5420f9e177261664b6d97df39090bc8348fd469786218a68`. Its one
  release directory is `cppgm-v0.9.0-linux-x86_64`, recorded as `prefix`.
- The published `SHA256SUMS` states the same archive digest. The package's
  provenance record (`cppgm-v0.9.0-linux-x86_64-provenance.json`, `8cb5cc9d…`)
  states the same digest, source `bb0980105`, host compiler g++ 13.3.0 on
  Ubuntu 24.04, and a hash for every file. All eleven locked hashes equal it.
- `scripts/setup_backend.py` checks the archive digest and then requires every
  entry to be a plain file inside the release directory. It extracts only the
  locked files, checks each against the lock, and publishes pins and
  configuration only after all of them match. The package's other six tools,
  its licence, notice and README are not installed.

| File | Bytes | SHA-256 |
|---|---|---|
| `bin/lowir` | 497,600 | `a0dac6661a497ed3797dc4710d3514f93f0e76fd1ab145abbfbbf4eb6378c95c` |
| `bin/lowir2native` | 5,024,560 | `a76101c81ad67c62fc12cc5738f4f43a2bd500ac139474bf52263a46c908a41b` |
| `bin/cppgm` | 9,802,656 | `a6aa218c3b9f4eb902017fa57ee6c743f86a64f5ea5fe73210857ca330fa3acb` |
| `lib/libcppgm_lowir.a` | 6,841,112 | `92b148da4763cf66296331025254f858bb185547f065ec15478e2860a662d880` |
| `lib/libcppgm_lowir.so` | 3,878,472 | `71f2cb0c9bf3ecf7fe78a39757acb02d1c000c8088611d07748a31220fcdb3bd` |
| `include/lowir_interface.h` | 5,449 | `48e09900667506f9e02885e22ff1a6a40dd1c8f7d2c6155be1b16e27b432dd3b` |
| `include/lowir_records.h` | 5,263 | `60f1d822476bd3b9bc38d9441ddb943e3f5bef2e97ab0891a5280154f0dc1910` |
| `include/lowir_c_vocabulary.h` | 9,589 | `ffcced61c048682c5a993470b97a1646a43cdb09cbf22c1682ec970eeb2d0294` |
| `include/lowir_model.h` | 29,423 | `847a06baa6d92ca178f07698743e1b0a4f4080c30355640fdc3d3654193887bd` |
| `include/lowir_local_model.h` | 7,012 | `c6cbe9d4f485cf4fe7c567d07fda7c84d51dbde9581aad07b17218bf022743ae` |
| `include/ir_symbol_model.h` | 20,468 | `feda1b6df64e2263f327889b4684074fd91e46a36b78bccda48d5f73cb7ba93b` |

The release bytes are upstream's, not a local rebuild. A local build of the
tag with g++ 15.2.0 produces different bytes, as a different host compiler
should, so `backend.py --configure CHECKOUT` no longer matches an ordinary
local build. Install with `python3 scripts/setup_backend.py`, or with
`--bundle PATH` for a package that is already downloaded. The v1, v2 and v3
releases, locks and pins stay unchanged for historical reproduction.

## Upstream CI at pin time

- The Release workflow for the tag, run 43701650820, succeeded in all four
  jobs: Linux x86-64 tools, library and source; Intel macOS; wasm; publish.
- On the tagged commit's push to main, Tests (run 43701437981), Upstream
  C++11 correctness (43701437985), Clang build (43701438200), Intel macOS
  port (43701438003), the Wasm32, simdjson, Godot, index-cpp, self-build,
  internal and public benchmark runs, Inception (43703494100) and Export
  Assignments (43703494105) succeeded. The Boost benchmark run 43701438216
  was still in progress when this was written.
- The same commit passed the pull-request runs before the merge, including
  PA36 release qualification (43698118118).

## LowIR contract changes

There are 563 commits from `17b024f69` to the tag. The changes to the reviewed
[LowIR contract](https://github.com/vishvananda/cppgm/blob/bb09801053e48486beebe014071a71353ce9be0b/pa8/lowir.md)
add forms. Its 68 replaced lines renumber upstream assignments, rename the
driver, or extend a rule with the additions below:

- Target profiles: an optional leading `target` line (for example
  `target wasm32 1`) sets pointer storage, alignment and index scaling. A file
  without one keeps the x86-64 profile. Wasm profiles add `import` and
  `export` records, `memory_size` and `memory_grow`.
- Function metadata `cold=yes`, and alias declarations that give an alias its
  own binding and typed interface.
- Boundary facts: `repeatable=yes`, `noalias_result=yes`, `return=nonnull`,
  `this_return=yes`, `alias=exclusive_result`, and the aggregate layout facts
  `shape=` and `result_shape=`.
- Call facts `argument_type=` and `argument_shape=` for unnamed variadic
  arguments.
- Instruction facts `index inbounds` and `copy nonnull`.
- `select` on `f32` and `f64`, and `frame_address`.
- Two storage rules are now stated. An `i1` store writes a byte holding 0 or
  1, and a read of a slot that nothing has written yields an unspecified
  value. RustGM lowers `bool` to `i1` and never stores another value in one.
- The assembly forms are now a required part of the upstream contract
  ([the assembly boundary](https://github.com/vishvananda/cppgm/blob/bb09801053e48486beebe014071a71353ce9be0b/pa29/assembly.md))
  instead of an optional extension. Their LowIR spelling is unchanged, and
  the lock still records `assembly` as a capability of the object profile.

There is no new arithmetic or bit operation. RustGM emits none of the new
forms. Each one needs a real typed producer proof before use, as before.

## The LowIR library

The release ships `libcppgm_lowir` (static and shared) with a C interface,
`lowir_interface.h`, at ABI version 2. A producer interns symbols and
signatures, writes instruction, operand, literal and slot records directly
into buffers that the library owns, and receives a host-linkable x86-64
object in the same process. No LowIR text is written or parsed. Upstream's
instructions are
[Embedding the LowIR backend](https://github.com/vishvananda/cppgm/blob/bb09801053e48486beebe014071a71353ce9be0b/docs/lowir-library.md).

What this checkpoint does with it:

- The lock's optional `library` profile records ABI version 2, the commit and
  the hash of both library forms and the six headers.
  `backend.library_profile` accepts it only for the locked commit and target.
- The installer pins the library and its headers together in one folder under
  `obj/backend/pins/` and records it as `library` in `backend.local.json`.
- A fourth handshake, `backend.py --test-library`, is part of
  `make test-backend`. An original C producer (`tests/backend/library.c`)
  refuses any other ABI version, builds a read-only global and a function
  that loads it, compiles them at `-O0` as position-independent code and
  writes the object. The handshake checks the ELF header and links the object
  with a C observer that must read the global and get the right sums.

What it does not do: RustGM still reaches the backend through LowIR text and
the subprocess bridge. An in-process bridge is separate work with two open
points.

- Calling the library from Rust is FFI. The crate forbids `unsafe` and the
  audit rejects FFI, and the repository guide keeps the backend bridge outside
  the frontend library. A bridge therefore needs its own reviewed boundary,
  which is a rule change for the user to decide.
- ABI version 2 states its own limits: no assembly templates, no debug
  information, no Wasm objects and no complete aggregate ABI metadata. A
  signature carries a result type, parameter types and a variadic flag only.
  RustGM's writer also states `classes=` and `result_classes=` for object
  parameters and results, explicit `as (...)` call boundaries and
  `return=noreturn`, so each emitted form needs a fit check first.

## Decisions

- The distribution is upstream's release package. Rejected: repackaging the
  three tools into a RustGM archive as v1 to v3 did. That would leave two
  archives to keep equal and would drop upstream's provenance record.
- The installer selects files by their full path inside the release
  directory. An entry outside it, a link, a duplicate or a missing `prefix`
  fails before anything is published.
- The library is an optional capability beside the object profile. A lock
  cannot advertise a library for another commit, target or ABI version.
- The driver's key in `backend.local.json` is now `cppgm`. A worktree that
  still holds the v3 configuration must run `scripts/setup_backend.py` again.

## Qualification

Everything below ran on main `2ef44dfe3` plus this change, with the release
package installed by `setup_backend.py`.

- `make test-backend`: the executable, object, assembly and library
  handshakes pass.
- `make test-maintainer-native`: all 523 release maintainer native tests
  (`--ignored`) pass. Its released native units pass too, with 2,059
  independent checks and 729 executable comparisons.
- The broad native course sweep, `check_native.py` for PA11 to PA14 and PA16
  to PA24 with the release compiler: all 13 stages pass, with 5,551
  independent checks, 3,106 executable comparisons, 358 codegen-only
  comparisons and 105 object comparisons. The seven failures that the v3
  checkpoint recorded in PA13 and PA18 no longer occur.
- A fresh root installed the published package from its URL. Every pinned
  file matched the lock and all four handshakes passed there.
- `make test-harness` passes.

Two contracts refuse any unreviewed lock, as before. Each now binds the
v0.9.0 lock and keeps the earlier bindings as history:

- the PA28 atomic capability contract
  (`pa28/tests/backend/atomics/contract.json`), with the seven reviewed CPPGM
  sources at the new commit
- the PA31 source-attribute CLI contract
  (`pa31/tests/spec/source-attribute-transport/cli-contract.json`)

No declared case, outcome, encoding count or diagnostic changed. The PA28
backend contract passes on the new pin with all 65 commands.

All seven reviewed CPPGM sources changed between the pins:

- `lowir_model.h`: adds `frame_address`, `memory_size`, `memory_grow`, the
  `index` bounds property, the `nonnull` instruction fact and pointer-width
  types. `frame_address` sits in the middle of the opcode table, so opcode
  numbers moved. RustGM writes text and never depends on them; a library
  client must use the constants of the pinned headers.
- `instruction.h`: the instruction, value and slot records moved to the
  shared `lowir_records.h`.
- `text_parser_body.cpp`, `validator.cpp` and `pa8/lowir.md`: additions for
  the new forms.
- `emit_memory.cpp`: the atomic changes apply at optimization level 1 and
  above, or to fused and tested flag forms. RustGM compiles at `-O0`.
- `intrinsic_lowering.cpp`: changes to cppgm's own C++ frontend only.

The atomic vocabulary is unchanged by name and order: the seven atomic opcode
strings and the `MemoryOrder` enumeration.

The measurement used immutable old and new tools on frozen inputs. Each input
ran in six exclusive blocks of A A A B B A, with repeats so that each
observation lasts roughly 0.04 to 0.2 s. The new backend generates different
code, so output bytes differ between the pins; every observer and native
behaviour matched.

| Input | Kind | A/A range | Paired B/A median |
|---|---|---|---|
| `deref` (2,573 B) | object + observer | 0.867–1.066 | 1.013 |
| `receivers-128` (182,027 B) | object + observer | 0.880–1.067 | 1.017 |
| `f64-saturation` (20,999 B) | program | 0.945–1.105 | 1.022 |
| `text-bytes` (14,250 B) | program | 0.849–1.247 | 1.035 |
| `alias-callable` (1,312 B, control) | program | 0.886–1.085 | 1.026 |
| `core-atomics` (1,377,713 B) | object | 0.823–1.246 | 1.040 |

`core-atomics` is a bring-up compilation of real core with one consumer; one
object build of it takes about 0.06 s. All six paired medians are above 1
and all fall inside their A/A ranges. They show neither a speedup nor a
reliable regression, and this is not a qualification of generated-code
quality.

Retained evidence, under `obj/backend-pin-v4/` unless a path says otherwise:

- `published/`: the downloaded asset, `SHA256SUMS` and provenance record
- `native-qualification.log` (release build and course sweep):
  `bb70d475905c12dba3753dcc9242ce187281626c44ca8c9b6dfd0e8f5c7ebf27`
- `maintainer-native.log`:
  `12304713ec95b679b06f5c1547145beddc5a09c62c3f3811419319651e4f0a88`
- `clean-install.log`:
  `4c3c2b6c720f266416734e5ea3fdb88f65dc59b7088ec70fb6a5b585652cd9a2`
- `measure/run1/receipt.json`:
  `5ab3f5efc8d9fbe1eed3bc8ca0cf477163a8af88ee69407332c9522fb8245c85`
- `obj/parallel/pa28/backend-v0.9.0-pin/receipt.json`:
  `ccadccfc5889fb8796977ecb30bf6e5ef2404e8fcdd8dc46f84b34e22fcaca85`
