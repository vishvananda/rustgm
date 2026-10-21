# Captured input conditions with explicit reference results

This checkpoint supersedes the `reference-result` boundary recorded by
`299806d3` without changing its fixture bytes. The original case independently
compiles in Rust but previously failed LowIR and constant inspection at offset
162. The source closure borrowed through its stored `&'b2 u8` and returned an
explicit `&'static u8`. Its input well-formedness debt was already retained, but
there was no inferred result port: the source Link had revision zero, no bridges
and an ordinary unconditional proof. The missing relation was `'b2: 'static`.
The actual parent image was expression-borrow region 4 in `make`.

A simpler accepted `&'static u8` input exposed the same failure without any
root-well-formedness Formation record. The diagnostic incoming bridge made the
whole parent/child proof pass; an opposing nonstatic actual input still failed
the parent proof. The original diagnostic used an artificial output flag solely
to exercise the existing conditional solver. Production retains input bridges
with `output=false`; revision continues to count inferred result ports only.

The producer observes only the actual retained capture field types. Cached type
properties skip fields with no surrounding lifetime context; one lazy typed fold
shares work across a field DAG. Nested callable-local binders remain private,
while captured outer slots survive. A bounded sparse membership index selects
incoming slots, with charged field, fold, lookup and prefix visits. Every new
bridge must map to a variable owned by the exact immediate parent; existing
closing and public-binder exclusions remain in force. No new public schema or
lifetime assumption was added.

A nonempty checked bridge list selects the existing conjunctive `prove_output`
route. It re-proves the whole source graph under only its exported scalar debts,
which the actual parent must discharge. The same checked catalog then requires
the exact actual images at callers. This changes neither capture permissions nor
source initialization, loans, storage, specialization or runtime layout.

Revision is not the proof seal. The summary retains the exact immutable Arc
identity of the checked Link and matches source/key ownership. A private test
uses an input-only Link with revision zero and proves its static condition, then
rejects both a different parent and a changed parent-region image with the same
revision and bridge count. The checked map cannot be mutated through a shared
Arc. Public callable-local and execution receiver slots are not incoming field
permissions.

The membership concern from review was not an observed regression: membership
obligations are attached to defining function contracts. The accepted captured
opaque-call control and opposing opaque static escape preserve their outcomes
using the existing solver unchanged. No membership fallback or speculative
proof route was added.

The nine independently predeclared controls are in
`pa24/tests/capture-input-hypotheses.json`: the unchanged original, simple static
input, nested reference result and opaque-call positives; nonstatic input,
owned-local escape, public input, opaque escape and nested callable-binder
negatives. Source and native evidence is under
`obj/parallel/pa24/reference-result/`. `capture-input-outcomes.json` records the
qualified checkpoint and explicitly supersedes the earlier boundary receipt.
No assignment activation or text-reference regeneration accompanies this work.

For the committed source inventory, run from the repository root:

```sh
CARGO_TARGET_DIR=obj/rust CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 \
  python3 scripts/storage.py run --prune-under obj/parallel/pa24 -- \
  cargo test --locked --manifest-path dev/Cargo.toml --test captured_dispatch \
  -- --skip execute --nocapture
```

For the committed native inventory, using the unchanged pinned CPPGM backend:

```sh
CARGO_TARGET_DIR=obj/rust CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 \
  python3 scripts/storage.py run --prune-under obj/parallel/pa24 -- \
  cargo test --locked --manifest-path dev/Cargo.toml --test captured_dispatch \
  -- --ignored --nocapture
```

`-- --include-ignored --nocapture` runs both inventories together. The two
focused filters for this slice are `incoming::` and `formation_truncation::`.
The latter requalifies the unchanged original reference-result bytes alongside
its earlier scalar and nested controls. Native tests remain ignored by default.
This is the committed captured-dispatch inventory, not every planned PA24 lane
or an assignment-release claim.

The four observer controls cover nested binders, an empty no-capture case with
zero observer work/index allocation, scalar fields with 4,096 unrelated types,
a shared diamond and repeated fields at depths/counts 8/32/128, and exhaustion.
The observer charges `3 * depth + repetitions + 4` steps for that diamond:
36/60/156, 108/132/228 and 396/420/516. These are bounded logical work results,
not wall-time neutrality or a claim about the separate CFG-point solver.

Qualification passes strict all-target/all-feature Clippy and the syntax audit
(1,491 files, zero findings). The input-only seal test, four observer tests,
conditional graph/caller/catalog/entry proofs, all ten captured-dispatch source
and native functions, seven reference/receiver-chain functions and twenty-six
source-closure tests pass. Four preexisting source-closure native tests were not
selected in that source-only batch. Upstream checks verify 24 fixtures and 126
Reference files; layout reports 38 stages. All seven independently declared
field-width/repetition 8/32/128 and ordinary source controls pass Rust object and
frontend LowIR checks. Every active PA1–23 text suite is exact; no oracle changed.

The earlier admission refusal (282.9 GiB free versus 285.2 GiB including shared
reservations) ran no compiler command; integrator-owned cache cleanup restored
headroom without changing limits. Historical diagnostic/lint failures remain in
the receipt. The accepted root helper extraction `3dd9e9d3` fixes the combined
inspection-function line limit and is a separate dependency of this snapshot.
