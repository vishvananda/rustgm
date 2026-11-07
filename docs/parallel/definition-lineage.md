# Definition side table, lineage ledger and impl-determined projections

Lane B batch of 2026-11-07 on top of `e84a85783`. It restores the expansion
`Definition` to 136 bytes and bounds the import-lineage ledger by its admitted
input. It repairs the largest group of the core census cause "associated type
requires an unambiguous implementation". It also applies std's two profile
decisions: std builds as an rlib only, and Rust 2015/2018 is implemented only as
far as cfg-if and rustc-demangle need.

## 1. The expansion `Definition` keeps 136 bytes

The macro metadata rows (`cbef4d469`) put `imported: Option<ImportedDefinition>`
(32 bytes) inline in every expansion `Definition`, so local definitions paid for
imported ones and the record grew to 168 bytes. Eight private layout invariants
assert 136.

Imported macro definitions are always installed first: `Engine::import_macros`
pushes them, in row order, right after `Engine::create`. Definition index `i`
below the installed count is therefore exactly bound row `i`. The bound row
(`crates::ImportedMacro`, already in the `Imported` cold box) now also keeps the
defining crate's id and the transported image's `FileId`. `install_macro`
records the image when the definition is pushed, and checks three things: rows
install in order, the row exists, and only declarative rows carry an image.
`transported_macros::imported(facts, index)` replaces the inline field at
every reader: the site scope, the spelling, diagnostics frames, permission
frames and sessions, unstable-call validation, panic dispatch, the derive
pairing's crate id and `local_inner_macros`.

- `Definition` is 136 bytes again and the `Imported` record keeps 160 bytes.
  The row grows inside the existing cold box at bind time, uncharged like the
  rows it extends. Installation allocates nothing.
- Callers that only had a `&Definition` now pass `(index, &Definition)`.
  `Engine::introduce` wraps the common `paths.invocation` call.
- The loader's separate `admitted: HashSet<PathBuf>` (32 bytes in `Engine`,
  from `054b91950`) is folded into its source cache. `cache` now maps every
  admitted identity to its parsed module unit, or to `None` for an identity
  that was only included. Each identity still grants once.
- Every exported macro table is byte-identical. The 72-case native runner
  publishes the same 27 `.rgm` hashes as the baseline run on main's tree
  (`obj/parallel/pa29-macro-metadata/maintainer-882970-true-1794072148518787485`,
  integration branch `c944b393b`, which differs from main only in docs).

Amended controls:

- `derived_locations` and `modifier_storage` assert `Report` 3104, not 3096,
  as the integrator accepted. `modifier_storage` asserts `Engine` 4152, not
  4144 (section 8). Each comment names the lineage grant word.
- `chosen_authority` builds its `Definition` without the removed `imported`
  field and passes `(0, &definition)` to `paths.invocation`. This is a
  mechanical edit; the asserted behavior is unchanged.

## 2. Import-lineage ledger

### Attribution (analysis only)

An analysis-only patch (`obj/parallel/definition-lineage/lineage-attr/apply.py`,
never committed) labelled every `lineage_budget` charge by caller. It also
dumped the retained edges of each import. The census of original core used the
same inputs and flags as census 4 and reproduced it: 42,109 declarations
checked, 74 failed, 127 blocked. Its records are in
`obj/parallel/definition-lineage/lineage-attr/census-1/`. Lane A's whole-run
attribution gives the total: 4,068,681 steps and 54.8 MB. That is 0.28 steps
and 3.74 attempted bytes per admitted byte (14,668,689 bytes).

| Payer, to the end of `finish_imports` | Calls | Steps | Attempted bytes |
|---|---|---|---|
| Glob member selection (`imports.rs:606` → `record_source` → `Lineage::record`) | 471,556 | 3,164,710 | 53,933,792 |
| Glob contribution withdrawal (`imports.rs:688` → `remove_source`) | 209,512 | 209,512 | 0 |
| Single-import selection (`imports.rs:494`) | 5,507 | 20,856 | 400,344 |
| Other (`imports.rs:528`) | 8,123 | 8,123 | 0 |
| **Subtotal** | | **3,403,201** | **54,334,136** |

