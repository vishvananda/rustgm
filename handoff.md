# RustGM handoff — 2026-11-13

The active goal is to compile the pinned original core, then alloc/std, completing
and releasing assignment components as their contracts qualify. Continue as one
worker. **No original library artifact exists yet.** Part A's zero-sized niche
companion change is published and clears core's actual `Result<(), Repr>` size
assertion. The current census has three failed and four blocked bodies; next is
bool's nonzero niche, followed by the borrowed closure output-lifetime model.

This supersedes earlier stop instructions. Historical checkpoints and retained
requirements are in Git (`git show 03003a86f:handoff.md`); every retained requirement
there still holds (see [Standing requirements](#standing-requirements)).

## Workspace and Git

- The main checkout `/home/vishvananda/rustgm` is Git-only and never builds.
  Edit and build in B, `/mnt/vizier/rustgm-scratch/rustgm-pa26-work`, branch
  `implement/scoped-inherent-20261110`. Publish with `git push origin HEAD:main`.
- Part A's published implementation is `34a28702e`, descending from handoff
  e6610fae3. The clean release candidate11 binds that source. Documentation-only
  descendants may follow without rebuilding it.
- Push CI is bounded smoke/harness/upstream, not a full qualification. The latest
  verified full success is [43239321921](https://github.com/vishvananda/rustgm/actions/runs/43239321921)
  on exact a7ce2784a; fast and qualification both pass. Current candidate11 has
  focused, course and original-census evidence; full successor qualification is
  pending and owned by scheduled CI. Do not dispatch a manual full per edit.
- The old uncommitted part-A patch is integrated. Its raw restoration patch and
  earlier failed receipts remain under `obj/parallel/zst-niche/`; do not reapply it.
- Next input-only work: `bool-batch2`, `bool-declared2.json` and
  `bool-independent2.json` under that root. All21 frozen Rust outcomes agree.
  Batch1's four rustc ICEs came from missing bool Clone/Copy declarations and are
  retained as infrastructure failures. No bool implementation has been made yet.
- Commits use the repository's single-line subject with no trailers.
- The backend stays pinned at v3 (17b024f69). Don't modify the C++ or v5rust
  checkouts or other checkouts' outputs. Work as one worker, with no new
  worktrees.

## Core census

Release candidate11 (`obj/parallel/capture-match/candidate11-release`, clean
34a28702e, CLI58d7312a) is the current observation. The unchanged120s observer
runs exclusively after load admission and finishes its body pass in24.56s:

- all42,310 declarations pass;
- 35,120 item attempts check25,851 bodies;
- three failed and four blocked bodies remain, against candidate10's four/four;
- actual initializer25396, `io/error/repr_bitpacked.rs:350`'s
  `size_of::<Result<(), Repr>>() == 8`, clears;
- no previously passing body fails; postcheck verifies722 sources,375 opens,
  six configuration bindings,3945 compiler sources and unchanged limits/profile.

The remaining failures:

| Body | Site | Failure | Next |
| --- | --- | --- | --- |
| `as_underlying`7631 | `slice/cmp.rs:52`, `transmute::<ControlFlow<bool>, u8>` | unequal sizes | bool niche |
| `into_searcher`26791, `is_contained_in`26792, blocking26793–26796 | `str/pattern.rs:956` | closure result retains an input-only or local lifetime | PA27 closure output-lifetime model |

Raw observer, comparison and postcheck are in
`obj/parallel/capture-match/original-candidate11*`.
The completion pass was not run. A core build also needs a backend contract for
ymm, zmm and kreg bodies beyond the initial assembly profile of v3.

## Completed: zero-sized niche companions (part A)

[Implementation and qualification](docs/pa31-zero-sized-niche-companions.md).
Source nullable eligibility now admits two single-field variants when one is
zero-sized and no more aligned than the other's existing carrier. Layout chooses
the actual orientation per instantiation; constant reads, construction, initializers
and imported-header re-derivation preserve those same variant identities.

Eleven declared PA31 outcomes agree with Rust. Fifteen private controls pass in
overall-timeout receipt1794558954797138354; ten fresh integration tests pass in
1794559299273190847 after fixing the ABI test's missing no_mangle exports. Every
active text case passes (9307/9307); upstream and315 Python harness tests,485
integration roots and44-stage layout checks pass. No references are changed.
Forty-eight frozen exclusive A/A+ABBA observations at widths8/32/128 and an ordinary
control preserve complete outputs. Short timings/noise make no speed claim.
Source34a28702e freezes release CLI58d7312a with3945 source hashes and an empty patch.

The larger tagged fallback remains I64, wider scalar patterns retain their
unsupported-profile error, alignment may not increase, and the declared orientation
is tried first. Course eligibility is unchanged. Raw evidence is
`obj/parallel/zst-niche/`; the actual core assertion is now verified clear.

## Next: `bool` niche (part B, 7631)

`ControlFlow<bool>` is `Continue(())` beside `Break(bool)`, so it needs part A's
companion plus a `bool` carrier whose niche value is 2. rustc encodes
`Continue(())` as 2, which batch 5's `controlflow-bool-continue` confirms.
Today every carrier is zero-niche. The sites that hardcode 0 or null are:

- lowering's `niche_zero` and `niche_test` (`lower/function/aggregate.rs`);
- cleanup's variant test (`lower/cleanup/emit.rs`);
- the initializer's `Work::Bits { bits: 0 }`;
- `nullable_zero` (`lower/data/zero.rs`), which must not report the empty
  variant as zero when the niche value isn't 0;
- the const-eval byte encoder (`fill(0)`, `const_eval/memory/bytes/enumerations.rs`)
  and decoder (`bytes/niches.rs`);
- transmutability's `tag(0, ..)`.

`carrier_type` (`lower/context/niches.rs`) must represent a `bool` carrier as
a byte so it can hold 2. Restrict the `bool` carrier to source policies with an
explicit policy flag. The course `Option` policy shares `nullable_layout`, and
an `Option<bool>` change there would move course references. Batch 5's
`controlflow-bool-*` and `option-bool-size` cases are declared and confirmed by
rustc.

**Known separate deviation, not needed by the census.** `repr(Rust)` enums get
an `I64` tag (`enum_tag` in `abi/layout.rs`), where rustc picks the smallest
fitting integer. So `Result<(), ()>` is 8 bytes against rustc's 1, and
`ControlFlow<u8>` is 16 against 2 (batch 5's `result-unit-unit` and
`controlflow-u8-size`). Changing it would move course layout references.

## Recent checkpoints

- 34a28702e, zero-sized niche companions:15 private/10 integration controls,
  all9307 course cases, upstream/harness and48 output-equal calibrated observations.
  Release candidate11 clears actual25396 without a reached-body regression.
- 99a409d03, body projection normalization
  ([evidence](docs/pa31-body-projection-normalization.md)): 17 cases, and six
  independently reviewed references. Candidate9 cleared `first_set` and both
  `try_from` bodies.
- 534f3cc67, global clause premises
  ([evidence](docs/pa31-global-clause-premises.md)): a normalizer's concrete
  premise selects implementations before a caller's global where clause
  (rust-lang/rust#50825), with 25 cases. Candidate10 cleared `div` 36996 and
  nine `div` and `rem` bodies.
  - Focused receipt 1794556164673085248 passed, and the frozen CLI passed all
    9,307 active text cases.
  - Recorded tradeoff: implementation-first frames skip the goal cache. 256
    clause-free `select` copies exhaust the crate-wide source method selection
    budget at function 222 instead of 229, and candidate9 already exhausts it
    at 128 copies.
- 0ceecbfe6 and 03003a86f, the candidate9 and candidate10 observations.

## Pins, artifacts and storage

- `obj/parallel/capture-match/candidate11-release` is the current comparison pin.
- After publishing this observation, retire the superseded candidate10 release
  pin and both `wipniche1-dev-fast`/`wipniche2-dev-fast` pins with scoped cleanup.
  Their producers and observations remain; do not treat retired paths as binaries.
- The pin script asserts3945 development Rust files. Update that count for actual
  new `.rs` files under dev/src, dev/bin or dev/tests; never drop test coverage to
  fit the count.
- Local free space is179GiB. The shared controller still has2 jobs,150GiB reserve,
  CPU800%,12GiB RAM and256 tasks, with a16GiB cap for this worktree. Storage's
 40GiB local default does not reduce the controller's150GiB reserve.

## Procedures

- Wrap every build, test, probe and benchmark:
  `python3 scripts/resources.py run --max-output-gib 16 -- /usr/bin/python3 scripts/storage.py run --no-prune --max-output-gib 16 -- CMD`.
  The wrapper chdirs to the repository root, so use repo-relative paths. Its
  "resources: waiting" lines can appear in stdout.
- Fastcheck: `python3 scripts/fastcheck.py --max-output-gib 16 [--pa paN]
  [--lib-case full::path] MODULE[::test] …`.
  - Selections name the test module, such as `global_clause_premises::case`,
    not the `suite_*` target.
  - Never pass `--pa pa41` or wrap fastcheck in `timeout`.
  - The architecture audit rejects 7 or more files sharing a filename stem, and
    Clippy rejects more than 3 bools in a struct.
- Declare outcomes before the first rustc or RustGM run of a new fixture.
  Record malformed batches and mispredictions.
  - rustc oracle: `obj/parallel/simd-divrem/oracle.sh BATCH OUT`.
  - RustGM: `obj/parallel/simd-divrem/rustgm.sh CLI BATCH OUT`.
  - The no_core prefix needs `legacy_receiver`, `clone_fn`, `freeze`, and
    `destruct` as `Destruct: PointeeSized`, plus `Neg` impls for negated
    literals.
- Packets are byte-for-byte copies of declared batch files, with `cases.tsv`,
  `outcomes.json` (sha256, declared, rustc, base, rustgm), a README and a
  `pa31/tests/README.md` entry. `obj/parallel/simd-divrem/make-packet.py` is the
  latest builder.
- Reference updates need a mechanical check, a base reproduction, an
  independent review and then scoped `course.py STAGE --bless --case PATH`.
- Core observation:
  1. Pin a clean head with `pin.py candidateN release`.
  2. Derive the observer with `sed 's/candidate11/candidateN/g' observe-original-candidate11.py`.
  3. Wait with `wait-load2.py 14 20 21600`, then run
     `scripts/resources.py run --exclusive … --prune-under obj/parallel/capture-match/original-candidateN -- python3 obj/parallel/capture-match/observe-original-candidateN.py`.
  4. Compare with `compare-original.py candidateN obj/parallel/capture-match/original-candidate(N-1)`.

  Keep the 120s observer window and all compiler limits. Don't build or clean
  during an exclusive measurement.
- Retire a pin by removing its literal `.keep-artifacts` path, then
  `python3 scripts/storage.py prune --apply --prune-under DIR`. Never use
  variable or command substitution, `xargs`, `find -delete` or globs with `rm`.

## Standing requirements

Everything under "Retained requirements and resource rules" and "Historical
checkpoints" in `git show 03003a86f:handoff.md` remains in force. That
includes:

- the symbolic-index native companions;
- the implicit-receiver Deref packet's two 128-layer positives;
- the const-adapter bootstrap and runtime positives;
- the three tuple-factory cleanup-owner cases;
- genuine source panic runtime routing for native and core artifacts;
- PA25-PA44 remain planned assignments that must fail, not pass empty.

No frontend, body or abort substitution is authorized.
