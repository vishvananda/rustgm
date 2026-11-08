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

## Second qualification: main `f98c5d7c8`

The full CI qualification of main `f98c5d7c8` (run 42450534500) failed 35
tests. Nineteen are the failures above, which this branch repairs but main did
not yet contain. The other sixteen are new since the 2026-11-07 nightly. Each
cause below was found by exact selection at a candidate commit and at its
parent. This checkpoint repairs the causes in this lane's area, the amended
controls and the accepted layout growth; the rest are listed at the end.

### Storage walks of declaration-time bodies (`65a9eae6c`)

`shape_constructors::early_type_arguments_share_declaration_dependencies`
(1,601,553 steps against a bound of 32,768) and
`shape_constructors::repeated_shape_defaults_reuse_completed_nominal_instances`
passed at the nightly `054b91950` and failed at `b55c0dc49`. Bisecting
`054b91950..b55c0dc49` with the two tests selected exactly, the first bad
commit is `65a9eae6c`; its parent `2a13ab4a9` passes both.

`65a9eae6c` made every declaration-time body walk its storage roots, through
`field_template_dependency`, to find a field template its declaration had not
published yet. That walk used the non-caching storage preparation. On success
it charged every visited type a second time and prepaid the dense success rows
up to the largest visited type id. The rows are reserved exactly, so each
growth also charges the copied capacity. Every body therefore paid again for
rows that earlier types already held, which is quadratic in the crate.
Missing-template failures were also no longer cached (next section).

Field instantiation steps for the test's chain, measured on the built test
harness:

| Constants | `2a13ab4a9` | `65a9eae6c` | This checkpoint |
| --- | --- | --- | --- |
| 64 | 1,727 | 28,177 | 1,728 |
| 128 | 3,455 | 105,489 | 3,456 |
| 256 | 6,911 | 407,569 | 6,912 |
| 512 | 13,823 | 1,601,553 | 13,824 |
| Repeated defaults | 18 | 74 | 19 |

The culprit is exactly 6n² + 56n + 17, the parent 27n − 1 and the repair 27n.
A chain whose structs are declared after its constants gives the same numbers.
The repair's one extra step is the body walk's own visit, which the parent did
not make.

The repair keeps batch 5's single wait per nominal:

- The walk is resumable. `PendingFields` holds its stack and visited set. A
  nominal without a published template stays on top of the stack, unvisited
  and uncharged, and the resumed walk continues there instead of at its root.
- Each visited type is charged once, when it is visited. The completed walk
  publishes its types as the crate-wide preparation does, without a second
  charge or a row prepayment.
- `FieldWaits` collects a body's solved storage roots once, since they are
  fixed while it waits. It also keeps the next root, so a completed root is
  never walked again after a wait.
- A second miss on the same nominal is still "missing aggregate field
  template".

Controls:

- The two `shape_constructors` tests pass unchanged, with their bounds.
- New `shape_constructors::early_type_argument_chains_keep_field_work_linear`
  requires equal increments per constant at 64, 256 and 512 constants. The
  culprit's measured steps fail it.
- New private
  `types::aggregate::preparation::tests::waited_field_templates_resume_their_storage_walk_linearly`
  publishes each template of a 64-, 256- and 512-nominal chain only after the
  walk waits for it. It requires linear steps, one wait per nominal and no work
  for the prepared root. As a control in the same test, a walk restarted at its
  root after every wait must cost more than 64 times the resumed one at 512.

A source-level wait control was not possible. A scratch probe counted waits
over every `pa31/tests/spec` control and over generated chains (struct
literals and array annotations, in and out of a module). Only
`signature-constant-privacy/signature-leaked-value-length.rs` waited, and only
once. In the generated chains, typing the bodies already publishes every
template.

### Final crate-wide preparation failures (`65a9eae6c`)

`pipeline::execution::layout_kernel::tests::structural_readiness_preserves_pending_fields_closedness_and_failed_work`
(queried_storage.rs:222) and
`nominal_instances::incomplete_field_graphs_never_publish_successful_descendants`
(17 steps against 11) fail at `65a9eae6c` and pass at `2a13ab4a9`.

