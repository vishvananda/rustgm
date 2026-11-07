# Input-proportional macro-contract budget

Lane B, task 1 (user-approved). The resolve-phase macro-contract and
library-inventory ledger (`State` in `dev/src/resolve/bindings/panic_contracts.rs`)
was capped by a fixed whole-crate default of 1M work units and 64 MiB of
attempted and retained storage. Original core crossed that default at
`net/ip_addr.rs` (open order 179), so the target-unit executor stopped there.
The census avoided it only with explicit limits. The ledger is now bounded by
the source it actually admits.

## Rule

- **Setup floor.** Every ledger starts at 1,000,000 work units, 64 MiB attempted
  and 64 MiB retained (`Allowance::SETUP`). Before most source is read, the
  ledger pays for provider tables, the prelude and builtin vocabularies. The
  floor equals the old default, so no input that was accepted before can now
  fail. A context-free scratch ledger keeps exactly this floor.
- **Per-source grant.** The first parse of each authenticated source identity
  grants a fixed amount per original byte (`OriginalImage` length): 2 work units,
  64 attempted bytes and 8 retained bytes. That covers the crate root (at ledger
  creation, in `Contracts::admitted`), file modules (`Loader::module`) and
  `include!` sources (`Loader::include`). The grant happens right after the parse,
  before expansion reads the file.
- **Nothing else grants.** Expansion output, copied module or include instances,
  path aliases of one identity and repeated requests grant nothing. The loader
  keeps one `admitted` identity set. `include_str!`/`include_bytes!` resources are
  literals, not parsed sources, and grant nothing either.
- **Explicit caps.** `SourceContractLimits` now holds three optional caps
  (`Option<u32>`). The CLI flags set only the cap they name. Each effective limit
  is `min(cap, granted)`. The caps are snapshotted once, as before, and zero is
  still a real cap.
- **Sticky failure.** A later grant only raises `granted`. It never clears
  `failed` and never refunds attempted charges.

Bytes are the cleaner measure. They are the quantity the observer authenticates
by hash, they are immutable after expansion appends to the normalized view, and
they bound tokens, attribute bytes and format-string bytes. Token counts are not
retained after parsing.

## Attribution (analysis only)

A temporary `#[track_caller]` patch recorded the immediate caller of every
ledger charge and the peak retained payload. It also printed the cumulative
usage at each source admission. The patch was never committed; it is preserved
as `obj/parallel/macro-budget-v1/attribution-cli2/attribution.patch`.

The run used unchanged original core: inputs `original-core-inputs-88d9e12a…`
(read-only), main `080659214`, `--cap-lints=allow --emit-types`, and u32::MAX
caps. It reached the same first error as the census frontier, `flatten.rs`
(open order 142). It admitted 375 files: the root, 371 modules and 3 includes,
14,668,689 original bytes in total.

| Total | Value | Per admitted byte |
|---|---|---|
| Work | 8,398,273 | 0.573 |
| Attempted bytes | 244,790,717 | 16.69 |
| Retained peak | 27,543,295 | 1.88 |

| Payer group | Work | Share | Attempted |
|---|---|---|---|
| Library inventory (`stability/*`) | 2,836,470 | 33.8% | 27.2 MB |
| Derive lists and output | 2,581,476 | 30.7% | 142.3 MB |
| Source-prelude lookups | 1,619,907 | 19.3% | 0 |
| Panic, format, eager and choice expansion | 1,001,506 | 11.9% | 75.0 MB |
| Macro header contracts | 358,710 | 4.3% | 0.2 MB |

All payers are linear in their input:

- **Inventory:** per definition, attribute and feature-name byte. `has_marker`
  adds a binary-search `log` factor over a tiny marker table.
- **Derive output:** per emitted derive token and origin.
- **Header contracts:** one walk per attributed owner.
- **Prelude lookups:** 134,990 resolver lookups at 12 units each. Each lookup
  re-authenticates the same prelude record. This costs a constant per lookup, so
  it is linear and not per-site re-derivation like LIB-193's `enable` parsing.
  Validating once per phase would remove 19% of core's work; that is listed
  under "For the integrator".

The cumulative charge-to-admitted-bytes ratio was largest at the end of the
run, after all admissions. No charge needed allowance from a file admitted
later.

## Factors

Each factor is a power of two with at least 3.5× headroom over core's linear
payers. The headroom is meant for payers not reached yet, such as sema method
probes that also validate the prelude. No factor absorbs repeated or
superlinear work.

