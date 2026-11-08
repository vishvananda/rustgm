# Broad native sweep failures

Lane B batch of 2026-11-08, branch `implement/chain-bytes-20261108` on
`15ba5e670`. Seven PA13 and PA18 native checks had been failing on main since
2026-10-28. Scheduled CI ran only the maintainer native subset, so nothing
reported them. Lane D's old-pin baseline on `347373f7f` found them:
`check_native.py` passed 5021 of 5028 cases.

| Stage | Case | Failure |
| --- | --- | --- |
| PA13 | `floats/f32-0-zeros.rs` | successful frontend stderr requires a declared warning code |
| PA13 | `floats/f64-0-zeros.rs` | same |
| PA13 | `scalars/irrefutable-or-arm.rs` | same |
| PA18 | `constructors/qualified-variants.rs` | same |
| PA18 | `shared-call-reborrow/copy-bound.rs` | successful LowIR emission requires exit 0 and nonempty stdout |
| PA18 | `shared-call-reborrow/generic-referent.rs` | same |
| PA18 | `shared-call-reborrow/generic-value.rs` | same |

None is a compiler regression. In every case the compiler's output equals its
reviewed reference, and the native harness had no way to declare that output.

## 1. Unreachable-pattern warnings

`d77116084` and `88777d097` regenerated these stderr references with
`warning[unreachable_patterns]` lines after an independent rustc review. Those
reviews ran only the text harness. `check_native.py` accepts a successful
case's stderr only when the manifest lists each code in `warnings` and the code
is in `REVIEWED_WARNING_CODES`. Neither was true for these four cases.

### Oracle

The predictions and controls were frozen before rustc ran
(`obj/lane-b5/predictions.json`). `obj/lane-b5/oracle.py` runs the native
checker's exact rustc 1.98.0 program command (default lints, as each of these
cases declares) with `--error-format=json`. A claim passes when every rustc
`unreachable_patterns` primary span starts at one RustGM reference offset, in
the same order and count.

| Case | RustGM offsets | rustc primary span starts | Span text |
| --- | --- | --- | --- |
| `f32-0-zeros` | 169 | 169 | `-0.0` |
| `f64-0-zeros` | 169 | 169 | `-0.0` |
| `irrefutable-or-arm` | 102 | 102 | `_` |
| `qualified-variants` | 222, 295 | 222, 295 | `_`, `_` |

Controls, all as predicted except where noted:

- PA11 `control/and-skips.rs` emits rustc `unused_must_use` and no
  `unreachable_patterns`.
- PA18 `constructors/enum-variants.rs` and PA13
  `floats/f32-0-adjacent-normal.rs` have reachable wildcards and emit no
  `unreachable_patterns`.
- Three frozen opponents emit no `unreachable_patterns`: `f32-0-zeros` without
  its `-0.0` arm, `irrefutable-or-arm` without its trailing wildcard, and
  `qualified-variants` with a second variant `W`.
- Prediction missed: PA13 `floats/f32-0-duplicate-zero-retries.rs`
  (`0.0|0.0 if …`). I predicted rustc would warn on the redundant alternative.
  It emits no `unreachable_patterns`, so it agrees with that case's empty
  RustGM reference.

rustc additionally emits `dead_code` for the unused constants in the float
cases. RustGM has no `dead_code` producer and the native checker makes no
claim about it.

### Repair

`unreachable_patterns` joins `REVIEWED_WARNING_CODES`. Exactly these four
cases declare `"warnings": ["unreachable_patterns"]`. No other successful
LowIR case has a nonempty stderr reference without a declaration.

## 2. Empty LowIR libraries

The three `shared-call-reborrow` libraries hold only private, unused
functions. RustGM emits a 0-byte LowIR unit for each, and the reviewed `.ref`
is 0 bytes. `accepted_frontend` required nonempty stdout, so these cases could
never pass.

### Oracle

