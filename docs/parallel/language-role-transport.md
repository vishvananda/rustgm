# Imported language-role transport: operator roles, Self defaults and checked rows

Lane C checkpoint on `implement/language-roles-20261107` (base `97b61c102`,
code commit `c0061f02f`). This is a transport slice, not a library milestone.
No original-core artifact, LIB-193 repair or whole-assignment claim follows.

## Design

**Role wire.** Version 13 (`module-bodies-v13`) keeps role rows as `(role,
declaration address)`. Roles with a compact ID (0-8, and 9 for `From`) keep
their byte. Any other role is written as escape 255 followed by its language
item spelling, at most 64 bytes. The decoder accepts the escape only under
`body_nodes()`. It rejects:

- an unknown spelling: `unknown source language role spelling`;
- a spelling that has a compact ID: `source language role spelling replaces
  its compact role ID`.

The spelling names the role only. The consumer binds that role to the row's
declaration address, never to a name or path. Earlier profiles keep their
encoders and bytes.

**Provider fence.** Under version 13, export no longer fences source-declared
compiler-language contracts. Two exceptions remain:

- The Future family (`future_trait`, `future_output`, `poll`,
  `async_drop_in_place`) and `Send` still need version 11.
- Each operator role re-runs the source operator contract before export
  (`operator_contract::exported`), and that contract's work is charged.

Other roles go through the existing `header_role`. Validation under
`BindingProfile::Languages` admits the same set: the five excluded roles are
rejected and every other role is allowed.

**Consumer binding.** A fresh consumer admits an imported operator role only
after it re-authenticates the declaration with the same checks that
authenticate a source operator. Today only the ten compound-assignment roles
are admitted. The checks are:

- the trait header from completed declaration validation, with exactly one
  written parameter and no supertraits;
- an `Rhs` parameter whose default is the trait's `Self`;
- exactly one member: the method named after the operator, with no generics
  of its own;
- a safe Rust ABI `(&mut Self, Rhs)` signature with a unit result;
- the exact function-item identity.

Fixtures exercise `add_assign`, `sub_assign` and `mul_assign`. The division and
shift assignments are admitted, but their bodies still stop at the rejected
flags 12-15. Roles with an `Output` result are rejected by name until associated
results transport. Every other imported role still reports `imported source role
requires its wider explicit header profile`.

A consumer's own operator use (`own-operator.rs`) is still rejected with
`source binary operator requires its actual language declaration`. Source
operator resolution still reads the local catalog. Design for the follow-on:
resolve the operator through `Catalog::validated_role`, the imported role's
declared identity. Neither its spelling nor a path may be used for this.

**Parameter defaults.** Under version 13, slot tag 2 is a sized type slot
followed by a default type ID. A region slot cannot carry a default. A default
is admitted only on a trait-owned slot, and it must be the trait's own `Self`
type. The producer checks this against its typed default. Fresh validation
checks it against `rows.traits[owner].self_type`, and
`formal_headers::parameters` restores `Argument::Type`. Defaults of any other
form are rejected by name.

**Checked Add, Sub and Mul.** Binary flags 9-11 now map to rows. Typing
accepts integer and float operands only, and the result has the operand type.
Flags 12-15 (Div, Rem, Shl, Shr) are allocated and rejected by name. No
overflow decision is serialized. A fresh consumer rebuilds `P::Plus`, `Minus`
or `Star`, re-proves each instance, and lowering emits the existing overflow
guard to the shared abort exit. Without a `panic_const_*` item, that guard
takes the course-abort path.

**Request ports.** `+=` on a primitive still emits Read, Binary and Store. Its
typing passes through method selection, and that leaves a
`WellFormed(FunctionItem(add_assign))` obligation. Three changes make that
obligation reconstructable:

- The domain index treats a direct function-item formation as an owning
  request for its application.
- Reconstruction demands every WellFormed root, charged per obligation.
- `applied_records` folds a `FunctionItem` with `apply_argument_charged`, so
  the callable's late binder is kept. The old `signature_charged(.., 0)`
  call dropped it and produced `missing lifetime argument`.

Ingestion also demands every slot default root.

## Evidence

Receipts are under `obj/parallel/language-roles/`. Nothing was hand-edited or
blessed.

The fixtures are in `pa29/tests/spec/language-roles/`. Their outcomes are in
`pa29/tests/language-role-outcomes.json`, declared before the oracle and
candidate runs.

**Independent Rust 1.98.0.** Run with `-Cpanic=abort` and `--cfg rustc_oracle`
(`rustc/receipt.txt`). Both providers compile. Results:

- The in-range clients for bump, lower, scale, own-operator and sum exit 42.
- The bump, lower and scale overflow clients exit 134 (SIGABRT).
- The mismatch client fails with E0308.

