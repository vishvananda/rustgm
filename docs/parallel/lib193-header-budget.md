# LIB-193 header budget: attribution, literal reuse and input-graph bound

LIB-193 was the unlocated `source header contract query work exceeds profile
limit` failure in `target_features::collect`, reached from
`Checker::from_facts` before `Checker::run`. The sema header ledger was capped by
`MAX_COMPONENTS`/`MAX_BYTES` (1,000,000 work, 16,000,000 bytes), constants that
began as metadata decoding bounds.

## Attribution

One analysis-only diagnostic of main `beabc1161` recorded every ledger charge by
`#[track_caller]` call site (CLI `8a8a7bba...`, never qualified). At the crossing
(1,000,002 work, 14,686,030 bytes) the totals were:

| Payer | Work |
| --- | ---: |
| Intrinsic-annotation owner-effect scans (`tracking.rs`) | 209,626 |
| Target-feature collection, 1,446 of 6,292 sites | 161,621 |
| Header definition rows (`bind_definition`) | 139,216 |
| MIR-boundary binding prepayment (definitions + 16 x annotations) | 125,566 |
| Header owner rows (`observe_selected_owner`) | 123,615 |
| Definition-binding prepayment (definitions + pointer markers) | 118,345 |
| Non-admitted owner checks | 77,268 |
| Legacy const-generic indices | 26,614 |
| Everything else | 18,131 |

838,381 units (84%) were spent before target-feature collection began. Those
payers are linear and visited once per owner, definition or effect. The repeated
work was in target-feature collection. 6,292 sites use only 85 distinct `enable`
values, and every site decoded, searched and implication-closed its value again.
Replaying the exact charges over the compiler's own census reproduces all 32
observed per-literal costs exactly. The full walk would cost 928,255 units.

## Repair: validate each spelling once

`collect` keeps a walk-local map from each plain string-literal `enable`
spelling, borrowed from the unit source, to its closed `FeatureSet` and its
explicitly named gated features. The first site decodes, validates and closes
the value as before, with the same diagnostics at its own entry. It then pays
the memo's growth before inserting. Each later site with that spelling still
does all of the following:

- pays two units per spelled byte to hash and compare its own literal;
- obtains its own crate gate result for each gated feature;
- keeps its own owner, function and safety checks;
- keeps its row, site and witness retention.

Raw, byte and nonliteral tokens keep the uncached path. Memoization removes
748,665 of the 928,255 parse units. Lookups cost 170,920 and the 85 first
validations cost 8,670.

The memo boundary is one collection walk; nothing is retained afterward. Work
can still multiply by the number of sites times literal length (linear), by
distinct spellings times names times the 77-entry search, and by closure size,
which is bounded by the 128-bit feature domain. Expected throughput relies on
the existing fast hash, which gives no worst-case bound against colliding
spellings.

## Input-graph bound

The local header ledger now snapshots an `InputBound` once in
`Declarations::initialize`, before any charge. The bound covers:

- definitions;
- every unit's owners, attribute effects, AST nodes and edges;
- retained attribute token trees and the bytes they spell.

Work is limited to 16 units and attempted storage to 128 bytes per element.
Failure stays sticky because charges are added before they are checked. The
configured `SourceContractLimits` and the observer profile are unchanged.
Errors now say `exceeds its input-graph bound`.

The factors were sized from one analysis-only measurement of the candidate,
which observed the bound without enforcing it (CLI `7fbc8cdc...`). Original
core has 3,389,691 graph elements before attribute bytes. Its header work through
collection and into `Checker::run` was 1,118,860 units and 20,507,722 bytes:
0.33 units and 6 bytes per element. Collection itself took 276,052 units.

The densest genuinely linear payer is a first-site literal validation, at about
13 units per spelled byte. Dense minimal target-feature functions retain about
49 bytes per element, including publication. A factor of 16 is the
declaration-checking precedent. A factor of 128 bytes leaves about 2.6x
headroom for those functions. Neither factor may be raised
to absorb repeated or superlinear work.

`Declarations` grows by 16 bytes, to 7,136, and `Headers` grows to 136 bytes.

## Controls

- Shared-spelling scaling: 1, 8, 32 and 128 sites of a 26-name value differ
  from a cheap value only by each site's own spelling read, with equal bytes.
  Reused rows are closed.
