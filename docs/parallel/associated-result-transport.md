# Declared associated-result transport: trait results, impl definitions and Output roles

Lane C checkpoint on `implement/associated-results-20261107` (base `bba7e42fb`,
code commits `6317621d9` and `4521eb176`). This is a transport slice, not a library milestone.
No original-core artifact, LIB-193 repair or whole-assignment claim follows.

## Design

**Rows.** Version 13 (`module-bodies-v13`) folds version 11's associated-type
and projection-recipe tables into its own header columns under tag 10. Two
row kinds share declaration kind 9:

- a trait's declared result keeps version 11's `AssociatedTypeHeader`;
- an impl's definition is a new `AssociatedDefinition { declaration, value }`
  row (`HeaderRef::AssociatedDefinition`). `value` is a type recipe in the
  impl's own frame. It is a definition, never a bound proof.

The impl's member links list the definition after its method links, keyed by
the trait member. The three columns (results, definitions, projections)
precede the optional const-body trailer. They are present exactly when the
declaration table names a kind-9 declaration. Presence is derived from rows
already decoded, never from trailing bytes, so every packet without associated
results keeps its exact bytes. Earlier profiles still reject associated
results with `named, associated or generic alias transport requires its wider
profile`.

**Validation** (`generics/results_contract.rs`, the membership ledger, the
domain index and the port planner):

- A result has no own inputs, formals or region captures. Its only promise is
  implicit sizing. Its canonical projection must be `<Self as Trait<P..>>::M`
  over exactly the trait's own type formals. Trait formals are allowed here;
  version 11 allowed only `Self`.
- Each definition is linked from exactly one impl of its own `Self` owner.
  Definition rows follow implementation and link order, so the table has one
  encoding. A definition has no default.
- A definition value is a root of its impl's frame. It cannot name the trait
  frame it satisfies.
- The port planner interns `Node::Projection { member, arguments }`: the
  member and a chain of `Self` then the trait arguments. The member names its
  trait, so the node needs no other field and `Node` keeps its size. Planning
  compares projections structurally and never selects a definition.
- A normalization proof whose quantified part binds no region and records no
  gap or relation (`Obligations::quantifies_nothing`) is exported as its flat
  evidence. Zero-slot callable openings leave such scopes when a rigid
  projection result is normalized. Any other quantified proof is still
  rejected by name.

**Consumer.** A fresh consumer trusts no provider proof:

- Each definition gets its own scope (`Inputs::Definition`, owned by its impl)
  and a cell whose type is its mapped value.
- `ImportedOwner.results` binds `(trait member, definition)` from the
  authenticated member links into the inventory's `type_members`, so
  projection selection finds the impl's own definition. It installs only on
  imported impls, with no duplicates or foreign members.
- `PrimitiveMethods` splits result links from method links, and kinds are
  checked against the header (`TypeAlias` and `Function`).
- `check_imported_results` re-proves every definition in the rebuilt impl
  environment: its binding, no families, a region-free value, formation, and
  each direct bound naming no trait argument (`Sized`, `Copy`, `Clone`, or a
  closed trait application such as the no-core `Sized` role). Regional
  evidence is rejected. Only then are `conformance::Definitions` built, so the
  impl's methods are checked against the definitions. The registry completes
  with the definitions.
- Projections normalize through the ordinary selector in every consumer.

**Output roles.** `operator_contract::imported` now admits by-value operators
with their own `Output` (`imported_value`). The checks are:

- the header profile and the `Rhs = Self` default;
- exactly the `Output` type and the operation method, both owned by the trait;
- a required, nongeneric, safe early-bound Rust `(Self[, Rhs])` signature
  whose result is `<Self as Op[<Rhs>]>::Output` (`output_projection`, shared
  with the source contract);
- the exact function-item identity.

Indexing roles stay rejected by name (`imported indexing operator roles
require their wider result profile`).

**A consumer's own operator use** (`4521eb176`). When the consumer has no
language declaration of its own for an operator role, `operator_contract::checked`
asks `Catalog::validated_role`. That answers only for a role whose imported
declaration completed validation, and the identity is never a spelling or path.
It then re-runs `imported_operator`, the same contract that admission runs:

- the indexing rejection;
- `require_imported_role`;
- the current imported header;
- `imported_assignment` or `imported_value`.

That contract now returns the `SourceOperator` (definition, method, signature,
`Rhs`, `Output`) that the source path produces. `pending_default` returns no
pending default for a validated imported role, because its defaults arrived
complete with its header. Without an imported or own role, the expression is
still rejected with `source binary operator requires its actual language
declaration`.

## Evidence

Receipts are under `obj/parallel/associated-results/`. Nothing was
hand-edited or blessed.

The fixtures are in `pa29/tests/spec/associated-results/`. Their outcomes are
in `pa29/tests/associated-result-outcomes.json`, declared before the oracle
and candidate runs (`rustc/declared.txt`).

**Independent Rust 1.98.0.** Run with `-Cpanic=abort` and `--cfg rustc_oracle`
(`rustc/receipt.txt`):

- `fetch-hit` and `sum-in-range` exit 42.
- `sum-overflow` exits 134 (SIGABRT).
- `fetch-mismatch` fails with E0308 and `fetch-unimplemented` with E0277.

**Candidate.** `crate_linking::normalized_results` checks:

- The packet carries `[13, 0]` and round-trips. Its columns hold 2 results, 2
  definitions and 4 projections.
- All three positive clients reach LowIR in fresh consumers.
- `fetch-mismatch` reports `incompatible types`.
- `fetch-unimplemented` reports `associated type requires an unambiguous
  validated implementation`.
- The v12 and v10 exports reject by name.

**Forged rows.** Each forgery is rejected:

| Forgery | Diagnostic |
| --- | --- |
| Definition rows swapped | `source associated definition belongs to another actual Self owner` |
| Definition names the trait result | `source declaration has duplicate or conflicting typed headers` |
| Value is the trait-frame projection | `source consuming type root captures another parameter owner` |
| Value is any other in-range recipe | a structural frame rejection, or `incompatible types` from the consumer's own impl method conformance for the impl's `u8` Self type (exactly one) |
| Value index 0xfffffff0 | `associated result definition lost its actual implementation owner` |
| Columns dropped | `truncated metadata artifact` |
| Version relabelled 12 | `unknown source declaration kind` |

Every strict prefix fails to decode. The swap was first accepted (both values
are `i32`). That exposed a second encoding of the same table, and the
canonical row-order rule was added.

**Amended control.** The frozen language-roles boundary for `output.rs` with
`sum-in-range.rs` no longer holds: the provider exports and the consumer
returns 42, the rustc outcome. `language-role-outcomes.json` carries an
`amended` reason, and `spelled_roles` now publishes `output.rs` and executes
`sum-in-range` natively.

**Native.** The ignored native tests run fresh consumers, compiled by CPPGM
(not modified):

- `fetch-hit` and `sum-in-range` exit 42.
- `sum-overflow` dies by signal 6.
- The language-roles natives, including the amended `sum-in-range`, are
  unchanged.

**Byte equality.** Baseline `bba7e42fb` (`baseline/`, SHA-256 `54421e8b…`) and
the candidate (`candidate/`) are pinned. `baseline-compare/receipt.txt` shows
byte-identical exports for:

- source-body-ir provider and wire, and the language-roles provider (v13);
- source-branches provider, wait and spin (v12);
- spin and source-nominals (v10).

All 25 consumer LowIRs are identical.

**Scaling.** Providers with 6, 24 and 96 generated traits (each with one
result and one `u8` impl) have exactly width+1 results and definitions and
width+2 projections. Each trait adds the same wire delta, and fresh `fetch`
consumers grow boundedly. Costs:

- Validation charges per row and argument.
- The planner interns one node per projection use.
- The result-link map is reserved only when a packet has links.
- Re-proof charges per definition and bound.

**Gates on the final tree.**

