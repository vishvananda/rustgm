# Census batch 1: premise causes and quantified impl constraints

The first original-core census ([tracker](../library-build-blockers.md)) found
two kinds of problem in Lane A's area. Its stop at `iter/adapters/flatten.rs:316`
hid the real cause and was counted as a budget stop. Five impl headers failed
because their parameters are determined only through `Fn(..) -> T` bounds. This
batch repairs both, then reruns the census on unchanged original core.

## Projection premise causes

`known_premises::normalized` errors used to be replaced by the fixed text
"invalid or exhausted canonical projection premise". The canonical-premise check
now runs in its own step. Its two outcomes are worded separately:

- `begin_with_scope` passes the inner error through unchanged.
- The cached `begin_reusing` path keeps a static message. It reports either
  "canonical projection premise work exhausted", which stays sticky, or
  "invalid canonical projection premise", which is an ordinary failure.

The census budget classifier stays lexical and conservative. It now also counts
"exceeds its input-graph bound" (LIB-193's ledger wording) as a budget.
`census_records` checks both sets of messages: the budget wording is sticky, and
the invalid-premise and missing-contract messages are not.

With the real error visible, the `flatten.rs:316` stop is
`missing associated result contract`. It comes from `opaque_normalized`, then
`results::has_equalities`, then `templates::for_result(Iterator::Item)`.
`prepare_associated_types` runs after trait impl headers, so
`facts.associated.type_contract` does not exist yet. rustc accepts the two
reductions in `obj/parallel/census-batch1/controls/flatten/`. This is a genuine
pass-order gap. Fixing it needs a pass-order change or a wait, so it is out of
scope here.

## Quantified constraining clauses

`impl_headers.rs` read in-scope predicates with `ordinary()`, which rejects every
quantified clause. `Fn(A) -> B` lowers to a quantified `FnOnce<(A,)>` clause even
when it binds no lifetimes. That clause carries an `Output = B` equality.
`Constraints::clause` now handles each kind of clause:

- A plain predicate adds a dependency at depth 0.
- A `ForAll` clause adds its body's dependency one binder deeper. The impl's
  lifetimes are read at depth 1, and the clause's own bound lifetimes are
  neither inputs nor outputs (`bound_uses::collect_beneath`). Type and const
  parameters keep their identities.
- Feature clauses determine nothing.

The impl's own trait-reference projection never determines its outputs. Beneath
a binder that comparison is conservative: a lifetime-bearing header matches any
lifetime-bearing subject of the same trait. It can only reject more. The result
matches rustc's `setup_constraining_predicates`, where an output is constrained
only once every input is constrained.

Outcomes were declared before any oracle, and all twelve agree with
`rustc 1.98.0 (88d9e12ae)`:

- **Accepted** (`pa24/tests/spec/constrained-call-outputs/`): `trait-output`,
  `inherent-output`, `chained-outputs` and `higher-ranked-output`.
- **E0207** (same directory): `input-only`, `unknown-input`, `inherent-input-only`
  and `impl-lifetime-input`. In `impl-lifetime-input`, `F: FnOnce(&'x u8) -> U`
  leaves `U` unconstrained because `'x` is not constrained.
- **PA21 binders** (`pa21/tests/spec/binders/constrained-results/`):
  `bound-result` (accepted) and `bound-unknown-subject` (E0207).

The private invariant
`sema::declarations::impl_headers::tests::quantified_outputs_constrain_impl_parameters_only_through_known_inputs`
runs all ten (eight PA24 and two PA21). Before the repair, every one failed
with "quantified clause requires a scoped consumer".

The PA24 positives are declaration-only. Selecting such an impl at a use site
still fails with "associated type requires an unambiguous validated
implementation". The census will reach that gap once it gets to bodies. The
first use-site sources and their oracle are kept under
`controls/superseded-use-site/`.

## Census 2

Analysis-only census, not a qualification. CLI `e6f4a7ec...`, 22.0 s, 375 opens,
inputs unchanged. Manifest conventions as before; only `dev/bin/crates.rs` is
rebound, unchanged since census 1. Limits 14,500,624 / 1 GiB / 64 MiB.

| Phase | Checked | Failed | Blocked |
| --- | ---: | ---: | ---: |
| frontend | 0 | 0 | 0 |
| declarations | 10,408 | 6 | 31 |
| bodies | 0 | 0 | 0 |
| completion | 0 | 0 | 0 |

Census 1 checked 1,197 declarations: it had 7 failures and a budget stop. All
five `Fn`-output failures from census 1 are gone, and so are the 22 more that
the misclassified stop had hidden. The remaining records:

| Records | Message | Site | Cause |
| ---: | --- | --- | --- |
| 1 + 31 blocked | source attribute header changed its semantic function | `ops/arith.rs` (`Sub::sub`, line 203); blocks `array/mod.rs:503` (30) and the `[T; 0]` arm (1) | pass order |
| 2 | object requires exactly one principal trait | `any.rs:162`, `:169` | Lane B |
| 1 | missing associated result contract | `iter/adapters/flatten.rs:316` | pass order |
| 1 | shorthand Self associated type requires a prepared trait owner | `pin.rs:1745` | header order |
| 1 | non-pointer non-null patterns require their wider profile | `pat.rs:85` | profile gap |

An instrumented analysis-only diagnostic (CLI `db2b2b49...`, records identical)
traced the arith record. `array_impl_default!` self types such as
`[T; (32 - 1)]` evaluate their length constants while trait impl headers are
being prepared. The binary-operator contract then queries `Sub::sub`'s attribute
header. Function declaration headers are prepared only in the later definition
loop, so `declaration_header` is `None`, and `validate_header` reports that
absence as a change.

`pin.rs:1745` uses `Self::Target` inside the where-clause of
`const impl<Ptr> DerefMut for Pin<Ptr>`. The implicit projection reads the impl's
own registered header, which apparently does not exist yet while its
where-clauses are being lowered.

`pat.rs:85` is `DispatchFromDyn` for `pattern_type!(T is !null)` over a type
parameter.

Read: three of the four causes outside Lane B (arith, flatten and pin), plus
all 31 blocked records, are ordering gaps. Each is a trait impl header that needs
a fact prepared later. The fourth is a missing profile.

The census ends with the crate record `implementation candidate inventory still
has pending headers` (`lib.rs`, file 0, byte 0). That check is
`Implementations::finish_constant_headers`, the first-error completion check
after the isolated trait-impl header loop in `prepare_trait_implementations`. It
refuses because the failed headers are still pending. It is not an independent
cause, and it stops the census before `prepare_associated_types` and the
definition loop.

Sixteen other `"invalid or exhausted ..."` messages remain, in ten files
(method selection, receiver inference and nominal trait proofs). The census
treats each one as a budget stop. If the census reaches one, split it the same
way.

Checks: fastchecks `1794031290105421940` (47 private cases) and
`1794031155598357318` (census mode, strict CLI Clippy). The pinned CLI matches
the source fingerprint. With that CLI, course selections pass without blessing:

| Course | Selection | Passed |
| --- | --- | ---: |
| PA19 | implementations and inherent headers | 65/65 |
| PA20 | coherence headers and contract equations | 80/80 |
| PA21 | binders | 76/76 |
| PA22 | impl sources with binders or `Fn` sugar | 7/7 |
| PA23 | same filter, plus the unconstrained marker | 27/27 |
| PA24 | same filter | 82/82 |

Evidence: `obj/parallel/core-census-2/` and `obj/parallel/census-batch1/`.
