# Override bounds, region-binding sugar and impl header projections

Lane B batch of 2026-11-07, on top of `0311a54ea`. It covers the three
remaining "associated type requires an unambiguous implementation" sites
of the [census plan's follow-up decisions](../core-census-plan.md):

- override conformance of a repeated `Self::Item` bound (`iter/mod.rs:376`);
- region-binding callable sugar (`iter/adapters/scan.rs:60`,
  `iter/adapters/map_windows.rs:245`);
- `ops/control_flow.rs:427`.

All four sites clear. The other three items assigned to Lane B (`--extern
core=`, variant re-exports and `compiler_builtins` injection) are diagnosed in
section 5. Each needs an integrator decision before implementation.

## 1. Override bounds in the impl's normal forms

Rustc substitutes the impl into the trait method's predicates and normalizes
them in the impl's param-env before it compares the override's predicates
with them. RustGM normalized the override's bounds but not the trait's. Core's
`impl_fold_via_try_fold!` overrides `fold` in `Scan` with
`FFF: FnMut(AAA, Self::Item) -> AAA`, repeating the trait's own bound. That
override was reported as a "stronger generic requirement". The census could
not see this cause, because conformance runs after the declaration loop.

- `conformance/clauses.rs` (new) normalizes the trait method's own bounds
  (`Contract::method_bounds` onward) and the override's obligations.
  Normalization goes through the impl's checked associated definitions
  (`Definitions::resolve`) and then ordinary selection, the same
  `projections::Session` the method signatures use.
- A quantified clause is normalized inside its own binder
  (`Obligations::scope`), and selection debts stay in the contract's region
  proof.
- An associated equality keeps its projected side as written. Only that
  side's subject, trait arguments and own arguments are normalized, with its
  result, so it still states that projection's result.
- When an assumption changed, the environment is re-formed from the
  normalized assumptions (`mapping::environment`, now `pub(super)`).
- `mapping::assumptions` was split out of `contract`, which stays under
  Clippy's line limit.

Controls: `pa31/tests/spec/override-projection-bounds/`, eight cases checked
by `dev/tests/invariants/override_projection_bounds.rs`, all matching rustc.

- Positives: the census `Scan` fold shape, a supertrait's `Self::Item` (the
  shape of Lane A's `Flatten` reductions), the impl's normal form written
  out, a nested projection and callable sugar.
- Opponents, each E0276: a wrong type in the repeated bound (two cases) and
  an added `Copy`.

Lane A's reduction `census-batch5/repro/wrap.rs` now passes. The four
`flatten-*.rs` reductions still stop earlier, on "trait argument does not
satisfy its declaration requirements", on the base and on this batch alike.

## 2. Impl parameters determined under a region binder

`F: FnMut(&mut St, I::Item) -> Option<B>` has an elided input lifetime, so
the sugar quantifies the condition over one region. RustGM proved quantified
conditions only after the parent resolved, so `B`, which only the `Output`
equality determines, stayed unresolved. Rustc instantiates the bound region
with a placeholder and equates the normalized output with `Option<B>`.

- `selection/instantiation.rs`: `Determined` adds, for each quantified
  associated equality whose result names a type or const parameter the impl
  header leaves undetermined, one instance at fresh regions (one per bound
  region) to the trial's ordinary goals.
- The universal condition is still proven in its own scope after the parent
  resolves. The instance is implied by it and only lets the trial infer the
  parameter.
- The fresh regions follow the captures. `Instance::existentials` counts them.
  `Instance::allocated` (captures plus existentials) replaces the capture
  count in the trial's region interval (`frames.rs`, `result_templates.rs`).
  The allocator record already covered `trial.end`.
- A parameter inferred to name one of those regions cannot leave the trial,
  as rustc's placeholder cannot leave its universe. That selection fails with
  "selected type retains an unmapped lifetime".
- The declared conditions moved to `instantiation::conditions`, which keeps
  `instantiate` under the line limit. Work is charged as before, plus
  `Determined::steps`.

Controls: `pa31/tests/spec/region-sugar-impl-parameters/`, eight cases checked
by `dev/tests/invariants/elided_sugar_parameters.rs`.

- Positives: core's `Scan` (with its `fold` override) and `MapWindows`
  shapes, a caller premise that normalizes `<Scan<u8, F> as Iterator>::Item`
  to `u16` in a signature, and an output that names a caller lifetime.
- Opponents: a mismatched result (E0308) and a premise whose output names its
  own bound region (rustc: E0308, "one type is more general than the other").
- Two recorded gaps: a generic body that selects such an impl (`s.next()`)
  stops with "invalid scoped proof evidence" when it attaches the selection's
  scoped proof. `body-fixed-output`, whose condition determines nothing,
  stops the same way on the base, so this is a separate body-selection cause.

The PA31 `fn-sugar-impl-parameters` gaps `elided-lifetime` and `map-adapter`
now pass and moved to that invariant's checked cases. Both were declared with
rustc outcome 0, so no declaration changed. `impl-lifetime-mismatch` stays a
gap.

## 3. Impl headers that name a projection of their parameter

`impl<R: Try> ControlFlow<R, R::Output>` failed even in ordinary mode, because
no environment existed while the header formed. `<R as Try>::Output` could
not prove `R: Try`.

- `demand_projection_environment` now demands the impl's own environment
  (`Work::ProjectionEnvironment` with `Position::Impl`) when the projection's
  nearest binder owner is an impl's lexical `Self`. `implementation` checks
  the owner's item kind, so a trait's lexical `Self` is unaffected.
- After publication the same work item finds the environment complete.

Controls: `pa31/tests/spec/impl-header-projections/`, ten cases checked by
`dev/tests/invariants/normalized_impl_headers.rs`.

- Positives: the census shape, the same header beside core's `Try`,
  `FromResidual` and `Residual` with `ControlFlow`'s impls, inline, qualified
  and where-clause bounds, a trait impl, and a method returning `Self`.
- Opponents: no `R: Try` bound (E0277) and `R` only inside a projection
  (E0207, "impl type and const parameters must constrain its target").
- One recorded gap: `self-where` adds `where Self: Sized`. That bound names
  the self type that the header projection's environment needs before the
  type is formed, so it stops with "nominal or associated types require
  later semantic assignments". The same bound beside a header without a
  projection passes, as it does on the base.

## 4. Controls, oracle and amendment

Every outcome in the three directories was declared before its frozen rustc
1.98.0 oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024
--crate-type=lib --emit=metadata`).

- The declared files and the oracle receipts are in `obj/lane-b/declarations`
  and `obj/lane-b/oracle-v1`/`oracle-v2`.
- Most reduced shapes had exploratory rustc runs in scratch first; each
  `outcomes.json` names the cases that had none.
- One amendment, recorded in `impl-header-projections/outcomes.json` before
  oracle v2: three new fixtures repeated the prefix's own `pub enum Option`
  line, so rustc rejected the fixtures themselves (E0428). v2 removes the
  repeated line. Case bodies and declared outcomes are unchanged, and no
  RustGM run preceded the amendment.
- All 26 cases then agree with their declarations.

The prefix of each fixture is the exact pinned core fragment of
`spec/fn-sugar-impl-parameters/map-adapter.rs`.

## 5. Diagnosed and not implemented

### `--extern core=`

`Dependency::new` rejects the alias `core`. Accepting it is one line. Then:

- the `no_std` injection (`standard_imports::provider_root`) has to resolve
  `core` through that extern-prelude entry instead of binding the source
  provider's root;
- the client gate (`resolve/bindings/client.rs`) admits a `no_std` client only
  over a provider named `core`, which a shim client does not have.

For the six std units, `core` is the `rustc_std_workspace_core` shim, whose
`pub use core::*` must carry core's public names. A source export's module
binding can target only its own artifact today
(`implementation_domains.rs`: "foreign module targets require the
mixed-provider namespace profile"). This needs one of two new version-13
forms, and the integrator should pick one:

- a glob row naming a foreign module, re-resolved by the consumer;
- per-name foreign bindings, which cost one row per core public name.

### Variant re-exports

A variant has a value-namespace constructor declaration with a `Variant` row.
Its type-namespace alias has no row: version 13 names each variant once, in
its enum's member scope. `pub use crate::option::Option::{self, None, Some}`
therefore fails twice:

- the type-namespace bindings of `None` and `Some` have no declaration
  (`authored_export::declaration`: "source header names an unreserved actual
  declaration");
- consumers accept module bindings only to modules, traits, structs, enums
  (type) and functions (value). Both `module_exports.rs` and
  `implementation_domains.rs` reject other targets.

Proposed shape, for the integrator to approve before use: no new tag. The
existing `ModuleBindingRow` may target a `Variant` declaration in the value
namespace, under a version-13 profile gate. The type-namespace variant alias
binding is not emitted, as the enum member scope already omits it. Older
profiles reject the target by name. Consumers re-derive the binding from the
validated `Variant` row, with per-row charging and a forged-namespace
opponent.

This does not reach a Rust type-namespace use of a re-exported variant, such
as `prelude::v1::Some { 0: x }`. Core does not use one.

### `compiler_builtins`

Rustc 1.98.0 was observed with an empty sysroot.

- A `#![no_std]` crate gets no `extern crate compiler_builtins` in its
  expansion (`-Zunpretty=expanded` shows only `extern crate core`).
- The crate loader injects it afterwards, unless the crate is `#![no_core]`
  or `#![compiler_builtins]`, or a `#![compiler_builtins]` crate is already in
  its graph.
- Missing, it fails with E0463 "can't find crate for `compiler_builtins`",
  even for `--emit=metadata`.

RustGM's metadata does not record `#![compiler_builtins]`, so a consumer
cannot apply the "already in the graph" rule or rustc's check that the
resolved crate is marked. That needs a version-13 descriptor flag.

Every PA29 `no-std-clients` control would also need a RustGM-built
`compiler_builtins` artifact. Its oracle used a rustc-only `builtins.rs`
stand-in. Both need integrator decisions.

## 6. Census

Each pin was built from a committed tree and run with census 4's manifest,
flags and pinned inputs, analysis-only.

| Pin | Base | Checked | Failed | Blocked |
| --- | --- | --- | --- | --- |
| Lane B census on base `0311a54ea` (pin `e9cc215f`) | main | 42,274 | 8 | 28 |
| Lane B census on base `16c57acb8` (pin `330ee2f8`) | this batch | 42,280 | 4 | 26 |

"associated type requires an unambiguous implementation" falls from 4/2 to
0/0. The four remaining failures:

- `pat.rs:85` (Lane A);
- three "canonical trait selection requires pending declarations" roots:
  `prepare` and two `Buf`s.

## 7. Checks

- Fastcheck with 18 exact private cases: the three new invariants, the
  fn-sugar, user-callable, determined-parameter and override-clause tests,
  the hot-record and derive-layout invariants. Passed; see
  `obj/qualification/fast/1794092764120901920`.
- `make test-upstream`: 127 pinned fixtures and 126 Reference files
  verified.
- `--pa pa1` to `pa12`, and `pa13` to `pa24` without `pa20`: every case
  passes (`obj/qualification/fast/1794093238195843574`,
  `1794093341557034320`). `pa20` alone: 1451/1451 (`1794093419034704707`).
- No hot record changed. `Instance` and `Contract` are transient.