Hidden-glob completion pays the remainder (about 0.67M steps, almost no
storage). The 238,444 retained edges come from 2,686 imports. 198 globs hold
235,771 of them, and 146 stdarch `core_arch` globs alone hold 227,320. Each
x86 intrinsic module glob-imports its parent's namespace of about 6,600 names.

Every payer is linear in its edges: one constant-work record per edge, plus
amortized map and vector growth charged as attempted bytes. The edges are the
resolution's own glob fan-out, which rustc's resolver also materializes. No
payer repeats work per edge.

### Bound

The lineage ledger now follows the lint-policy ledger:

- the floor is the former fixed limits, 10,000,000 steps and 64 MiB;
- each admitted source identity grants a fixed number of steps and attempted
  bytes per original byte;
- the import worklist is granted every source admitted so far when it is
  created (`imports::Work::new(facts, admitted)`), before its first charge.
  The engine grants it again after each module load and `include!`
  (`Engine::grant_admissions`, which also grants the lint ledger);
- an exhausted ledger is never granted again (`producer::Work::admit`);
- expansion output is not an admission and grants nothing.

The two ledgers share `producer::Work`. A zero-sized `Factors` marker
(`producer::Lint`, `producer::Lineage`) selects the per-byte factors, so neither
record grows. The lint factors stay 8 steps and 64 bytes. The lineage factors
are 8 steps and 128 bytes, sized from the densest linear import families:

| Family (512 → 4096 edges) | Steps/B | Attempted B/B | Per edge |
|---|---|---|---|
| Original core | 0.28 | 3.74 | |
| Single imports, one per item | 0.84 → 0.80 | 22.8 → 21.6 | 23.6 → 23.5 steps, 640 → 637 B |
| Glob blocks: 8 modules import one 8-item module | 2.32 → 2.40 | 40.8 → 42.0 | 13.0 → 13.0 steps, 228 → 227 B |
| Glob re-export chain | 0.57 → 0.54 | 16.8 → 16.1 | 21.6 → 21.5 steps, 640 → 637 B |

The factors leave 3.3× (steps) and 3.0× (bytes) of headroom over the glob
blocks, and 28× and 34× over core. For core the allowance is 127M steps and
1.94 GB attempted. The first run asserted the lint factors and failed the glob
blocks at 40.8 bytes per byte (`fast-t2-private-2.lib.stdout`). The lineage
factors were then separated and resized.

### Controls

Declared before the runs, in `limit_tests`
(`dev/tests/invariants/exhaustion_limits.rs`):

- `lineage_ledger_grants_each_admitted_identity_once_before_reading_it`: the
  floor, the factors and the grant at 1, 4 and 32 copies of aliased modules and
  includes, plus macro-generated requests that grant nothing.
- `lineage_grants_only_grow_and_never_reset_an_exhausted_ledger`.
- `linear_import_families_scale_and_square_fanout_still_fails`: the three
  families stay within half of each factor, and per-edge work changes by at
  most 5/4 from 512 to 4096 edges. The opponent is a square fan-out:
  `width` modules each glob-import one module of `width` items. Doubling the
  width from 64 to 128 doubles the admitted bytes (2,790 → 5,406) and
  quadruples the work (48,781 → 193,801 steps, 0.72 → 2.81 MB). At width 1024
  expansion fails with "source lint policy storage budget exceeded".

Amended controls:

- `import_lineage::imported_missing_edges_never_become_direct_and_failed_work_remains_paid`
  set the expansion ledger's steps to the literal 10,000,000 floor. It now
  reads the ledger's actual limit and asserts the grant equals the admitted
  bytes. The asserted behavior is unchanged.
- `item_journals` (4 tests), `item_globs` (2) and
  `glob_payer::completion_checks_both_failed_owners_and_the_actual_cap_before_empty_results`
  build `imports::Work` directly. They pass `0`, the bare floor, like the lint
  tests that build `Producer::new`.
- `lint_limits` names `Work::<Lint>`, since a struct expression does not apply
  the default marker.

## 3. Impl parameters that only callable sugar determines

Census 4's largest cause, "associated type requires an unambiguous
implementation", has 35 failed and 26 blocked records. Grouped by root cause
from its records (`rustgm-pa29-work/obj/parallel/core-census-4/`):