- Gate checks: 16 shared gated sites pay one gate result each.
- Duplicate attributes keep two sites.
- Distinct permutations: each pays at least full name and implication work.
- A later unknown feature still rejects at its own entry.
- An exact budget boundary admits or rejects the final charge.
- Paired linear inputs: 64 and 256 callees stay within half of their bound,
  and the bound scales with the graph. At 64 callees they use 4,486 of 48,304
  work and 148,184 of 386,432 bytes. Dense 7,000- and 28,000-item
  source headers stay within half of theirs.
- Adversarial superlinear control: one function with 512 sites called 512 times
  revalidates its whole row at each call. It fails with the bound error, while
  the 32 x 32 case passes.

## Re-probe

The qualified CLI (`ce4c689d...`, base `46b3e3bb0` plus this checkpoint) probed
unchanged original core once. The manifest, the six configuration-source pins,
cap-lints `allow` and the source-contract limits (14,500,624 work, 1 GiB
attempted, 64 MiB retained) were unchanged. LIB-193 is cleared. The first error
is now `file 97: 6032: object requires exactly one principal trait`, in 20.09 s
with 375 opened sources. File 97 is original `core/src/any.rs`, and byte 6032 is
the `Send` bound in `impl fmt::Debug for dyn Any + Send` at line 162. No
original artifact or whole assignment is complete.

## Amendment: lowering payments (2026-11-08)

The input-graph bound also covers lowering. `Payments::charge` is the payer
behind `charge_caller_inputs`, and `d2bcb1f89` gave it the same bound. The
sizing above measured header work only up to `Checker::run`.

That bound turned the ignored native test
`needs_drop_provider::linked::source_needs_drop_scaling_native_answers` red. Its
1,024-nominal input lowered to 121% of the byte bound. Each consumed body's
storage was paid a second time as `Originals::shrink` released it.
[The needs-drop native repair](needs-drop-native-repair.md) makes that shrink
work-only. Its margins are:

- all 13 native scaling inputs use 1% to 66% of their byte bound, the
  1,024-nominal input 66%, and at most 10% of their work bound;
- literal constant items, the densest bodies measured, take 115 of 128 bytes
  per element (90%);
- trivial functions take 244 bytes per element. A source-core crate with Drop
  rows fails to lower at 256 of them.

Lowering needs its own measured per-phase constants. The 128-byte header
factor must not be raised to absorb lowering. The next section gives lowering
its own account.

## Lowering's own account (2026-11-08)

Lowering no longer pays the header account. Its storage is split by what it
grows with:

- Storage sized by the original bodies pays a lowering account. It opens
  once, on the crate's existing ledger, at the start of `inspect_staged`, and
  only for the report whose input graph the declarations snapshotted. Its
  bound is linear in the input.
- Storage that demanded instances, factories and adapters add pays the
  existing demand `Budget` (100,000 bodies, 64 MB). Each pays its own actual
  retained bytes once.
- Header queries and caller inputs charged while a demanded instance is
  traversed or emitted pay the demand budget too, in work and bytes, through
  a lexical demand scope. Demanded-instance cost is proportional to the
  output, and its work follows the same rule as its bytes.

Monomorphization is legitimately superlinear in its input. In a chain of `n`
generic functions, where each forwards to the one before and each is
instantiated by its own root, `n` items demand n(n+1)/2 instances. No
input-proportional bound accepts that chain and still rejects a superlinear
defect. rustc bounds instantiation with fixed limits, and this split does the
same:

- the demand budget's fixed cap bounds output;
- the lowering account stays proportional to the input, and keeps rejecting
  superlinear retention sized by the originals.

The accepted cost is that a large valid program can reach the fixed cap (open
item 1).

### What pays where

| Payer | Charge | Account |
| --- | --- | --- |
| `Builder::new` | the proof vector, reserved exactly for the originals: 480 bytes each | lowering |
| `Originals::take` | the row owner, the exact replacement row vector (96 bytes per row) and the moved row vector (at most 192 per row after doubling) | lowering |
| `Originals::shrink` | one work unit per moved body and row | lowering |
| Original proofs and rows pushed back | nothing new: they fill the exact reservations | lowering, if they ever grow |
| Demanded proofs and rows (`push_instance`, factories, adapters) | the actual added slots, plus one work unit per moved element | demand |
| `Budget::enter` | the instance body's retained bytes and its contract slices (unchanged) | demand |
| Header queries and caller inputs for original bodies while the account is open | their existing work and bytes | lowering |
| Header queries and caller inputs inside a demand traversal step (dispatch, callsite, coercion and instantiation) | their existing work and bytes, collected by the scope | demand: work to `canonical_steps`, bytes to the 64 MB cap |
| Header queries and caller inputs while a demanded body is emitted (`emit_pending`, `index_bodies`) | their existing work and bytes, collected by the scope | demand: work to `canonical_steps`, bytes to the 64 MB cap |
| Header-row storage (`ledger()`) | unchanged | header |