| Dimension | Measured | Factor | Headroom | Core allowance |
|---|---|---|---|---|
| Work | 0.573/B (0.463/B without prelude lookups) | 2/B | 3.5× (4.3×) | 30.3M |
| Attempted | 16.69/B | 64/B | 3.8× | 1.0 GB |
| Retained | 1.88/B | 8/B | 4.3× | 184 MB |

## Qualification

These controls were declared before the runs.

**Private tests.** `make fastcheck` passed 12 exact tests, plus strict private
Clippy, format and architecture (`obj/qualification/fast/1794041287780502150`):

- `limit_tests::` (6 tests):
  - `exact_finite_boundaries…` (amended: the default is no caps);
  - `both_actual_owner_constructors…`;
  - `immutable_limit_layout…` (amended sizes: limits 24, Config 96, State 152,
    Contracts 160, SourceCore 240);
  - new: `each_admitted_identity_grants_once_and_expansion_output_grants_nothing`
    covers aliases, copies and macro-generated `mod`/`include!` at 1, 4 and 32
    copies;
  - new: `explicit_caps_bound_the_grant_and_grants_never_reset_failure`;
  - new: `linear_invocation_family_scales_and_exponential_expansion_still_fails`.
- `panic_contracts::tests::exhausted_macro_contract_work_denies…` (amended: it
  exhausts the actual allowance, not the literal 1M).
- Two inventory-budget tests, the request-payer test, the local macro owner test
  and the include/instance replay test.

**Integration tests.** `make fastcheck` passed 4 exact tests, plus CLI Clippy
(`obj/qualification/fast/1794041419129360102`):

- `inventory_ordering::actual_source_inventory_crosses…`: the 131,072-function
  input now also completes without caps, and an explicit 1M cap still fails;
- `quota_options` (2 tests);
- `census_mode::budget_exhaustion_is_sticky_and_ends_the_census`.

**Linear family.** Source-core `m!{}` invocations cost about 1.0 work unit each:
1,024 calls used 1,160 work for 5,385 bytes, and 16,384 calls used 16,520 work
for 82,185 bytes. Both are about 0.2 work per byte. The inventory family is
linear as well; its 131,072 case passes the old 1M default only through the
proportional grant.

**Superlinear control.** Doubling `macro_rules!` recursion grants nothing: the
allowance equals the root admission at depths 9 and 12, and work grows at least
7× over those three levels. At depth 40 the CLI fails within 57–65 s with a
sticky budget error, with or without lint caps. The source lint-policy storage
ledger fires before this one. Evidence: `obj/parallel/macro-budget-v1/exponential/`.

**Original core** (pinned dev-fast CLIs: baseline `f349649e…`, candidate
`3a4faa4c…`):

- **Ordinary mode:**
  - baseline, default limits: `ip_addr.rs`, exhausted budget;
  - baseline, observer caps: `flatten.rs` 8286;
  - candidate, default limits: `flatten.rs` 8286;
  - candidate, observer caps: `flatten.rs` 8286.
- **Census mode:** the baseline-capped, candidate-capped and candidate-default
  `census.tsv` files are byte-identical. Each checked 10,410 declarations, with
  4 failed and 31 blocked.

The original-core observer records `null` for an absent cap and still requires
matching manifest and producer caps. `scripts/tests/test_original_core.py`
passes.

**Target units** (candidate, `obj/parallel/macro-budget-v1/units-*`):

- **Core, `--analysis-cap-lints allow`, default limits:** past `ip_addr.rs`; the
  new first gap is `missing associated result contract` at file 142, byte 8286
  (`iter/adapters/flatten.rs`).
- **Core, Cargo policy (uncapped):** `strict provenance lint producer is not
  implemented` at `lib.rs:3519` (`implicit_provenance_casts`), as expected. It is
  recorded here, not fixed.
- **Std (combined run, capped lints):** stops first at core (unit 5) with the
  same `flatten.rs` gap. The planning gap stays std's dylib (unit 14). After core,
  the next unit that needs new support is cfg_if (task 2).

## Source-prelude validation once per phase

Follow-up (Lane B, task 1). Each resolver lookup that reached the source prelude
re-authenticated the whole retained selector record: its owner, effect, use tree,
source image and module (12 work units). That was 19.3% of core's ledger work.

**Rule.** Each phase that reads the selection now re-authenticates it once:

- when it is selected (`select_source_prelude`, during expansion);
- when expansion completes (`finish_analysis`);
- when resolution completes (`analyze_expanded`, and the plain `analyze`);
- once per sema method index (`visible::Index::new`), which keeps the
  authenticated scope.

