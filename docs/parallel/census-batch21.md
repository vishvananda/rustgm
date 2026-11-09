# Census batch 21: method lookup reads an empty callable binder as its body

Census 23 ([batch 20](census-batch20.md), pin `93fb6b89`, `census.tsv` sha
`91c6af8e`) checked 42,310 declarations of original core, with none failed or
blocked, and one record. Its crate record is `local_annotations` at
`array/mod.rs:643` (file 98, byte 22026), inside `<[T; N]>::try_map`:
`drain::Drain::new(&mut me, &mut f)` reports "quantified clause requires a
scoped consumer". `Drain::new` is a `const unsafe fn` of
`impl<'l, 'f, T, U, F: FnMut(T) -> U> Drain<'l, 'f, T, F>`
(`array/drain.rs:5`).

This batch, Lane A, was declared in a notice (`ab59bd13`) written under the
integrator hold for batch 20's join and approved with fix A before any code
(section 1). It started on main `0bcba9e3f` and has four steps:

- D0, located by code reading and four rounds of declared probes
  (section 2);
- the approved fix: method lookup reads an empty binder, one that quantifies
  no lifetime and captures nothing outside it, as its body (fix A). Of such a
  condition and an equal flat clause of the same scope, lookup checks the flat
  clause once (D1, section 3);
- 22 declared PA31 controls and the flip survey (section 4);
- qualification on the declared sets and census 24 (sections 5 and 6).

Census 24 (pin `88273b11`) checks 42,310 declarations, with none failed
or blocked, and has a single record. Record 1 at `array/mod.rs:643` is gone.
`local_annotations` now runs past it and stops at `iter/traits/iterator.rs:698`
(file 175, byte 23994), inside `Iterator::intersperse`:
`Intersperse::new(self, separator)` reports "candidate properties require
associated type normalization" (section 6).

Out of scope:

- Lane B owns `selection/search/*.rs`, `sizing_fallback` and
  `overlap/inputs.rs`; C3b owns `traits/implementations*`, `overlap*` and
  `selection/search/coherence.rs`; Lane C owns transport. This batch touches
  none of them, and no `selection::Caller` construction site.
- `generics/traits.rs` (`elaborate_trait_bounds`) did not change; see D3 in
  Open items.
- The static `Sync` check in `validation.rs` and `constrained_scalars.rs` did
  not change.
- The CPPGM backend and its configuration did not change.

## 1. Declarations and base runs

Everything below was declared in `obj/parallel/census-batch21/` before the
run it governs:

| File | sha | Content |
|---|---|---|
| `notice.txt` | `ab59bd13` | D0 location, fix A, the alternatives, the controls and survey |
| `probes/r1` to `r4` `declared.txt` | `37e13264`, `a75f098e`, `fac77a20`, `c76fb2ff` | each probe round, before its run |
| `fixtures/declared.txt` | `408efeb6` | the 22 controls' rustc, base and candidate outcomes, before the oracle |
| `survey/declared.txt` | `b18ec69d` | the flip survey's inputs and allowed movements |
| `declared/invariants.txt` | `756ac241` | the invariants (F), (i), (ii) and (iii) and their placement |
| `declared/amendment-1.txt` | `36d82f2d` | the invariant file is `empty_binder_lookups.rs` (filename stem rule) |
| `declared/amendment-2.txt` | `9e5e3e99` | (i)'s row count includes elaborated supertraits |
| `ii/procedure.txt` | `37db7a76` | the (ii) base observation and reread |
| `qualify/declared-qualification.txt` | `6c39ca98` | the qualification sets, before any candidate |
| `declared/amendment-3.txt` | `7390eb05` | D1, its charge, the scaling assertion and its `cfg(test)` row |
| `declared/amendment-4.txt` | `2223b9c7` | (i)'s bounds and capture layout, labelled errors |
| `declared/census-24.txt` | `abfa0fc6` | the census 24 procedure |

`decisions.txt` records the integrator's decisions:

- Fix A was approved. A+O (also opening a quantified lifetime) waits for
  f01 to close, and B was rejected.
- The approved flips, by name, are d01, d02, d02b, d03, d04, d09, d10, e05
  and e06 (reject to accept). The approved stderr moves are e01 and e01b,
  both still rejected. d07, d08, e04, e06b, e06c and e08 are known
  rejects-valid under the bound-region rule.
- After `fc-1`, D1 and amendment 3 (section 3). After `fc-2` and `fc-3`,
  the reader repair and amendment 4 (Checks).