A full proof or row vector that a demanded body grows adds slots for as many
bodies as have been demanded so far, at least four. The first demanded body
therefore never doubles the originals' exact reservation, and demanded growth
stays proportional to the demanded bodies. With `Vec`'s own doubling, one
instance after 1,024 trivial functions would have paid about half a megabyte
of proof slots sized by the originals. This figure is derived from the growth
policy, not measured. Builder grows from 1,776 to 1,784 bytes, for the
`originals` count.

### The demand scope

Lowering's `Account` holds an optional scope tally. While a scope is open,
`Account::charge` adds work and bytes to that tally instead of to lowering's
totals and checks no bound of its own. The scope's opener pays the tally to
the demand budget when the scope closes.

- Scopes are lexical. `Declarations::pay_demand_traversal` and
  `pay_demand_emission` open a scope, run one closure and close the scope in
  the same function, whatever the closure returns. A charge cannot stay
  misrouted after the instance finishes, and an error still closes the scope
  before it propagates. A nested scope leaves its charges to the outer one.
- The traversal scopes each `traversal.step`. `Budget::pay_scope` adds the
  step's work to `canonical_steps` and its bytes to the budget's bytes, then
  checks the 64 MB cap.
- When the traversal ends, `materialize` carries the first demanded proof
  index and the budget's bytes to emission (`carry_demand`). Emission scopes
  each body at or after that index. `Context` keeps a `[bytes, steps]` tally
  that starts from the traversal's bytes and is capped by the same 64 MB
  (`BODY_BYTES`). `run` adds its steps to `canonical_steps`. `settle_demand`
  clears the carried boundary after emission, and reopening the account
  clears it too.
- Original bodies, and work before the first traversal step, stay on
  lowering's account.

An instrumented run of the chain below found that its traversal steps charge
nothing to lowering's account. All of its per-instance work is emission's
header queries, such as `caller_tracking`, asked by `function::lower`.

Lowering's `Option<Account>` field in `Headers` grows from 40 to 88 bytes for
the scope and the carried boundary.

### Bound

Lowering's bound is 4·E + 64·O work and 32·E + 1,536·O bytes, where:

- E is the header bound's input-graph element count;
- O is the number of body owners: function, constructor, constant and static
  definitions, and every closure, inline constant, async block, array length,
  repeat count and explicit discriminant.

| Constant | Value | Basis |
| --- | ---: | --- |
| `BYTES_PER_BODY_OWNER` | 1,536 | twice the worst measured 768: one 480-byte proof slot plus `take`'s ceiling of 288 (three 96-byte row slots) |
| `BYTES_PER_INPUT` | 32 | a floor for payers that scale with body size; with owners paid separately the measured residual is zero |
| `WORK_PER_BODY_OWNER` | 64 | more than twice the worst measured 27, a trivial `const fn` whose lowering also asks 17 header queries |
| `WORK_PER_INPUT` | 4 | the measured residual is at most 0.09 per element |

The 128-byte header factor is unchanged.

### Measurements

These are per item, measured from 256 to 1,024 items inserted into the
`needs_drop` scaling base:

| Shape | E | O | Lowering bytes | Byte share | Lowering work | Work share | Demand bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `fn fK(){}` | 4 | 1 | 768 of 1,664 | 46% | 9 of 80 | 11% | 0 |
| `const CK:u8=0;` | 5 | 1 | 768 of 1,696 | 45% | 8 of 84 | 10% | 0 |
| `const _:u8=0;` | 5 | 1 | 768 of 1,696 | 45% | 8 of 84 | 10% | 0 |
| `static SK:u8=0;` | 5 | 1 | 768 of 1,696 | 45% | 8 of 84 | 10% | 0 |
| `const fn kK()->u8{0}` | 6 | 1 | 768 of 1,728 | 44% | 26 of 88 | 30% | 0 |
| `fn dK(){let _l=Leaf;}` | 9 | 1 | 768 of 1,824 | 42% | 9 of 100 | 9% | 0 |
| `fn pK(_r:Root){}` | 9 | 1 | 768 of 1,824 | 42% | 9 of 100 | 9% | 0 |
| `fn gK(x:u8)->u8{x}` | 11 | 1 | 768 of 1,888 | 41% | 9 of 108 | 8% | 0 |
| closure call in a fn | 14 | 2 | 1,536 of 3,520 | 44% | 14 of 184 | 8% | 0 |
| closure call in an exported fn | 39 | 2 | 1,536 of 4,320 | 36% | 16 of 284 | 6% | 0 |
| exported `fn rK()->u8{0}` | 31 | 1 | 768 of 2,528 | 30% | 10 of 188 | 5% | 0 |
| constructor and drop glue | 44 | 2 | 1,536 of 4,480 | 34% | 16 of 304 | 5% | 320 |
| `struct NoiseK([u8;K]);` | 11 | 2 | 768 of 3,424 | 22% | 8 of 172 | 5% | 0 |
| nested array lengths | 23 | 5 | 768 of 8,416 | 9% | 5 of 412 | 1% | 0 |
| discriminant, alias | 8 | 1 | 0 | 0% | 0 | 0% | 0 |
| shared generic, one instance each | 37 | 2 | 768 of 4,256 | 18% | 13 of 276 | 5% | 1,824 |
| own generic, one instance each | 49 | 3 | 768 of 6,176 | 12% | 13 of 388 | 3% | 1,824 |
| generic Drop instance | 58 | 3 | 768 of 6,464 | 12% | 13 of 424 | 3% | 3,944 |
| generic instance calling a plain fn | 51 | 3 | 768 of 6,240 | 12% | 14 of 396 | 4% | 1,908 |

No shape sorted by the originals exceeds 50% of its grant after the
preallocation charge. The worst is the trivial function at 46%.

The last row's instance body calls `h()`, so emitting it asks a header
query. It pays one demand work unit per instance, and its lowering work is
its originals'. The other three instance shapes ask no query while their
instances lower, and pay no demand work.

Header-account margins are unchanged by this split, as slope shares of the
header bound:

- trivial functions: 100 of 128 bytes per element (78%);
- `const fn`: 66 of 128 bytes (52%), and 12.8 of 16 work units per element
  (80%), or 72% of the bound at 1,024 items. That work was 98% before header
  queries asked during lowering moved to lowering's account.

The 13 declared `needs_drop` scaling inputs use at most 22% of lowering's
byte bound (`noise1024`) and at most 27% of the header byte bound. Lowering
leaves the header account exactly as analysis left it.

PA31 source-provider corpora, measured whole:

| Input | Lowering bytes | Share | Header bytes share |
| --- | ---: | ---: | ---: |
| source-const-drop work128 | 100,680 of 683,104 | 15% | 3% |
| source-index-places bound-128 | 94,204 of 676,704 | 14% | 60% |
| source-assign-ten bound-128 | 83,520 of 699,136 | 12% | 6% |
| source-va-runtime work128 | 126,056 of 1,125,984 | 11% | 55% |
| 14 further inputs | | at most 7% | at most 49% |

Five inputs remain unmeasured, because the probe cannot build them with a
generic tag:

- source-panic-handler provider: resolve fails with `unresolved value name`;
- source-integer-niches some-zero and source-nonzero-methods some-zero: `value
  is outside its scalar pattern range`;
- source-binary-five bound-128: it needs a nonempty aligned `repr(C)` struct;
- source-promotable parent-policy: its stability belongs to another
  declaration.

Each needs its own report tag and sources. The static shape is measured
through a `sync` lang item and the `auto_traits` feature.

### The superlinear chain

| n | Instances | Lowering bytes | Byte share | Lowering work | Work share | Demand bytes | Demand work |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | 528 | 30,984 | 12% | 545 | 3% | 806,688 | 992 |
| 64 | 2,080 | 55,560 | 12% | 1,022 | 3% | 3,190,912 | 4,032 |
| 128 | 8,256 | 104,712 | 12% | 1,979 | 3% | 12,690,080 | 16,256 |