| Group | Failed | Sites |
|---|---|---|
| Impl parameter determined only by a callable-sugar `Output` | 14 | `filter_map.rs` ×5, `map.rs` ×4, `map_while.rs`, `from_fn.rs`, `repeat_with.rs`, `scan.rs`, `map_windows.rs` |
| Specialization (`default fn`, `spec_*` traits) | 14 | `fuse.rs` ×4, `step_by.rs` ×7, `zip.rs` ×3 |
| A user callable-trait impl's own `Self::Output` | 5 | `array/drain.rs` ×2, `sync/sync_view.rs` ×3 |
| Other | 2 | `ops/control_flow.rs:427`, `iter/mod.rs:376` |

The first group is core's `impl<T, F> Iterator for FromFn<F> where
F: FnMut() -> Option<T>` shape: `T` appears only in the sugar's `Output`
equality. Parenthesized sugar always lowers through `bound_clause`, which makes
a `Clause::ForAll` whenever a lexical binder scope exists, even one that binds
no region. Impl instantiation routed every `ForAll` to the quantified
conditions, which run only after `resolved_parent`: universal children have
isolated inference and cannot refine the parent. `T` therefore stayed an
unresolved variable, the trial stayed deferred and `Self::Item` had no single
implementation. Probes isolated it: the same impl with an explicit
`F: FnOnce<(), Output = Option<T>>` passed, and every sugar form (where-clause
or inline, `FnMut` or `FnOnce`, in the impl or from a caller premise) failed.

### Repair

`selection::instantiation::instantiate` now lowers a quantified clause whose
binder binds no region out of that binder (`Bound::lower(1)`, one fold shared
by every such clause and charged with the instance's fold work) and adds it to
the ordinary conditions. The lowered body is the clause written without a
quantifier: the impl's captured lifetimes move from binder depth 1 to 0, and
the later capture substitution maps them as for any ordinary clause. This is
rustc's `no_bound_vars` treatment of an empty binder. Clauses that bind a
region keep their scoped proof. The store already refuses an empty binder
whose body names its own binder, so the lowering cannot meet one.

This touches one shared selection file,
`dev/src/sema/traits/selection/instantiation.rs`. It does not touch
`premise_index`, `alternatives::choose`, storage-opaque, canonical selection
or outlives.

### Controls

`pa31/tests/spec/fn-sugar-impl-parameters/` holds eleven no_core libraries
on a pinned core prefix. Their outcomes were declared before the rustc 1.98.0
oracle: nine before the implementation changed, and the two lifetime controls
after it but before their oracle and before any RustGM run of them. One fixture
amendment came before its re-run: `const-call-adapter` v1 lacked the `freeze`
lang item that rustc's const checking needs. Declarations, oracle results and
the amendment are in `outcomes.json`; receipts are in
`obj/parallel/definition-lineage/oracle-fn-sugar/`.

| Control | rustc | Before | After |
|---|---|---|---|
| `from-fn-item` | 0 | cause | 0 |
| `inline-bound` (`RepeatWith`) | 0 | cause | 0 |
| `caller-premise` | 0 | cause | 0 |
| `impl-lifetime-output` (impl lifetime inside the sugar output) | 0 | cause | 0 |
| `explicit-binding` (no sugar) | 0 | 0 | 0 |
| `caller-mismatch` | E0308 | cause | "incompatible types" |
| `output-not-option` | E0271 | cause | cause (no impl applies) |
| `map-adapter` (`Map` with a `fold`-like override) | 0 | cause | "implementation method imposes a stronger generic requirement" |
| `const-call-adapter` (`array::Drain`) | 0 | cause | cause |
| `elided-lifetime` (`Scan`) | 0 | cause | cause |
| `impl-lifetime-mismatch` | region error | cause | 0 |

The last four are recorded gaps in
`dev/tests/invariants/fn_sugar_impl_parameters.rs`, which fails when any of
them changes:

- `elided-lifetime`: `FnMut(&mut St) -> Option<B>` binds the elided region, so
  its condition still waits for a resolved parent. This is the remaining gap of
  the cause (`scan.rs` and `map_windows.rs` in core).
- `map-adapter`: the projection now selects, and the override then fails
  conformance. That failure needs no sugar. Every probe of an impl method that
  repeats a trait method's where-clause naming `Self::Assoc` fails the same way
  before and after this repair, including a four-line ordinary crate
  (`probes/conf4-ordinary.rs`). Census 4 could not see it, because
  conformance runs after the declaration loop.
- `const-call-adapter`: a user impl of `FnOnce` cannot normalize its own
  `Self::Output`, with or without sugar, and even for a concrete
  `impl FnOnce<(usize,)> for S` (`probes/callable-concrete.rs`). This is the
  third group above.
- `impl-lifetime-mismatch`: rustc reports the region error in borrow checking.
  This analysis does not region-check a generic body: the loans phase rejects
  generic body construction by name.

The same file checks the lowering itself on a bare store. An empty sugar binder
whose body names the impl's first capture lowers to the ordinary clause.
Substituting the capture after lowering gives the same predicate as
substituting beneath the binder and lowering afterwards. Region-binding and
ordinary clauses come back unchanged, and lower nothing.

Amended control: `implementation_conditions` declared
`an_empty_quantifier_cannot_enter_the_flat_completed_cache`. It asserted that a
hand-built empty-binder impl condition was proven in its own scope and stayed
out of the flat completed cache. That was a property of the old routing. The
amended test, `an_empty_quantifier_is_proven_and_cached_as_its_ordinary_condition`,
proves the same impl chain with and without the empty quantifier. It asserts
equal completed-cache sizes and scope counts, no regions and valid evidence.
The other 29 selected selection invariants pass unchanged, among them
`empty_quantifiers_keep_the_enclosing_binder_and_canonical_subject_scope`,
which proves a quantified clause directly, outside impl instantiation.

### Census 5 (Lane B run)

This is Lane B's own run on base `e84a85783`. Lane A's
[census batch 4](census-batch4.md) independently ran its own census 5 (pin
`5cb7e00c`); the two are different runs and not directly comparable. The
analysis-only pin `4cb211bb` was built from the committed repair
(Lane B branch commit `c3e34e5b2`, no uncommitted `dev/` change), following
[census batch 3 §5](census-batch3.md#5-census-4). It uses census 4's manifest,
flags and pinned inputs with an explicit producer binding. None of the six
configuration sources changed since census 4's base. It opened 375 sources in
51 s. Records, receipt and the by-cause comparison
(`census_compare.py`) are in `obj/parallel/definition-lineage/census-5/`.

| Census | Checked | Failed | Blocked | End |
|---|---|---|---|---|
| Census 4 (`6e53028c`) | 42,109 | 74 | 127 | declaration loop complete |
| Census 5, Lane B (`4cb211bb`) | 42,122 | 62 | 126 | declaration loop complete |

The cause falls from 35 failed and 26 blocked to 23 and 25. Exactly the twelve
predicted sites are gone: `filter_map.rs` ×5, `map.rs` ×4, `map_while.rs`,
`from_fn.rs` and `repeat_with.rs`. Every other cause keeps its census 4 count,
and no new failure appears. Of the remaining 23, specialization has 14, user
callable-trait `Self::Output` 5, region-binding sugar 2 (`scan.rs`,
`map_windows.rs`), and `control_flow.rs:427` and `iter/mod.rs:376` one each.
Once declarations pass, the `map.rs`-style `fold` overrides will meet the
conformance gap above.

## 4. Std builds as an rlib only

The executor now applies the [std profile decision](../core-census-plan.md#std-profile-decisions).
In `scripts/target_units.py`, `kind_gaps` treats `dylib` beside `lib` or
`rlib` as dropped, not as a gap. The unit still maps to `--crate-type=lib` with
`--metadata-output`. Each `Step` carries `dropped_crate_types`, and each
completed receipt row records it, so original std's unit records
`["dylib"]`. A target with only `dylib`, `cdylib` or `staticlib` keeps its
typed `crate_types` gap.

- `scripts/tests/test_target_units.py`: the course-profile std unit
  (`["dylib","rlib"]`) is no longer a gap; it builds the rlib with
  `--crate-type=lib` and `std.rgcrate` metadata and records the dropped
  `dylib`. A new authored graph builds `["dylib","rlib"]` and
  `["lib","dylib"]` as rlibs. Its opponents stay typed gaps: dylib-only,
  cdylib-only, and a `staticlib` or `cdylib` beside an rlib. A fake-RustGM run
  records the dropped type in its step and receipt rows.
- `scripts/check_target_units.py` adds the real case `rlib-beside-dylib`
  (complete, `records` drops `dylib`). The `unsupported-crate-type` opponent
  now lists `["dylib"]` explicitly.

Amended control: `test_target_units` asserted that std's unit had a
`crate_types` gap for `dylib`. Under the decision, that is the dropped type.

## 5. Rust 2015/2018 as cfg-if and rustc-demangle need them

### Rule list

Read from cfg-if 1.0.4 (2018) and rustc-demangle 0.1.27 (2015) in the std
target graph, non-test cfg (`obj/parallel/legacy-editions-v1/std-run/inputs/vendor/`).
Both are `--source-client core@...` units.

| Rule | Needed by | State |
|---|---|---|
| A source client in Rust 2015 or 2018 | both | implemented here |
| Pre-2021 `unreachable!()` and `unreachable!(fmt, args)` select `unreachable_2015` | rustc-demangle (`v0.rs:305`, `v0.rs:348`) | implemented here |
| `#![no_std]`'s injected `extern crate core` is nameable at the 2015 crate root, so `use core::{char, fmt, ...}` resolves in modules and bodies | rustc-demangle | implemented (legacy editions batch) |
| 2015 `use` paths are crate-root relative | rustc-demangle | implemented |
| Closures capture whole variables before 2021 (`*f`, `(*self).field`) | rustc-demangle | implemented |
| The `expr` fragment keeps its 2021 rules before 2021 | rustc-demangle's local macros | implemented (no difference) |

