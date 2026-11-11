# Actual source Unsize for nominal storage tails

Original core's `as_request` reaches a nominal Unsize goal after the ordinary
PA22/23 reference-cast correction. Compiler `c31afe8d7` now proves that goal
through its actual stored final field. PA31 owns the source protocol reductions;
ordinary slice/object casts retain their earlier owners.

## Typed proof and inference

The existing completed unsizing rules identify parameters confined to the final
field of the same struct owner. They supply eligibility, not conversion proof.
Every unchanged type, const and lifetime argument goes through the existing bulk
input equality operation. A prefix lifetime needs equality; one-way shortening
is insufficient. Eligible marker arguments need not themselves unsize.

The actual final-field template is instantiated in each endpoint's complete
argument environment. Its source Unsize goal becomes an ordinary child, followed
by both existing PointeeSized endpoint goals and their retained formation and
lifetime debts. Object erasure still pays Sized, principal, equality, auto and
outlives requirements; known dynamic upcasts retain intrinsic-principal policy.
Deferred heads cannot publish completed absence. Physical layout grants no proof.

The first implementation resolved whole nominal applications before selecting
the field. It passed44 outcomes but rejected two independently accepted element-
inference goals. The corrected implementation reuses the body's existing typed
`Fields` environment, preserving unresolved type/const terms while applying its
actual lifetimes. The argument-environment builder is shared with body field
access; its owner check, cached environment and projection continuation remain.
The selected template's region fold charges its actual work/storage to the
existing source payer, and inference charges the rule traversal and field work.

No dense record fields, global scans, new cache, compiler limits, LowIR vocabulary
or backend contract change. Declaration failures retain their original diagnostic
separately from inference mismatch, normalization, unresolved input and budget
outcomes. The existing private budget guard still requires the same exhaustion;
a formation guard additionally checks nominal identity gives Unsize absence.

## Owning reductions

[The packet](../pa31/tests/spec/nominal-unsize/README.md) declares46 inputs:
33 accept,13 reject. All46 object outcomes are independently checked with pinned
Rust1.98. All31 executable positives return `9 23` in Rust and through CPPGM.
Two positives check original-shaped source bodies and explicit lifetime equality
without claiming native execution. The unchanged earlier
`unsizing-roles/nominal-unsize-required.rs` also reaches complete LowIR acceptance.
The manifest and `independent-current.json` bind every current source hash.

Native observations cover real prefix/tail data, metadata length, mutation,
reborrows, aliases, generic/nested/projected fields, marker and const inputs,
borrowed data, exact single evaluation, empty/ZST/non-Copy tails, C padding and
shared/mutable virtual dispatch. Explicit source and target element-inference
cases retain the initial candidate's failure before the correction. Negatives
preserve element/prefix/owner identity, no-op and reverse restrictions, nested
reference limits, object proof, lifetime formation, permissions and active loans.

Independent drafts caught missing no_core operator prerequisites, mutable
DispatchFromDyn and shared-reference Clone/Copy implementations. Active cases use
actual slice-pattern reads/writes and a bounded state transition for evaluation
count, with real source prerequisites. Rust delayed ICEs are retained as failed
infrastructure evidence; none is a language-rejection expectation. Original
versions, source-faithful bytes and the corrected C representation attribute are
retained under B's `obj/parallel/nominal-unsize/`.

## Focused qualification

Evidence is under B's `obj/parallel/nominal-unsize/`:

- `public1.log`, receipt1794354554515240647,188.67s: initial44 complete outcomes,
  the earlier required positive, width/noise bounds and four older control groups
  pass. Its immutable5e119e53 candidate rejects the two later independent inference
  cases; `inference-before/` preserves both diagnostics.
- `public2.log`, receipt1794355091892551059,196.49s: eight exact tests across
  frontend and memory pass after preserving field inference. All46 complete LowIR
  outcomes, the older required positive, eight-counter width8/32/128 with0/128
  unrelated-type invariance, source array/object/upcast/erasure controls,41 ordinary
  nominal casts and28 old array-reference cast outcomes pass.