All three rows come from the committed control. Each instance pays about
1,530 demand bytes, so the 64 MB cap is reached near 41,800 instances
(n ≈ 288).

Both of lowering's shares stay flat. Lowering work grows by about 15 units
per item, and before the demand scope its share was 9%, 17% and 32%. Demand
work is exactly 2·(instances − n): each instance that forwards to another
instance pays two units for its emission's header queries.

`forged_instance_charges_pay_only_the_demand_budget` forges 64 work units
and 1,000 bytes into a `charge_caller_inputs` call at the start of every
instance's `specialize`, at n = 64. That is 133,120 units, more than four
times lowering's work bound of 30,516. Lowering's account stays at exactly
1,022 units and 55,560 bytes, the unforged figures. The demand budget gains
65·2,080 work units (each forged charge also pays one authentication unit)
and 1,000·2,080 bytes. With the cap preloaded so that the unforged chain
still fits, the forged chain is rejected with the demand budget's message,
never lowering's.

### Rejected options

- A, keeping demanded growth on lowering's account: `generic Drop instance`
  took 56% of its grant, failing the 2× rule, and the account rejected valid
  chains from n = 16.
- B, a fitted per-element constant large enough for the measured instance
  shapes: any fitted constant still rejects a valid chain at some n, a false
  rejection that grows with the input.
- D, B with more slack: the same false rejections, later, and a looser bound
  for genuine superlinear defects.

### Open items

1. The demand budget's fixed 64 MB and 100,000-body cap has not been measured
   against large std clients. Its adequacy is open.
2. The originals' `Vec<Body>` (456 bytes per body) is not charged on any
   ledger. `body::build::concrete_selected` reserves it with
   `Vec::with_capacity(facts.functions.len())` and pushes uncharged. It belongs
   to the body phase's ledger, charged at build where it is allocated, not to
   lowering's account.
3. Closed: header queries and caller inputs charged while a demanded instance
   is traversed or emitted pay the demand budget through the lexical demand
   scope (section above). They no longer reach lowering's account.
4. A crate without a ledger opens no lowering account. Its demand growth pays
   only the demand budget.

### Size pins

- `Declarations` stays 7,152 bytes, pinned at compile time in
  `dev/tests/invariants/prototype_headers.rs` and asserted at run time by
  `registry_retains_distinct_identities_and_shared_finite_profiles`.
- `Headers` gains `lowering: Option<Account>` inside the boxed `Ledger`, one
  per crate. It grows from 136 to 224 bytes: 176 with the account, and 224
  once the account holds the demand scope and the carried boundary. `Headers`
  stays inline: the 88 bytes are paid once per crate inside the boxed
  `Ledger`. Boxing the account would add an allocation and an indirection to
  every lowering charge on the hot path. The pin's comment records this
  reason. A grep for every spelling, summed or single, found one assertion:
  `ledger_capacity.rs` pins `Headers`. That pin is updated, and its test is
  selected. The 176 and 224 spellings elsewhere in `dev/` pin unrelated
  structs. The other spellings are diagnostic prints:
  - `attribute_storage` in `declared_origins.rs`, read by
    `compose_operands.rs` and `cpu_policy.rs`;
  - `pointee_consumers.rs`, `origin_integrity.rs` and `always_const.rs`.

  The `Headers` spellings in `typed_headers` and `metadata_contracts` name
  other structs. Creating the ledger charges no size of its own, so no header
  byte total moves.
- The proof `Builder` pin moves from 1,776 to 1,784 bytes. No other spelling
  of that size is tested; the historical checkpoint documents keep their
  observed 1,776.

### Controls

`dev/tests/invariants/lowering_ledger.rs`, mounted in
`lower::instances::proof::origins` as `ledger_tests`:

- `declared_scaling_inputs_lower_inside_both_accounts`:
  - the generated 64- and 1,024-nominal inputs equal the declared fixtures;
  - all 13 declared inputs stay inside half of lowering's bound;
  - lowering leaves the header account as analysis left it.
- `ordinary_shapes_pay_at_most_half_of_what_their_inputs_grant`: trivial
  function, literal and unnamed constant, static, noise, Drop local,
  constructor and `const fn`, from 256 to 1,024 items. For each:
  - lowering's bytes and work grow by at most half of what its bound grows by;
  - the header account grows by less than its bound does;
  - both sizes stay inside half of lowering's bound.
