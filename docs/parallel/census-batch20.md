# Census batch 20: method lookup through `[const]` supertraits ignores constness

Census 22 ([batch 19](census-batch19.md), pin `7df1f865`, `census.tsv` sha
`9b678716`) checked 42,310 declarations of original core, with none failed
or blocked, and one record. Its crate record is `local_annotations` at
`ptr/metadata.rs:251`, inside `impl<Dyn> Ord for DynMetadata<Dyn>`:
`<*const VTable>::cmp(&self.vtable_ptr(), &other.vtable_ptr())` reports "no
applicable trait method for this prefix". Core 1.98 declares
`const trait Ord: [const] Eq + [const] PartialOrd<Self> + PointeeSized`
(`cmp.rs:982`), and `impl<T: PointeeSized> Ord for *const T` is not const.

This batch, Lane A, was declared in a notice (`2db17f41`) and an addendum
(`79dc69ab`). Both were written under the integrator hold for batch 19's
join and approved before any code (section 1). It started on main
`4df942467` and has four steps:

- D0, located by code reading and eight rounds of declared probes
  (section 2);
- the approved fix: method applicability maps a `[const]` condition to its
  ordinary bound, because rustc's method lookup ignores constness. Of a
  `[const]` condition and an equal ordinary one, lookup keeps the first
  (amendment 2, section 3);
- 22 declared PA31 controls, the probe reruns and the flip survey
  (section 4);
- qualification on the declared sets and census 23 (sections 5 and 6).

Census 23 (pin `93fb6b89`) checks 42,310 declarations, with none failed
or blocked, and has a single record. Record 1 at `ptr/metadata.rs:251` is
gone. `local_annotations` now runs past it, through file 64, and stops at
`array/mod.rs:643` (file 98, byte 22026), inside core's `<[T; N]>::try_map`:
`drain::Drain::new(&mut me, &mut f)` reports "quantified clause requires a
scoped consumer". That is batch 21's lead (section 6).

Out of scope:

- Lane B owns `selection/search/*.rs`, `sizing_fallback` and
  `overlap/inputs.rs`; C3b owns `traits/implementations*`, `overlap*` and
  `selection/search/coherence.rs`; Lane C owns transport. This batch touches
  none of them, and no `selection::Caller` construction site.
- `trait_requirements` and its `selection::Caller` construction are
  byte-unchanged; only their lines move, to `applicability.rs:282` and
  `:306` (`:222` and `:246` before this batch). By design they receive the
  mapped ordinary goals: that is the method-call half of the fix. Lane B's
  placeholders row touches that Caller site, not `declaration_conditions`.
- The static `Sync` check in `validation.rs` and `constrained_scalars.rs`
  did not change. LIB-061 stays deferred.
- The CPPGM backend and its configuration did not change.
- `capability()`, `equations.rs`, impl validation, `callee_premises`,
  `bodies/bounds.rs` and the std prelude model did not change.

## 1. Declarations and base runs

Everything below was declared in `obj/parallel/census-batch20/` before the
run it governs:

| File | sha | Content |
|---|---|---|
| `notice.txt` | `2db17f41` | D0 location, option A, its gate and cost, the declared controls and census 23 |
| `notice-addendum-1.txt` | `79dc69ab` | the grep gate, the binder controls, b1, the survey's allowed movements |
| `probes/r1` to `r8` `declared.txt` | `e11b93c9`, `ebb503dc`, `2f21c728`, `046a03d6`, `af0c3332`, `ca427026`, `69efc9a9`, `22b98ec3` | each probe round, before its run |
| `fixtures/declared.txt` | `85b8ab98` | the 22 controls' rustc, P3 and candidate outcomes, before the oracle |
| `qualify/declared-qualification.txt` | `077b8d56` | the qualification sets, before any candidate |
| `declared/amendment-1.txt` | `727c3173` | the invariant file is `supertrait_lookups.rs` |
| `survey/declared.txt` | `cb4342e8` | the flip survey's inputs and allowed movements |
| `probes/rerun-declared.txt` | `752185e5` | the probe reruns on the candidate |
| `d0/declared.txt` | `f4859f46` | the work-invariant diagnostic, after `fc-1` failed |
| `declared/amendment-2.txt` | `a897c915` | A-dedupe: rule, footprint, cost, the extended invariant and the M = 0 base comparison |

