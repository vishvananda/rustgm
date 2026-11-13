# RustGM handoff — 2026-11-13

The active goal is to compile the pinned original core, then alloc/std, completing
and releasing assignment components as their contracts qualify. Continue as one
worker. **No original library artifact exists yet.** Part A's zero-sized niche
companion change is published and clears core's actual `Result<(), Repr>` size
assertion. Candidate12 clears bool's nonzero niche in actual core. The last census now has
two borrowed-closure lifetime failures and four blocked bodies; that lifetime
model is next.

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
- Bool's implementation and current input contracts are qualified; see
  [the checkpoint](docs/pa31-bool-niches.md). There are21 active PA31 cases,
  three native observers and one required nested-niche positive in pending.json.
  Independent Rust22/22, API21/21,18 private controls and current native programs
  pass. Full9307 course references are unchanged. No new oracle is generated.
- Clean source8950ad99c freezes candidate12, CLIbca8b9e1,3945 source hashes and
  an empty patch. Original observation/postcheck/comparison and48 exclusive
  A/A+ABBA controls pass; actual7631 clears with no reached-body regression.
- Commits use the repository's single-line subject with no trailers.
- The backend stays pinned at v3 (17b024f69). Don't modify the C++ or v5rust
  checkouts or other checkouts' outputs. Work as one worker, with no new
  worktrees.

## Core census

Release candidate12 (`obj/parallel/capture-match/candidate12-release`, clean
8950ad99c, CLIbca8b9e1) is the current observation. The unchanged120s observer
runs exclusively after load admission and finishes its body pass in23.29s:

- all42310 declarations pass;
- 35120 item attempts check25852 bodies;
- two failures and four blocked bodies remain, against candidate11's three/four;
- actual7631 (`ControlFlow<bool>` transmute), plus previously cleared25396
  (`Result<(), Repr>` size assertion), pass;
- no previously passing body fails; postcheck verifies722 sources,375 opens,
  six configuration bindings,3945 compiler sources and unchanged limits/profile.

The remaining failures are into_searcher26791 and is_contained_in26792 at
str/pattern.rs:956, both "closure result retains an input-only or local lifetime".
They block26793–26796. Raw observation/comparison/postcheck is in
`obj/parallel/capture-match/original-candidate12*`.

The completion pass was not run. A core build also needs backend support beyond
v3's initial assembly profile for ymm, zmm and kreg, and full original metadata,
provided-body and native consumer qualification. Passing the census will not
itself complete the original-core build or PA32.

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

## Completed: bool niche (part B)

[Implementation and qualification](docs/pa31-bool-niches.md). `Carrier::Boolean`
provides empty_bits=2 and byte storage. An explicit bool_niche flag admits it
only for source enum policies and validated imported source field groups. Both
cache query orders preserve the course layout; standalone transparent-wrapper
ABI keeps its existing profile. CTFE, validity, transmutability, construction,
matching, cleanup, initialization and zero-data classification consume the actual
carrier value. No backend or compiler-limit change is made.

Current API receipt1794562814860019382 checks all21 outcomes; native
1794562983528985835 passes three programs, including false/true destructor counts
`0:0 1:0 2:1 2:1`. Four unchanged cell programs retain passing native evidence.
Initial11 integrations pass1794561838421695979,18 private controls pass
1794562152284043406. All9307 active references reproduce with CLIf042c27d,73.58s.
Upstream/harness/485root/44-stage checks pass. Only test expectations/declared
cleanup inputs changed after the initial run; production hashes and CLI match.

Twenty-two independently declared outcomes agree with Rust. Batch1's four rustc
ICEs and later incomplete no_core prefixes remain setup failures, not language
rejections. The exact21 inputs and three C observers are bound by the owning
packet manifest. pending.json retains the Rust-positive second unused-value
contract; f042c27d still rejects it at its size assertion. No unsupported positive
is inverted and PA31 stays planned.

Candidate12's clean source8950ad99c/CLIbca8b9e1 clears actual7631. Its
postcheck and reached-body comparison pass, as do48 immutable A/A+ABBA
observations with complete output equality. Short timings make no speed claim.

Next reduce actual26791/26792. Eleven frozen ordinary controls and their
independent Rust/baseline11 observations are in `obj/parallel/closure-return-inputs/`.
Inferred immediate/named/reference identity closures reproduce the output-lifetime
error; annotated/contextual twins pass typing. The first LowIR probe omitted main, and the library retry supplied artifact
flags to a typing view, producing usage status2; neither qualifies prepared
bodies. ordinary2-observed/results.json now uses executable consumers: independent
Rust accepts/runs eight positives and rejects three opponents. Baseline12 rejects
five inferred positives at typing; three annotated/contextual twins lower. All
three intended negatives reject in typing or prepared lowering, including the
static escape and annotated-elided input. Keep those phase distinctions.
Capture bounded typed graphs, exact binder coordinates, opening/closing maps and
region equalities before changing the shared kernel.
Do not relax all output lifetimes: annotated elided returns and local/static
escapes must still reject. Remove temporary trace code before qualification.

A separate source-layout deviation remains: repr(Rust)'s fallback I64 tag differs
from Rust's smallest fitting tag. Result<(),()> is8 versus1, and ControlFlow<u8>
is16 versus2. General scalar/multifield and further unused-value nesting remain
required; course outputs must not be moved without a reviewed contract.

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

- `obj/parallel/capture-match/candidate12-release` is the current comparison pin.
- Candidate10 and wipniche1/2 were retired after publication of11; cleanup
 1794560721836625856. Their producer/raw files remain, their binaries are absent.
- After publishing this observation, retire superseded candidate11-release and
  wipbool1-dev-fast with scoped cleanup. Retain their producers and observations.
  Current managed dev-fast CLI is f042c27d; use candidate12 for new comparisons.
- Pin.py asserts3945 development Rust files. No new compiler .rs file is added
  by part B; update the count only for actual new files, never by dropping tests.
- Free space is about166GiB; monitor the unchanged reserve carefully. Controller2jobs/150GiB reserve/CPU800%/RAM12GiB/
 256tasks and the16GiB output cap remain unchanged. Storage's40GiB local default
  does not reduce that shared reserve. Check free space before every batch.

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
  2. Derive the observer with `sed 's/candidate12/candidateN/g' observe-original-candidate12.py`.
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
