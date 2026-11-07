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

## 2. Builtin binary operand types

Census 3 blocked 478 items behind `{ $size * $elems }` in portable-simd
`to_bytes.rs` and `($n - 1)` in `array_impl_default`. With two candidate
impls (`Mul` or `Sub` for `usize` and for `u8`), trait selection alone cannot
choose. rustc first applies its builtin rule (`is_builtin_binop` and
`enforce_builtin_binop_types` in `op.rs`):

- It applies only when neither operand is a general type variable. Integer and
  float inference variables count as known.
- One shared reference layer is peeled from each operand.
- Arithmetic (`+ - * / %`): both integral or both floating. The operands are
  unified and the result is the left operand's type.
- Bitwise (`& | ^`): as arithmetic, or both `bool`.
- Shifts: both integral. Only the result is related, to the left operand; the
  right operand is never unified.
- Comparisons: both scalar. The operands are unified; the result stays `bool`.
- Compound assignment uses the same categories; its result stays `()`.
- The selected method's `Output` must still equal the builtin result.

The body checker now does the same in the existing source operator paths
(`source_binary_operator`, `source_comparison_operator` and
`source_assignment_operator`), after the ordinary trait obligation and method
instantiation. No operator contract changed, so the trait proof, method
clauses and const effect are still checked. One deviation is deliberate:
comparisons apply the rule only to primitive scalars (integers, floats, `bool`,
`char` and their inference variables). Raw and function pointer comparisons
keep ordinary trait selection. In core those impls only compare a type with
itself, so this changes only the wording of an error, not an outcome.

Operand classification never forces a pending projection that waits for a
definition (see section 3).

### Controls

Twelve no_core controls, declared before the rustc 1.98.0 oracle and before the
implementation, are in `pa31/tests/spec/builtin-binop-types/` with
`outcomes.json`. They cover: literal `Sub` and `Mul` (batch 2's control, a
`{ 2 * 4 }` const argument and impl self-type lengths), shift result and an
unresolved shift right operand (E0277), floats, a `&u8` right operand,
compound assignment, a literal comparison, `u8 - usize` and `u8 == u16`
(E0308), and `3 * m` with a non-numeric `M`, which still selects the only
`Mul<M>` impl by trait.

The first oracle run hit a rustc internal compiler error in
`shift-result-lhs.rs`: a `fully_perform` delayed bug at the no_core
`const impl` bodies `self<<r`. That is not an outcome. Before the re-run, the
two impl bodies were replaced with `self`. The typed constant and the declared
outcome did not change; the amendment is recorded in `outcomes.json`. Both
oracle runs and the ICE reports are in `obj/parallel/census-batch3/controls/`.

| Control | Declared | rustc | Pin before | After |
|---|---|---|---|---|
| sub-literals | accept | accept | reject | accept |
| mul-const-argument | accept | accept | reject | accept |
| sub-impl-lengths | accept | accept | reject | accept |
| shift-result-lhs | accept | accept | reject | accept |
| shift-rhs-not-unified | reject | E0277 | reject | reject |
| float-literals | accept | accept | reject | accept |
| reference-rhs | accept | accept | reject | accept |
| user-impl-by-trait | accept | accept | accept | accept |
| mixed-width | reject | E0308 | reject | reject |
| assign-literal | accept | accept | reject | accept |
| compare-literal | accept | accept | reject | accept |
| compare-mixed-width | reject | E0308 | reject | reject |

The opponents' messages are "associated type requires an unambiguous
validated implementation" (shift) and "incompatible types" (both
mismatched widths). `dev/tests/invariants/binop_operand_types.rs` checks all
twelve.

