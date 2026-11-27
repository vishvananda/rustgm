# Performance, validation speed and self-hosting plan

Adopted 2026-11-20 by the user, with all seven recommendations in
[Decisions](#decisions-needed) accepted. This is the current execution path. It
supersedes the execution order in the [self-hosting plan](self-hosting-plan.md);
source pins, Rust semantics and the frozen CPPGM backend boundary are
unchanged. Lanes may run in parallel as long as the host is not heavily loaded:
the controller admits two heavy jobs at a time.

It answers three requests: make the frontend much faster using what the CPPGM
Rust port (`~/work/private/v5literal/checkout`, read-only) does, make
validation fast, and find the fastest route to self-hosting and assignment
close-out. Numbers marked *estimate* are inferences from profile shares; the
rest were read from receipts, code or the saved profiles.

## The short version

1. **The frontend is 99% of compile time and repeats its own work.** The
   joint core + alloc + std graph spends 46.8 s in the frontend and 0.46 s in
   CPPGM. RustGM retires about 243K instructions per loaded source line on the
   core graph; rustc about 126K; CPPGM's frontend about 33K. CPPGM gets there
   with no clever algorithms: it computes each fact once, keys every memo on
   identity, never re-validates, and allocates almost nothing. RustGM solves the
   same trait problem tens of times, copies inference graphs to speculate,
   re-authenticates immutable facts on every read, and has a work ledger whose
   exact-replay rule has blocked the caches that would fix this.
2. **The validation loop loses most of its time outside the tests.** Only
   28.5% of 5,864 fastcheck launches passed. A no-op run costs 36–60 s of gates,
   every source edit compiles the frontend two or three times at two build jobs
   on a 64-thread machine, and 15% of wall time is compiles killed at the
   deadline. The fixture lane already runs 9,374 cases in about 20 s.
3. **The strict self-hosting path is unbounded; the joint-graph path is
   short.** Strict core is in the first of about 13 export passes on crate 1 of
   22, at 0.56 gates per hour and falling, with no census of what remains. The
   joint graph already compiles and runs with 14 unproven obligations and 50
   waivers. RustGM has never been run on its own source in any mode.

The plan is therefore: fix the loop first (days), take a census of the
compiler's own source in the joint graph immediately, and start the
algorithmic performance work in parallel. The seven decisions in
[the last section](#decisions-needed) are adopted as recommended.

## 1. Frontend performance

### Where the time goes

Frozen graph: original core at `88d9e12ae`, `unicode-ident`, one `no_std`
consumer. RustGM retires 88.0B user instructions; a cold rustc build of the
same targets retires 45.5B.

| Phase | Share of cycles |
| --- | ---: |
| Parse, configure, expand | 5.5% |
| Name resolution | 2.2% |
| Declarations (29 crate-wide passes) | 16.1% |
| Body typing | 57.8% |
| Template CFGs, contracts and proofs | 5.0% |
| Concrete bodies and instances | 10.5% |
| Finish constants and lints | 1.5% |
| LowIR and everything after | 1.5% |

By mechanism, trait selection is present in 35.7% of samples and inference in
20.7%. The leaf profile is flat: 130 symbols make up half the cycles. A third
of all cycles are container and runtime overhead rather than compiler logic:

| Overhead (leaf cycles) | Share |
| --- | ---: |
| `malloc`/`free` and `RawVec` shims | 12–15% |
| Hash table probes, inserts, key equality | 7.5–15% |
| `memmove`/`memcpy` | 3–5% |
| Drop glue | 3–5% |
| Lint/attribute owner revalidation | 4–7% |

More than a dozen optimizations have landed since October. Each saved under 3%
because each trimmed a constant inside a per-query path; none removed queries
or copies.

### What CPPGM does that RustGM does not

Evidence for the CPPGM column is in the port under
`dev/src/translated/src/`; for the RustGM column, in `dev/src/`.

| CPPGM practice | RustGM today | Change |
| --- | --- | --- |
| **Answer each semantic question once.** Memo keys are pointer tuples (`SpecializationKey`, conversion sets, concept results); 75,827 specialization requests produce 30,341 records. | Strict core makes 768,399 candidate lookups over 13,878 distinct keys and 742,205 impl trials; the 169,870 selection rows kept in typed bodies name only 3,337 distinct concrete `(impl, arguments)` answers. The crate-wide memo rejects any goal containing a lifetime, parameter or projection, needs two identical full searches before reuse, and applies only in an assumption-free caller (`sema/traits/selection/memo.rs:158`). | A canonical query cache keyed on `(environment id, goal with regions and inference variables replaced by indices)` that stores a canonical response and instantiates fresh regions on a hit. |
| **Cheap exits before real work.** Overload resolution filters by arity, tries non-templates first, and a perfect match skips deduction. | 97% of trials (718,871 of 742,205) carry no inference edges, yet each builds a `Trial`, a fresh inference context and a substitution folder. 204,141 trials that pass the index still end in a header mismatch. | A one-way structural matcher over interned ids for goals with no inference variables: no context, no allocation. An allocation-free deep reject before any trial is built. |
| **Speculate in place.** Candidates and conversion sequences live in reused flat vectors; nothing is copied to try a candidate. | `infer::Context` has no undo log (`types/infer.rs:1`). Isolation is a fresh context plus a graph import per frame, per trial and per nested condition; publishing a result builds a third context. A type crosses between the store and term graphs four times. | One per-body union-find with an undo log. A trial is snapshot, unify, commit or roll back. |
| **Never re-validate.** The validator runs only under `--validate`; sema sets one `facts` pointer per expression and lowering reads it. The frontend has no runtime asserts. | Immutable facts are re-authenticated on every read: `Owner::validate` re-walks every attribute witness per call (31,992,265 lint read steps against 970,361 AST nodes); impl polarity costs 3–4 hash lookups several times per trial; trait classification and language-item roles are re-derived per query; sessions are authenticated three times. | Validate once when a table is sealed, then read plain fields. Keep revalidation behind a self-check mode, as CPPGM does. |
| **Bump arenas, plain records, persistent scratch.** 0.84 mallocs per token; 2% of instructions in the allocator. | Each search builds a `Query` with five hash maps and three vectors; each trial allocates six more containers; a search `Frame` is 1,312 bytes and is moved through a `Vec`. | A pooled search scratch owned by the session, ranges into shared index arenas instead of per-node vectors, a small hot frame with boxed cold state, and per-impl instantiation recipes built once. All of this is safe Rust. |
| **Field reads instead of lookups.** An `Identifier` carries its keyword, macro and directive role; visited sets are epoch marks stored in the record; one name table is probed once. | About 2,190 hash map uses, mostly keyed by dense `u32` ids (`DefId`, `NodeKey`, `ItemId`, `TypeId`). Attribute names are compared against source text as strings. | Dense id-indexed vectors for the hottest side tables, per-body vectors indexed by a body-local node ordinal, epoch arrays for visited sets, attributes classified into an enum when configured. |
| **Do work on demand, at most once.** Naming a specialization only creates an entity; bodies are analyzed when demanded (26,007 requests, 8,903 bodies). | In a joint graph every body of every dependency is typed and proved for every consumer, though the consumer emits 39 functions. Each instance clones its CFG and is fully re-proved, because `spec.md` requires ownership, region and loan checks "under each complete instance's own inputs". | Type and prove only the demanded bodies of dependency crates. Prove borrows and regions once per template and do only dispatch and drop elaboration per instance. |
| **Three walks per body:** parse, one semantic pass, one lowering pass. | About ten AST walks and fifteen CFG passes per body, plus nine post-expansion validators and 29 declaration passes. | Fuse walks once the items above land; this is last because its measured share is small. |
| **One coarse budget.** The only step counter is the constant interpreter's 2^24 cap. | About 6,100 metering sites and 167 `charge` implementations. A warm cache hit must reproduce the original debits and the first budget failure, and about 470 test files assert exact work counts. | Budgets become coarse runaway guards. See decision 1. |

RustGM already has interned types with `u32` identity, a slice interner, a
string interner and the Fx hasher adapted from the port. The type store is not
the problem.

### Work order

Savings are *estimates* of total cycles on the frozen graph. They overlap and
do not add.

| Step | Work | Estimate | Size and risk |
| --- | --- | ---: | --- |
| A1 | Sealed fact tables: trait class, impl polarity, language-item roles, lint level per owner, attribute enums | 6–9% | Small, low. Needs decision 2. |
| A2 | Concrete-goal matcher and allocation-free pre-reject | part of A3 | Small to medium, low. |
| A3 | Canonical query cache, first for goals without parameters where the environment has no clause for the trait, then environment-keyed | 20–28% | Large, highest risk: regions, ambiguity, coinduction. Needs decision 1. Build a shadow that counts hits by phase first. |
| B1 | Pooled search scratch, small frames, per-impl recipes | 6–10% | Medium, mechanical. |
| B2 | Undo-log inference with in-place trials | 8–12% | Large. Body-phase searches must rerun on wake rather than hold a snapshot open. |
| B3 | Dense id tables for the hottest twenty side tables | 4–7% | Medium, wide but mechanical. |
| C1 | Demand-driven bodies for dependency crates | Removes most of a consumer's compile; not visible on the core-only graph | Medium. The dependency's own strict build still checks every body. |
| C2 | Per-template borrow and region proof | 5–7% here, far more for a 309K-instance compiler build | Large. Needs decision 3. |
| C3 | Fuse AST and CFG walks; emit event lists from the resolver walk | 2–4% | Medium. |

A1, A2 and B1 start first: they are low risk and need no cache design. A3 is
the largest single lever and starts with its shadow census the same week.

Targets on the frozen core graph, in user instructions:

| Target | Instructions | Meaning |
| --- | ---: | --- |
| Today | 88B | |
| After A and B | 45B or less | At or below rustc on the same graph (*estimate*: the steps above remove 50–65%) |
| After C | 25B or less | |
| Stretch | 15B or less | CPPGM-class cost per line |

The profile is flat, so the list above is not the whole route to the stretch
target. Re-profile after each lettered group and apply the same three rules
(answer once, never re-validate, do not allocate to speculate) to whatever
is then on top, starting with the declaration phase.

### Measurement rules

- **Primary metric: user instructions and deterministic work counters** on
  frozen inputs with byte-identical output. Wall time on this host has A/A
  ranges of 28–63%, so paired ABBA wall runs are for milestones only.
- **Stop sub-1% work.** A change that does not remove at least 1% of
  instructions on the core graph, or a whole class of repeated queries, is not
  worth a checkpoint.
- **Fair comparison with rustc.** Same crate graph, target, edition, cfg and
  features; same opt-level, overflow checks, debug assertions, panic strategy
  and codegen units; incremental off; one job; both compilers producing the
  same artifact kinds. Report the frontend-only pair (RustGM through proofs
  against `rustc --emit=metadata`) and the end-to-end pair separately. The
  current 42.8 s against 37.0 s figure mixes scopes and should be retired.
- **Benchmark set:** core alone, the joint core + alloc + std consumer, the
  three small native controls, and the trait C×Q scaling family.

### Deliberately later

- **Host build tuning.** LTO, one codegen unit, `panic = "abort"`, PGO and an
  allocator swap are worth single-digit percent each (the port measured 2–3%
  for fat LTO). They wait until the algorithmic work lands so they do not mask
  structural cost. Distributed rustc is itself built with LTO and PGO, so
  matching that configuration afterwards is the fair comparison.
- **Parallel body checking.** CPPGM is single-threaded and still several
  times faster than GCC (2.8–6.8× by workload). Threads are blocked today by `&mut Declarations`, the
  order-dependent interner and the ordered ledgers; revisit after C.
- **LowIR transport.** Text emission is 0.01% today. It becomes relevant at the
  projected 280–415 MB for a stage build; handle it in the self-hosting track.

## 2. Validation speed

### Where the time goes

From 5,874 fastcheck launch records across five checkouts, all under the
controller's 8-CPU cgroup with two build jobs and two test threads.

| Finding | Evidence |
| --- | --- |
| 47.6% of launches are refused at admission | fastcheck passes `--wait-seconds 0` |
| A no-op run still costs 36–60 s | `cargo fmt --all --check` 19–25 s, assignment metadata 2–15 s, audit 3 s, accounting scans up to 30 s in a worktree with 1.5M `obj` entries |
| Any `dev/src` edit costs about 37 s of check and 57 s of build, regardless of module | clippy checks the library, then `cargo test --no-run` compiles it again; a private selection compiles it a third time |
| Private unit tests cost 66 s for a tests-only edit | 184K lines of `dev/tests/invariants` are included into the library under `cfg(test)`, about 742K lines in one unit |
| 15% of wall time is runs killed at the deadline | 72 of 99 private-lint timeouts come in streaks: a killed rustc does not finalize its incremental session, so the next run starts over |
| Compilation uses 2 of the 8 CPUs it is granted | `CARGO_BUILD_JOBS=2`, `RUST_TEST_THREADS=2` in `scripts/resource_runner.py` |
| The 13 suite binaries take about 19 minutes serially in dev-fast | 177 test files spawn rustc as a live oracle; table-driven tests hide a process per row |
| Fixtures are about 100 times cheaper per case | 9,374 PA1–PA24 cases in about 20 s; 2,078 PA25–PA30 cases in about 24 s; adding one needs no Rust build |

Median admitted run time rose from about 60 s in early November to 150–180 s
now, with the 90th percentile pinned at the kill point.

CPPGM's loop is fast for plain reasons: tests are fixtures only (6,440
oracles, no test binaries), the runner uses every CPU with persistent
`--batch-stdin` workers that fork per request, an edit recompiles one
translation unit, and there is no gate chain. The Rust port's own build is not
fast (median 405 s); the test side is what to copy.

### Step 1: fix the loop we have (days, no architecture change)

1. **Cache and parallelize the static gates.** Format only changed files with
   a content cache like the audit's, skip assignment metadata unless a manifest
   changed, run gates beside the compile, and skip them when the source
   fingerprint matches a passed receipt. Saves 40–60 s per run.
2. **One compile per edit.** Use `clippy-driver` as the compiler for the
   public build, as the private lane already does. Saves about 35 s per edit.
3. **Use the granted CPUs.** Raise build jobs and test threads from 2 to 8
   inside the existing quota.
4. **Never kill a compile.** Let rustc finish and persist; apply the budget to
   test execution. Queue for admission instead of failing instantly.
5. **Stop rescanning `obj`.** Account declared output roots or keep a running
   tally, and archive the frontier worktree's 1.5M entries.

Expected result: a typical source-edit run drops from 150–180 s to 60–80 s
(*estimate* from stage medians) and the timeout streaks disappear.

### Step 2: fixtures first (weeks, incremental)

1. **New tests are fixtures.** Input, declared outcome, oracle, run by the
   course runner against one prebuilt binary. A Rust harness test is allowed
   only when it needs in-process access.
2. **Persistent batch mode in the CLI**, modeled on CPPGM's worker, and the
   runner at full granted width. Target: the whole 11,452-case corpus in under
   15 s.
3. **Migrate the process-spawning tests.** The 159 files that spawn the CLI
   and the 177 that spawn rustc become fixtures with rustc outcomes recorded
   when the fixture is created and rechecked nightly.
4. **Take the private harness out of the edit loop.** It runs at checkpoints.
   The roughly 470 files that assert exact work counts are redeclared as
   output and scaling-bound checks under decision 1.
5. **Cache results** by compiler binary hash, input hash and flags.

The edit loop then is one lint-compile of the CLI plus the affected fixtures.

| Tier | Contents | Target |
| --- | --- | ---: |
| Edit | One compile, affected-PA fixtures | 60–90 s, compile-bound |
| Checkpoint | All fixtures, suites and library harness at 8 threads | under 10 min |
| Nightly | Release build, native lane (44 min today), live rustc oracles, full Clippy | unattended |

### Step 3: make the compile itself fast

- **Crate split.** The fifteen top-level modules already form a strict
  dependency order that the audit enforces, so a workspace split is possible.
  `sema` is 46% of the code and one strongly connected component of 64
  submodules, and most edits land there, so the split pays off fully only once
  `sema` is broken up. Measured and declined for now: see decision 7.
- **Self-hosted builds.** Once stage 1 exists and the frontend reaches its
  targets, RustGM builds RustGM. CPPGM self-compiles in a quarter of GCC's CPU
  time; the equivalent here turns the 100 s compile into seconds and is the
  real end state of this section.

## 3. Self-hosting and assignment close-out

### State

| Rung | State |
| --- | --- |
| M1 core | Done in the bring-up profile, zero skips |
| M2 alloc | Partial: core + alloc emit and link; Vec and String match; 2 of 12 consumers recorded |
| M3 std | Partial: 9 of 14 consumers match. `env-args`, `fs` and `io-stdin` are open and the compiler uses all three |
| M4 Unicode crates | Partial: `unicode-ident` works; `tinyvec` stops at 12 body blockers |
| M5 stage 1 | Not started. RustGM has never compiled its own source |
| M6 stage 2 | Not started; the comparator exists |
| M7 strict | Core unpublished |

A demand list for the compiler exists: 309,093 instances, reaching about 10%
of core's functions, 27% of alloc's and 18% of std's. The milestone consumers
already cover 87% of them.

### Why the strict path is unbounded

A strict library build produces a metadata packet. The packet is a
hand-written table per declaration family; the producer admits only recognised
shapes; the consumer decodes, re-validates and re-proves every imported fact
in a fresh process; and fourteen profile versions are kept byte-compatible.
Each new construct therefore needs a schema row, an admission check, codec
columns, consumer validation, a reproof and tests: about 505 compiler lines
per gate. The machinery is about 84K lines.

- The current gate is in the first of about 13 export passes. The later
  passes and body-row collection have never run on real core.
- Export errors are `&'static str` with no location. Each gate was found with
  a debugger breakpoint.
- There is no keep-going mode, so the number of remaining gates is unknown.
- The export must cover all 25,858 core bodies, though self-hosting demands
  about a tenth of them, and then 21 more units, several of which
  (`compiler_builtins`, `panic_unwind`, `proc_macro`, the compiler itself)
  have never been compiled in any mode.
- The rate fell from 1.7 gates per hour to 0.56.

### Fastest path

1. **Park the alias patch and stop strict transport fixes.** The patch stays
   saved as it is.
2. **Take the stage-1 census now.** Run the existing guest command with
   `tinyvec_macros`, `tinyvec`, `unicode_normalization` and `dev/src/lib.rs`
   as guests and `dev/bin/rustgm.rs` as the application: first the type-check
   census, then the bring-up census through lowering. This is the largest
   unknown in the project and costs one observer run.
3. **Fix by family, ranked by the demand list.** The `tinyvec` projection
   blockers, the three open std consumers, then the compiler's own records.
4. **Scale the output path.** Raise the 64 MB instance budgets for the
   projected 280–415 MB of LowIR, stop writing global data one line per byte,
   and measure CPPGM above `-O0` on that volume.
5. **Stage 1 (M5).** Link, then run the 9,374 PA1–PA24 cases through
   `RUSTGM_TEST_APP`.
6. **Stage 2 (M6).** Rebuild with stage 1 and compare LowIR and the linked
   compiler byte for byte. Determinism does not depend on metadata.
7. **Strict ratchet in the joint graph (M7).** Burn down the 14 unproven
   obligations (12 outlives, 2 loan conflicts) and the 50 waivers (46 are
   intrinsic qualification) plus whatever the compiler crate adds, to zero
   records and zero trapping stubs.
8. **Close PA45** under decision 4.

In parallel, and cheaply: **build the strict export census.** Give export
errors a `DefId` and source location and add a mode that records and
continues. One run then sizes the transport work, which decision 4 needs.

### Assignment close-out

| Assignments | Route |
| --- | --- |
| PA25–PA31 | Publish the two implemented units (`simd-abi`, `provided-profile`). Release the partial units that the demand list reaches as the ladder exercises them. Close each assignment at its published manifest plus deferred list under decision 5. |
| PA32, PA33, PA36 (core, alloc, std) and PA34, PA35 | Fall out of M1–M4. Their unit exits currently require separately compiled artifacts, so they share decision 4. |
| PA29 `generic-transport` | Resized by the export census. If kept, replace per-family admission with one generic encoding of the declaration model. |
| PA38–PA44 (proc-macro host, derives, unwinding, edition and target closure, async) | Independent and unstarted. Schedule after M5 when lanes are free; none blocks self-hosting. |
| PA45 acceptance | Last. |
| PA37 | Steps 1–8 above. |

### Risks

- **The compiler's own 558K lines** have never been seen by RustGM. The census
  in step 2 turns this from unknown to a list.
- **Volume.** 309K instances against budgets sized for small programs, and
  per-instance re-proof (performance step C2) multiplies frontend time.
- **Stage-1 speed.** At about 9.5K lines per second today, and with stage 1
  built at CPPGM `-O0`, stage 2 could take tens of minutes; the 225 s unit
  timeouts will need raising.
- **Conflicts between tracks.** Performance work and self-hosting fixes both
  touch `sema`. Integrate daily once the loop is faster.

## 4. Sequence

| Lane (existing worktree) | First week | Then |
| --- | --- | --- |
| Loop (`rustgm-async-work`) | Step 1 of section 2 | Batch runner and fixture migration; later publication and close-out |
| Self-hosting (`rustgm-pa26-work`) | Park the alias patch; stage-1 census; export census | Family fixes toward M5 and M6 |
| Runtime (`rustgm-backend-work`) | `env-args`, `fs`, `io-stdin`; instance budgets and LowIR volume | Stage link and comparison |
| Performance (`rustgm-pa29-work`) | Instruction-count harness and benchmark set; A1, A2, B1; cache shadow census | A3, B2, B3, then C |

The loop lane goes first if only one lane can run: every other lane is paying
its cost on every edit.

Process rules for all lanes: one qualification per batch by cause; receipts
stay under `obj/` and are not narrated in documents; the handoff and roadmap
change at milestones only. Since November 19, 121 of 245 commits were
record-only commits.

## Decisions needed

All seven were adopted as recommended on 2026-11-20. `AGENTS.md`, `spec.md` and
the PA45 handout carry the resulting rules.

| # | Decision | Recommendation (adopted) | What it unblocks |
| --- | --- | --- | --- |
| 1 | Is the work ledger an exact, replayable contract or a coarse runaway guard? | Coarse guard. Caches may skip debits; tests assert outputs and scaling bounds instead of exact counts. | The query cache (the largest performance lever) and about 470 test files |
| 2 | May sealed data be trusted without revalidation on read? | Yes, with revalidation behind a self-check mode. | Sealed fact tables |
| 3 | May borrow and region proofs be done once per template instead of per instance? | Yes; amend `spec.md` (the "each complete instance's own inputs" rule). rustc checks generic bodies once. | Per-template proof, and a tractable stage build |
| 4 | Does PA45 require separately compiled strict artifacts with metadata for every crate? | Amend it to a strict whole-graph session: zero stubs, obligations and waivers, comparing merged LowIR and the linked compiler. Leave separate compilation to PA29 and PA32–PA36, sized by the export census. | A bounded route to PA45 |
| 5 | Do the finite contracts close PA25–PA31? | Yes, at the published manifest plus deferred list, with deferred items carried by PA44. | Closing seven assignments |
| 6 | May the loop rules in `AGENTS.md` change? | Yes: compiles are never killed, admission queues, 8 build jobs, fixtures first, harness suites at checkpoints. | Section 2 |
| 7 | Should the crate split be allowed? | Not now (measured 2026-11-21). Splitting into base, `sema` and upper crates saves a mean 10–12 s of a 63 s one-module rebuild (nothing for a `lex` edit, about 18 s for `sema`, 45 s for `pipeline`) against 10,545 path rewrites, 2,489 items widened to `pub`, 917 `#[path]` test files split across harnesses and a multi-crate audit. Revisit if edits concentrate above `sema`. | Section 2, step 3 |

## Evidence

- Performance attribution and controls:
  `/mnt/vizier/rustgm-scratch/rustgm-pa29-work/obj/parallel/performance-gap/report.md`
- Trait lookup and trial counts: [self-hosting plan](self-hosting-plan.md),
  "Trait query representation and reuse"
- Joint-graph run:
  `/mnt/vizier/rustgm-scratch/rustgm-pa26-work/obj/parallel/m4-xid-frozen/unicode-consumer-xid/receipt.json`
- Fastcheck receipts: `obj/qualification/fast/` in each worktree; controller
  receipts under `obj/parallel/resources/`
- CPPGM numbers: `docs/port/frozen-performance.md` in the port;
  `pa36/calibration.tsv` and `docs/performance/` in
  `/mnt/vizier/cppgm-v091-reference`;
  `/mnt/vizier/v5literal-perf-evidence-20261109`
- Demand list and runtime probes: [self-hosting runtime](self-hosting-runtime.md)
