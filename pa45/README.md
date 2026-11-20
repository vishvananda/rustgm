# RustGM Programming Assignment 45: Reproducible self-hosting

Status: **planned**. This is a buildout brief, not a released grading contract.
Work prerequisites: [ordered dependency units](../docs/assignment-order.md). The source profile is Rust 1.98.0, edition 2024 and
Linux x86-64, with aborting panics.
See the [self-hosting plan](../docs/self-hosting-plan.md) for the milestone
ladder and the [library bootstrap plan](../docs/library-bootstrap-roadmap.md)
for source pins.

This is the required capstone. It was an optional extension until 2026-11-15.

## Required implementation

RustGM compiles RustGM. The stage 0 compiler is the maintainer build made by
pinned rustc. Stage 1 is the RustGM CLI and library compiled by stage 0 through
CPPGM. Stage 2 is the same source compiled by stage 1. The contract has three
parts, and all three are required:

1. **Stage 1 is a working compiler.** Stage 1 passes every committed PA1–PA24
   text oracle when it is the compiler under test, selected through
   `RUSTGM_TEST_APP`. No oracle is regenerated or relaxed for this.
2. **Stage 2 reproduces stage 1.** For the same inputs and configuration, every
   output of stage 2 is byte-identical to the corresponding output of stage 1:
   the LowIR and metadata of each crate in the graph, and the linked compiler.
3. **Strict mode, zero bring-up records.** Both stages are built in strict
   mode. The build of original core, alloc, std, the compiler's dependency
   crates and the compiler itself reports zero bring-up records and emits no
   trapping stub. An unqualified bring-up artifact never satisfies this
   assignment.

The input graph is the real one: original pinned core, alloc and std in the
abort profile, the compiler's actual dependency crates (unicode-ident, tinyvec
and unicode-normalization) and the unmodified `dev/` source. Self-hosting needs
atomics beyond U32, SIMD lowering for hashbrown, allocator, start and TLS shims
and a backend that handles the LowIR volume. It does not need procedural
macros, unwinding or async, so those assignments are not prerequisites.

CPPGM stays the external LowIR-to-native backend at every stage. No stage may
call rustc, CPPGM or another frontend to lex, parse, check or lower Rust. The
only rustc-built compiler in the chain is stage 0.

The maintainer implements compiler responsibilities in `dev/src/` and introduces
typed interfaces before text views. Follow [spec.md](../spec.md), the pinned Rust
Reference and CPPGM's efficient representation principles. Parse once, retain
completed facts and avoid speculative or repeated whole-graph work. The compiler
implementation remains idiomatic safe Rust; strict Clippy all/pedantic and the
syntax-aware file/architecture audit are required.

## Tool surface

Use the existing `dev/rustgm --emit-lowir` view and explicit crate-build
interfaces; this assignment adds no language view. Freeze the stage build
procedure before activation: the ordered crate graph, every source,
configuration and backend-tool identity, and the summary that records each
stage's output hashes. The procedure must be deterministic: no timestamps,
absolute paths, host randomness or unordered iteration may reach an output.

`RUSTGM_TEST_APP` already selects the compiler under test for the course
runner, so stage 1 needs no new test entry point.

Link the binary target and its recorded dependency closure after each strict
target build. Use the binary unit's index from the frozen plan and the host
linker from its tool inventory.

```sh
python3 scripts/link_self_hosting.py obj/stage1/target \
    --unit "$BINARY_UNIT" --out obj/stage1/native \
    --linker "$LINKER" --linker-sha256 "$LINKER_SHA256"
```

The output directory contains the linked `rustgm`, its command log, the native
objects and their hashes. Use the two target-unit output directories and linked
executables for the stage comparison.

```sh
python3 scripts/compare_self_hosting.py obj/stage1/target obj/stage2/target \
    --stage1-binary obj/stage1/native/rustgm --stage2-binary obj/stage2/native/rustgm
```

The comparison checks the frozen graph, strict receipts, compiler diagnostics
and stage 2 compiler pin. It reads each unit's LowIR and metadata and then the
linked compiler byte for byte. Each binary crate needs its own metadata
artifact for this check.

## Tests to develop with the reference

- Stage 1 against the PA1–PA24 oracles.
- Stage 1 against stage 2, byte for byte.
- Strict build with zero bring-up records.
- Reproducible inputs and deterministic output.
- Bounded build resources.

Declare expected outcomes before any stage runs: the oracle suites stage 1 must
pass, the set of outputs compared between stages and the bring-up count of
zero. Keep a negative control for each part: a deliberately perturbed stage 1
input must fail the comparison, and a build with one bring-up record must fail
the strict gate. Keep original upstream bytes, attribution and checksums. Never
hand-edit generated references.

## Completion and review

Publish the stage build procedure, a nonempty `tests/manifest.json` and the
recorded stage hashes together after independent review. Until then, this
assignment must fail explicitly and stays outside pass totals. A reduced
example, a bring-up build or an empty suite does not activate it.

At publication, run the required coherent checkpoint checks from
[TESTING_AND_REFERENCES.md](../TESTING_AND_REFERENCES.md), including prior active
suites, strict Clippy, file/architecture audits, harness checks and the build
time and memory measurements for both stages. Use the resource/storage guards
and scoped cleanup; stage builds are heavy jobs.