`decisions.txt` records the integrator's decisions:

- Option A was approved with a grep-plus-comment gate.
- b2 to b5 are declared binder controls, predicted identical after the fix.
  b1 is the third tracked accepts-invalid (Open items).
- c19's text was frozen as first observed.
- The survey allows only c11 and c14 (accept to reject) and c01, c02, c15
  and c16 (reject to accept) to move. Anything else stops the batch.
- The std-mode E0034 accepts-invalid is tracked in docs only.
- After `fc-1`, the integrator chose A-dedupe (amendment 2).

The base pin `pins/base-4df942467/rustgm` (`1f69d999`) was built from clean
`4df942467` with an empty patch. Census 21's pin `30d6eb8a` was retired
when `4df942467` reached main (`retired-census21-pin.txt`).

## 2. D0: where the candidate is dropped

`declaration_conditions` (`dev/src/sema/methods/selection/applicability.rs`)
iterates `predicates_in_scope(owner)`. For a trait method that scope holds
the trait's supertraits. Before this batch it pushed every `Trait`,
`ConstTrait`, `MaybeConstTrait`, `ProjectionEqual` and `Auto` predicate as a
goal with no effect mapping, so `Self: [const] Eq` became the conditional
goal `*const VTable: [const] Eq`.

- `capability()` (`traits/selection/instantiation.rs:269`) keeps an
  ordinary impl ordinary, and `types/infer/predicates/equations.rs:232`
  refuses a non-ordinary goal whose constness differs from the candidate's.
  The impl candidate is absent.
- Type-qualified form: `paths.rs:282` calls `prepare`, then
  `paths/conditions.rs:57` returns absent, and `implicit_methods.rs:85`
  reports "no applicable trait method for this prefix".
- Method-call form: `receiver.rs:423` and `:515` call `applicable`, and
  `receiver.rs:156` reports "no applicable method for this receiver".
- A parameter candidate fails the same way: an ordinary `T: Cmp` premise
  elaborates to ordinary supertraits, which cannot prove the conditional
  goal (c15, c16).

Only `[const] Super` through an ordinary impl or an ordinary premise fails.
A plain supertrait is an ordinary goal and matches a candidate of any
constness (c03, c04). A const trait without `[const]` supertraits pushes no
conditional goal (c05, c06). A const impl adopts the requested effect
(c07b, c08b). A `T: [const] Cmp` premise elaborates to `[const]`
supertraits (c17, c18). rustc 1.98.0 accepts every non-const use and
rejects the const uses at the call with E0277 (c09, c19, c20). That is
consistent with probing that ignores constness, followed by an effect check
in const contexts.

Impl validation is not affected. `instantiation.rs:147` maps an impl's
conditions through `effects::in_context` under the use's effect, and a
body's chosen callable is proven the same way (`bodies/bounds.rs:507`,
`callee_premises.rs:96`). Method applicability was the one consumer of
`MaybeConstTrait` predicates that mapped no effect.

The integrator took the code reading and probes as D0, with the declared
flips as confirmation. The probe rounds and their header repairs
(the `const impl` syntax in r2, the `freeze` lang item in r3, and the
`copy` and `clone_fn` lang items in r7 and r8) are recorded in each
`roundN.txt`.

## 3. The fix

### Rule

Method lookup ignores constness, as rustc 1.98's probe does. In
`declaration_conditions`' push branch, a condition is mapped through
`effects::in_context(.., Constness::Ordinary)` before instantiation, so a
`[const]` condition becomes its ordinary bound. The chosen callable still
proves its complete clause under the use's effect, so a call in a const
context is still rejected (c09, c19, c20).

- Gate: `grep -rnw declaration_conditions dev/src` gives one call, in
  `prepare`. `prepare` has two callers, `applicable` (from `receiver.rs:423`
  and `:515`) and `paths.rs:282`. A comment at the mapping names both
  lookups.
- `Bound::ConstTrait` (`const Tr`) is unchanged.
- The mapping runs after `.ordinary()?`, so a `ForAll` clause keeps today's
  path. Core 1.98 has no `[const]` supertrait under a binder. b2 to b5 are
  the binder controls.

### A-dedupe (amendment 2)