`65a9eae6c` added `&& missing.is_none()` to the failure cache of the
crate-wide `prepare` and `prepare_equality`. A root whose walk met an
unpublished template was then never cached as failed. A repeated query walked
again, and a query after the template's publication succeeded, where these
controls require the first failure to be final.

The condition is not needed for batch 5's rule that a pending miss is never
cached. That rule concerns the storage preparation of declaration-time bodies
and layout demands, which never caches failures: before `65a9eae6c`, in it and
after this repair. The crate-wide preparation requires completed declarations,
so for it a missing template is a final failure. The repair restores that
cache, including a root that is itself the missing nominal. The storage path is
unchanged.

Batch 5's repaired case still passes with the cache restored. Its invariant
`early_constant_execution.rs` passes over all 12 `declaration-constant-execution`
controls, and `privacy_of_signature_constants.rs` and
`residual_object_equalities.rs` pass over the other 15 batch 5 controls.
`signature-leaked-value-length.rs`, the one control that waits, is among them.
The original-core census (diag5 to diag10) was not rerun, because it needs a
release pin.

### Amended control: `ctfix` (`65a9eae6c`)

`ctfix::no_drop_repair_keeps_reached_intrinsic_body_fences` fails at
`65a9eae6c` ("authored unsupported intrinsic executed") and passes at
`2a13ab4a9`. Batch 5 gave source `ctpop` a validated constant handler, so
`pa28/tests/spec/intrinsic-declarations/execution/const.rs` now evaluates
`ctpop(7u32)` to 3. Lane A's `e2c0ead9e` amended the sibling
`prototype_headers.rs` control and missed this one.

The control now mirrors `e2c0ead9e`:

