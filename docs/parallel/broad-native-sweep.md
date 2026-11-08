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

Release binary `obj/rust/release/rustgm` sha256 `a3c9d445…` was built from
`15ba5e670` with no `dev/` changes. Native evidence records the `dev/rustgm`
wrapper's hash, `ad053a19…`. The backend is the current lock, b7c6b8e72.
Logs are in `obj/lane-b5/`.

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