A trait's `[const] S` supertrait is also in scope as its self bound's
elaborated ordinary consequence `Self: S`. After the mapping, both are the
same ordinary goal. The first focused run, `fc-1`
(`1794232866347702137`), failed the declared work invariant at depth 1:
probes and steps `(2, 509)` against the plain twin's `(2, 365)`. The
22-case outcome test passed. The declared diagnostic (`d0/observed.txt`,
`55b379a3`, rows reverted) confirmed this hypothesis, H-dup: the scope is
`[const] S0`, `PointeeSized`, `Cmp`, `S0`, where the twin's is `S0`,
`PointeeSized`, `Cmp`, so every probe proved `S0` twice, 72 extra steps per
probe. Outcomes were unaffected. `fc-1` is recorded as a failed declared
invariant.

The integrator chose A-dedupe:

- **Rule.** A trait condition is skipped only when it equals, by canonical
  value, a condition mapped from `[const]` in the same scope, in either
  order. Of such a pair, the later one in scope order is skipped and the
  first keeps its position. No other duplicate is compared: two ordinary
  conditions, two `[const]` conditions, `const`, auto and projection
  conditions are never skipped, and a scope without a `[const]` condition is
  unchanged. A general dedupe would change the work of non-const lookups.
- **Footprint.** One `let mut mapped = Vec::new();` before the loop.
  Everything else is in the push branch: a `[const]` condition scans the
  scope up to its own clause for an equal ordinary clause, and an ordinary
  `Trait` condition is skipped when `mapped` holds it.
- **Cost.** The scan visits and comparisons are added to
  `body_work.canonical_steps`. Each pair of scope positions with a `[const]`
  member is paid at most once, so a call with P predicates and M `[const]`
  conditions pays at most M(2P − M + 1)/2 ≤ P·M steps, and nothing when
  M = 0.

One declared `cfg(test)` `copy_trace` row per trait condition reports the
scope's P and M, the charge, and whether the condition was kept. The only
other production-file `cfg(test)` row is the invariant's module row.

## 4. Controls, probes and the survey

### Controls (`pa31/tests/spec/const-supertrait-lookup/`)

Twenty-two no_core libraries on pa31's `spec/function-pointers/foundation.rs`
prefix, generated by `tools/gen_fixtures.py`. Their outcomes were declared
(`fixtures/declared.txt`) before the frozen rustc 1.98.0 oracle of their
exact bytes and before any RustGM run. `outcomes.json` records them
(declaration sha `f94ec580`, 22 of 22 agree):

- 16 accepts, both lookup forms: record 1's shape, the generic `T: Cmp`
  form, the plain hierarchy, a plain const trait, const impls from ordinary
  and const fns, `T: [const] Cmp` callers, and the four binder controls.
- E0277: the call in a const fn, in a const item, and under an ordinary
  premise in a const fn (`generic-const-fn`, c19).
- E0034: the `[const]` hierarchy's `cmp` beside a plain `Seq::cmp`, in both
  forms, and the plain twin of that pair.

On P3 the four flips were rejected, and the two `[const]` ambiguity rows
were accepted: an accepts-invalid with record 1's root. On the candidate,
every row matches rustc. The E0277 rows are now rejected in bodies with
"trait obligation requires an unambiguous matching assumption or
implementation". That is c19's text, frozen as first observed.

PA31 is a planned assignment with no graded manifest. The controls are
registered by the `pa31/tests/README.md` paragraph and checked by the private
invariant `dev/tests/invariants/supertrait_lookups.rs`.

### Invariants and bounded work

- `const_supertrait_lookups_match_declared_outcomes`: the 22 outcomes.
- `const_supertrait_lookup_work_equals_its_plain_twin`, unchanged from the
  notice: a `[const]` hierarchy of depth D pays its plain twin's probes and
  steps, `(2, 365)`, `(2, 1373)` and `(2, 9437)` at D = 1, 8 and 64.
- `const_supertrait_lookup_conditions_equal_the_plain_twin`: for each lookup
  form in its own source, the kept conditions equal the twin's in content
  and order, with D + 2 kept rows in one call. The charges are 4, 116 and
  6,304 at D = 1, 8 and 64, as predicted (D(D+1)/2 + 2D + D²), against
  P·M = 4, 144 and 8,320. The plain twin has M = 0 and is charged nothing.