Each lookup (`Resolution::selected_source_prelude`) still pays its own constant
cost: 1 work unit and the `prelude_import` request check. It also rechecks the
selected module row (a module whose member scope is the selected scope). A sema
probe also requires the selection to equal its index's scope. Lookups between
two validations use the selection without re-reading the record. A changed
record or owner makes that phase's completion fail, so no accepted phase result
rests on a changed selector. Validation never moved out of a phase; no payer was
refunded, and the grant factors are unchanged.

**Attribution** (the same analysis-only patch, now tagged by phase, never
committed: `obj/parallel/prelude-once-v1/attribution-baseline.patch`). Both
pinned dev-fast CLIs ran unchanged original core (722 files verified against
`original_core_manifest.json`) with u32::MAX caps, `--cap-lints=allow
--emit-types`. Baseline `b72400cd…` is head `ed325cbe1`; candidate `40be00df…`
is this change. Both stop at `flatten.rs` 8286, and their stdout and diagnostics
are identical. The census files (10,410 declarations checked, 4 failed, 31
blocked) are byte-identical.

| Payer group | Before | Share | After | Share |
|---|---|---|---|---|
| Library inventory (`stability/*`) | 2,836,470 | 33.8% | 2,836,470 | 41.0% |
| Derive lists and output | 2,581,476 | 30.7% | 2,581,476 | 37.3% |
| Source-prelude lookups | 1,619,907 | 19.3% | 135,041 | 2.0% |
| Panic, format, eager and choice expansion | 1,001,506 | 11.9% | 1,001,506 | 14.5% |
| Macro header contracts | 358,710 | 4.3% | 358,710 | 5.2% |
| Total work | 8,398,273 | | 6,913,407 | |

The 134,990 lookups split into 9,629 during expansion and 125,361 in the
post-expansion resolution walk. They now cost 134,990 units. The rest is three
12-unit validations (selection, expansion end, resolution end) and 15 units of
one-time selection setup. This core run
stops before the sema method index is built. Attempted bytes (244,790,717) and
the retained peak (27,543,295) are unchanged.

| Dimension | Measured now | Factor | Headroom (before) |
|---|---|---|---|
| Work | 0.471/B | 2/B | 4.2× (3.5×) |
| Attempted | 16.69/B | 64/B | 3.8× (3.8×) |
| Retained | 1.88/B | 8/B | 4.3× (4.3×) |

### Qualification

These controls were declared before the runs.

**Private tests.** `make fastcheck` passed 9 exact tests, plus strict private
Clippy, format and architecture (`obj/qualification/fast/1794046564195156058`):

- new: `prelude_scope::tests::lookups_pay_a_constant_cost_and_phases_reauthenticate_the_selector_once`.
  On the `constructor-method` fixture, 1, 64 and 4,096 lookups each cost exactly
  that many work units and no bytes, and return the authenticated scope. A phase
  completion costs 12. A changed effect passes a lookup but fails the phase
  completion and the authenticated read. A changed selected scope fails the
  lookup itself.
- `prelude_scope::tests::source_prelude_failure_payment_precedes_selector_publication`
  (amended: it exhausts the actual admitted allowance through a test-only
  accessor rather than the literal 64 MiB, which the input-proportional grant of
  `054b91950` had already made stale on main);
- five more `prelude_scope::tests` (owners, privacy, `no_implicit_prelude`,
  shadowing, re-authentication of owner, effect and module incarnations, the
  builtin macro);
- `visible::prelude_tests::source_lang_trait_is_not_a_method_prelude_without_actual_import`;
- `container_scopes::tests::scope_storage_is_linear_and_hot_records_do_not_gain_provenance`.

**Integration tests.** `make fastcheck` passed 8 exact tests, plus CLI Clippy
(`obj/qualification/fast/1794046842006443262`): both `legacy_editions` tests,
the four `edition_fragments` tests, the 131,072-function `inventory_ordering`
input and `census_mode::independent_declaration_failures…`.

## For the integrator

- Amended frozen controls:
  - `pa31/tests/spec/source-contract-limits/contract.json` and its README: the
    old defaults are now the setup floor and all predictions are unchanged;
  - the `inventory-budget` README;
  - the three private test assertions above.
- The census no longer needs explicit `--source-contract-*` limits for this
  ledger. Its explicit caps still cap the allowance and give the same frontier.
- Optional cleanup: validate the source prelude once per phase instead of once
  per lookup (19% of core's work in this ledger). Done: see
  [Source-prelude validation once per phase](#source-prelude-validation-once-per-phase),
  which amends one more private control (the selector-publication payment test).
