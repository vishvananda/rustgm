# Needs-drop native scaling repair

Lane B repair of 2026-11-08 on main `e02716c40`.

## 1. Regression

| Item | Value |
| --- | --- |
| Test | `needs_drop_provider::linked::source_needs_drop_scaling_native_answers` (`suite_frontend`, ignored, native) |
| Failure | `width8-calls8-noise1024: input: caller inputs exceed source work/storage budget`, inside `lower(source)` before the backend, deterministic |
| Seen | full runs 42471803252 and 42472038917, both on `ad05f4d61` |
| Last native pass | nightly 42208093492 on `31c3a6077`; every full run between them skipped native |
| First bad commit | `d2bcb1f89` "Validate each target-feature enable spelling once and bound header work by its input graph" |

An exact `--ignored` fastcheck on `e02716c40` reproduced the failure. The
other 12 inputs passed natively. Direct builds of `suite_frontend` then ran
the same test at both sides of `d2bcb1f89`:

- its parent `aa7881ae3` passes all 13 inputs, `noise1024` included;
- `d2bcb1f89` fails exactly as main does.

The bound tightened a ledger that other phases share. Only the ignored native
test lowers 1,024 unrelated bodies, and no lane selection included it, so the
failure stayed invisible for two days.

## 2. Cause

`d2bcb1f89` replaced the header ledger's fixed limits (1,000,000 work,
16,000,000 bytes) with its input-graph bound of 16 work units and 128 bytes per
element. It changed `Payments::charge` the same way, and `Payments::charge` is
the payer behind `charge_caller_inputs`, which lowering uses. The
[LIB-193 sizing](lib193-header-budget.md) measured header work only up to
`Checker::run`.

An analysis-only attribution of every byte charge on `e02716c40` gave these
figures. The instrumentation was not committed. For `noise1024` the bound is
191,552 work and 1,532,416 bytes:

- Analysis ends at 9,820 work and 417,904 bytes.
- Lowering first crossed the bound in `Originals::shrink`, reached from
  `demand::prepare_ordinary`. That single charge was 764 work and 210,864
  bytes.
- With enforcement off, lowering completes at 19,076 work and 1,848,856 bytes:
  121% of the byte bound and 10% of the work bound.
- All of lowering's byte growth comes from `Originals`: 594,120 bytes in `take`
  and 836,832 in `shrink`.

Each added noise nominal adds 11 input elements, which grant 1,408 bytes. It
also adds one anonymous array-length body, which cost:

- 400 bytes of header analysis;
- 576 bytes in `take`;
- 823 bytes in `shrink`.

`shrink` charged `len × (456 + 96)` bytes, the sizes of `Body` and
`Requirements`, each time fewer than half of the consumed bodies were live. So
it paid for that storage a second time while releasing it. At 1,799 bytes
against 1,408 per nominal, the input fails above about 216 noise nominals.
`noise64` passed only on fixed headroom, at 67%.

## 3. Repair

- `Originals::shrink` pays work only: one unit per moved body and row. This
  follows the path_compatibility precedent: bytes are charged when retained
  storage newly grows. `shrink_to_fit` never raises retained storage above
  what the builder and `take` already paid for.
- `Originals::take` is unchanged. It still charges:
  - the row owner;
  - the replacement row vector;
  - the moved pending row vector.

  The moved vector was never charged when `body::drops::collect` built it, so
  `take` is its only payment, not a double count. The charge stays at `take`
  because the allocation site runs in the body phase, before lowering creates
  the row payer from the report.
- Test only: `source_contract_bound` and `source_contract_costs` become
  `pub(crate)` under `cfg(test)`, so lowering controls can read the ledger.

No bound, budget, wire byte or layout changes.

## 4. Controls

`dev/tests/invariants/shrunk_originals.rs`, mounted in
`lower::instances::proof::origins` as `shrink_tests`:

- `declared_scaling_inputs_lower_inside_their_input_bound`: every input the
  native test lowers stays inside its bound, without the backend.
- `unrelated_constant_bodies_pay_less_than_their_input_bound_grants`:
  - the generated 64- and 1,024-nominal inputs equal the declared fixtures
    byte for byte;
  - from 64 to 256 and from 256 to 1,024 nominals, both total and lowering
    bytes grow less than the bound does;
  - the per-nominal increments stay linear, apart from row capacity doubling.