- `plain_supertrait_lookup_counters`: the plain twin's 26 work counters
  (six method-selection counters and every `BodyWork` counter) at each
  depth. The same test file was run against `4df942467`'s
  `applicability.rs` plus only the module row (fastcheck
  `1794235214942594088`). Its lines (`a2/base-counters.txt`, `096c5b2b`)
  are byte-identical to the candidate's (fastcheck `1794235489075551343`).
  So with M = 0 every counter equals the base's. Fastcheck captures a
  passing test's output, so both lines were read by rerunning each receipt's
  own hash-checked lib binary with `--nocapture` (`a2/procedure-note.txt`).
- Observed, not asserted: the whole-analysis `canonical_steps` differ
  between the twins by 8, 232 and 12,608, exactly two calls' charges.

### Probe reruns

On the candidate pin, every probe round reran byte for byte against P3's
recorded outputs (`probes/rerun-observed.txt`, `7992bf71`). The rustc
columns are byte-identical. Only the declared rows moved: c01, c02, c15 and
c16 to accept; c11 and c14 to E0034-shaped rejections; and c09, c20 and
both c19 rows to the bodies obligation text. The std rows s1 to s8 and the
binder rows b1 to b5 are unchanged.

### Flip survey

`survey/inputs.txt` lists every committed `pa*/tests` input at `4df942467`
whose bytes contain `[const]`, `const trait`, `const impl`, `derive_const`
or `const_trait` (3,299 files), plus the 22 new controls. The base pin and
the candidate pin each ran all 3,321 (`tools/survey.py`;
`survey/observed.txt`, `b003a33f`):

- Every committed input's status, stdout and stderr are byte-identical.
- Nine rows moved, all in the new packet and all declared: four rejected
  rows now accepted, two accepted rows now E0034-shaped rejections, and
  three E0277 rows with the new stderr text.

## 5. Qualification

The candidate tree is `4df942467` plus the uncommitted fix, the invariant
and the fixtures. It was qualified one fastcheck at a time
(`tools/qualify.sh`, `candidate/`) against the declared sets
(`qualify/declared-qualification.txt`, `077b8d56`). Every receipt passed with
sources equal to the tree (`tools/verify-qualification.py`,
`candidate/verify.txt`):

| Set | Fastcheck | Tests |
|---|---|---|
| lib chunk 00 | `1794236486565208337` | 1,000 |
| lib chunk 01 | `1794236617635026163` | 1,000 |
| lib chunk 02 | `1794236690795281887` | 998 |
| `suite_traits` roots, part 00 | `1794236780903627205` | 129 |
| `suite_traits` roots, part 01b | `1794236931323462950` | 227 |
| `suite_projections` roots | `1794237198773215393` | 131 |
| `suite_regions` roots | `1794237334813924243` | 142 |
| `suite_inference` roots | `1794237421771913898` | 149 |
| pa31 frontend roots (53) | `1794237555628009094` | 265 |
| pa31 other roots, part a | `1794238101681492619` | 115 |
| pa31 other roots, part b | `1794238587073446386` | 96 |
| PA19 to PA24 course text and `census_mode` | `1794237922376090060` | 1,199, 1,482, 851, 356, 415 and 490 text cases; 12 tests |

- **Lib set.** Every non-ignored lib test of the candidate, 2,998 of 3,001,
  from the candidate's own `lib-inventory` and `lib-ignored`
  (`qualify/lib-set.txt`). The other three are ignored native tests. It is
  batch 19's 2,978 plus this batch's four tests and the 16 that main added
  after `4ac1361c3`.
- **Roots.** Batch 19's five lists, unchanged, plus `roots-pa31-frontend`
  (53 roots) and `roots-pa31-other` (`compiler_recipes`, `crate_linking`,
  `generic_functions`, `negative_literal_patterns`). Each of the 190 listed
  roots executed at least one test.
- **pa31 other.** Its first run, `1794237641425758820`, executed all 211
  selected tests to a pass, then exhausted the wall-clock budget (`make`
  exit 124) at a host load average near 28. A timeout fails, so that
  receipt is not used. The same 211 tests ran as two fastchecks, with
  `crate_linking` split by exact test name (`qualify/note-pa31-other.txt`,
  written before the rerun).
- **No `--pa pa31`.** PA31 is a planned assignment without a graded
  manifest. Its controls are qualified through the private invariant and
  the roots above, and its committed inputs through the flip survey.