cfg-if's one `macro_rules! cfg_if` uses only `tt` fragments, `$crate` and
`$(...)?`. No other rule differs for it in 2018. Outside `cfg(test)`,
rustc-demangle uses no other legacy rule:

- no `async`, `await`, `dyn` or `try` identifier;
- no anonymous parameter, bare trait object or `...` pattern;
- no array `into_iter`;
- no `panic!` or `assert!`. Its `assert!` macros are test-only.

It imports `TryFrom` explicitly, so the 2015 prelude's missing `TryFrom`
changes nothing. Every other legacy rule keeps its named `LegacyRule` rejection.
That includes pre-2021 `panic!` and `assert!` (`panic_2015`), which still fail
with "the pre-2021 panic macro form is not implemented for edition 2015|2018".

**`core::char` and `unreachable!` in the bootstrap model.** Neither is an
edition rule. RustGM's bootstrap model of `core` has no `char` module and no
`unreachable!`. Against it, rustc-demangle's names stop at `use core::char`
(`legacy.rs:1`; it needs `char::from_u32`), and expansion stops at
`unreachable!` (`v0.rs:305`), in 2015 and 2018 alike
(`obj/parallel/legacy-editions-v1/rustc-demangle/`). Both names come
from real core once rustc-demangle builds as a source client of core, so the
bootstrap model is not extended.

