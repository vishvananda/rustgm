# Callable-role transport: `FnOnce`, `FnMut`, `Fn` and call operators

Lane C checkpoint in `rustgm-async-work` (base `621108866`). This is a
transport slice, not a library milestone. No original-core artifact, LIB-193
repair or whole-assignment claim follows.

## Design

**Provider roles.** Under version 13 (`module-bodies-v13`) a no-core provider
may declare:

- the `tuple_trait` marker;
- `FnOnce<Args: Tuple>` with its `fn_once_output` declared result and
  `extern "rust-call" fn call_once(self, args: Args) -> Self::Output`;
- `FnMut<Args: Tuple>: FnOnce<Args>` with `call_mut(&mut self, args)`;
- `Fn<Args: Tuple>: [const] FnMut<Args>` with `call(&self, args)`.

All are `const trait`s with `#[rustc_paren_sugar]`.
`generics/rust_call_roles.rs` checks each role's exact header: one written Args
parameter with its Tuple obligation, the written hierarchy, and exactly its
call member (plus `Output` for `FnOnce`). The conditional effect (tag 2) is
admitted only on these roles' own clauses. `fn_once_output` is a kind-9
result row owned by the bound `FnOnce`.

**Wire additions**, version 13 only:

- type recipe tag 9 is a non-unit tuple over `roots` entries (unit keeps its
  scalar row);
- requirement tag 4 is `ProjectionEqual { projection, value }`, the `-> U` of
  `F: FnOnce(T) -> U`;
- body IR node 29 is `Rvalue::Invoke [Flag(call trait), Type(signature),
  Range(arguments)]`, with flags 0/1/2 for `FnOnce`/`FnMut`/`Fn`;
- node 12 flag 1 is a two-phase borrow, which `apply_mut` and `apply_ref`
  use for their receivers. Flag 2 (unique) stays rejected by name.

Earlier profiles reject every one of these by name, and packets without them
keep their exact bytes.

**Parenthesized sugar.** `F: FnOnce(u8) -> u8` opens a clause binder that
binds no region. Version 13 lifts exactly that vacuous binder: the environment
assumption, the `ScopedWellFormed` input over a zero-count frame and the
rebuilt obligation. A binder that binds any region is still rejected. Under
version 13 a `ProjectionEqual` declared binding is a builtin premise, not a
re-proved debt. Each consumer normalizes against it.

**Consumer re-authentication.** A fresh consumer trusts no provider proof:

- `invocation::imported` admits the imported Tuple marker and each callable
  role only after re-reading its current header. That covers the profile
  (const, not unsafe or auto), the one Args type parameter, the Tuple premise
  on Args, the hierarchy premise with its effect, the member set, a required
  nongeneric call method, `method_contract` (receiver per kind, Args,
  `<Self as FnOnce<Args>>::Output`, rust-call) and the exact function-item
  identity (`operator_contract::imported_item`).
- `Catalog::validated_role` gives the neutral identity of a validated
  imported role. The consumer's own closure bindings, Tuple checks and
  `invocation::definition` use it, never a spelling or path.
- `untupled_calls::check` re-proves every transported `Invoke` before the
  body is accepted. The receiver must exist. An `FnMut`/`Fn` receiver must be
  a reference of the kind's mutability. The signature binds no region and its
  input count is `arguments - 1`. Its result must be the callee's declared
  `Output`, either as an environment `ProjectionEqual` premise on exactly
  `<C as FnOnce<Args>>::Output` or as that projection itself. Then
  `C: Kind<Args>` is proved in the rebuilt environment. Any remaining
  lifetime obligation is rejected by name.

**A consumer's own closures.** A source client's closures pay the inference
work budget, because a client has no local source retention ledger
(`closures/completion.rs`). A receiver adapter now accepts the source catalog's
own Copy carrier, provided it adds no predicate and no defining request
(`adapters/preconditions.rs`).

**Fences.** Closures inside provider bodies stay rejected by name (`nested
callable source arguments require a wider transport profile`), and so do
closure call entries (`source closure call entries require their closure
transport`).

## Evidence

Receipts are under `obj/parallel/callable-roles/`. Nothing was hand-edited or
blessed.

The fixtures are in `pa29/tests/spec/callable-roles/`. Their outcomes are in
`pa29/tests/callable-role-outcomes.json`, declared before any oracle or
candidate run (`rustc/declared.txt`). The provider reuses lines 2-57 of
`spec/source-nominals/provider.rs` and keeps their attribution.

**Independent Rust 1.98.0.** Run with `-Cpanic=abort` and `--cfg rustc_oracle`
(`rustc/receipt.txt`):

- all nine accepted clients exit 42 (`once-`, `mut-` and `ref-` crossed with
  `item`, `closure` and `capture`);
- `wrong-output` fails with E0271, `ref-mutating` with E0594 and
  `mut-consuming` with E0507.

