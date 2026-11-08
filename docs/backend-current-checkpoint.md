# CPPGM backend v3 checkpoint (17b024f69)

All three tools now pin cppgm-extended
`17b024f692a5b537a4db6c4d3b6dc33bc930c680`, replacing the v2 pins at
`b7c6b8e72` ([v2 checkpoint](backend-v2-checkpoint.md)). The user lifted the
backend freeze on 2026-11-08 for inline assembly only. The new pin adds the
optional LowIR `assembly` extension. RustGM does not yet lower Rust `asm!` to
it; that is a separate PA41 slice.

## Determinism control

The tools were rebuilt in separate clones outside the active C++ checkout,
with `make -C dev lowir lowir2native cppgm++` and its default configuration:
g++ 15.2.0 (Ubuntu 15.2.0-16ubuntu1), `-std=gnu++11 -Wall -O3
-fno-strict-aliasing` and the `-Dmain=test_runner_real_main` test-runner
variant. Those flags are identical to the v2 stamps.

- Control: the same procedure on `b7c6b8e72` reproduces all three v2 pins byte
  for byte (`32b1e67c…`, `27b3138e…`, `0d216449…`). The procedure therefore
  rebuilds a known pin exactly.
- Candidate: two clones of `17b024f69` at different absolute paths produce
  byte-identical tools.

| Tool | Bytes | SHA-256 |
|---|---|---|
| `lowir` | 422,456 | `65a8ebde18ead6ee6498024de93f246aae21c8292fecd8e1e34c25b34271b9ba` |
| `lowir2native` | 2,295,904 | `aa740c24e8eb5749176ad32f36669bdcef17ccb925947d5c8bbe110f049beb4c` |
| `cppgm++` | 8,208,808 | `614689ec481576a83227076181eb165c038347cc4473a160d483f3654c8192c3` |

## Upstream CI at pin time

`17b024f69` was the newest cppgm-extended main commit with green Tests
(run 42516011691) and Upstream C++11 correctness (run 42516011688) when it
was chosen. Some runs were still in progress at pin time and finished later:

- Succeeded: Boost advisory 42516011690, simdjson 42516011731, Godot
  42516011709, index-cpp 42516011682, Inception 42516830851 and Export
  Assignments 42516830899.
- Intel macOS port: the push run 42516011674 was cancelled. A
  workflow_dispatch run on the same commit, 42509140066, succeeded.
- Public benchmarks: push run 42516011716 was cancelled by the next push,
  `6a6a15f44`, whose run 42517561890 succeeded. That successor changes only
  benchmark workflows, scripts and READMEs, not tool sources.

## Distribution

`backend.lock.json` binds the new commit and tool bytes. Its object profile
records the capabilities `lowir-to-object` and `assembly`.
`backend.distribution.json` binds the deterministic `backend-pins-v3` archive.
The archive holds the three tools, their manifest, and the inherited NOTICE
with the Apache license appended. The NOTICE is built the same way as v2's.
The v1 and v2 releases, locks and pins stay unchanged for historical
reproduction.

## LowIR contract changes