- `native.log`, receipt1794355334645613177,65.37s: three exact groups pass:31 new
  real conversions, the source upcast observer and original nominal wrapper
  transfer/cleanup observer. Strict selected production/CLI/test Clippy, full
  formatting and architecture checks pass.
- `controls.log`, receipt1794355520793219579,41.37s: existing nominal coercion
  declaration/value opponents and all three separately required tuple-factory
  cleanup-owner boundaries pass. Those valid unsupported factories remain work.
- `harness.log`, receipt1794355460066739241:315 harness checks,468 roots/13
  binaries,44 assignment layouts,128 pinned upstream fixtures and126 Reference
  files pass. No oracles or original upstream files changed. Whole PA31 remains
  planned. The known cold private-harness link exceeds the local five-minute
  budget; private/full successor qualification remains pending in CI.

Final immutable production CLI SHA-256:
`00b2af11b5d3e0396043305fbf4164ed5ea9bdf89ba7a3d16561f9fb367a486e`.
It binds committed compiler `c31afe8d78f3e6487b3c4524574dd40a6c341f4e`, an empty
source patch and3892 Rust source hashes. The initial5e executable was retired
following replacement (scoped prune1794355873725556381). Tidy1794355873834134744
removed82 disposable outputs; source, diagnostics and observations remain.

Forty exclusive A/A+twoABBA observations preserve complete ordinary import and
array-cast LowIR, a source known-representation output and74 existing source
outcomes. Candidate/baseline ratios are0.979/1.084/0.959/0.997; A/A ratios are
0.637/1.348/1.159/1.046. These noisy observations make no speed or neutrality
claim. Frozen sources, raw timings and full output hashes remain in `measure/`,
cleanup1794355792172199438; the source baseline is the preceding2cab6320 CLI.

## Original core and next requirement

The same120s original observation still FAILS its deadline at121.1401s. It retains
220 failure records and3743 item attempts through function7809,file226 byte27629,
node2318 in slice/index.rs. All722 original hashes, six source-code bindings,
profile and work/storage limits remain fixed. Partial progress SHA-256:
`6feb6dc8314b77a07327749901e41fb58afffa72d08c9dcc6e703177279e3341`.
No nominal-tail fence or as_request failure remains in that prefix. Prefixes
have different endpoints; their counts are not speed or body-completion evidence.

The first failure is now `IndexRange::fold`, ops/index_range.rs file93 byte4858:
field access requires a known tuple or struct type at
`self.try_fold(init, NeverShortCircuit::wrap_mut_2(f)).0`. Its reverse counterpart
has the same error. The actual adapter returns `impl [const] FnMut(...) -> Self`
and uses a const move closure. Its own wrap_mut_1/2 bodies separately report
invalid canonical trait selection. Reduce ordinary wrapped-call result/field
inference first, then retain actual source callable/const/Try prerequisites where
needed; select the earliest owner from that evidence.

Full42918510671 on216364b72 passed every stage for the preceding projection-input
checkpoint. Full42925327481 on the PA22/23 checkpointcba471220 also passed every stage.
New full42930363606 runs the published nominal-source checkpoint4a9409a58. It
was dispatched after both predecessors completed; do not cancel it for later
documentation-only updates. No complete census or original core/alloc/std artifact exists; earlier
optimized nested-proof checks and the three tuple factories remain required.


The following input-only probes independently qualify21 ordinary Rust objects.
All reproduce or control the same behavior on frozen2cab and00b: generic
FnOnce/FnMut/Fn result forwarding can leave a tuple head unknown before `.0`,
including without opaque or nominal types. Custom trait output forwarding,
inherited output, function pointers and explicit result/type arguments pass.
PA24 therefore owns the next ordinary reduction. Direct opaque invocation has a
separate known-function error; actual source const/callable/Try counterparts
remain required. Plans and exact outputs are under `obj/parallel/callback-result/`;
no PA24 compiler change or passing new contract is claimed yet.
