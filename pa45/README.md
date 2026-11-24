# RustGM Programming Assignment 45 — Reproducible self-hosting

**Self-hosting demonstrated.** The strict whole-graph build passed the
PA1–PA24 suites and reproduced the executable with zero bring-up records
using CPPGM v0.9.3. The student grading suite is pending.

Build your completed RustGM compiler with RustGM, then repeat the build
using that compiler. Use the Rust 1.98.0 Linux x86-64 panic-abort profile.
Complete this milestone after [PA36's library bootstrap](../pa36/README.md).
Use the original `core`, `alloc`, and `std` sources and the compiler's
actual target dependencies. The selected graph uses the panic-abort
profile. This completes the initial track. The remaining procedural
macro, unwind, edition, target, and async work follows in PA37–PA43.
PA44 integrates the broader frontend profile.

## Required result

| Stage | Build |
| --- | --- |
| 0 | Build your completed RustGM once with the pinned Rust toolchain. |
| 1 | Use stage 0 to compile the entire source graph and link the resulting LowIR with CPPGM. |
| 2 | Use stage 1 to compile the same graph, then link it with the same backend and linker settings. |

Each self-hosted stage is one strict whole-graph build. Keep each crate's
identity, edition, configuration, and dependencies while passing them
through the same compilation session. Use CPPGM for LowIR-to-native
generation. RustGM implements each Rust frontend phase.

Freeze the source inventory, configuration, compiler and linker identities,
and backend lock before the builds. Reuse the same source bytes, graph
order, options, and backend optimization level in both stages. Use the
original library sources and the compiler's real dependencies. Prevent
timestamps, machine paths, or unordered traversal from changing artifacts.

## Tool surface

Record each stage's LowIR, native link manifest, and linked executable.
Report the strict checks and artifact hashes. The checked graph must have
zero bring-up records, unresolved obligations, and waivers, and must
emit no trapping stubs.

## Check your implementation

Use the stage 1 executable as the compiler under test for every PA1–PA24
suite. For example, the commands below use a compiler linked at `obj/stage1/native/rustgm`.

```sh
stage1="$PWD/obj/stage1/native/rustgm"
result=0
for number in $(seq 1 24); do
    make -C "pa$number" test RUSTGM_TEST_APP="$stage1" || result=1
done
test "$result" -eq 0
```

Compare stage 1 and stage 2 LowIR and native link manifests byte for byte
before the second native link. Compare the final linked executables byte
for byte as well. Keep evidence of source and tool identities with both
outputs. Check that a differing input or a nonzero bring-up record causes
the validation to fail. Record build time and peak memory for each stage.