There are 595 commits from `b7c6b8e72` to `17b024f69`. The changes to the
reviewed [LowIR contract](https://github.com/vishvananda/cppgm-extended/blob/17b024f692a5b537a4db6c4d3b6dc33bc930c680/pa8/lowir.md)
add forms. Its four removed lines are replaced by extended versions: one
sentence about bodyless declarations and three vector-operation rows, each
keeping every operation it listed:

- Address-only `declare function @name`. A direct call of it needs an explicit
  `as (...)` boundary.
- An `xstate_query` function role (XGETBV). The caller keeps the CPU and OS
  support checks.
- A clarification of slot reads, slot storage operands, call arguments and
  `phi` incoming values. RustGM writes no `phi`. Its slot uses validate
  unchanged: every canonical LowIR in the native comparison below is
  byte-identical.
- Optional `unary popcount`.
- New vector operations: `phminposuw`, `pclmulqdq`, `dpps`, `dppd`,
  `insertps`, `roundss`, `roundsd`, `roundps` and `roundpd`. The caller must
  ensure CPU support.
- `read_mxcsr` and `write_mxcsr`.
- `stack_save` and `stack_restore`.
- Advisory `branch likely` and `branch unlikely`.
- The optional [assembly boundary](https://github.com/vishvananda/cppgm-extended/blob/17b024f692a5b537a4db6c4d3b6dc33bc930c680/pa36/assembly.md):
  - `assembly` templates and `asm` invocations
  - `assembly_function`, `assembly_global` and template `environment`
  - `code_label` and `indirect_jump`

RustGM emits none of the new forms yet. Each one needs a real typed producer
proof before use, as the v2 vector vocabulary did.

## Decisions

- Assembly is an optional capability of the reviewed object profile.
  `backend.assembly_profile` accepts it only when the profile names the same
  commit as the executable tools. A lock cannot advertise assembly for a
  different object compiler.
- The assembly handshake lives in `tests/backend`, beside the executable and
  object handshakes, and `make test-backend` now runs all three. It checks:
  - an object `movntps` (`0f 2b`) in both a by-value and a stored-operand
    function
  - a C observer with full bits, 16-byte alignment and sentinels on both sides
  - a standalone executable that must exit 42
- `lowir2native` writes executables without section headers. The handshake
  therefore disassembles the executable `PT_LOAD` segments, and `objdump`
  becomes a requirement for the assembly handshake only.
- Rejected: pinning the active checkout's `dev/` binaries. They were built
  before the assembly commits.
- Rejected: a frontend-only change. The v2 tools reject `assembly` items
  ("expected top-level LowIR item").

The `movntps` evidence is LowIR-level only. It shows that the pinned tools
accept the extension and emit a real non-temporal store. It is not evidence
that RustGM supports `asm!`. That support comes with the PA41 slice, which
must lower the checked asm body nodes itself.

## Qualification

The old and new pins ran the same broad native pass on main `347373f7f`, with
the same release frontend and test binaries. Their hashes and the source
fingerprint were recorded before and after each pass, and nothing changed.
Each pass ran:

- the executable, object and assembly handshakes
- the release maintainer native suite (`--ignored`)
- `check_native.py` for every PA set that `make test-native` runs

Native output equality with the old pin is the regression criterion. It holds:

- All 5028 independent native cases have the same result and error. All 2834
  native runs have byte-identical stdout and stderr. All 12 object cases have
  identical object bytes, and every validated LowIR is byte-identical.
- The old pin's 7 failures repeat with identical messages:
  - 3 in PA13 and 1 in PA18: an undeclared `unreachable_patterns` warning
  - 3 in PA18 `shared-call-reborrow`: empty library LowIR
  These predate this change and are harness and fixture gaps on main, routed
  to Lane B. CI runs only the maintainer native suite, so it never saw them.
- 133 independent rustc panic messages differ only in the OS thread id that
  rustc's runtime prints. The comparison normalizes only that field.

On the new pin, three maintainer tests stopped at their frozen backend
bindings before running anything:

- the PA28 atomic capability contract
  (`pa28/tests/backend/atomics/contract.json`)
- the PA31 source-attribute CLI contract
  (`pa31/tests/spec/source-attribute-transport/cli-contract.json`)

Both refuse any unreviewed lock. As with v2, each now binds the v3 lock and
the CPPGM source hashes it reviews, and keeps the v2 and v1 bindings as
history. No declared case, outcome, encoding count or diagnostic changed.

Of the seven reviewed CPPGM sources, six changed between pins, with 39
removed lines. The changes are additions plus behaviour-preserving
generalizations:

- `validator.cpp`: the fixed-role signature check now takes a parameter count,
  so `xstate_query` takes one parameter and returns `i64`, and the other roles
  keep three parameters returning `void`. The function and symbol checks are
  restructured, and the "invalid function signature" and entry-role checks
  remain.
- `intrinsic_lowering.cpp`: cppgm++'s C `prefetch` now lowers non-constant
  immediate controls as source-immediate guard calls instead of discarding
  them. This changes behaviour only in cppgm++'s own C frontend. RustGM does
  not use it.
- `lowir_model.h`: string tables are re-flowed around new entries, so `sqrt`
  is no longer the last unary operator and `equivalent_entry` is no longer
  the last keyword.
- `instruction.h`: one `Branch` comment line now mentions the advisory
  prediction.
- `text_parser_body.cpp` and `pa8/lowir.md`: additions for the new forms.

`emit_memory.cpp` is unchanged. The atomic vocabulary is unchanged by name and
order, in every reviewed source:

- the seven atomic opcode strings, `atomic_load` through `atomic_signal_fence`
- the `MemoryOrder` enumeration and `kMemoryOrderCount`
- the eight atomic operand-shape rows in `validator.cpp`
- every atomic line in the parser

With the new bindings, all three tests pass on the same release binaries.
Every other maintainer native test passed in the full pass.

A clean root installed the local bundle with `setup_backend.py --bundle`.
Each pinned tool matched the lock, and the executable, object and assembly
handshakes passed there.

The measurement used immutable old and new tools on frozen inputs. Each input
ran in six exclusive blocks of A A A B B A, with repeats so that each
observation lasts roughly 0.04 to 0.2 s. Every output was byte-identical between
the pins, and every observer and native behaviour matched:

| Input | Kind | A/A range | Paired B/A median |
|---|---|---|---|
| `deref` (2,573 B) | object + observer | 0.927–1.069 | 1.012 |
| `receivers-128` (182,027 B) | object + observer | 1.000–1.276 | 1.036 |
| `f64-saturation` (20,999 B) | program | 0.936–1.033 | 1.019 |
| `text-bytes` (14,250 B) | program | 0.947–1.089 | 0.983 |
| `alias-callable` (1,312 B, control) | program | 0.934–1.018 | 1.017 |

The paired medians fall inside the A/A ranges. They show neither a speedup nor
a reliable regression, and this is not a qualification of generated-code
quality.

Retained evidence:

- `obj/parallel/backend-asm-pin/build.json`: the rebuild, the reproduction
  control and the build stamps
- `obj/parallel/backend-asm-pin/old/receipt.json` (1102 s):
  `7b2632dfd1941cbd96b90c7ada29ec52da42fa316bfad05966b4901bcc44a344`
- `obj/parallel/backend-asm-pin/new/receipt.json` (1118 s, before the
  rebinding): `0ea2c2264724c69b8922e6d1a982e5766009fe22c8edc7c542b846a1186ce0fb`
- `obj/parallel/backend-asm-pin/native-equality.json`:
  `4c19ea795bafc5326ff51cb02b13430741510be9c0f1759dc5a663e18e83602d`
- `obj/parallel/backend-asm-pin/rebind/focused.log`:
  `8f9ea7874868d6baf22b711afa0a27a791e44f9d4f279ae66d2f2119687fc7e3`
- `obj/parallel/backend-asm-pin/clean-install.log`:
  `29d720a44b437415f03c052fd902ca58c2527fc62c5d1797995cf57b00b2004d`
- `obj/parallel/backend-asm-pin/measurement/receipt.json`:
  `88de1745857075c8845fafc037d5ffd9435711dcb28da7e5832ea0c78df40101`

`make test-backend`, `make test-harness` (307 tests) and `make test-upstream`
all pass. The v3 archive SHA-256 is
`581be6bf83bf625e1a191c189e872bbeb735b88cf843c2aaeb7f85a040866aa9`.
