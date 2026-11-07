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
configured `--source-contract-work-limit`. That constant began as a metadata
decoding bound (`74662c8e4`, "1,000,000 path/type atoms"). Draft commit
`d9b3ea28e` reused it, together with the 16 MB artifact bound, as this ledger's
work and storage caps. Declaration checking has used an input-proportional
guard since `a66678f30`. Replaying the charging code over the
opened sources estimates about 932K units for target-feature parsing and
implication insertion alone: 6,337 x86 sites using only 87 distinct `enable`
strings. That cost is linear and repeated; it is an estimate, not attribution.

1. Measure per-payer totals once with an analysis-only diagnostic.
2. Validate each distinct `enable` literal once and reuse its closed feature
   set. Every site still pays its lookup, owner/function/safety validation, row
   and witness retention, and its own gate result.
3. Replace the borrowed constants with the input-proportional work and storage
   bound that the amended `spec.md` budget rule requires: a documented factor
   of the authenticated input graph, computed once before the first charge, with
   sticky failure. Size the factor from attribution of genuinely linear work.
   Qualify it with paired linear-scaling inputs and an adversarial superlinear
   control. Never refund failed work, skip validation or raise the factor to
   absorb repeated work.

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

The census does not enter bodies while declarations fail (integrator decision,
2026-11-07, after [census batch 4](parallel/census-batch4.md)).
- Why: about 30 later declaration passes each consume every declaration's
  facts and stop at the first error. Continuing into bodies would need per-item
  isolation in every pass, plus a body-to-declaration dependency map. That work
  is no longer needed once declarations pass.
- Trade-off: body causes stay invisible until then.
- Instead, batches drive declaration failures to zero (census 5: 62 failed,
  55 blocked). Per-pass isolation is reconsidered only if that stalls.

Where RustGM is stricter than rustc 1.98.0 on an observed program, it follows
rustc, for example type privacy in array lengths in signatures.

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

## Std profile decisions

Made by the integrator on 2026-11-07 at the user's request, so that std work
does not wait on them. Neither decision blocks core or alloc.

**Std builds as an rlib only.** Upstream std lists `crate-type = ["dylib",
"rlib"]`. Under the course profile the target-unit executor builds the rlib,
drops `dylib` and records the dropped type in each unit receipt. A target that
lists only `dylib`, `cdylib` or `staticlib` remains a typed gap.

- For: rustc links std statically from its rlib unless `-C prefer-dynamic`
  is given, so default-std consumers behave the same. A Rust dylib needs
  position-independent code, a shared-object link with an exported-symbol list
  and dylib metadata. That is backend work, and the backend is frozen.
- Against: RustGM ships no `libstd-*.so`, and supports neither
  `-C prefer-dynamic` nor dylib consumers. Its std artifact set therefore
  differs from upstream bootstrap's. The decision is reversible: if the backend
  ever gains shared objects, the dylib becomes an added output.
- Rejected: implementing dylib output (it needs the frozen backend), and
  keeping the gap (it blocks milestone 6 for an artifact no default consumer
  links).

**Rust 2015/2018: need-driven, ahead of PA40.** The std closure contains
cfg-if (2018) and rustc-demangle (2015). RustGM implements exactly the edition
rules those two crates exercise, each with controls compared against rustc
1.98.0. Every other 2015/2018 rule stays rejected by name through the
`support::edition` legacy-rule table until PA40 closes the editions. That
table already holds the 2015/2018 switch and named per-rule rejections.

- For: this is the smallest work that unblocks the std graph. Each rule is
  checked independently, and an unimplemented rule fails by name instead of
  silently getting 2021/2024 semantics.
- Against: edition support stays partial, so arbitrary 2015/2018 crates still
  fail; the full closure stays PA40's. A later need (another std dependency or
  a test client) adds its rules the same way.
- Timing: both units depend on core, so neither can build until core passes.
  Lane B first reports the exact rule list, including rustc-demangle's use of
  `core::char` and `unreachable!` in the bootstrap model. It implements the
  rules after its current core work, in the same batch as the executor's dylib
  rule.

## Process

- Qualify a batch of related repairs with one focused gate and one short
  checkpoint under `docs/parallel/`. Add one tracker entry per batch.
- Change `spec.md` only for new normative rules, ROADMAP only at milestones.
- Fixtures stay in the earliest owning assignment; keep them minimal and declare
  outcomes before generating any oracle.
- Component publication of the ten implemented units runs in CI when lanes are
  idle; it does not gate core work.
- Commits use the repository's single-line imperative subjects without trailers.
