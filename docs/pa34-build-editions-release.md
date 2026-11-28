# PA34 build-editions component release

This releases the finite **build-editions** work unit and closes PA34 at its
finite contract, deferring `host-graph-closure` to PA45.

## Released contract

The canonical manifest is [build-editions.json](../pa34/tests/units/build-editions.json):
48 declared outcomes (35 accept, 13 reject), all `--emit-lowir` with file input,
under Rust 1.98.0 across editions 2018, 2021, and 2024 on x86-64 Linux with
checked overflow and aborting panics.

| Family (`tests/grading/`) | Cases | Accept | Reject | Executed | Object |
| --- | --- | --- | --- | --- | --- |
| `foreign-editions` | 14 | 10 | 4 | 10 | 0 |
| `temporary-editions` | 10 | 8 | 2 | 8 | 0 |
| `never-editions` | 8 | 6 | 2 | 6 | 0 |
| `opaque-editions` | 6 | 5 | 1 | 5 | 0 |
| `pattern-editions` | 6 | 3 | 3 | 3 | 0 |
| `or-patterns` | 3 | 2 | 1 | 0 | 2 |
| `mixed-crates` | 1 | 1 | 0 | 1 | 0 |

In addition, [pa34/tests/build-driver/](../pa34/tests/build-driver/suite.json)
is executed by [grade_pa34_driver.py](../scripts/grade_pa34_driver.py): it
builds and runs a host `build.rs` script, verifies generated `OUT_DIR` source
and `cargo::rustc-cfg`/`cargo::rustc-env` propagation into a Rust 2021 library,
compiles a Rust 2024 consumer against a renamed `--extern` binding, links and
executes the resulting native binary through CPPGM v0.12.0, and checks
byte-identical replay across two independent builds.

## Deferred work

`host-graph-closure` remains unpublished and is carried by PA45 under PA34's
finite contract in [assignments.json](../assignments.json).

## Qualification

Candidate: release CLI `obj/rust/release/rustgm`, SHA-256
`8ee7c0197c4b1d8a23e7913ddf2917934645748f91950adc7e9323c3a43c6ebc`, with pinned
CPPGM v0.12.0 ([backend.lock.json](../backend.lock.json)). Qualified on
2026-11-28:

| Check | Result |
| --- | --- |
| `assignment_metadata.py --unit build-editions` | 48 cases valid |
| `check_syntax.py pa34 --unit build-editions` | 48 independent rustc checks agree |
| `course.py pa34 --unit build-editions --bless` | 48/48 declared outcomes, references written |
| `check_layout.py --unit build-editions` | complete oracle provenance |
| `course.py pa34 --unit build-editions` | 48/48 against committed text |
| `check_native.py pa34 --unit build-editions` | 48 checks; 33 executable comparisons, 2 object comparisons, 1 multi-edition provider crate graph |
| `grade_pa34_driver.py` | PASS (host `build.rs` + generated source + mixed 2021/2024 + replay) |