The base pin `pins/base-0bcba9e3f/rustgm` (`b53683f1`) was built from clean
`0bcba9e3f` with an empty patch. Census 22's pin `7df1f865` and batch 20's
survey base pin `1f69d999` were retired when `0bcba9e3f` reached main
(`retired-pins.txt`).

## 2. D0: where the candidate is dropped

- **Emitter.** `Clause::ordinary` on a quantified clause
  (`types/parameters/clauses.rs:166`), called from `declaration_conditions`
  (`dev/src/sema/methods/selection/applicability.rs`). The destructor site
  `sema/destructors.rs:213` rejects only a quantified outlives body; adding
  core's `const impl Drop for Drain` (d10) changes nothing.
- **Route.** `Drain::new` is an associated path with a nominal head.
  `complete_method_path` takes a unique inherent candidate directly only
  when no trait declares a method of that name. Core's `trait ZipImpl`
  (`iter/adapters/zip.rs:127`) declares `fn new`, so the path goes through
  `PathQuery::inherent` and `prepare` to `declaration_conditions`, with the
  impl as owner. The method-call form reaches the same function for every
  inherent candidate (`receiver.rs`).
- **The binder.** `impl<'l, 'f, T, U, F: FnMut(T) -> U>` writes no binder,
  but callable sugar always opens a bound scope
  (`generics/bounds/lexical.rs` `call_bound_scope`, then `bound_clause`).
  `F: FnMut(T) -> U` is published as two zero-region `Clause::ForAll`
  clauses, `F: FnMut<(T,)>` and `<F as FnOnce<(T,)>>::Output == U`.
- **Controls** (round 1). The effect, the unsafe call, the `&mut` borrow and
  the destructor do not matter (d01 to d04, d10). The unique route (d05) and
  a plain `F: Call<T, Out = U>` bound (d06) pass. An explicit `for<'a>` bound
  (d07), `FnMut(&T) -> U` (d08) and the method-call form (d09) fail the same
  way.
- **rustc** (rounds 2 to 4). An inherent candidate whose sugar condition does
  not hold is rejected by the probe, and the path falls back to a trait
  method of the same name (e06, e06b, e06c). The chosen callable's own proof
  still rejects a failing condition (e01, e01b, e02, f02).

The integrator took the code reading and the probes as D0.

## 3. The fix

### Fix A

`lookup_condition` reads a `Clause::ForAll` whose `bound_regions` is 0 and
whose body has no escaping binder as its body, the same predicate. Both checks
are stored properties, O(1). Every other quantified clause still errors with
"quantified clause requires a scoped consumer", so the bound-region shapes
keep their outcome by name. The three `declaration_conditions` reads that used
`Clause::ordinary` (the condition, batch 20's `[const]` twin scan and its
trace row) use `lookup_condition`.

### D1 (after `fc-1`)

The first focused run, `fc-1` (`1794244393032956033`), failed declared
invariant (i) at one bound: the sugar impl kept 11 lookup conditions against
its flat twin's 9. `elaborate_trait_bounds` lifts a quantified clause's
implied consequences into the same binder, so a sugar bound also publishes
`ForAll(0, F: MetaSized)` and `ForAll(0, F: PointeeSized)`. These are not
deduplicated against the flat `F: MetaSized` and `F: PointeeSized` that F's
implicit `Sized` elaborates to. Fix A flattened them into two duplicate goals
per sugar bound. The 22 outcomes were unaffected (`fc-1-observed.txt`,
`9a9fd8a9`).

The integrator chose D1, in `applicability.rs` only:

- **Rule.** A condition read from an empty binder that equals, by interned
  predicate identity, a flat clause of the same scope is skipped; the flat
  clause is read at its own position. No other duplicate is compared.
- **Cost.** A lookup-local reader, `ScopeConditions`, builds the scope's flat
  set on the lookup's first such read: P steps, one per scope clause. Each
  read is one more step. All of it is charged to `body_work.canonical_steps`.
  A lookup pays P + M for M reads, so L lookups cost L·(P + M). A scope with
  no empty binder builds nothing and pays nothing. The reader is built per
  `declaration_conditions` call and never shared.
- **Trace.** One declared `cfg(test)` `copy_trace` row per read reports P, the
  read's charge and whether it was kept (amendment 3). It is separate from
  batch 20's row, whose lines (ii) keeps byte-identical. The only other
  production-file `cfg(test)` rows are batch 20's trace row and the
  invariant's module row.