- `trivial_functions_with_drop_rows_lower_beyond_the_shared_header_bound`: 256
  and 1,024 functions lower. At 1,024, analysis and lowering together need
  more than the header bound, so the old shared account would fail.
- `demanded_instances_pay_the_demand_budget_once_by_their_own_size`:
  - four instance shapes each pay constant demand bytes and constant demand
    work per instance;
  - only the shape whose instance calls a plain function pays demand work;
  - lowering's account follows only the originals;
  - one instance pays 3,248 demand bytes after either 256 or 1,024 trivial
    originals;
  - demanding each instance twice yields the same symbols and the same demand
    bytes.
- `superlinear_instantiation_lowers_until_the_fixed_demand_cap`:
  - at n = 32, 64 and 128 the chain emits exactly n + n(n+1)/2 symbols beyond
    the base, a private LowIR equality;
  - demand bytes and demand work grow quadratically;
  - lowering bytes and lowering work grow linearly, both inside half of
    lowering's bound;
  - with the cap preloaded to twice the 32-item demand, 32 still lowers and 64
    fails with the demand budget's own message.
- `forged_instance_charges_pay_only_the_demand_budget`: the forged chain
  above. Its lowering account and header account equal the unforged run's,
  and only the demand cap rejects it.
- `superlinear_lowering_charges_are_rejected_once_inputs_grow`: a forged
  quadratic charge on the opened account, through both
  `charge_caller_inputs` and `pay_lowering_growth`, fits at 256 functions and
  fails at 1,024 for work and for bytes. The header account is untouched.

`dev/tests/invariants/lowering_account.rs`, mounted in
`sema::header_attributes::local::lowering_account` as `tests`:

- `charges_move_to_lowering_only_once_its_account_opens`:
  - the same header query and payment land on the header account before
    opening and on lowering's account after;
  - a different crate's report cannot open the account;
  - reopening keeps its totals;
  - charges beyond its bound fail with lowering's message, leaving the header
    account untouched.
- `demand_scopes_collect_charges_only_while_open`:
  - before the account opens, a scope collects nothing;
  - once it opens, a scope collects a query, a payment and a charge beyond
    lowering's bound, a nested scope collects nothing, and lowering's account
    does not move;
  - a scope whose closure fails still closes, so the next charge lands on
    lowering's account;
  - emission scopes only bodies at or after the carried first index;
  - after `settle_demand`, and after reopening, nothing is scoped;
  - the header account does not move.
- `crates_without_a_ledger_open_no_account`: also, scopes collect nothing and
  nothing is carried.

`consumed_originals_pay_their_rows_once_and_shrinking_pays_only_work` now
measures lowering's account. Its other controls moved to `ledger_tests`.

### Checks

Receipts are under `obj/qualification/fast/`.

| Selection | Receipt | Result |
| --- | --- | --- |
| Measurement: the uncommitted probe and the first controls | `1794147536548509120` | the tables before the demand scope; two controls amended (constructor demand, forged charge rounding) |
| Demand scope: the 7 ledger controls, the 3 account controls and the `Headers` pin | `1794150551512766930` | chain and measurement tables above; three controls amended (per-item demand work is constant rather than positive, the calling-instance shape added, a forged charge's authentication unit) |
| The three amended controls | `1794150812994707362` | 3 passed |
| Private: the 7 ledger controls, the shrink control, 6 `Originals` row tests, the `needs_drop` kernel test, `d2bcb1f89`'s 8 header tests and the 3 account controls | `1794151386139192365` | 26 passed |
| Private: the `Headers` pin test, the chain control, 8 proof tests, 10 adapter tests and 3 sealed-permission tests | `1794151663494627357` | 23 passed |
| Integration: `access_sources`, `stream_sources`, `needs_drop_provider`, `cstr_sources`, `vactx` and `asynccleanup` | `1794151722083032424` | 25 passed |
| Integration, demand-budget consumers: `demanded_cleanup`, `instance_values` and `source_closures` growth limits, and `instance_bodies` | `1794152027775849598` | 10 passed |
| Exact `--ignored` native scaling test | `1794152153688188003` | 1 passed, all 13 inputs |

The last five receipts cover the final tree, both commits rebased onto
`89cb62ee5`: each receipt's source fingerprint equals that tree's.
`asynccleanup` and `instance_bodies` are selected for the changed emission
and future-entry charges in `lower/context.rs`.
