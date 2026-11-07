# Census batch 2: demanded header facts

The second original-core census ([batch 1](census-batch1.md)) left three
ordering gaps in Lane A's area. In each one a trait impl header needs a fact
that a later fixed pass prepares. This batch prepares those facts on demand. It
also keeps failed impl headers item-local at the end of the trait-impl pass,
and it gives the remaining fixed "invalid or exhausted" messages separate
wordings.

## Design

Rust's header semantics are demand-driven, but the checker runs its passes in a
fixed order. The repair keeps that order. When a pass meets a fact that a
later pass owns, the fact is prepared on demand through the existing
declaration worklist. Each demand meets three conditions:

- It goes through a guarded state, so re-entering an active state is reported
  as a cycle.
- It makes monotonic progress. A demand fires only while its fact is absent, and
  each demand completes or fails one state.
- It is charged to the shared step budget of the drain that demanded it.

The three causes:

1. **Operator methods behind array lengths.** Body readiness already defers a
   binary operator until its `Rhs` default is checked. It now also defers until
   the operator method's declaration header exists, through
   `Requirement::Definition(method)`. The suspended constant then resumes,
   whether it is an anonymous impl self-type length or an initializer. The
   function's existing `Active` state makes a re-entry a cycle. The header
   attribute check now treats a missing function header as an unprepared
   prerequisite, worded separately. It reports "changed" only when a header
   exists for another declaration.
2. **Associated result contracts behind impl where-clauses.** Before a
   declaration projection selection opens a fresh query, it checks the subject
   projection. If that projection names a source trait associated type with no
   type contract yet, the selection demands the member's
   `Work::Definition`. That declares the one member, as `prepare_associated_types` would, and
   lowers its result bounds with the ordinary bound worklist
   (`Work::AssociatedBounds`). A per-member
   state records active and complete members. The later pass skips members that
   are already prepared. Declaration is synchronous, so a demand made while
   the member's own bounds are being lowered sees the declared contract. That
   is the partial-contract view the fixed pass already has. No new cycle can
   arise at this step.
3. **An impl's own trait reference behind `Self::` shorthand.** In rustc,
   `Self::X` in a trait impl resolves through `impl_trait_ref`. That needs the
   trait path's arguments, but not the where-clauses. Here the where-clauses
   are lowered first, and the impl's registry header is published only at the
   end. A shorthand `Self::` in a trait impl whose header is still active now
   demands the impl's trait reference. It lowers the trait path arguments once
   and records the reference predicate. The `Arguments` and `Finish` phases
   reuse that predicate. `Self::` inside the trait reference's own arguments
   meets the active reference and is reported as a cycle, as in rustc (E0391).

**Census isolation of the terminal check.** `finish_constant_headers` accepts
an isolation predicate. Ordinary mode passes none and keeps the exact
check. In a census, a pending header whose impl is census-poisoned or settled
stays pending, so selections that could use it keep waiting on it and are
recorded as blocked. Any other pending header is still a crate-level failure.

The inherent-method index has the same census rule: pending inherent owners
whose headers failed stay isolated, and ordinary mode keeps "unfinished
inherent member headers". In a census, a member of an impl whose header failed
is recorded as blocked by that header, not as a separate failure.

Operator readiness covers unary `-` and `!` the same way as binary operators.
It demands the method header and, for an operator with its own `Output`, that
member's result contract.

## Controls

Every outcome was declared before its rustc 1.98.0 oracle, and the
declarations came before the implementation changed. The evidence is in
`obj/parallel/census-batch2/controls/`: `declared-outcomes*.json`,
`rustc-oracle*.json` and the `probe-*.json` RustGM observations. One
disclosure: exploratory rustc runs of earlier `self-*` reductions came before
the first declaration.

| Set | Cases | rustc | Pre-repair pin | This branch |
| --- | --- | --- | --- | --- |
| `pa31/tests/spec/demanded-header-facts/` | 5 (2 accept) | 5/5 agree | 2 positives rejected | 5/5 |
| `pa20/tests/spec/projections/header-demand/` | 8 (5 accept) | 8/8 agree | 5 positives rejected | 8/8 |