- **Size.** `declaration_conditions` is 99 lines by Clippy's count, against
  its limit of 100. The next batch that adds lines there factors it first,
  in its own behavior-preserving commit with a flip survey.

D3, lifting an empty binder's closed consequences as flat clauses in
`elaborate_trait_bounds`, would remove the duplicates at their source. It is
a candidate root fix and is not queued (Open items).

## 4. Controls, invariants and the survey

### Controls (`pa31/tests/spec/quantified-lookup-conditions/`)

Twenty-two no_core libraries on pa31's `spec/function-pointers/foundation.rs`
prefix, each byte-identical to a declared probe (`tools/gen_fixtures.py`,
`fixtures/sources.map`). Their outcomes were declared
(`fixtures/declared.txt`) before the frozen rustc 1.98.0 oracle of their
exact bytes and before any RustGM run. `outcomes.json` records them
(declaration sha `f73a56e7`, 22 of 22 agree):

- 12 accepts: record 1's shape and its non-const, safe, by-value, method-call
  and `Drop` twins, a sugar supertrait, the trait fallback, and three
  controls. The nine flips were rejected on the base and are accepted on the
  candidate.
- 6 known rejects-valid that rustc accepts, all bound-region:
  `explicit-binder`, `sugar-lifetime`, `split-shape` (core's `Split::new`),
  `trait-fallback-region`, `trait-fallback-leak` and
  `receiver-sugar-lifetime`. Their stderr is byte-identical between base and
  candidate.
- E0277 (`fail-inferred`) and E0599 (`fail-explicit`, `fail-unique`,
  `fail-region-unique`). `fail-inferred` now fails in bodies with "trait
  obligation requires an unambiguous matching assumption or implementation",
  and `fail-explicit` in `local_annotations` with "no applicable trait method
  for this prefix", as approved.

PA31 is a planned assignment with no graded manifest. The controls are
registered by the `pa31/tests/README.md` paragraph and checked by the private
invariant `dev/tests/invariants/empty_binder_lookups.rs`, mounted from
`supertrait_lookups.rs`.

### Invariants and bounded work

- (F) `quantified_lookup_conditions_match_declared_outcomes`: the 22
  outcomes.
- (i) `sugar_lookup_conditions_equal_the_flat_twin`. An inherent impl under N
  sugar bounds `Fi: FnMut(T) -> Ui`, against its flat twin
  `Fi: FnMut<(T,), Output = Ui>`, at N = 1, 4 and 8, in the type-qualified
  and method-call forms, with two callers each:
  - the kept conditions are equal in content and order, every row is kept,
    and the six method-selection counters are equal;
  - the twin reads no empty binder and pays nothing; each sugar lookup pays
    P + M, and the analysis pays L·(P + M), asserted below L·P·M at N = 4
    and 8;
  - exact three-point affine fits, the same in both forms: rows per lookup
    7N + 2, P = 9N + 2, M = 5N reads and 2N skipped reads (`MetaSized` and
    `PointeeSized`), with L = 2. The charges are 32, 116 and 228, against
    L·P·M = 110, 1,520 and 5,920.
  - Observed only: whole-analysis body work, which includes the chosen
    callable's own proof (`fc-4-observed.txt`).
- (ii) Batch 20's counter, work and charge lines, reread from `fc-4`'s
  hash-checked lib binary (`1cfe9350`), are byte-identical to the base's
  (`ii/base-lines.txt`, `080211da`, from `0bcba9e3f`'s fastcheck
  `1794244108076865560`). So with no empty binder in scope every counter
  equals the base's.
- (iii) The six bound-region controls fail by name, as (F) and the survey
  show.

### Flip survey

`survey/inputs.txt` lists every committed `pa*/tests` input at `0bcba9e3f`
whose bytes contain callable sugar or `for<` (1,086 files), plus the 22 new
controls. The base pin and the candidate pin (`pins/candidate-1`,
`e8dcf143`) each ran all 1,108 (`tools/survey.py`; `survey/observed.txt`,
`64f18e1d`):

- Every committed input's status, stdout and stderr are byte-identical.
- Eleven rows moved, all in the new packet and all declared: the nine flips
  to accept, and `fail-inferred` and `fail-explicit` with their approved
  texts.

## 5. Qualification

The candidate tree is `0bcba9e3f` plus the uncommitted fix, the invariants
and the fixtures. It was qualified one fastcheck at a time
(`tools/qualify.sh`, `qualified/`) against the declared sets
(`qualify/declared-qualification.txt`, `6c39ca98`). Every receipt passed with
sources equal to the tree (`tools/verify-qualification.py`,
`qualified/verify.txt`):

| Set | Fastcheck | Tests |
|---|---|---|
| lib chunk 00 | `1794247890418450258` | 1,005 |
| lib chunk 01 | `1794248049288293017` | 1,005 |
| lib chunk 02 | `1794248152219992964` | 1,000 |
| `suite_traits` roots, part 00 | `1794248228458305507` | 129 |
| `suite_traits` roots, part 01b | `1794248451380496273` | 227 |
| `suite_projections` roots | `1794248594106509041` | 131 |
| `suite_regions` roots | `1794248939229508769` | 142 |
| `suite_inference` roots | `1794248998739528164` | 149 |
| pa31 frontend roots (53) | `1794249066453115715` | 265 |
| pa31 other roots, part a | `1794249144569820413` | 115 |
| pa31 other roots, part b | `1794249314133244338` | 96 |
| pa31 other roots, part c | `1794249446367676622` | 5 |
| PA19 to PA24 course text and `census_mode` | `1794249489557675471` | 1,199, 1,482, 851, 356, 415 and 490 text cases; 12 tests |

- **Lib set.** Every non-ignored lib test of the candidate, 3,010 of 3,013,
  from `fc-4`'s own `lib-inventory` and `lib-ignored`
  (`qualify/lib-set.txt`). The other three are ignored native tests. It is
  batch 20's 2,998 plus this batch's two tests and ten that main added after
  batch 20's qualification.
- **Roots.** Batch 20's lists, unchanged, plus `roots-pa31-other-c`: the five
  `crate_linking` tests added since batch 20's split, in
  `called_pointers.rs` and `enabled_callables.rs`. The three pa31-other
  parts ran split from the start, as declared. Each of the 190 listed roots
  executed at least one test, and each part's executed count is the declared
  one.
- **No `--pa pa31`.** PA31 is a planned assignment without a graded
  manifest. Its controls are qualified through the private invariants and
  the roots above, and its committed inputs through the flip survey.

No status flipped beyond the declared control flips. The focused checkpoint
was `1794246727453069423` (`fc-4`, six tests). It is not a qualification.

## 6. Census 24

Census 24 uses the pin `obj/parallel/census-batch21/qualified-1/pin/`
(`88273b11`). `run.py build` built it, bound to `fc-4`'s receipt, and
`run.py census` is bound to the thirteen qualification receipts. Both carry
the uncommitted `dev` diff; the sources fingerprint also covers the new
invariant file. The manifest (`e870634f`) chains from census 23's
(`e1951bc7`), and only `expected_cli_sha256` and
`predecessor_manifest_sha256` changed. `run.py`'s census stage now accepts
status 0 or 1 (`declared/census-24.txt`). The ordinary observation's first
error is `iter/traits/iterator.rs` byte 23994. The census
(`qualified-1/census/census.tsv`, sha `7443bfac`) checks 42,310
declarations, with none failed or blocked, and has one record:

- Record 1 at `array/mod.rs:643` is gone, as declared. `local_annotations`
  runs past file 98.
- The new crate record is `local_annotations` at `iter/traits/iterator.rs:698`
  (file 175, byte 23994), in `Iterator::intersperse`:
  `Intersperse::new(self, separator)` reports "candidate properties require
  associated type normalization". `Intersperse::new` is a `const fn` of
  `impl<I: Iterator> Intersperse<I> where I::Item: Clone`
  (`iter/adapters/intersperse.rs:19`). It was not predicted.
- By code reading, not yet traced: the path takes the applicability route
  (core's `ZipImpl` declares `new`), `declaration_conditions` pushes
  `<I as Iterator>::Item: Clone` as a builtin `Clone` condition, and
  `builtin_requirements`' property query on that projection returns the text
  (`types/infer/properties.rs:70`). No quantified clause is involved, and the
  route and text exist on the base.
- The bound-region sugar sites, led by `slice/iter.rs`'s `Split::new` (used in
  `slice/mod.rs`, file 210), come after file 175 and were not reached.
- Bodies are still not reached: the census stops at the crate record.

## Checks

- Focused runs:
  - `fc-1` (`1794244393032956033`) failed declared invariant (i): fix A
    duplicated elaborated conditions (section 3). The ruling was D1.
  - `fc-2` (`1794245468988084266`) failed strict Clippy before any test ran:
    `declaration_conditions` had 126 lines with D1 written inline. It also
    reached the 16 GiB output cap. The approved repair moves D1 unchanged
    into the lookup-local `ScopeConditions` reader.
  - `fc-3` (`1794246140924823233`) passed everything except (i), which
    exceeded the trace capture's 256-row bound. The error carried no label;
    by code reading one analysis matches about 24N + 4 rows, so N = 64 was
    never compatible with the bound. Amendment 4 (option A) uses N = 1, 4
    and 8 and one capture per prefix, and labels every capture and analysis
    error.
  - The base observation `1794244108076865560` and `fc-4`
    `1794246727453069423` passed.
- (ii): byte-identical to the base (section 4).
- Flip survey: the 1,086 committed inputs byte-identical; eleven declared
  new-packet rows moved (section 4).
- `make test-upstream`: 128 pinned upstream fixtures and 126 Rust 1.98.0
  Reference files verified.
- Census 24: `run.py build` (ordinary first error `iter/traits/iterator.rs`
  byte 23994), then `run.py census`, bound to the thirteen qualification
  receipts.
- Pending in scheduled or manual CI: full release regular and native tests,
  uncached audits, all-target strict Clippy and the harness.

## Open items

- **Census 24's record 1** (section 6): `local_annotations` at
  `iter/traits/iterator.rs:698`, "candidate properties require associated
  type normalization" on a type-qualified call of `Intersperse::new`, whose
  impl has a `where I::Item: Clone` condition. The integrator names the next
  batch's lead.
- **f01, batch 22's lead-in** (tracked in
  [library-build-blockers.md](../library-build-blockers.md)): a higher-ranked
  inherent bound met only by a `'static` impl, `F: for<'r> Tr<'r>` with
  `impl Tr<'static> for u8`, is accepted on the unique route, where rustc
  1.98.0 reports E0599. The pin proves the bound from the `'static` impl; only
  rustc's leak check rejects it. Batch 22 closes f01; then A+O opens a
  quantified lifetime in lookup for the bound-region shapes, the six known
  rejects-valid here and the twelve bound-region sugar impls in core
  (`survey/inherent-sugar-impls.tsv`), led by `slice/iter.rs`'s `Split::new`.