| Input | RustGM stdout | rustc object, defined code or data | cppgm++ on that LowIR |
| --- | --- | --- | --- |
| `copy-bound`, `generic-referent`, `generic-value` | 0 bytes | none | ELF64 ET_REL, none |
| `pub fn read` control | nonempty | `read` | — |
| `generic-value` with `pub fn f` | nonempty | `f`, `consume::<&u8>` | — |

rustc 1.98.0 collects mono items lazily for a library, so a private,
unreachable function is never code-generated. Both pinned `cppgm++` binaries,
`0d216449…` (b7c6b8e72) and `614689ec…` (17b024f69, the v3 pin), compile a
0-byte or newline-only LowIR unit to a 456-byte ET_REL object with no defined
code or data symbol (`obj/lane-b5/backend/results.json`).

### Repair

A case may declare `native.lowir = "empty"`, and only a library object may.
The checker now enforces four rules:

- `accepted_frontend` accepts empty stdout exactly when the case declares
  it. A declared case with nonempty stdout fails, and so does an undeclared
  case with empty stdout.
- A declared empty unit still goes through `emit_object`. That proves the
  pinned backend accepts it and produces a relocatable object.
- `empty_objects` reads both objects' ELF symbol tables. It fails if either
  rustc or CPPGM defined a function, object, common, TLS or indirect-function
  symbol. A declaration therefore cannot hide a function that rustc
  generates and RustGM drops.
- The pass line counts declared empty units among the object comparisons.

Tradeoffs:

- The check proves the bridge rather than skipping it. One extra backend
  invocation per declared case costs little, and a skip would leave the empty
  unit unexercised. If a future backend pin rejects empty units, the
  declaration surfaces that as a failure instead of hiding it.
- The ELF reader is about 30 lines of `struct` parsing in `check_native.py`,
  not a call to `nm`. It avoids a new host tool dependency and keeps the
  backend bridge in `backend.py` unchanged, so `make test-backend` is not
  invalidated. It understands only ELF64 little-endian symbol tables, which
  `verify_object` already requires.
- The declaration sits inside `native`, beside the object reason, rather than
  as a top-level case key. The text harness needs no change because it already
  compares stdout bytes exactly.

## 3. Reference receipts

Each edited manifest entry's contract digest changed. Before any receipt
existed, `check_layout.py` failed with "manifest changed since reference
generation: pa13", which is the expected control. The normal selected harness
generated one scoped batch per stage, after a selected comparison without
`--bless` passed 3/3 and 4/4 byte for byte:

- `pa13/reference-batches/1794175013924476258-d80d25d5fc6a.json`, for the
  three PA13 cases.
- `pa18/reference-batches/1794175032616597333-2d3ab5ad5e01.json`, for the four
  PA18 cases.

Only those two receipts are new. `git status` after the blessed run showed no
modified `.ref`, `.ref.stderr` or `.ref.exit_status` sidecar, and no compiler
source changed. The blessing therefore re-records the manifest and case
contract hashes. No output was accepted that the comparison had not already
matched.

## 4. Checks

### First run, before integration

Release binary `obj/rust/release/rustgm` sha256 `a3c9d445…` was built from
`15ba5e670` with no `dev/` changes. Native evidence then recorded only the
`dev/rustgm` launcher's hash, `ad053a19…` (section 6). The backend was the v2
lock, b7c6b8e72. Logs are in `obj/lane-b5/`.

| Check | Result |
| --- | --- |
| `scripts/tests/test_native.py` | 18 tests OK, including 3 new |
| `make test-harness` | 307 tests OK, maintainer suites registered, 44-stage layout |
| `make test-upstream` | 128 pinned upstream fixtures, 126 Reference files |
| `make test-native-pa13` | PASS 532/532 (was 529); 365 executable comparisons; evidence `obj/native/pa13/evidence.json` |
| `make test-native-pa18` | PASS 500/500 (was 496); 307 executable, 3 codegen-only, 3 object comparisons, all 3 declared empty |
| `course.py pa13`, full text | 805/805 |
| `course.py pa18`, full text | 505/505 |
| Real-binary opponents (`obj/lane-b5/frontend/results.json`) | A `pub fn` library and `generic-value` with `pub fn f` emit 237 and 852 bytes of LowIR. Both are rejected under the declaration, by stdout and by both objects' symbols. The three declared cases are rejected without the declaration. |