**Candidate.** Three tests in `crate_linking::spelled_roles` check the declared
outcomes:

- The packet carries `[13, 0]`, round-trips and has three spelled role rows.
- It has 2, 3 and 2 Add, Sub and Mul nodes.
- Six positive clients reach LowIR in fresh consumers.
- The mismatch client reports `incompatible types`.
- The v12 and v10 exports reject with `source compiler language contract
  transport is not implemented`.

Two boundaries are recorded with their diagnostics:

- The own-operator client is rejected; rustc accepts it.
- The `Output` provider is rejected with `named, associated or generic alias
  transport requires its wider profile`. The declared guess was the
  associated-result message; the same gap rejects one stage earlier.

**Forged rows.** Each forgery is rejected:

| Forgery | Diagnostic |
| --- | --- |
| Unknown spelling `add_assigx` | `unknown source language role spelling` |
| Duplicate `sub_assign` | `source role binding is duplicated or belongs to a foreign provider` |
| Wrong owner kind `panic_impl` | `imported declaration changed its exact identity or kind` |
| Missing declaration index | `source header query has an unknown declaration` |
| Flag 9 relabelled 12 | the division/shift message |

A version relabelled to 12, and every strict prefix, fail to decode. The two
"recorded at first observation" diagnostics in the table are first
observations, and the outcomes file notes them.

**Amended control.** The frozen source-body-ir flag-9 control
(`Add` unreconstructable) is no longer true. It moved to flag 12, and a new
flag-9 result-type control was added (`changed its primitive result type`).
`pa29/tests/source-body-ir-outcomes.json` marks the change as `amended`.

**Native.** The ignored native test runs fresh consumers, compiled by CPPGM
(not modified). The in-range clients exit 42 and the overflow clients die by
signal 6, which agrees with rustc.

**Byte equality.** Baseline `97b61c102` and candidate `c0061f02f` are pinned
under `baseline/` and `candidate/` with their SHA-256s.
`baseline-compare/receipt.txt` shows byte-identical exports for:

- source-body-ir provider and wire (v13);
- source-branches provider, wait and spin (v12);
- spin and source-nominals (v10).

All 21 consumer LowIRs are identical. Only the language-roles provider differs:
the baseline rejects it.

**Scaling.** Widths of 6, 24 and 96 `total += step` statements give a constant
wire delta per statement, width + 1 Add nodes, width `binary add i32` guards,
and LowIR growth bounded by a fivefold ratio. The costs:

- Each spelling is bounded to 64 bytes of the packet itself.
- Validation charges per row and operand.
- Ingestion demands each default root through the lazy cursor.
- Reconstruction charges per obligation.

No cap is borrowed.

**Gates on the final tree.**

| Gate | Result | Receipt |
| --- | --- | --- |
| `make fastcheck`, 11 exact linking tests | passed | `obj/qualification/fast/1794035733598288133` |
| Native fastcheck, 3 tests (roles, nodes, branches) | passed | `obj/qualification/fast/1794035845633669854` |
| `make test-upstream` | passed: 127 fixtures, 126 references | — |

## Gap list toward `Option::map`

1. **`FnOnce`/`FnMut`/`Fn`.** This did not fit in this checkpoint. It needs
   `fn_once_output` as an associated result, rust-call tuple arguments, and
   consumer admission of the three callable roles.
2. **Associated results: the version-11 fold.** Version 13 should absorb
   version 11's associated-type and projection-recipe tables as header columns
   under tag 10. `Add`'s `Output` and item 1 both need this.
3. **Enum headers and match.** This covers variants, discriminants, layout,
   IsVariant, Select, BindView, FalseEdge and guard tables. These codes are
   allocated and rejected by name.
4. **Closures and captures** at the call site, with CaptureBoundary
   projections.
5. **A consumer's own use of imported operators**, through validated-role
   identity.
6. **Not on the `Option::map` path:**
   - Div, Rem, Shl and Shr rows, with their divisor and shift guards;
   - non-`Self` defaults;
   - the version-12 leftovers: drops, impl methods, references and constants.

## Pending

These checks were not run here:

- The private `scopes::tests` invariants, after the `ParameterSlot.default`
  constructor change. One cold attempt
  (`obj/qualification/fast/1794036184417610409`) ran out of the 300 s budget in
  `clippy-private` (status 124) without reporting a diagnostic. Later attempts
  waited for a heavy-job slot. The selection is in
  `obj/parallel/language-roles/private-selection.txt`.
- Full release and native suites, all-target strict Clippy and the harness.
  Scheduled CI owns these.
- `make test-backend` was not needed, because the bridge and the backend
  contract are unchanged.