- `literal_constant_items_pay_less_per_element_than_their_input_bound`: the
  tiny-body case, measured from 256 to 1,024 items (section 5).
- `consumed_originals_pay_their_rows_once_and_shrinking_pays_only_work`, at
  1,024 nominals:
  - retained growth stays charged: `take` pays exactly the row owner, the
    replacement estimate and the built row vector;
  - consuming every body adds no bytes;
  - consumption costs one work unit per moved body and row, plus one per
    payer charge;
  - the old-charge control: the halving charges this repair removes total
    836,832 bytes, at least half of the consumed body and row storage.

Lowering's own account later moved the first three controls to
`lowering_ledger.rs` and measures the fourth on that account. See
[lowering's own account](lib193-header-budget.md#lowerings-own-account-2026-11-08).

## 5. Margins after the repair

| Input | Bytes after lowering | Bound | Share |
| --- | ---: | ---: | ---: |
| width8-calls8 | 14,136 | 90,624 | 15% |
| width8-calls32 | 27,960 | 155,136 | 18% |
| width8-calls128 | 83,256 | 413,184 | 20% |
| width32-calls8 | 14,136 | 99,840 | 14% |
| width32-calls32 | 27,960 | 164,352 | 17% |
| width32-calls128 | 83,256 | 422,400 | 19% |
| width128-calls8 | 14,136 | 136,704 | 10% |
| width128-calls32 | 27,960 | 201,216 | 13% |
| width128-calls128 | 83,256 | 459,264 | 18% |
| width8-calls8-noise64 | 75,064 | 180,736 | 41% (67% before) |
| width8-calls8-noise1024 | 1,012,024 | 1,532,416 | 66% (121% before) |
| ordinary-unused-intrinsic | 9,912 | 68,864 | 14% |
| primitive-calls128 | 6,584 | 407,808 | 1% |

Work stays at or below 10% of its bound for every input. These are bytes paid per
added input element, measured from 256 to 1,024 items:

| Item | Total | Lowering | Share of 128 |
| --- | ---: | ---: | ---: |
| `struct NoiseK([u8;K]);` | 88 | 52 | 69% |
| `const CK:u8=0;` | 115 | 115 | 90% |
| `const _:u8=0;` | 115 | 115 | 90% |
| `type AK=[u8;0];` | 0 | 0 | 0% |

## 6. Open: lowering needs its own measured bound

Resolved by
[lowering's own account](lib193-header-budget.md#lowerings-own-account-2026-11-08).
The record below is kept as it was reported.

These figures were reported to the coordinator rather than fixed here. The
input-graph bound stays in place: the fix would be separately measured
per-phase ledger constants, never a raise of the header bound.

- Literal constant items take 90% of the bound per element, above the 75%
  guidance. Each item has five input elements. All 576 bytes per item are
  `Originals::take`:
  - 384 bytes for the row-vector `Growth` estimate (four slots per row);
  - about 192 bytes for the moved row vector, which grew by doubling.
- Trivial functions such as `fn fK(){}` have four input elements each and pay
  244 bytes per element: 100 bytes of header analysis and 144 of lowering.
  That is 190% of the bound. Measured from 16 to 64 and from 64 to 128
  functions, the base crate with its Drop rows lowers 192 such functions at
  99% of its bound and fails at 256 with this repair's error. This was an
  exploration probe, not a committed control.

## 7. Checks

Receipts are under `obj/qualification/fast/`.

| Selection | Receipt | Result |
| --- | --- | --- |
| Reproduction, `--ignored` native test on `e02716c40` | `1794138339901193495` | failed as reported |
| Private: 4 new controls, 6 `Originals` row tests, the needs_drop kernel test, and `d2bcb1f89`'s header tests (4 target-feature, 3 ledger-capacity, 1 prototype-header) | `1794142059008640675` | 19 passed |
| Integration: `access_sources` and `stream_sources` from `d2bcb1f89`, and the three non-native `needs_drop_provider` tests | `1794142227328358662` | 5 passed |
| Exact `--ignored` native scaling test | `1794142419018985848` | 1 passed, all 13 inputs |
