# PA37 fixture ownership

Active self-hosting grading suite (`suite_kind: "self-hosting"`, `self-hosting`
released). [suite.json](suite.json) is driven by `scripts/pa37_self_host.py`
and `pa37/Makefile` (modeled on CPPGM PA35 `inception`):

- **Stage 0 (seed):** compiler binary built by pinned `rustc 1.98.0`.
- **Stage 1 (`stage1` / `rustgm-self`):** strict zero-bringup whole-graph build
  of unmodified Rust 1.98.0 `core`/`alloc`/`std`, locked dependencies, and
  compiler sources, linked through pinned CPPGM v0.12.0.
- **Verification ladder (`test-through-pa24`, targets up to `test-through-pa30`):**
  runs committed earlier assignment oracle suites with Stage 1 via
  `RUSTGM_TEST_APP` (no regenerated or relaxed oracles).
- **Stage 2 (`stage2` / `rustgm-inception`):** Stage 1 rebuilds the exact same
  frozen source graph with the exact same CPPGM and linker pins.
- **Byte comparison (`compare`):** verifies `program.lowir`,
  `program.native-links.json`, and the linked `rustgm` executable match byte
  for byte between Stage 1 and Stage 2 (`MATCH rustgm`).
- **Negative controls (`negative-controls`):** verifies perturbed Stage 2 LowIR
  and nonzero bring-up records are rejected by strict comparison and receipt
  verification.

See [pa37-self-hosting-release.md](../../docs/pa37-self-hosting-release.md) and
[pa37/MAINTAINER.md](../MAINTAINER.md).