Two declarations were amended after the first oracle run, and no candidate run
on frozen files preceded either. rustc needs a `drop_glue` lang item for the
owned `FnMut`/`Fn` argument, so the provider gained a rustc-only
`#[cfg(rustc_oracle)]` shim. `mut-consuming` was declared E0525 and rustc
reports E0507; the frozen outcome follows the oracle.

**Candidate.** `crate_linking::fn_roles` checks the following:

- The packet carries `[13, 0]` and round-trips, with one Invoke per call trait
  and the two-phase receiver.
- All nine clients reach LowIR in fresh consumers.
- The three opponents reject with `trait obligation requires an unambiguous
  matching assumption or implementation`.
- v12 and v10 exports reject by name, and the provider-closure control is
  fenced.

**Forged wires.** Each is rejected in a fresh consumer:

| Forgery | Diagnostic |
| --- | --- |
| `apply` Invoke FnOnce relabelled Fn or FnMut | `source call operator receiver changed its call trait borrow` |
| Invoke flag 3 | `body IR invoke node has an unknown call trait` |
| `apply_mut` Invoke FnMut relabelled FnOnce | `source call operator result is not its callee's declared Output` |
| Two-phase receiver relabelled unique | `encoded unique Rvalue::Borrow is not reconstructable` |
| Role spelling `fn_mut` relabelled `unsize` | `source const/auto/unsafe traits require a wider explicit profile` |
| Version relabelled 12 | `unknown source declaration kind` |

The two-phase receiver relabelled immediate is accepted, as it is sound. Every
strict prefix fails to decode. A first version accepted the FnMut-to-FnOnce
relabel, because nothing re-proved the call trait. That gap is the reason for
`untupled_calls`.

**Native.** The ignored native test runs fresh consumers through CPPGM (not
modified). All nine clients exit 42 with empty stdout. The
`normalized_results` and `spelled_roles` natives are unchanged.

**Byte equality.** Baseline main `621108866` (`baseline/`, SHA-256
`68e52c0d…`) and the candidate (`candidate/`, SHA-256 `0f2ea0ce…`) are pinned.
`identity/receipt.txt` compares 330 files: exports, LowIR, statuses and
diagnostics for every fresh consumer. It covers source-body-ir provider and
wire, language-roles and associated-results (v13), source-branches provider,
wait and spin (v12), and spin and source-nominals (v10). 0 files differ.

**Scaling.** Providers with 6, 24 and 96 `applyNNN<F: FnOnce(u8) -> u8>`
appliers grow exactly linearly in packet bytes. A fresh closure client's LowIR
stays bounded. The Invoke re-proof is canonical body work: one step per call
operator and per `Output` premise it consults, which are drawn from the
environment's `Output` bucket only. Bodies without call operators charge
nothing, so existing counters are unchanged.

**Gates on the final tree.**

| Gate | Result | Receipt |
| --- | --- | --- |
| `make fastcheck`, 9 exact linking tests (new and v13 neighbours) | passed | `obj/qualification/fast/1794047892609223474` |
| `make fastcheck`, 7 of those rerun after the charged re-proof | passed | `obj/qualification/fast/1794048654738104874` |
| `make fastcheck`, 14 related linking controls (calls, borrows, regions, versioned clients, roles, nodes, branches) | passed | `obj/qualification/fast/1794048393686952912` |
| Native fastcheck, 3 tests (final tree) | passed | `obj/qualification/fast/1794048748389168796` |
| Private fastcheck, 4 exact lib cases (required exports, invocation roles) | passed | `obj/qualification/fast/1794048102028586993` |
| Private fastcheck, 7 exact lib cases (wrapper premises with the new carrier control, receiver adapters, invocation roles) | passed | `obj/qualification/fast/1794048483522171441` |
| Byte-identity compare | identical | `obj/parallel/callable-roles/identity/receipt.txt` |
| `make test-upstream` | passed: 127 fixtures, 126 references | `obj/parallel/callable-roles/test-upstream-1.log` |

## Gap list toward `Option::map`

1. **Enum headers and match.** This covers variants, discriminants, IsVariant,
   Select, BindView, FalseEdge and guard tables. It is the next task.
2. **Closures inside provider bodies**, including closure call entries and
   capture boundaries. These are fenced by name. Iterator adapters need them.
3. **Consumer-written callable bounds on imported roles.** A consumer cannot
   write `G: FnOnce(u8) -> u8` against the imported family. Paren sugar
   needs its marker attribute on the imported header. The angle-bracket form
   stops at `trait argument does not satisfy its declaration requirements`.
4. **Region-binding callable sugar** (`for<'a> FnOnce(&'a T)`, a non-vacuous
   `ScopedWellFormed`).
5. **Invoke results that need non-premise normalization**, for example an
   `Output` reached only through an impl selection.
6. **`match` inside a consumer closure's capture analysis.**

## Pending

These checks were not run here:

- Full release and native suites, all-target strict Clippy and the harness.
  Scheduled CI owns these. No integration root or suite was added (`fn_roles`
  is inside `crate_linking`), so `make test-harness` was not needed.
- `make test-backend` was not needed, because the bridge and the backend
  contract are unchanged.