With real core, `unreachable!()` reaches `$crate::panicking::panic`. The
format arm of `unreachable_2015` calls `$crate::panic!` with core's own 2024
tokens and no `edition_panic` allowance. The edition walk therefore selects
`panic_2021`, then `format_args!`, which is Lane C's transport.

Source clients still require `#![no_core]`. cfg-if and rustc-demangle are
`#![no_std]` clients of core, as alloc is. That gate is shared with alloc, is
not an edition rule, and stays as it is.

### Implementation

- `resolve::bindings::client::identity` admits Rust 2015 and 2018 source
  clients. Rust 2021, which no std unit needs, keeps "source client requires the
  Rust 2024 source profile".
- `resolve::expansion::panic_dispatch` maps a 2015 or 2018 `unreachable` call
  site to the new `delegation::Handler::Unreachable2015`, which spells
  `$crate::panic::unreachable_2015`. The same rule serves a local source role
  and an imported row. `panic`, `assert` and `std_panic` before 2021 keep the
  `LegacyRule::PanicMacros` rejection, and its doc now says so.
- `spec.md` (macro envelope version 4) records both rules.

### Controls

Plan 4 declared three new rows and restored two amended ones on 2026-11-07,
before the rustc 1.98.0 oracle and before the implementation changed. The plan
is `obj/parallel/definition-lineage/legacy-unreachable/plan4.json`; its hash
(`plan4_sha256`), the oracle hashes and the result are in
`pa29/tests/imported-panic-outcomes.json`. The oracle reran all 75
macro-metadata cases natively (`legacy-unreachable/oracle-1/`), and every
declared rustc outcome held. The provider fixture's rows return distinct
values, so each native result names the rule that ran.

