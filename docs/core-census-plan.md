# Core census and transport plan

Adopted 2026-11-07. This replaces the serial first-error probe loop as the
operational path to original core, alloc and std. It changes execution order
and process only. Assignment requirements, the [unit graph](../assignment-plan.json),
`spec.md`, source pins and the frozen CPPGM backend boundary are unchanged.
The [blocker tracker](library-build-blockers.md) still owns concrete failures.

## Assessment

The probe/fix/reprobe loop cleared LIB-019 (10-24) through LIB-192 (11-07), but
every blocker was in loading, expansion or header-attribute collection:

- LIB-193 crosses its budget in `target_features::collect`, called from
  `Checker::from_facts` (`dev/src/sema/declarations.rs:350`). That precedes
  `Checker::run` (`:457`). Original core has never run the declaration passes,
  body inference of its 21,230 functions, constant evaluation of 8,697 `const fn`,
  borrow checking, lowering or metadata emission.
- The observer uses `--emit-types`, so even a passing run stops before borrow
  checking, LowIR and metadata.
- All 49 distinct `rustc_*` attribute kinds in the 375 opened files are now
  recognized. The next blocker stream is body semantics.
- stdarch supplies 13,819 of the 21,230 functions, 6,338 `target_feature` sites
  and most of the 10,207 `simd_*` calls. Its failures will be highly repetitive.
- Every phase propagates its first `Err`; one run reports one blocker.
- A typical blocker repair carried about 30 lines of fixtures, tests and prose
  per compiler line. From 11-01 to 11-07 documentation grew by 89K lines and
  compiler code by 74K.

Structural gaps that a first-error loop would reach only after many more runs:

| Gap | Evidence | Consequence |
| --- | --- | --- |
| Transported bodies cannot branch | `support/crates/generics/rows.rs:180` has Return/Jump/Unreachable; body IR also has Branch/Panic/FalseEdge (`body/ir.rs:354`) | No separately compiled consumer can call `Option::map` or an iterator adapter |
| Scalar-only LowIR | `lowir/vocabulary.rs` | Demanded SIMD bodies cannot lower, although the backend has `v128` |
| Only U32 atomics | `atomic-u32` unit | alloc/std need other widths, which the backend already supports |
| Rust 2015/2018 absent | `support/edition.rs` | rustc-demangle and cfg-if in the std graph |
| No target-unit executor | `scripts/host_graph.py` skips target units | alloc/std graphs cannot be driven |
| No allocator/start/TLS runtime | no `__rust_alloc`, `lang_start` | alloc and std consumers |

The backend stays frozen (user decision, 2026-11-07). Inline assembly, f16/f128
and non-temporal stores remain explicit locked-backend capabilities; demanded
paths that need them stay fenced and are recorded, never worked around.

## LIB-193

The sema header ledger is one crate-wide counter capped by the fixed
`MAX_COMPONENTS` constant (`declared_origins.rs:570`), independently of the
configured `--source-contract-work-limit`. Replaying the charging code over the
opened sources estimates about 932K units for target-feature parsing and
implication insertion alone: 6,337 x86 sites using only 87 distinct `enable`
strings. That cost is linear and repeated; it is an estimate, not attribution.

1. Measure per-payer totals once with an analysis-only diagnostic.
2. Validate each distinct `enable` literal once and reuse its closed feature
   set. Every site still pays its lookup, owner/function/safety validation, row
   and witness retention, and its own gate result.
3. Only if attribution shows remaining genuine linear work above the cap, bind
   the header ledger to the explicitly configured, snapshotted
   `SourceContractLimits` instead of the hard-coded constant. The ordinary
   default stays one million, as `spec.md` requires. Never derive an allowance
   from input size, refund failed work or skip validation.

## Census mode

Add an analysis-only keep-going census to the CLI. In the per-definition
declaration loop and the body/initializer loops, a failed item is marked failed,
its dependents are reported as blocked rather than rechecked, and checking
continues. Budget exhaustion remains sticky and ends the census. A census always
exits nonzero, writes no artifact or metadata and never counts as a pass.
It prints deterministic records: phase, category, location and message.
Ordinary mode keeps first-error behavior byte-for-byte.

The census supplies the first progress measures (declarations and bodies checked
out of the whole crate). Repairs are then grouped by cause rather than by
discovery order.

## Lanes

| Lane | Worktree | Work | Exit |
| --- | --- | --- | --- |
| A | `rustgm-pa29-work` | LIB-193 attribution and repair | Unchanged original core passes target-feature collection with unchanged inputs and default limits |
| B | `rustgm-pa26-work` | Census mode and observer support | One census lists all failures in the reached loops; ordinary-mode outputs unchanged |
| C | `rustgm-async-work` | Typed body transport: Branch/Panic first, then complete body IR | A separately compiled provider exports a branching generic body; a fresh consumer executes it natively |
| D | next free lane | Target-unit executor, Rust 2015/2018, allocator/start shims | Small real crates build and run through RustGM |

Two heavy jobs run at a time. A and B hold the heavy slots first; C begins with
reading and design and takes a slot when one frees. A capacity refusal is an
infrastructure wait, not a failure. The isolated const-callable WIP stays on
`implement/const-closures-20261107`, unqualified and off main; the census decides
when const closures return.

Milestones:

1. LIB-193 cleared; first declaration census of original core.
2. Original core passes declaration analysis.
3. `--emit-types` passes for original core.
4. Borrow checking, LowIR and metadata for original core; a separate consumer
   executes its real `Option`/iterator bodies.
5. Original alloc and an allocation/collection consumer.
6. Original std and a default-std consumer.

## Process

- Qualify a batch of related repairs with one focused gate and one short
  checkpoint under `docs/parallel/`. Add one tracker entry per batch.
- Change `spec.md` only for new normative rules, ROADMAP only at milestones.
- Fixtures stay in the earliest owning assignment; keep them minimal and declare
  outcomes before generating any oracle.
- Component publication of the ten implemented units runs in CI when lanes are
  idle; it does not gate core work.
- Commits use the repository's single-line imperative subjects without trailers.
