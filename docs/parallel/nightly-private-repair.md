# Nightly private-test repair

The nightly qualification has failed since 2026-11-07. Its last good run was
`31c3a6077` and its first bad run `054b91950`. Joined checks select only each
lane's own tests, so they missed these failures. Re-run by exact name on main
`4a9a3441f`, 17 private tests and 2 integration tests still failed. Three
further assertions were masked behind the first failing assertion of a test:
the expansion `Report` (3,096 to 3,104 bytes) and the lint `Producer` (368 to
384 bytes).

Every cause was found from the diff of the range, then confirmed by
measurement. Layout sizes were printed from the built test harness. Budget
failures were attributed per call site with a temporary `#[track_caller]`
probe on every header-ledger charge. The probe was removed before this
checkpoint. No intermediate commit was rebuilt.

Two causes are real defects, and this checkpoint fixes them in the compiler.
The input-graph bound of `d2bcb1f89` exposed both. The other failures are
justified layout growth, or tests that hard-coded a fixed budget that
`054b91950` replaced with an input-proportional grant. For those, the assertion
is updated with its reason, or the test now reads the actual grant.

| Test | Cause commit | Fix | Reason |
| --- | --- | --- | --- |
| `resolve::bindings::admission::tests::ordinary_admission_inspects_no_syntax_and_language_requests_stay_packed` | `054b91950` | `SourceCore` 200 → 240 | The macro-contract ledger gained its 24-byte admitted-source grant, and its three limits became optional caps (12 → 24 bytes). `Contracts` is 160 bytes. The provider is one cold box; `Package` stays 256. |
| `resolve::bindings::policy::targets::foreign_requests_retain_no_cpu_permissions_or_ordinary_binding_owner` | `054b91950` | `SourceCore` 200 → 240 | As above. |
| `resolve::bindings::policy::tests::ordinary_requests_do_not_allocate_a_provider_or_library_policy` | `054b91950` | `SourceCore` 200 → 240 | As above. |
| `sema::compiler_items::panic_declarations::appended_claims_preserve_ordinary_storage_ordinals_and_closed_wire_profiles` | `054b91950` | `SourceCore` 200 → 240 | As above. |
| `sema::traits::selection::search::refutation_layout::source_auto_and_refutation_layouts_are_measured` | `054b91950` | `SourceCore` 200 → 240 | As above. |
| `resolve::expansion::decorated_calls::tests::costs::ordinary_attributes_do_not_spend_or_revive_panic_permissions` | `054b91950` | Stop one unit below `Contracts::work_limit()` | The test exhausted the old fixed 1,000,000-unit default. The grant is now the setup floor plus 2 units per admitted byte. A new test-only accessor reads the capped grant, beside the existing `attempted_limit()`. `054b91950` amended its own panic-permission control the same way. |
| `resolve::expansion::derive_lists::tests::union_copies::whole_clones::rejected_actual_union_payment_leaves_source_and_effect_unpublished` | `054b91950` | Exhaust `attempted_limit()` | The test exhausted the old fixed 64 MiB attempted-byte limit. |
| `resolve::expansion::permission_session::tests::unsafe_lint::unsafe_lint_metadata_keeps_witness_and_global_failed_budget_on_warm_queries` | `054b91950` | Charge `work_limit() + 1` | The test charged the old fixed 1,000,001 units. |
| `resolve::hygiene::tests::an_import_alias_keeps_its_own_identifier_context_when_published` | `6f8c51a46` | Exact copied fixture origins | Before Rust 2018 a `use` path is relative to the crate root, so lowering now reads the edition of the path's first token. That needs an exact copied origin. The fixture copied every generated token from the whole `install!();` span, which is longer than the token. Each token is now copied from a same-spelling span in a trailing comment, as a transcribed definition token is. Real expansion always records exact copies. |
| `resolve::hygiene::tests::an_opaque_star_retains_root_segments_without_relabeling_them` | `6f8c51a46` | Exact copied fixture origins | As above. |
| `resolve::hygiene::tests::grouped_imports_keep_each_segment_alias_and_star_context_independent` | `6f8c51a46` | Exact copied fixture origins | As above. |
| `resolve::impl_restrictions::tests::parsed_restrictions_keep_feature_origins_disabled_headers_and_ordinary_layouts` | `054b91950`, then `ab99d98c6`; masked `254ff6828` | `Source` 208 → 296, new `SourceClient` 296 assertion, `Report` 3,096 → 3,104 | The provider variant grew to 240 bytes. Then `ab99d98c6` held the client's imported-allowance ledger (a local macro context with its own contracts) inline in `SourceClient`, which made the client the largest variant. `Source` is one cold box. The lineage ledger's admitted-byte grant adds one word to `Report`; `6e202b72f` recorded it in other layout controls. |
| `resolve::lint_policy::producer::cap_tests::caps_preserve_retained_attribute_debt_and_compact_records` | `054b91950`; masked `254ff6828`, `5b632546e` | `Config` 80 → 96, `Report` 3,104, `Producer` 368 → 384 | `Config` holds the source-contract caps as three optional words. Both lint and lineage ledgers record their admitted-source grant; `deprecation_suggestions.rs` already asserts 384. |
| `resolve::stability::predicate_transport_tests::malformed_non_type_rows_and_legacy_profiles_cannot_drop_or_invent_debt` | `1f5e4c773`, settled by `18232e371` | `SourceRows` 784 → 792 | Version thirteen added inline associated definitions (808 bytes at the nightly) and enum tables. `18232e371` moved both into one cold box, so raw rows pay one pointer and frozen `Columns` stay 768. It amended `policy_transport.rs` but missed this control. |
| `support::crates::generics::source::descriptor::module_contracts::tests::complete_module_costs_prepay_sort_forest_transactions_and_enclosing_layouts` | `1f5e4c773`/`18232e371`; masked `254ff6828` | `SourceRows` 792, `Report` 3,104 | As above. |
| `verbatim_links::c_callable_safety_privacy_and_nominal_boundaries_remain_independent` | `1f5e4c773`/`18232e371` | `SourceRows` 792 | As above. This is a private test: `dev/src/lib.rs` includes `tests/invariants/verbatim_links.rs` as the crate-root module `verbatim_links`. Select it with `--lib-case verbatim_links::c_callable_safety_privacy_and_nominal_boundaries_remain_independent`. It is not in `suite_frontend`, which is why a suite selection is refused. |
| `sema::bodies::effect_facts::protocol_rows::tests::retained_try_storage_and_empty_paths_keep_actual_enclosing_layouts` | `d2bcb1f89` exposed an older defect | Geometric Try-plan growth (code) | Publishing each body's Try plans reserved exactly the new plans. Each growth copied every earlier plan and charged `length × 2 × size_of::<Plan>()`, which is quadratic in the crate's Try sites. Once the fixed `MAX_BYTES` cap became 128 bytes per input element, that one payer reached 317,952 of the 397,824-byte bound at width 128. Capacity now at least doubles. The test now also requires the per-function header bytes to stay flat from widths 8–32 to 32–128. |
| `const_eval::use_inputs::selectors::width_tests::reference_caller_width_and_instances_keep_one_paid_pass_and_lazy_descriptor_storage` | `d2bcb1f89` exposed repeated queries from `14d55ebfa` | Query the enclosing context once per body (code) | `callee_use_effect` runs at every callee use. Each run re-queried the enclosing function's source context: its comptime contract (8 units), its unchecked-const-body policy (8 units, from `14d55ebfa`) and its attribute words. At width 128 that was 15,244 queries and 121,952 of the 135,392 work units allowed at 16 units per element. The answer is fixed for one body, so the body checker now memoizes it per (environment owner, body). |
| `selected_references::selected_reference_callbacks_preserve_caller_allocations_and_whole_signatures` | `d2bcb1f89` (as above) | The memoized enclosing context | This test compiles the same `caller-scaling-128` fixture and failed with "defining const policy work budget exhausted". |
| `selector_bounds::wide_and_nested_constraints_keep_finite_completed_work` | `d2bcb1f89` | None here | Projected-bound staging overran the header bound. `4d9ade297` repaired this after the nightly: it counts written bounds in the bound (see `census-batch2.md`). It passes with this checkpoint, but it was not run separately on unmodified main. |

`sema::compiler_items::prototype_validation::tests::reached_intrinsics_cannot_execute_their_authored_fallback`
is stale and belongs to Lane A, so it is left unchanged. Five more of the
nightly's 25 names already passed on `4a9a3441f` because of later amendments,
such as `48c5ad537` for the prelude control:
`choice_dispatch::...selected_builtin...`,
`default_warnings::...charged_projection...`,
`prelude_scope::...source_prelude_failure_payment...`,
`fn_allowances::...genuine_original_const_bodies...` and
`transport_tests::legacy_availability...`.

On this checkpoint rebased onto main `85dbaaff0`, fastcheck
`1794107808283337632` passes all 33 selected private tests (the 17 repaired
ones and their neighbours in the same modules) with `selector_bounds` and
`selected_references`. The lib probe of `85dbaaff0` showed the same 17
private failures as `4a9a3441f`.

Neither code fix changes any wire byte, diagnostic or accepted program. The
Try-plan change affects only allocation growth and its accounting. The memo
caches a per-body answer that the body checker already treated as fixed.