The pa20 cases are manifest entries with the rustc `program` status, which was
also checked with `--emit=obj`. The private invariant
`demand_header_facts.rs` checks all 13 cases and the exact first errors of
the rejections. The rejections keep their own causes:

- overlap: coherence;
- missing `Sub`: no validated implementation;
- `const N = N - 1`: constant cycle;
- `Self::Out` in the impl's own trait arguments: header cycle (E0391);
- `Self::Missing`: no such associated type (E0220).

A second unary declaration (`declared-outcomes-2.json`) disagreed with rustc
on `static-length-neg.rs`. The fixture lacked the `sync` lang item that a
`static` requires. The corrected copies are in `declared-outcomes-3.json`, and
both sets of records are kept. With the corrected copies, rustc accepts
`static-length-neg-sync.rs` and RustGM still rejects it, with "missing
completed associated type definition" at the `-1` of a `const` that a
`static` array length reads through another `const`. The same shape passes
when the impls come first, or when the operator is binary. That is a separate
open gap: the scalar unary path does not wait for the impl's `Output`
definition. It is not a fixture. `literal-sub-two-impls.rs`
(`3 - 1` with `Sub` for `usize` and `u8`) is accepted by rustc and rejected by
RustGM. That is rustc's builtin integer binop rule, recorded below as the next
blocker.

## Messages

No fixed "invalid or exhausted" message remains in `dev/src`. Each of the 16
sites now chooses between "… work exhausted" and "invalid …" through
`census::fold_cause`. Inference `Budget`/`Overflow` and selection failures with
budget wording keep the sticky budget wording. Everything else is an ordinary
item failure. The proof source extension site distinguishes "capacity
exhausted", and the call result site distinguishes `Error::Budget`.
`census_records.rs` checks that the census's lexical classifier reads each
split wording correctly.

## Cause 4

`pattern_type!(T is !null)` over a type parameter is left explicit. The check
in `refined_annotations.rs` requires a `Type::Pointer` subject. Accepting a
parameter needs a pattern-type representation that is not a pointer, and
deferred well-formedness at instantiation. That is not a small change.

## LIB-193 regression: projected-bound staging

Four PA20 cases failed on main with "source header attribute producer exceeds
storage budget": `assoc-bounds/wide16`, `wide64`, `deep16` and `deep64`.
The 4-bound and 4-level cases still passed.

A measurement with the bound raised a thousandfold, and each charge attributed
to its call site, found the cause. Nearly all the bytes come from
`charge_bound_staging` in `generics/bindings/proj.rs`. In an ordinary crate,
projected associated-binding staging charges the header ledger. In a source
core or client it charges source retention instead, so LIB-193's sizing, which
was measured on original core, never saw this cost.

The growth is linear, and the setup cost is negligible (an intercept of about
1.9 KB):

| Case | Elements | Work | Bytes | Bytes/element |
| --- | ---: | ---: | ---: | ---: |
| `wide4` | 136 | 163 | 9,376 | 69 |
| `wide16` | 232 | 439 | 37,808 | 163 |
| `wide64` | 616 | 1,543 | 145,584 | 236 |
| `deep4` | 197 | 446 | 14,748 | 75 |
| `deep16` | 425 | 1,579 | 63,436 | 149 |
| `deep64` | 1,337 | 6,087 | 249,356 | 187 |

Each added written bound costs about 2,250 bytes (2,369, then 2,245), and each
nesting level about 3,400 to 4,060 bytes. That is a marginal cost of about
280 bytes per graph element of bound syntax, above the 128-byte factor. Work
stays at or under 4.6 units per element.

A setup floor would not fix this, because the cost is a slope: a 256-bound
input would fail again. Raising the shared factor would weaken the bound for
every producer. Instead, the bound counts the missing axis. In ordinary crates
only, which are the only crates where staging charges this ledger, every
`Bound` and associated `Constraint` AST node adds 4,096 attempted bytes. That
leaves 1.8x headroom per written bound and 2.4x per nesting level. Original
core and clients are unchanged.

`projected_bound_staging_keeps_linear_headroom_in_ordinary_crates` checks two
things for the 4, 16 and 64 cases of both shapes:

- each case stays within two-thirds of its byte bound and half of its work
  bound;
- the cost per added bound or level stays flat, which is the linear-scaling
  pair.