One existing control is amended. `binary_contract::tests::nested_fallback_respects_predeclared_order_and_shared_frontiers`
asserted that `pa31/tests/spec/source-shift-or/readiness/shared.rs` stays
rejected. That case, `let rhs=1;(n<<rhs)|rhs` with `n: usize`, was recorded as
an "explicit unresolved frontier"
([source shift checkpoint](../source-shift-or-checkpoint.md)). Its frozen
independent observation is acceptance in both editions
(`readiness/observations.json`). The builtin rule now settles it: the shift
result is `usize`, and `|` unifies the literal with it. The control now
requires acceptance and a lowered `shared` body. The fixture and its
observations are unchanged.

## 3. Unary operands behind a definition wait

Batch 2's `static-length-neg-sync.rs` (rustc accepts) failed with "missing
completed associated type definition" at the `-1` of `SMALLEST`. That `const`
is read through `COUNT` into a static array length, and the `Neg` impl follows
the static.

The cause was not readiness of the `Neg` members. After instantiating
`neg`, the unary path classified its operand with `contextual_head`. That call
advances every pending projection inline, including the operator's own
`Output`. Normalizing `<i32 as Neg>::Output` needs the impl's associated type
definition, which is not declared yet. The inline advance cannot suspend, so it
reported the wait as an error. The binary path made no such call, which is why
the binary shape already passed.

The new `settled_head` advances pending projections as far as they can go. It
still prepares the on-demand dependencies the inline advance prepares (impl
headers for range contracts and source result contracts). In declaration
availability, a projection that waits for an absent definition stays queued.
The body loop polls it before its next work item and suspends, so the
definition is prepared before the body resumes. Classification only sees an
unknown head, which selects no builtin rule. Elsewhere `settled_head` is the old
inline advance. Both operator paths classify with it.

The two batch-2 controls (declared and oracle-checked in batch 2's third
declaration) are now in `pa31/tests/spec/demanded-header-facts/`, and
`demand_header_facts.rs` checks them with the other 13. `static-length-neg-sync.rs`
is accepted. `static-length-missing-neg-sync.rs` (E0600) is still rejected,
with "associated type requires an unambiguous validated implementation".
All 16 batch-2 controls keep their outcomes and messages
(`probe-binop1-regress.json`).

## 4. The 256-bound "type constraint work budget exceeded"

Batch 2 left a generated case at this budget once the header ledger admitted
it. The family is one function with `n` associated-type bounds on one
parameter:

```rust
fn f<T: A<Out: B<0> + B<1> + /* ... */ + B<{n-1}>>>() {}
```

The budget is the fixed per-`Context` cap of one million inference steps in
`types/infer.rs`. The work under it is superlinear, so it is not a linear
workload that needs a larger cap. An analysis-only attribution with the cap
raised (`budget256/attr-wide*.infer.tsv`) shows two regimes:

| Bounds | Inference steps | `alternatives::choose` probes |
|---|---|---|
| 16, 64, 128, 160 | 4 per bound (64 to 640) | none |
| 192 | above 1,000,000 | 31,144 sampled |
| 255, 256 | above 2,000,000 | 62,346 sampled |

The payer is declaration bound validation: `validate_trait_bound`, then
`check_trait_assumption`, then `Assumptions::unambiguous_declaration`, then
`select`, then `alternatives::choose`. To prove each own bound unambiguous,
`choose` trial-unifies against every assumption with the same trait and head.
All `n` bounds share trait `B` and head `u8`, so the scan is `O(n^2)` trial
unifications. Each one imports, resolves and equates a constant argument
(`import.rs`, `resolve.rs`, `equations.rs` and `constants.rs` in the table's
source). With the binop build, 192 bounds pass in 1.1 s and 256 fail. Why the
scan starts between 160 and 192 bounds was not explained; below that,
`choose` is not reached.

This batch does not fix it. A cap increase would hide the slope. The intended
repair is a discriminator in `premise_index` (or a pre-filter before `choose`),
so that assumptions whose closed constant or type arguments differ from the
goal are excluded without a trial unification. That makes the own-bound check
linear in the matching candidates. Its controls would be this family at 64,
256 and 1,024 bounds with flat work per bound, plus a genuinely ambiguous
opponent (two bounds that unify with the goal), which must still be rejected.
