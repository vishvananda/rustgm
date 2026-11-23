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
   the LowIR of the whole-graph build and the linked compiler.
3. **Strict mode, zero bring-up records.** Both stages are built in strict
   mode. The build of original core, alloc, std, the compiler's dependency
   crates and the compiler itself reports zero bring-up records, zero unproven
   obligations and zero waivers, and emits no trapping stub. An unqualified
   bring-up artifact never satisfies this assignment.

Amended 2026-11-20: each stage is one strict whole-graph build. The compiler
binary is the application, and core, alloc, std, the dependency crates and the
compiler library join the same session from source, each resolved as a crate of
its own. Separately compiled artifacts with exported metadata are not part of
this assignment; they stay with PA29 and PA32–PA36. Until 2026-11-20 this
handout also compared the metadata of each crate.

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

Put the original crate trees under one frozen source directory. Describe them
in a configuration with a `core` object containing its relative `path`,
`disambiguator` and `cfg` list. The ordered `guests` list contains each
crate's `name`, `edition`, relative `path` and optional `cfg` list. An
omitted guest cfg inherits the session cfg. An explicit list, including an
empty list, replaces it. The last guest is the RustGM binary under the
application name `app`.

Use the [ordered configuration](configuration.json) for the Linux x86-64
abort-profile build. Place the pinned Rust 1.98.0 library sources in `library/`,
their locked registry packages in `library/vendor/`, and RustGM's locked registry
packages in `vendor/`. Export `dev/` from the compiler revision under test into
the same source directory. Preserve the original package bytes.

Use a maintainer binary built with the [pinned Rust toolchain](../rust-toolchain.toml)
for stage 0. Keep its bytes and SHA-256 so successive source batches can reuse it.
Stage 1 and stage 2 use the same frozen target inputs.

Freeze the source inventory and tool identities before building either stage.

```sh
python3 scripts/storage.py run --no-prune -- \
    python3 scripts/build_self_hosting.py freeze pa45/configuration.json \
    --sources "$SOURCES" --stage0 "$STAGE0" --linker "$LINKER" \
    --out obj/stage45/graph.json
```

Build the graph with stage 0 and link its single LowIR artifact through the
pinned CPPGM object backend.

```sh
python3 scripts/storage.py run --no-prune -- \
    python3 scripts/build_self_hosting.py build obj/stage45/graph.json \
    --sources "$SOURCES" --compiler "$STAGE0" --compiler-sha256 "$STAGE0_SHA256" \
    --out obj/stage1/target
python3 scripts/storage.py run --no-prune -- \
    python3 scripts/link_self_hosting.py obj/stage1/target --out obj/stage1/native \
    --linker "$LINKER" --linker-sha256 "$LINKER_SHA256" --optimization 1
```

Run all committed PA1–PA24 oracles with the linked compiler. The batch reports
every suite even when one suite fails.

```sh
export RUSTGM_TEST_APP="$PWD/obj/stage1/native/rustgm"
python3 scripts/storage.py run --prune-under obj/tests -- bash -c '
    status=0
    for ordinal in {1..24}; do
        python3 scripts/course.py "pa${ordinal}" --app "$RUSTGM_TEST_APP" || status=1
    done
    exit "$status"
'
```

Build stage 2 with the same graph and source directory. Read the stage 1
compiler's SHA-256 from its native link receipt.

```sh
python3 scripts/storage.py run --no-prune -- \
    python3 scripts/build_self_hosting.py build obj/stage45/graph.json \
    --sources "$SOURCES" --compiler obj/stage1/native/rustgm \
    --compiler-sha256 "$STAGE1_SHA256" --timeout 3600 --out obj/stage2/target
```

Compare the frontend outputs before spending time on the second native link.

```sh
cmp obj/stage1/target/program.lowir obj/stage2/target/program.lowir &&
cmp obj/stage1/target/program.native-links.json obj/stage2/target/program.native-links.json &&
python3 scripts/storage.py run --no-prune -- \
    python3 scripts/link_self_hosting.py obj/stage2/target --out obj/stage2/native \
    --linker "$LINKER" --linker-sha256 "$LINKER_SHA256" --optimization 1
```

Compare the independently built targets and their linked executables.

```sh
python3 scripts/storage.py run --no-prune -- \
    python3 scripts/compare_self_hosting.py obj/stage1/target obj/stage2/target \
    --stage1-binary obj/stage1/native/rustgm --stage2-binary obj/stage2/native/rustgm
```

The comparison verifies the frozen source graph, the compiler chain, the
strict diagnostics and the link receipts, including the CPPGM optimization
level. It compares the LowIR, native link manifest and executable byte for byte.
Target and link receipts record the output hashes, command time and peak
resident memory.

### Focused reproduction

Reuse the pinned stage compilers for diagnosis. Start with a small consumer
and include only the source guests it needs. Compare `--count-types` to check
analysis, then `--count-lowir` to exercise instance demand and typed LowIR
construction without writing the full LowIR text or running CPPGM.

The private test
`lower::instances::proof::origins::ledger_tests::superlinear_instantiation_lowers_until_the_fixed_demand_cap`
uses a small generic call chain and starts the demand ledger near its ceiling.
Run it through `make fastcheck` when changing the whole-graph demand account.
Use the frozen whole-graph build and byte comparison for qualification.

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