- `const.rs` with a `[u8; 3]` length is accepted and with `[u8; 7]` (the
  fallback's value) is rejected as incompatible types.
- The same source with `bswap`, which has no constant handler, still fails with
  "source intrinsic ... not implemented".
- The unchanged `runtime` and `reified` fixtures still fail as before.

The no-drop repair therefore still keeps a reached intrinsic body fenced, and
the "authored unsupported intrinsic executed" assertion is unchanged.

### Accepted growth: `Declarations` 7,136 → 7,144 bytes (`c6b5642dc`)

Seven controls assert `Declarations` as
`7048 + size_of::<Vec<()>>() + 6 * size_of::<usize>() + 2 * size_of::<usize>()`:

- `body::iterator_factories::tests::{cloning::iterator_clone_preserves_ordinary_record_sizes_and_lazy_owner_cost, iterator_layout_keeps_ordinary_records_compact, mutable_collections::mutable_iteration_keeps_ordinary_record_sizes_and_records_lazy_catalog_cost, progression::range_catalog_preserves_ordinary_record_sizes}`;
- `sema::declarations::library_types::collection_tests::collection_layout_preserves_scalar_owners_and_padded_derive_entries`;
- `sema::methods::names::tests::actual_source_receiver_and_path_sites_retain_their_identifier_context`;
- `sema::methods::visible::tests::scope_walk_rejects_unknown_definition_sites_and_counts_failed_work`.

`c6b5642dc` added `pending_copy_selection: Option<selection::Dependency>`, one
word. `census-batch6.md` section 1.3 records the growth, but only
`prototype_headers.rs` (7,144) was updated. Each of the seven now adds one
`usize` with that reason. A search for `7048`, `7136`, `7144` and their
underscored forms finds no other exact assertion of this size; other uses only
print it.

### Accepted growth: `Imports` gains the deferred-use vector (`c5d056327`)

`resolve::stability::availability_imports::path_compatibility::tests::repeated_groups_share_real_event_dedup_and_warm_lookup_has_no_retained_growth`
asserts `Imports` as its legacy tuple plus 32 bytes (path uses and their
completion flag). `c5d056327` added `deferred: RefCell<Vec<Deferred>>`, the
denied uses inside expansions that await their frames' allowance lists. That is
exactly 32 bytes: 216 against 184. The control now adds
`size_of::<RefCell<Vec<Deferred>>>()` with that reason.

### Warm feature lookups charged retained bytes (`c5d056327`)

`resolve::stability::availability_imports::path_compatibility::tests::real_path_width_and_duplicate_feature_imports_keep_touched_cost_bounded`
fails at compatibility_routes.rs:212. After 128 warm `features` calls with the
same policy, the retained bytes are 40,456 instead of 37,768.

`c5d056327` split a `spelling` helper out of `feature`. The helper returned an
owned `String` even for an ASCII name and charged that name's length in
retained bytes on every call, including warm lookups of a feature already
recorded. The policy's feature is `availability_provider`, 21 bytes:
128 × 21 = 2,688 = 40,456 − 37,768. Before `c5d056327`, `feature` charged work
only and borrowed an ASCII spelling.

`spelling` now returns a borrowed or normalized `Cow<str>` and charges work
only. Normalization still pays its own growth. `feature` charges bytes only
when it inserts a new name, as before. The deferred use is the one caller that
retains the spelling, so it now charges the owned spelling's bytes beside its
vector growth. That is what the earlier code charged it, so the deferred path
is not under-charged.

The warm-lookup control is unchanged. The same test now also requires a
deferred use to grow the deferred list by one and the retained bytes by exactly
its vector growth plus its spelling's length. No wire byte, diagnostic or
accepted program changes.

`028bd3e16` passes both path-compatibility tests and `c5d056327` fails both,
at :148 and :212.

### Routed to their owning lanes

Two further new failures were bisected and left to their owners:

| Test | Passes at | First fails at | Owner |
| --- | --- | --- | --- |
| `sema::crates::typed_headers::unpaid_provided::template_demands::tests::private_callable_templates_keep_failed_attempt_and_peak_costs` | `18232e371` | `2b0b5cf07` (late-region free originals in version thirteen) | Lane C |
| `abi::storage_relation::tests::actual_feature_origins_null_values_and_documentary_markers_grant_no_permission` | `902923015` | `f9a2c96d4` (impl candidates for a callable-trait subject) | Lane A |

- `template_demands` expects `fn declared<'a>(x:&'a u8)->&'a u8{x}` to stop at
  its earlier catalog boundary. Since `2b0b5cf07`, version thirteen transports
  late-region free originals, so the function passes that boundary.
- In the storage-relation control, the opponent without
  `impl<T:PointeeSized> Copy for *const T {}` now fails with "canonical trait
  selection work exhausted" instead of a Copy error (nonnull_types.rs:234).
  It also fails at `fa75f65b8`, `0311a54ea`, `c6b5642dc` and `a06677563`, and
  passes at `65a9eae6c` and `933e401c1`.

### Checks on main `71f4bc1e9`

With these checkpoints rebased onto `71f4bc1e9`, 33 of the 35 failing tests
pass. The remaining two are the routed `template_demands` and storage-relation
controls.

- The 28 private CI failures pass in fastcheck `1794123702416754649`.
- The five integration failures (`shape_constructors` ×2, `nominal_instances`,
  `ctfix`, `selected_references`) and the new linear control pass in
  `1794124021336383601`.
- The resumable-walk invariant, batch 5's invariants, the metadata, prototype,
  stability-transport and derive-list controls, and this branch's shim, loader
  flag and impl-header invariants pass in `1794123969582307044`.
- The nine `census_mode` tests and three `transported_rows` tests pass in
  `1794124262606592422`.
- PA1 to PA12 pass in `1794124335756971467`, and PA13 to PA19 with PA21 to
  PA24 in `1794124397628595354`. PA20 passes alone, 1451/1451, in
  `1794124477320290836`.

The only record whose size changes is the boxed per-body `Publication`. Its
one-word wait becomes `FieldWaits`: the last wait, the roots, the next index
and the resumable walk. No hot record changes, and no wire byte changes.