| Control | Edition | rustc 1.98.0 | RustGM |
|---|---|---|---|
| `panic-legacy-unreachable-2015-client`: `unreachable!()` and `unreachable!("at {}", 3)` | 2015 | 0, prints `47 48` | 0, prints `47 48` |
| `panic-legacy-unreachable-2018-client`: the same file | 2018 | 0, prints `47 48` | 0, prints `47 48` |
| `panic-legacy-native-client` (restored): a wrapper without `edition_panic` | 2018 | 0, prints `41` | 0, prints `41` |
| `panic-legacy-client` (restored opponent): a skipped wrapper reaching a 2018 `panic!` | 2018 | 0 | "the pre-2021 panic macro form is not implemented for edition 2018" |
| `panic-legacy-2015-client` (opponent): a 2015 `panic!()` | 2015 | 0 | "the pre-2021 panic macro form is not implemented for edition 2015" |

`47 48` is the provider's `unreachable_2015` row; `unreachable_2021` would
print `43 44`. The candidate's native evidence is in
`obj/parallel/definition-lineage/macro/native-t4/`.

- Private: the new
  `resolve::bindings::client::tests::legacy_source_clients_pass_the_edition_gate_and_2021_stays_rejected`.
  A 2015 and a 2018 header pass the edition gate and stop at the next check,
  "source client requires an authenticated dependency inventory". A 2021
  header keeps "source client requires the Rust 2024 source profile". The
  other three source-client tests and the delegation spacing test pass
  unchanged.
- `legacy_editions`: every frozen legacy rule keeps its outcome or its typed
  fence, and the declared `LegacyRule` table still mirrors the outcome file.
  No rule was added to or removed from the table.

Amended controls:

- `transported_rows`'s runner count rises from 72 to 75, the three plan-4
  rows.
- `panic-legacy-native-client` and `panic-legacy-client` return to their
  originally declared plan-1 outcomes. Candidate round 1 had amended both for
  the 2024-profile gate, which no longer rejects 2018. The reasons are in
  `amended_controls`, marked "(plan 4)", and both case notes now say so.

## 6. Checks

Receipts are under this worktree's `obj/`. `qualification/fast/N` stands for
`obj/qualification/fast/N`; other paths are under
`obj/parallel/definition-lineage/`.

