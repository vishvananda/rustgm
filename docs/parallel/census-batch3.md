# Census batch 3: lint ledger, builtin operands and the fourth census

The third original-core census ([batch 2](census-batch2.md)) ended at
"source lint policy work budget exceeded" in `ops/arith.rs`. That was the fixed
ten-million-step cap of the source lint-policy ledger. This batch attributes
that ledger, bounds it by its admitted input and repairs the next language
causes in Lane A's area.

## 1. Lint-policy ledger

### Attribution (analysis only)

A temporary patch recorded the immediate caller of every lint ledger charge,
with the cap raised. It was never committed and is preserved as
`obj/parallel/census-batch3/attribution.patch`, with the census in
`obj/parallel/census-batch3/attr1-census/`. The inputs were unchanged original
core (`census-batch2/qualified-2/manifest.json`), at main `7a54504d7`. The run
reached the end of the declaration loop: 41,618 checked, 76 failed, 616
blocked. Core admits 14,668,689 original bytes.

| Payer | Steps | Share |
|---|---|---|
| Expansion frames and calls | 2,088,708 | 20.5% |
| Per-node `changes()` and owner validation | 1,645,098 | 16.1% |
| Decorated-call attribute reads | 1,585,551 | 15.6% |
| Owner production, attribute parsing, ineffective capture | 1,573,517 | 15.4% |
| Sema safety: one charge of retained nodes | 1,373,705 | 13.5% |
| Sema header owner re-authentication (`validate_owner`) | 969,705 | 9.5% |
| Final `snapshots::validate` | 685,798 | 6.7% |
| Other sema | 228,831 | 2.2% |
| Other | 36,064 | 0.4% |
| **Total** | **10,186,977** | 0.69/B |

Attempted storage was 53.5 MB (3.65 bytes per admitted byte). Owner production
paid 40.3 MB of it.

Every payer is linear in its input. The one repeated payer is header owner
re-authentication: 9,463 validations of 1,661 owners, 5.7 per owner. Each use
re-checks a constant-size owner record. That is linear in uses, and the LIB-193
control `header_work_stays_inside_its_input_graph_bound_only_while_linear`
covers that path, so it is left unchanged and listed for the integrator. No
factor below absorbs it: it is under 10% of core's work, and the densest
families below do not use it.

The separate import-lineage ledger (`lineage_budget`, the same `Work` type)
used 4.07M steps and 54.8 MB, 82% of its fixed 64 MiB. It is not changed here.

### Bound

The ledger now follows the macro-contract precedent:

- A fixed setup floor equal to the former limits (10,000,000 steps, 64 MiB), so
  no previously accepted input loses budget.
- Each admitted source identity grants 8 steps and 64 attempted bytes per
  original byte. The loader counts each identity's original bytes once, in
  `Package::admitted_source_bytes`, at the same admission that grants the
  macro ledger.
- A producer created after admissions is granted their total before its first
  charge. The expansion engine grants an existing producer after each module
  load and `include!`, before it visits the new source.
- An exhausted producer is never granted again. A later admission neither
  refunds the failed attempt nor clears it.

The factors are sized from the densest linear families measured, not from core
alone:

| Family (512 → 4096 items) | Steps/B | Attempted B/B |
|---|---|---|
| Original core | 0.69 | 3.65 |
| One `#[allow(unused)]` item per line | 0.67 → 0.65 | 24.4 → 23.8 |
| Sixteen lint names per attribute | 0.49 → 0.48 | 11.1 → 11.0 |
| One `m!{}` invocation per line | 3.49 → 3.76 | 30.5 → 31.8 |

Each factor is a power of two with at least 2× headroom over the densest
family (2.1× steps, 2.0× bytes) and 11× over core. A first draft sized the
factors from core alone (4 steps and 16 bytes per byte). Its first run failed
the one-attribute family at 24.4 bytes per byte (`fast-lint1.log`), so the
families were measured and the factors resized. Those families are linear, not
repeated work. For core the allowance is
127M steps and 1.0 GB attempted. The invocation family's frame lookup is a
binary search over the sparse frame table. Its log factor is bounded by the
32-bit frame domain, and per-call work did not grow from 512 to 4096 calls
(19.3 and 19.0 steps).

### Qualification

These controls were declared before the runs. `make fastcheck` passed 18 exact
private tests plus strict private Clippy, format and architecture
(`obj/qualification/fast/1794069155529993010`).

New, in `limit_tests` (`dev/tests/invariants/exhaustion_limits.rs`):

- `lint_ledger_grants_each_admitted_identity_once_before_reading_it`: the
  floor, the factors and the grant at 1, 4 and 32 copies of aliased modules and
  includes, plus macro-generated requests that grant nothing.
- `lint_grants_only_grow_and_never_reset_an_exhausted_ledger`.
- `linear_attribute_family_scales_and_exponential_output_grants_nothing`: the
  three families above stay inside the factors and scale within 5/4. Doubling
  attributed macro output at depths 9 and 12 grants only the admitted bytes,
  while lint work grows 7.98× and storage 7.95×. At depth 16 expansion fails with
  "source lint policy storage budget exceeded", before the macro ledger leaves
  its floor.

The existing `each_admitted_identity_grants_once_and_expansion_output_grants_nothing`
now builds its input with the shared `admission_family` helper; its assertions
are unchanged. The existing depth-40 macro control still fails.

Amended controls (each now reads the producer's actual limits instead of the
literal floor; the asserted behavior is unchanged):

- `deprecation_suggestions`: `Producer` grows from 368 to 384 bytes, because
  both of its `Work` records hold the grant.
- `decorated_calls::tests::costs::exhausted_attribute_payer_blocks_children_and_failed_retries`
- `endpoint_permissions::tests::generated_endpoint_permission_rejects_copied_nodes_changed_roots_and_foreign_reports`
- `hidden_globs::payer_tests::failed_new_diagnostic_allocations_are_sticky_and_publish_no_prefix`
- `hidden_globs::payer_tests::completion_checks_both_failed_owners_and_the_actual_cap_before_empty_results`
- `implementations::future_tests::reservation_source_witnesses_are_current_and_failed_queries_retain_work`
- `ineffective_headers::tests::exhausted_annotation_observation_keeps_each_failed_attempt_paid`
- `expectations::completion_tests::failed_completion_payment_retains_attempted_work_without_finalization`

Tests that build `Producer::new` directly keep the bare floor and are unchanged.
`Package` stays 256 bytes: its admitted-byte count is a saturating `u32` in
existing padding.
