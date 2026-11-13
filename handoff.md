# RustGM handoff — 2026-11-13

The user asked to wrap up. Implementation work is stopped, and no local build,
test or measurement is running. Resume only on a new instruction. The objective
is still to compile the pinned original core, then alloc and std. **No library
artifact exists yet.** The current milestone is the core census: `--emit-types`
passes original core.

This file replaces the earlier handoff. Its historical checkpoints and retained
requirements are in Git (`git show 03003a86f:handoff.md`), and every retained
requirement there still holds (see [Standing requirements](#standing-requirements)).

## Workspace and Git

- The main checkout `/home/vishvananda/rustgm` is Git-only and never builds.
  Edit and build in B, `/mnt/vizier/rustgm-scratch/rustgm-pa26-work`, branch
  `implement/scoped-inherent-20261110`. Publish with `git push origin HEAD:main`.
- `origin/main` is this handoff's commit, which descends from 03003a86f. The
  "Reference and course checks" workflow passed on every commit through
  03003a86f. The last full CI run that passed every stage was
  [CI43157950706](https://github.com/vishvananda/rustgm/actions/runs/43157950706)
  on 03b2ad7e0. No full run has been dispatched since; scheduled CI owns it.
- **Uncommitted in B:** the zero-sized niche companion change (part A, below).
  It has 13 modified tracked files, plus the untracked
  `dev/tests/zero_sized_niche_companions.rs` and
  `pa31/tests/spec/zero-sized-niche-companions/`. It is not qualified. The
  complete patch, including the untracked files, is
  `obj/parallel/zst-niche/part-a-wip.patch` (sha256 26f37766…). It
  reverse-applies on the WIP, so it can restore the work, not be applied on
  top of it.
- Commits use the repository's single-line subject with no trailers.
- The backend stays pinned at v3 (17b024f69). Don't modify the C++ or v5rust
  checkouts or other checkouts' outputs. Work as one worker, with no new
  worktrees.

## Core census

Release candidate10 (`obj/parallel/capture-match/candidate10-release`, clean
534f3cc67, CLI 070d8da9) is the current observation
([tracker](docs/library-build-blockers.md)). The unchanged 120s observer ran it
exclusively from load 11.4:

- it reached the end of the body pass in 23.1s;
- all 42,310 declarations pass;
- it made 35,120 item attempts and checked 25,850 bodies;
- **4 failed and 4 blocked bodies** remain, against candidate9's 5 and 13;
- no previously passing body failed
  (`obj/parallel/capture-match/original-candidate10-comparison.json`).

The remaining failures:

| Body | Site | Failure | Plan |
| --- | --- | --- | --- |
| const `_` 25396 | `io/error/repr_bitpacked.rs:350`, `static_assert!(@usize_eq: size_of::<Result<(), Repr>>(), 8)` | incompatible types | part A, in progress |
| `as_underlying` 7631 | `slice/cmp.rs:52`, `transmute::<ControlFlow<bool>, u8>` | source transmute requires equal sizes | part B, `bool` niche |
| `into_searcher` 26791, `is_contained_in` 26792, blocking 26793-26796 | `str/pattern.rs:956` | closure result retains an input-only or local lifetime | PA27 owns the closure output-lifetime model |

A core build would also stop at the initial assembly profile for the ymm, zmm
and kreg bodies, which need a backend contract the v3 pin lacks.

## In progress: zero-sized niche companions (part A, 25396)

**Diagnosis.** `Repr` is `#[repr(transparent)] struct Repr(NonNull<()>,
PhantomData<..>)`, and rustc lays out `Result<(), Repr>` in 8 bytes, with
`Ok(())` in the null niche. RustGM's source nullable policy
(`types/aggregate/nullable.rs`) admits only a fieldless empty variant beside
one single-field variant, with an orientation fixed per item. `Ok(())` has one
zero-sized field, and `Result<Repr, ()>` needs the reverse orientation.

**Change (uncommitted).**

- `NullableEnum` gains `companion: Companion`: `Fieldless`, which is today's
  rule, or `ZeroSized`, for two single-field variants.
  - The source rule (`sema/declarations/aggregates/nullable_members.rs`) and
    the importer's re-derivation
    (`sema/crates/typed_headers/staged_headers/mapped_records.rs`) admit field
    counts (1, 1). Both rules must stay identical; no policy bytes are
    transported.
  - A `ZeroSized` policy has no `U8` fallback tag, so its tagged layouts don't
    change.
- `abi/layout/niches.rs` `orientation` tries the declared orientation, then the
  reverse. The empty side's field must be zero-sized and no more aligned than
  the payload (`fits_beside`), and the payload must be a carrier. Otherwise the
  enum keeps its tagged layout.
- `enum_niche` (`sema/constants/values/niches.rs`) checks the record's
  orientation with `NullableEnum::admits` and reads its identities from the
  record.
- Lowering construction, static initializers and zero data
  (`lower/function/aggregate.rs`, `lower/data/initializer.rs`,
  `lower/data/zero.rs`) admit an empty variant with one zero-sized field.
- The test constructors gain `companion` (`nullable_layout.rs`,
  `composite_classes.rs`).
- `dev/tests/zero_sized_niche_companions.rs` is registered in the memory
  suite. It has two tests:
  - `zero_sized_companions_niche_fill_like_rustc` types the 11 packet cases
    through the CLI and lowers the accepted ones;
  - `zero_sized_companion_values_lower_through_the_carrier` checks that the
    exported `make` and `swapped` return a scalar `Ptr`, and that
    `Result<usize, &u8>` keeps the out-pointer ABI.

**Evidence so far** (`obj/parallel/zst-niche/`):

- `declared.json` records five declared batches:
  - batch 1 is malformed: rustc 1.98 has no `pattern_type_macro` feature;
  - batch 2 is malformed: rustc requires the `legacy_receiver` lang item,
    and RustGM requires `clone_fn`;
  - batch 3 is malformed: rustc requires the `destruct` lang item;
  - batch 4 is malformed for RustGM only: its pinned profile requires
    `Destruct: PointeeSized`;
  - batch 5 has 20 cases, and rustc agrees with every declaration
    (`batch5-rustc/`).
- Base candidate10 (`batch5-rustgm/`) fails every `Result` companion case.
  The WIP pin `wipniche1-dev-fast` (CLI 2334c71b, built before the tests were
  added) matches rustc on all 11 packet cases (`batch5-wip1/`).
- Focused fastcheck 1794558203661636385 passed metadata, formatting,
  architecture and strict Clippy. It then **ran out of the five-minute budget**
  in uncached private Clippy, after 180.8s (status 124), so no test ran yet.

**To finish part A:**

1. Rerun the same fastcheck, `sh obj/parallel/zst-niche/fastcheck-part-a.sh`. The private-lint incremental cache from that attempt should make it
   fit. If it doesn't, run the integration selection and the `--lib-case`
   selections as two receipts.
   - Integration: `zero_sized_niche_companions`,
     `option_storage::nullable_enum_source_contracts`, `rust_enum_storage` and
     `cell_sources`.
   - Library: the six `abi::layout::niches::tests`, the
     `sema::constants::values::niches::tests` reader, the three
     `const_eval::memory::bytes::niches::tests`, the three
     `lower::context::niches::tests`, and two `abi::classification::tests`.
2. Write `docs/pa31-zero-sized-niche-companions.md` (diagnosis, change,
   tradeoffs, contract table, qualification), the packet `README.md`, and a
   `pa31/tests/README.md` entry. Extend spec.md's "Source-mode
   empty/single-field Rust enums publish physical nullable eligibility…"
   paragraph (near line 1866). Update ROADMAP and the tracker.
3. Record these tradeoffs:
   - a `ZeroSized` fallback keeps the `I64` tag;
   - a scalar-range payload still errors, as for `Option`;
   - the companion may be no more aligned than the payload;
   - orientation tries the declared order, then the reverse.
4. Run the full PA1-PA24 text run with a frozen CLI, as
   `obj/parallel/simd-divrem/text-all.sh` does. Course mode should be unchanged
   because source nullable eligibility is source-mode only. Review any changed
   reference independently before any scoped bless.
5. Commit, push, pin candidate11 release, and observe it exclusively against
   candidate10. Expect 25396 to clear. Then retire `wipniche1-dev-fast`.

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

## This session's checkpoints

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

- `obj/parallel/capture-match/candidate10-release` is the only release pin.
- `obj/parallel/capture-match/wipniche1-dev-fast` is the part A WIP pin. Retire
  it after candidate11.
- Every other `capture-match` directory has had its binary retired; their
  observations remain. Never treat a pruned path as an available binary.
- `obj/parallel/capture-match/pin.py` asserts 3945 source files, which includes
  the uncommitted test. If part A is abandoned, set it back to 3944. Add 1 for
  each new `.rs` under `dev/src`, `dev/bin` or `dev/tests`.
- At handoff, `/mnt/vizier` had 180G free. Keep the 40 GiB local reserve.

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
  2. Derive the observer with `sed 's/candidate10/candidateN/g' observe-original-candidate10.py`.
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