| Task | Check | Receipt | Result |
|---|---|---|---|
| 1 | private: layouts, imports, expansion | `qualification/fast/1794073799516254886` | 39 passed |
| 1 | integration: exported macro tables | `qualification/fast/1794074336241642232` | 18 passed |
| 1 | native macro-metadata runner | `macro-native-1.log` | 72 cases, native; 27 `.rgm` hashes equal main's |
| 2 | private: ledgers and limits | `qualification/fast/1794075593278535456` | 26 passed |
| 2 | PA1–12, PA13–19/21–24, PA20 | `qualification/fast/1794075962510203336`, `…/1794076485974890291`, `…/1794076715301613819` | passed; PA20 1451/1451 |
| rlib | `make test-harness` | `checks/test-harness-1.log` | 304 script tests; 464 roots in 13 binaries; 44 stages |
| rlib | `make test-target-units` | `obj/qualification/target-units/1794077897389065829` | passed |
| 3 | private: selection, fn sugar | `qualification/fast/1794078364887770565` | 30 passed |
| 3 | PA1–12, PA13–19/21–24, PA20 | `qualification/fast/1794078518676062594`, `…/1794078657237435493`, `…/1794078791755482442` | passed; PA20 1451/1451 |
| 3 | `make test-upstream` | `checks/test-upstream-1.log` | 127 fixtures, 126 Reference files |
| 3 | rustc oracle, fn sugar | `oracle-fn-sugar/` | 11 declared outcomes held |
| 3 | census 5 | `census-5/receipt.json` | §3 |
| 4 | private: source-client gate, delegation | `qualification/fast/1794079045317369279` | 5 passed |
| 4 | integration: macro rows and runner manifest | `qualification/fast/1794079192127492263` | 6 passed |
| 4 | integration: `legacy_editions` | `qualification/fast/1794079480944664037` | 2 passed |
| 4 | rustc oracle, plan 4 | `legacy-unreachable/oracle-1/` | 75 declared outcomes held, native |
| 4 | native macro-metadata runner | `macro-native-t4.log`, `macro/native-t4/` | 75 cases, native |
| 4 | PA1–12, PA13–19/21–24, PA20 | `qualification/fast/1794079542326198128`, `…/1794079589008930745`, `…/1794079664254213940` | passed; PA20 1451/1451 |
| 4 | `make test-upstream` | `checks/test-upstream-2.log` | 127 fixtures, 126 Reference files |
| 4 | integration: final manifest, notes and `legacy_editions` | `qualification/fast/1794079796477621769` | 4 passed |

Every fastcheck ran full formatting, the architecture audit and strict Clippy
for its targets. Pending for scheduled CI: full release regular and native
tests, all-target strict Clippy, the uncached audit and the independent suites.

## 7. Storage

The only retained executable is the census 5 pin,
`obj/parallel/definition-lineage/census-5/pin/rustgm` (sha256 prefix
`4cb211bb`), with `.keep-artifacts`. It is the comparison baseline for the next
census. A scoped prune (`storage.py prune --apply --prune-under
obj/parallel/definition-lineage`) removed the four probe CLIs, the attribution
build and the oracle objects: 102 files, 0.50 GiB. `make tidy` then removed 150
files. Receipts, diagnostics and stdout remain. The batch keeps 101 MB, and
the worktree's `obj/` is 14 GB, within its 16 GiB cap.

## 8. Decisions for the integrator

1. **`Engine` reads 4152 bytes and `Report` 3104.** Main asserted 4144 and
   3096. Census batch 3 added a `granted` word to the shared `producer::Work`.
   The inline import `Work` holds it as the lineage grant, adding 8 bytes to
   each. `Engine` measured 4184 before section 1 folded the loader's 32-byte
   admitted set into its source cache. `Report` 3104 was accepted earlier. The
   `Engine` amendment (`modifier_storage.rs`, "The engine's import Work holds
   the same lineage grant") needs the same acceptance.
2. **Lineage factors.** These are 8 steps and 128 attempted bytes per admitted
   byte, above the old fixed floor. The lint ledger keeps 8 and 64. A
   zero-sized `Factors` marker on the shared `producer::Work` selects the
   factors, so neither record grows. The alternative is one factor pair for
   both ledgers at 128 bytes, which would loosen the lint bound twofold.
3. **The remaining "unambiguous implementation" groups** (section 3) each need
   an owner:
   - specialization (14);
   - a user callable-trait impl normalizing its own `Self::Output` (5);
   - region-binding sugar (`scan.rs`, `map_windows.rs`), whose scoped
     condition still waits for a resolved parent;
   - override conformance of a repeated `Self::Assoc` where-clause. This
     fails without sugar and will meet `map.rs`'s `fold` overrides after the
     declaration loop.
4. **Edition gate.** Rust 2015 and 2018 source clients are admitted, and Rust
   2021 stays rejected because no std unit needs it. The same rule could admit
   2021 with no extra rule.
5. **The `#![no_core]` source-client gate** is shared with alloc and is not an
   edition rule. cfg-if and rustc-demangle are `#![no_std]` clients of core.
   Building them against real core needs the gate decided for alloc. They also
   need `format_args!` through `panic_2021` (Lane C) for rustc-demangle's
   formatted `unreachable!`.
