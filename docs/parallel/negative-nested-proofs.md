# Negative premises in nested trait proofs

This is Lane B's item B of 2026-11-09, on origin/main `c2132742c`. It follows
the [placeholder sizedness item](nested-sizedness.md). That item fences a
projected-size nominal under placeholders, because the nested trait proof
that answers it does not carry the other header's premises. Item B carries
them into that proof. It lands in four checkpoints:

1. it moves the `MetaSized` premises onto the placeholder handle;
2. it propagates the handle through the nominal trait `State` and the
   selection `Caller`, with the fences kept;
3. it retires the fences under declared controls;
4. it records the evidence and this notice.

## 1. The MetaSized move (3330eac46)

`Placeholders` is one shared handle, `Arc<Sets>`, over the other header's
Sized and `MetaSized` sets. Each set is sorted and deduplicated.
`Placeholders::new(sized, meta_sized)` checks or sorts each set. Its work is
`|S|.max(1) + |M|.max(1)` for ordered sets, plus `n (ilog2 n + 1)` for each
set that must be sorted. `sized(ty)` and `meta_sized(ty)` are binary
searches at `len.max(1).ilog2() + 1`.

The overlap inputs build one handle per side, in `premises()`, and charge its
construction there. The negative query (`may_overlap_with`) and its payment
(`pay_negative`) clone the handle and construct nothing. `NegativePremises`
is removed.

Work accounting was declared before any code or run, and the measurements
agree with it:

| Scaled test, c = 8, 16, 32 | Base `c2132742c` | Step 1 | Declared change |
| --- | --- | --- | --- |
| `MetaSized` premise pair steps | 1946, 3775, 7492 | 1965, 3810, 7559 | 2c + 3 |
| `PointeeSized` control | 1681, 3236, 6391 | 1693, 3256, 6427 | c + 4 |

Each harness was run twice, and both runs agreed. No pinned work assertion
changed.

- **Observed, not declared:** `Shapes` and `sizes::Query` each shrink by one
  word. `Option<Arc<[TypeId]>>` is 16 bytes and `Option<Arc<Sets>>` is 8.
  - The only consumer of either size is the typed-headers byte reservation
    in `contract_completion.rs`, which reserves `size_of::<Proof>()` plus
    `size_of::<Shapes>()` against the retained-storage budget. That counts
    bytes, not work units, and nothing pins it.
- **Unchanged outputs:** the sixty negative-premise twins (status lines and
  every stderr) and the nested reach survey (79 files) are byte-identical to
  `c2132742c`.

## 2. Propagation with the fences kept

The set now travels with every proof that a negative coherence query starts
for its nested obligations.

- **Callers:**
  - `selection::Caller` has a `placeholders: Option<&Placeholders>` field,
    and `OwnedCaller` has the owned handle;
  - every construction site chooses: the State-derived sites pass their
    State's set, and the others write `None`;
  - a closed requirement (in `select_opened`, `prove_quantified` and a closed
    normal form) passes `None`, because it mentions no parameter and its
    reuse slot stays premise-independent.
- **Searches:**
  - `Search::assuming` is the one production constructor. With a set, it
    installs `NegativeSizing`, with fresh debts, and the builtin proof's
    placeholder walk, and it rejects an environment together with a set;
  - the tests' `Search::new` and `assuming` share one infallible body, and
    `new` passes no set;
  - `may_overlap_with` builds its Caller with the set, and no longer installs
    it after construction.
- **Rigid parameter sizing:** a search with a set decides a rigid
  parameter's Sized and `MetaSized` by the other header's premises, at any
  depth. Only the root, which retains Sized debts, records a `MetaSized`
  debt for `pay_negative`; a nested search has decided it.
- **Nominal trait State:** it holds the set of the placeholder proof that
  owns it. Its normalization `Session`, its clause-term and shape cursors,
  and the nested State of `normal_form::begin_reusing` (through
  `normal_form::Sizing`) carry it. Each of these rejects an environment
  together with a set.
- **Fences:** `PLACEHOLDER_SIZE_FENCE` and `sizes::Query::fence` stay, so
  the shape walk still refuses a projected-size nominal under placeholders.

Layout:

| Type | Before | After | Pin |
| --- | --- | --- | --- |
| `nominals::traits::State` | 800 | 808 | `early_conformance.rs`, `closed_callers.rs` |
| `selection::Caller` | 24 | 32 | `placeholder_sizing.rs` (new) |
| `selection::OwnedCaller` | 24 | 32 | `placeholder_sizing.rs` (new) |

- `State` grows by one word, its `Option<Placeholders>`, so both existing
  pins move from 800 to 808.
- `selection::Pending` and `normal_form::Query` hold an `OwnedCaller` and a
  `State` inline, so each grows by 8. The projections `Cursor` grows by one
  word as well.
- Two retained-byte charges that use `size_of::<selection::Pending>()` rise
  by 8 per charge: the source-retention charge in
  `normal_form/data_result.rs` and the projection-requirement charge in
  `specialization/obligation_chain.rs`.

The step's outcomes were declared before any code or run, and the evidence
agrees with them. The base is the step-1 CLI; the candidate CLI was built from
this step's tree before its two test-only edits, the `closed_callers.rs` pin
and the test-only State accessor.

| Gate | Result |
| --- | --- |
| Sixty negative-premise twins | status lines and every stderr byte-identical |
| Flip survey, 336 files | byte-identical |
| Census lookahead | `census.tsv` byte-identical, 23 records; `result.json` differs only in CLI, producer and timing identity |
| Golden table, premise-less pass | byte-identical |
| Golden table, placeholder pass (empty set, every type parameter) | byte-identical, so no placeholder row moved |
| Scaled `MetaSized` pair and `PointeeSized` control | unchanged: 1965, 3810, 7559 and 1693, 3256, 6427, in both runs |

- **Coverage:** every placeholder row that normalizes still fails at its first
  trait step, as at the base. In this step, only the new unit test runs a
  nested search under a set.
- **Checks:** focused fastchecks passed:
  - the private bodies, methods and dispatch chunks (459, 325 and 213 cases);
  - the trait integration roots (116 and 240);
  - crate linking's absence and modifier transport (5) and their native
    clients (2);
  - native negative coherence (1);
  - `pa20`.