The existing adversarial control, repeated whole-row validation per call, has
no bound nodes and still exhausts. A generated 256-bound case now passes the
header ledger and then stops at a separate budget, "type constraint work
budget exceeded". That is recorded for the integrator and not changed here.

## Checks

All checks ran on the final sources, which have the same fingerprint as the
pin:

- Private fastcheck `1794065004108351246`: 41 exact tests, including the new
  controls, census records, binary/unary operator authority, pending headers,
  waited queries, the impl constraint and projection normalization
  regressions, and the ten LIB-193 `cpu_policy` controls.
- Integration fastcheck `1794065297556463111`: `census_mode`, shorthand and
  definition readiness, and inherent headers, with strict CLI Clippy.
- `--pa pa20` fastcheck `1794065154997624281`: 1,451/1,451 cases.
- `make test-upstream`: passed.

The eight new PA20 references were generated with `--bless` only for those
cases, after their outcomes had been declared and checked with rustc. Their
exit statuses match. With pin `e4a04221`, course selections pass without
blessing:

| Course | Selection | Passed |
| --- | --- | ---: |
| PA19 | implementations, inherent and trait headers | 140/140 |
| PA20 | projections, header projections, coherence | 280/280 |
| PA21 | binders, assoc-bounds | 83/83 |
| PA24 | constrained call outputs | 8/8 |

## Census 3

The analysis-only census used qualified pin `e29c6ab1`
(`obj/parallel/census-batch2/qualified-2/`), with inputs unchanged and 375
opens in 30 s. Its records are byte-identical to the run with the pre-fix pin
`e4a04221` (`core-census-3/`, now retired by hash). Only `dev/bin/crates.rs`
is rebound; the recorded reason is that it adds only an unused
`--source-profile` value.

It checked 37,780 declarations: 92 failed and 622 blocked. Bodies were not
reached. The census no longer stops at either crate-level completion check.
It ends with budget record 715, "source lint policy work budget exceeded"
(`ops/arith.rs`, byte 0). That is the fixed work cap in
`resolve/lint_policy/producer.rs`, which this census reaches for the first
time. The ordinary first error is `any.rs` byte 6032, "object requires exactly
one principal trait" (Lane B).

| Failures | Message | Examples |
| ---: | --- | --- |
| 35 | associated type requires an unambiguous implementation | `control_flow.rs` impl header, `array/drain.rs` `call_once`, `step_by.rs` |
| 18 | invalid canonical trait selection | `flatten.rs` `try_fold`, `fold` |
| 16 | object requires exactly one principal trait | `any.rs` (Lane B) |
| 8 | outlives bound requires a named lifetime | `filter_map.rs` `find`, `flatten.rs` |
| 4 | semantic type or associated item is private | `io/error/repr_bitpacked.rs` `const _` |
| 3 | invalid associated equality selection | `array/mod.rs` `try_from_fn`, `try_map` |
| 3 | missing storage opaque declaration | `mem/mod.rs` `to_be_bytes` (74 blocked), `uint_macros.rs` |
| 2 | incompatible types | `portable-simd to_bytes.rs` alias `Bytes` (426 blocked), `array_impl_default` `($n - 1)` (52) |
| 1 each | `pat.rs` non-pointer `!null` (cause 4); `from_try` later semantic assignments; `int_macros.rs` `Buf` projection normalization (11 blocked) | |

Compared with the diagnostic census of the same tree before the last
repairs:

- `static POWER_OF_FIVE_128` now passes, because unary readiness applies.
- 19 "trait method receivers require trait semantics" failures are now
  blocked by their failed impl headers.

The dominant cause is rustc's builtin integer binop rule (478 blocked). rustc
unifies the operand and result types of an integral or float binary operator
before trait selection. Without that rule, `{ $size * $elems }` and
`($n - 1)` cannot choose among the many source `Mul`/`Sub` impls.
`literal-sub-two-impls.rs` is the declared rustc-accepted control for it. The
next causes are the lint-policy work cap, Lane B's object principals, and the
storage opaque declarations of `to_be_bytes`.

Evidence:

- `obj/parallel/census-batch2/` (controls, receipts, pins, course logs);
- `obj/parallel/core-census-3/` and `obj/parallel/core-census-3b/`;
- `retired-*.json` (hashes of the retired pins).