The empty units' objects are identical across the three cases: rustc
`ffb876a8…` and CPPGM `6e93d358…`.

### Rebased run under v3

On `59ec4ba99`, rebased onto origin/main `2a27e11ca`, the backend was
reconfigured to the v3 lock (cppgm-extended 17b024f69): lowir `65a8ebde`,
lowir2native `aa740c24`, cppgm++ `614689ec`. `make test-backend` passed its
executable, object and assembly checks. The release binary was rebuilt as
sha256 `38e7124d00a16d55…`.

The scheduled step's exact loop then ran locally across all 13 lanes and
passed 5028 of 5028 cases in about 760 s (`obj/lane-b5/sweep.log`). Every
evidence file has scope `complete`, backend 17b024f69, and every case passed.

| Lane | Checks | Executable | Codegen-only | Object | Time | `evidence.json` sha256 |
| --- | --- | --- | --- | --- | --- | --- |
| PA11 | 191 | 180 | 0 | 0 | 121 s | `53505eb5f385f288` |
| PA12 | 113 | 64 | 0 | 0 | 36 s | `152e2e0e46b2e3ec` |
| PA13 | 532 | 365 | 0 | 0 | 64 s | `956a1b71deae6319` |
| PA14 | 12 | 12 | 0 | 0 | 23 s | `7bccf69eeb1416c4` |
| PA16 | 131 | 123 | 0 | 2 | 34 s | `22ce444e9bd5a93e` |
| PA17 | 205 | 135 | 0 | 0 | 36 s | `4c4f6c6c6b8c2b79` |
| PA18 | 500 | 307 | 3 | 3, all declared empty | 59 s | `8e09cd798c50c7e7` |
| PA19 | 731 | 309 | 0 | 0 | 64 s | `edec33bd81ca8cd5` |
| PA20 | 868 | 528 | 0 | 0 | 81 s | `cf8e500ba8616e52` |
| PA21 | 484 | 283 | 0 | 0 | 56 s | `e2f0b0f1a546e959` |
| PA22 | 356 | 120 | 134 | 0 | 48 s | `c4256c17245eff56` |
| PA23 | 415 | 86 | 185 | 0 | 49 s | `877d0493e1df25f2` |
| PA24 | 490 | 326 | 3 | 10 | 89 s | `78123a768f094076` |

The other checks on the same tree also passed:

- `test_native.py`: 18 tests.
- `make test-harness`: 310 tests and the 44-stage layout.
- `make test-upstream`.
- `course.py pa13`: 805/805.
- `course.py pa18`: 505/505.

These lane evidence files predate section 6, so their `candidate_sha256` is
still the launcher's hash. The binary that produced them is the
`38e7124d…` build above.

## 5. Scheduled sweep

I recommended adding the broad native sweep to the scheduled `qualification`
job, and the integrator approved it. That job already configures the qualified
backend for `make test-maintainer-native`, but these seven failures sat on main
for eleven days because only that maintainer subset ran there.