| Gate | Result | Receipt |
| --- | --- | --- |
| `make fastcheck`, 5 exact linking tests (new and amended) | passed | `obj/qualification/fast/1794041406586050351` |
| `make fastcheck`, 16 related linking/recipe controls | passed | `obj/qualification/fast/1794041508645434836` |
| Native fastcheck, 2 tests | passed | `obj/qualification/fast/1794041456808191386` |
| Private fastcheck, 30 exact lib cases (codec, scopes, domains, numbering, addressed recipes) | passed | `obj/qualification/fast/1794041921094922135` |
| Byte-identity compare | identical | `obj/parallel/associated-results/baseline-compare/receipt.txt` |
| `make test-upstream` | passed: 127 fixtures, 126 references | `obj/parallel/associated-results/test-upstream-1.log` |

**Own operator use.** Expected outcomes for the generated controls were
declared first (`obj/parallel/own-operator/declared.txt`). Independent Rust
1.98.0 (`obj/parallel/own-operator/rustc/receipt.txt`) gives:

- frozen `own-operator` exits 42;
- generated `own-add` (own `+` against `output.rs`) exits 42;
- generated `own-missing` (own `+=` against `output.rs`, which has no
  `add_assign` role) fails with E0368.

`crate_linking::spelled_roles` now checks the following:

- `own-operator` lowers two checked adds and returns 42 natively.
- `own-add` lowers three checked adds: two of its own and one in the `sum`
  instance.
- `own-missing` is rejected by name.
- The five forged role wires reject the `own-operator` client with the same
  diagnostics as imports.
- Own clients with 6, 24 and 96 `+=` statements lower exactly two checked adds
  per statement and grow linearly.

The frozen `own-operator` boundary in `language-role-outcomes.json` carries an
`amended` reason. Byte identity was rechecked against `bba7e42fb` with the
pinned candidate `obj/parallel/own-operator/candidate/` (SHA-256 `47e1fc0d…`).
`obj/parallel/own-operator/baseline-compare/receipt.txt` shows the same 8
exports and 25 consumer LowIRs, all identical.

| Gate | Result | Receipt |
| --- | --- | --- |
| `make fastcheck`, 7 exact linking/operator tests | passed | `obj/qualification/fast/1794042401964648311` |
| `make fastcheck`, 3 amended `spelled_roles` tests with the own scaling control | passed | `obj/qualification/fast/1794042725763711592` |
| Native fastcheck, `operator_role_instances_execute_natively` | passed | `obj/qualification/fast/1794042479821275717` |
| Private fastcheck, 8 exact operator-contract lib cases | passed | `obj/qualification/fast/1794042522992073964` |
| `make test-upstream` | passed: 127 fixtures, 126 references | `obj/parallel/own-operator/test-upstream-1.log` |

## Gap list toward `Option::map`

1. **`FnOnce`/`FnMut`/`Fn`.** These need `fn_once_output` as a declared
   result. That is now possible, but the tuple type recipe, rust-call tuple
   arguments and consumer admission of the three callable roles are missing.
   An exploratory no-core `FnOnce` provider stops before transport. With
   `tuple_trait`, it fails with `unsupported active source-core feature`. With
   only `unboxed_closures`, its paren-sugar header fails with `source trait
   header is outside the pinned provider profile`. Closures stay fenced by
   name.
2. **Enum headers and match.** This covers variants, discriminants, layout,
   IsVariant, Select, BindView, FalseEdge and guard tables. These codes are
   allocated and rejected by name.
3. **Closures and captures** at the call site, with CaptureBoundary
   projections.
4. **Wider declared results:** own generics (GATs), indirect and argumented
   bounds, regional definitions, results of foreign traits re-exported by a
   consumer (projections must be local), and quantified normalization
   evidence.
5. **Not on the `Option::map` path:** Div, Rem, Shl and Shr rows; non-`Self`
   defaults; the version-12 leftovers (drops, impl methods, references,
   constants).

## Pending

These checks were not run here:

- Full release and native suites, all-target strict Clippy and the harness.
  Scheduled CI owns these. No integration root or suite was added (the new
  module is inside `crate_linking`), so `make test-harness` was not needed.
- `make test-backend` was not needed, because the bridge and the backend
  contract are unchanged.