No status flipped beyond the declared control flips. The focused checkpoint
was `1794235489075551343` (`fc-2`, five tests). It is not a qualification.

## 6. Census 23

Census 23 uses the pin `obj/parallel/census-batch20/qualified-1/pin/`
(`93fb6b89`). `run.py build` built it, bound to `fc-2`'s receipt, and
`run.py census` is bound to the twelve qualification receipts. Both carry
the uncommitted patch `54174f08`, the `applicability.rs` diff; the sources
fingerprint also covers the new invariant file. The manifest (`e1951bc7`)
chains from census 22's (`d581a71e`), and only `expected_cli_sha256` and
`predecessor_manifest_sha256` changed. The ordinary observation's first
error is `array/mod.rs` byte 22026. The census
(`qualified-1/census/census.tsv`, sha `91c6af8e`) checks 42,310
declarations, with none failed or blocked, and has one record:

- Record 1 at `ptr/metadata.rs:251` is gone, as declared.
  `local_annotations` runs past file 64.
- The new crate record is `local_annotations` at `array/mod.rs:643`
  (file 98, byte 22026), in `<[T; N]>::try_map`:
  `drain::Drain::new(&mut me, &mut f)` reports "quantified clause requires a
  scoped consumer". `Drain::new` is a `const unsafe fn` of
  `impl<'l, 'f, T, U, F: FnMut(T) -> U> Drain<'l, 'f, T, F>`
  (`array/drain.rs:5`). It is called from the const fn `try_map`, with `F`
  the argument-position `impl [const] FnMut(T) -> R + [const] Destruct`.
  The text comes from `Clause::ordinary` on a quantified clause
  (`types/parameters/clauses.rs:166`) or `sema/destructors.rs:213`. Which
  consumer it is was not traced; that is batch 21's first step.
- Bodies are still not reached: the census stops at the crate record, so
  the local annotations after it and the bodies phase are not covered.

## Checks

- Fastchecks: as in section 5. The focused run `fc-1`
  (`1794232866347702137`) failed only the declared work invariant (H-dup,
  section 3). The base observation `1794235214942594088` and `fc-2`
  `1794235489075551343` passed.
- Probe reruns: every round byte-equal to P3's outside the declared moves
  (section 4).
- Flip survey: 3,299 committed inputs byte-identical; nine declared new
  packet rows moved (section 4).
- `make test-upstream`: 128 pinned upstream fixtures and 126 Rust 1.98.0
  Reference files verified.
- Census 23: `run.py build` (ordinary first error `array/mod.rs` byte
  22026), then `run.py census`, bound to the twelve qualification receipts.
- Pending in scheduled or manual CI: full release regular and native tests,
  uncached audits, all-target strict Clippy and the harness.

## Open items

- Tracked in [library-build-blockers.md](../library-build-blockers.md):
  - the std-mode E0034 accepts-invalid (s6 to s8): a user trait's `cmp`
    does not compete with the std prelude's `Ord::cmp`;
  - the std-mode rejects-valid (s2 to s5): the std prelude model offers no
    comparison impls for raw pointers;
  - b1, a `[const]` where clause on a method of a non-const impl, accepted
    where rustc 1.98.0 rejects it ("`[const]` is not allowed here"). The
    root is `[const]` placement checking, not method lookup.
- **Census 23's record 1, batch 21's lead** (section 6): `local_annotations`
  at `array/mod.rs:643`, "quantified clause requires a scoped consumer" on
  a type-qualified call of `Drain::new`, a `const unsafe fn` whose impl
  has an `F: FnMut(T) -> U` bound, from the const fn `try_map`.

## Storage

- Kept: census 23's pin `obj/parallel/census-batch20/qualified-1/pin/`
  (`93fb6b89`, with `.keep-artifacts`), the new census baseline. Census 22's
  pin `7df1f865` and the survey base pin `pins/base-4df942467/` (`1f69d999`)
  stay pinned until this batch is on main. They are then retired by
  removing their `.keep-artifacts` and pruning their directories.
- Analysis-only pin built and retired: `candidate-1` (`067a5b28`, the fix
  without the dedupe). Its outputs remain under `fixtures/out-c1`.
- obj stayed under 16 GiB throughout. The reserve and the cap were not
  changed.