- **f03a, low priority:** a const trait with a `[const]` callable-sugar
  supertrait, `pub const trait Run: [const] FnMut(u8) -> u8 {}`, is rejected
  with "ambiguous trait assumptions in declaration" in
  `validate_owner_bounds`, where rustc accepts it. Plain `[const] FnMut<(u8,)>`
  and ordinary sugar supertraits pass.
- **Inherent-impl `[const]` bounds** (probe g01, accepts-invalid):
  `impl<T, U, F: [const] FnMut(T) -> U> S<T, F>` is accepted, where rustc
  1.98.0 rejects it ("`[const]` is not allowed here"; inherent impls cannot
  have `[const]` trait bounds).
- Batch 20's `[const]` twin scan now reads clauses through
  `lookup_condition`. Across an empty binder it has no valid-program
  fixture: a valid program reaches it only through a const trait's `[const]`
  sugar supertrait, which f03a blocks first. f03a's fix owns adding that
  fixture.
- `methods/overlap.rs:111` still reads method conditions with `.ordinary()`.
  It is an open related site and was not changed.
- D3 is a candidate root fix and is not queued: lifting an empty binder's
  closed consequences as flat clauses in `elaborate_trait_bounds` would
  remove D1's duplicates at their source. It changes every consumer's
  elaborated scope, and possibly exported bytes.
- `declaration_conditions` is at 99 of Clippy's 100 lines (section 3).

## Storage

- Kept: census 24's pin `obj/parallel/census-batch21/qualified-1/pin/`
  (`88273b11`, with `.keep-artifacts`), the new census baseline. Census 23's
  pin `93fb6b89`, the survey base pin `pins/base-0bcba9e3f/` (`b53683f1`) and
  the survey candidate pin `pins/candidate-1/` (`e8dcf143`) stay pinned until
  this batch is on main. They are then retired by removing their
  `.keep-artifacts` and pruning their directories.
- obj reached the 16 GiB output cap once, during `fc-2`'s build, and was
  pruned and tidied before `fc-3`. The reserve and the cap were not changed.