A new step, "Broad native course sweep", follows "Native maintainer
qualification". It runs `check_native.py` for exactly the lanes `make
test-native` lists: PA11–PA14 and PA16–PA24, without PA15. Each lane runs
through `storage.py run`. `make test-native` itself would repeat the maintainer
step.

The step finishes every lane and then fails if any lane failed, so one run
reports all of them. It runs on the schedule and on manual dispatch only. The
push and pull-request `fast` job is unchanged.

Tradeoffs:

- **Time.** The step adds scheduled CI time to catch failures like these
  sooner.
  - The course lanes took 447 s of Lane D's 1102 s local baseline. The rest
    was the maintainer step.
  - The integrator estimates about 20 minutes on a hosted runner.
  - The last green scheduled run, on 2026-11-06, took 35 m 43 s of the job's
    60-minute limit. Even the high estimate fits, with little margin. If it
    nears the limit, `timeout-minutes` needs raising.
- **Skipped on maintainer failure.** The step uses the default `success()`
  condition, so it is skipped when an earlier step fails. That happened on
  2026-11-08, when "Native maintainer qualification" failed.
  - Making the sweep independent of that failure would need the backend
    setup split into its own step, with the sweep gated on setup alone.
  - I left this as it is, since a failed maintainer step already fails the
    run.

## 6. Binding evidence to the compiler

`check_native.py` recorded `candidate_sha256` as the hash of its `--app`. The
default `--app` is `dev/rustgm`, a launcher script that execs
`obj/rust/release/rustgm`. Every lane therefore recorded the launcher's
`ad053a19…`, and no evidence identified the compiler that ran.

`candidate_identity` now resolves the program that actually runs:

- An ELF `--app` is the compiler itself, and `wrapper` is null.
- Otherwise the launcher must have exactly the repository form:
  - `#!/bin/sh`
  - `set -eu`
  - `root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)`
  - `exec "$root/RELATIVE" "$@"`

  Comments may appear anywhere. The checker resolves it as the shell does,
  to the launcher's parent directory's parent joined with `RELATIVE`, and it
  never runs the launcher. `RELATIVE` may not contain `..`.
- Any other script fails closed with a clear message before any case runs.
  So does a target that is missing or not an ELF executable.

The evidence records `candidate` and `candidate_sha256` for the executed
binary, and `wrapper` and `wrapper_sha256` for the launcher. The
end-of-run check compares the whole identity, so a rebuild or a launcher edit
during the run fails it.

Checks:

- `test_native.py`: 21 tests, 3 new.
  - Both repository launchers resolve to `obj/rust/release/rustgm`.
  - A synthetic launcher and binary pair binds both hashes. A rebuilt binary
    changes only `candidate_sha256`.
  - Seven launcher mutations fail: environment target, parent escape,
    absolute target, extra statement, other root, other interpreter and
    dropped arguments. So do non-UTF-8 bytes, a script target and a missing
    target.
- `test_course_units.py`'s unit-identity test gave `check_native` a Python
  script as its candidate, which now correctly fails closed. Since `run_case`
  is mocked there, it now passes an ELF-headed stub, and it asserts that the
  receipt's `candidate_sha256` is the stub's and `wrapper_sha256` is null.
- `make test-harness`: 313 tests OK.
- `make test-native-pa14` under v3 passes 12/12. Its evidence
  (`a9f3a1fb…`) records `candidate_sha256` `38e7124d…`, equal to the release
  binary, and `wrapper_sha256` `ad053a19…`.
- `check_native.py pa14 --app scripts/course.py` fails closed:
  "cannot resolve the compiler this launcher execs without running it".

Other evidence writers, not changed here:

- `check_layouts.py` hashes `--app` the same way, and the Makefile passes
  `dev/rustgm`, so it has the same gap.
- `course.py` results and reference receipts record the command and
  `compiler_source_sha256`, but no binary hash. `check_syntax.py` and
  `check_literals.py` record no candidate hash at all. These are unbound
  rather than misbound.
- These metadata checkers hash `--app` directly: `check_storage_metadata`,
  `check_aggregate_metadata`, `check_macro_metadata`, `check_public_names`,
  `check_source_attributes` and `check_transitive_metadata`. Their
  maintainer tests pass `CARGO_BIN_EXE_rustgm`, a real binary, so today's use
  is bound. They would hash a launcher if one were passed.
- `fastcheck_run.py` and `maintainer_batch.py` hash the Cargo test
  executables they run directly, so they have no gap.
