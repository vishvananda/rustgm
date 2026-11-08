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

Decisions after [census batch 5](parallel/census-batch5.md) (census 6: 32
failed, 52 blocked):

- "Canonical trait selection requires pending declarations": a copy decision
  in a declaration-time constant waits for derived impls, such as
  `Copy for Option<T>` and `NonZero`'s `Copy`.
  - Chosen: widen early validation to ADT subjects whose arguments are exactly
    the impl's own parameters. A pending copy decision becomes a dependency of
    the waiting constant, not a failure. This follows the on-demand header
    facts (batch 2) and the opaque storage wait (batch 4). It costs more
    dependency edges, which the worklist's existing cycle detection must
    cover.
  - Rejected: answering copy decisions from declaration-level availability.
    That would decide `Copy` from impls not yet validated, breaking the rule
    that semantic decisions use validated facts.
- `pat.rs` (core's ordinary first error) is Lane A's: generic `!null` pattern
  types and pattern-type `DispatchFromDyn` coherence.

Decisions after Lane B's [specialization and no_std batch](parallel/specialization-nostd.md)
and Lane C's [lifetime nominals](parallel/lifetime-nominals-transport.md):

- **`--extern core=` names the direct dependency, as rustc does.** std's
  graph gives six units (cfg-if, hashbrown, panic_abort, rustc-demangle,
  std_detect, unwind) the `rustc_std_workspace_core` shim (`pub use core::*`)
  as `core`. Only alloc, compiler_builtins, the shim and std get real core.
  The no_std injection resolves `core` through the extern prelude, so through
  the shim a client reaches core's prelude by the shim's glob re-export.
  - Rejected: refusing `--extern core=` and passing core as inventory only.
    The executor would then have to rewrite Cargo's edges and hide the shim's
    real semantics.
  - Cost: cross-crate glob re-exports must resolve through the shim.
- **`compiler_builtins` follows rustc's injection rule.** RustGM injects it
  where rustc does, bound through the ordinary dependency path. A missing
  crate fails by name, as rustc requires. alloc and the shim also depend on it
  directly.
- **Variant re-exports and override conformance go to Lane B.** Variant
  re-exports (core's `prelude::v1`: `pub use crate::option::Option::{self,
  None, Some}`) block no_std clients of real core. Override conformance of a
  repeated `Self::Assoc` where-clause (`traits/conformance/mapping.rs`) is the
  other item. Lane B diagnosed both. If a variant re-export needs a new
  version-13 row, Lane B reports the tag to the integrator before using it,
  because Lane C is adding version-13 shapes at the same time.
- **Lane C's next slice:**
  1. methods of `impl<'a> X<'a>` (captured callable frames), which core's
     `Arguments`, `Location` and `PanicInfo` need;
  2. tuple and unit structs, for `NonNull` and `PhantomData`;
  3. writes through a `&mut` field of an immutable binding. rustc accepts
     them, so the fix lands under version 13 only.
  Declared nominal lifetime bounds and region function-pointer fields stay
  fenced by name until a demanded item needs them; core's panic types declare
  none.
- **After [census batch 6](parallel/census-batch6.md)** (census 7: 5 failed,
  25 blocked):
  - `intrinsics::assume`, reached by `hint::assert_unchecked` from
    `fmt/num_buffer.rs`, gets its own validated constant-execution handler,
    owned by Lane A. A true condition returns unit. A false one fails as a
    constant-evaluation error, as rustc's E0080 does. This follows the
    bit-count intrinsics; intrinsic fallback bodies stay unexecuted.
  - The rule to follow rustc where RustGM is stricter covers stable language
    and the features the pinned libraries use. Unstable internal forms that
    nothing pinned uses may stay rejected by name until something demands
    them. `!null` pattern types over integer or reference bases are one such
    form: rustc accepts them and core does not use them. This is a recorded
    deviation.
- **After Lane B's [core sites and re-exports batch](parallel/core-sites-reexports.md)**,
  which cleared the last four sites of its cause:
  - **Shim glob re-exports use one version-13 glob row naming the foreign
    module**, and the consumer re-resolves it against that crate's own exports
    with rustc's shadowing and ambiguity rules. The shim is
    `rustc_std_workspace_core`'s `pub use core::*`.
    - Why: a row per shim keeps metadata proportional to what the shim
      declares.
    - Cost: the consumer needs cross-crate glob resolution, charged per name it
      resolves.
    - Rejected: one foreign binding per core public name. It would copy core's
      export table into every shim and add a consistency proof between the
      two.
  - **Variant re-exports reuse `ModuleBindingRow`**, targeting a `Variant`
    declaration in the value namespace under a version-13 gate. There is no
    new tag. Older profiles reject it by name. Consumers re-derive it, with a
    forged-namespace opponent.
    - The variant's type-namespace alias is not emitted, matching local enum
      member scopes. It is added only if a demanded use appears.
  - **`compiler_builtins` follows rustc 1.98's crate-loader rule**, which Lane
    B observed with an empty sysroot. rustc injects it after expansion and
    skips `#![no_core]` and `#![compiler_builtins]` crates, and graphs that
    already contain a `#![compiler_builtins]` crate. A missing crate is E0463.
    - RustGM therefore transports a `#![compiler_builtins]` descriptor flag
      under version 13. The PA29 no_std controls compile their existing
      stand-in with RustGM as well as rustc.
    - Rejected: skipping injection. That would accept graphs rustc rejects and
      leave the builtins link implicit.
  - Lane B also owns "invalid scoped proof evidence": a generic body that
    selects a region-binding impl fails this way, on main too. Core's adapter
    bodies are likely to meet it.
  - Lane B reports any new version-13 shape to the integrator before using it,
    because Lane C is adding shapes at the same time.
- **Forged shared region rows** may be rejected by the earlier Self-identity
  or field-substitution check. A forgery must be rejected by name, but it
  needs no check of its own.
- **After Lane C's [lifetime-capturing methods](parallel/impl-region-methods-transport.md)
  and [unique-reference writes](parallel/unique-writes-transport.md)**,
  2026-11-08:
  - **Tuple and unit struct constructors are variant declarations (tag 11)
    parented by their struct**, bound in the struct's module value namespace,
    with tuple fields named by their ordinals. There is no new tag. Local sema
    already treats a struct's constructor as the constructor of its single
    `Variant`, and Lane B's value-namespace variant re-exports use the same
    row.
    - The consumer re-derives everything from the parent it installed:
      - a struct-parented variant must be that struct's only variant;
      - it must have tuple or unit shape and carry the struct's name;
      - a record struct has no constructor.
    - The constructor's visibility is computed as rustc computes it: the
      narrowest of the struct's and its fields' visibilities, narrowed to the
      defining crate by `#[non_exhaustive]`. It is never taken from the wire.
    - Forgeries include:
      - a record struct with a constructor;
      - a second variant under a struct;
      - an enum variant claiming a struct parent;
      - a widened constructor visibility;
      - every strict prefix.
    - `struct T();` stays fenced by name until a demanded use appears.
    - Cost: every consumer path that reads a variant dispatches on its parent's
      kind. Lane B's variant re-export gate rejects a struct-parented target by
      name until Lane C extends it.
    - Rejected: a new constructor tag. It would duplicate the variant's shape
      and field rows and diverge from the local IR.
  - **The const fence on methods of lifetime-capturing inherent impls lifts
    next**, after struct constructors. Core's `Arguments::from_str` is such a
    method, and `PanicInfo::new` probably is too. A fresh consumer gives these
    methods the same const checks and evaluation that version-13 const free
    originals get, with the captured regions as leading early parameters. No
    const evidence is transported.
  - **Lane C's order** after that:
    1. other-provider region nominals;
    2. the caller-location ABI (`#[track_caller]` imports and
       `Location::caller()`);
    3. the `panic_impl` extern, together with the unsizing coercion fence.
  - **Lane C's amended controls are accepted:**
    - the three capture-count forgeries now reject earlier, by name;
    - the held client binds a `&'static str`, because its provider never
      makes `u8` `Copy`;
    - the lifetime-nominal gate moved to the trait-impl form;
    - the unique-write provider gained its leading field before its first run.

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

Follow-up decisions after [Lane B's batch](parallel/definition-lineage.md),
2026-11-07:

- **Rust 2021 source clients are admitted too.** libc (edition 2021, two
  build units and a build-script run) is in std's graph, and it builds
  against source core. It goes through the same source-client gate as 2015 and
  2018. Any 2021 rule that RustGM has not implemented still fails by name.
- **`#![no_std]` source clients are required, not deferred.** alloc, cfg-if,
  rustc-demangle and libc are `#![no_std]` crates over core. A source client
  must therefore get the implicit `core` extern-prelude entry, bound to the
  authenticated `--source-client core` identity, and its edition's
  `core::prelude::rust_20xx` glob, checked against rustc. The `#![no_core]`
  form stays available.
  - Trade-off: this opens alloc's first gate before core passes, and its
    controls can use small original-shaped cores. A real alloc build still
    waits for core.
- **Ledger factors stay per ledger.** The lint ledger keeps 8 steps and 64
  bytes per byte; the import-lineage ledger uses 8 and 128. Each is sized from
  its own attribution, with at least 2× headroom over its densest linear
  family.
  - Trade-off: one shared pair would be simpler, but it would double the lint
    ledger's runaway bound.
- **The per-crate `Engine` grows from 4144 to 4152 bytes**, one grant word,
  as `Report` did. It is accepted.

## Process

- Qualify a batch of related repairs with one focused gate and one short
  checkpoint under `docs/parallel/`. Add one tracker entry per batch.
- Change `spec.md` only for new normative rules, ROADMAP only at milestones.
- Fixtures stay in the earliest owning assignment; keep them minimal and declare
  outcomes before generating any oracle.
- Component publication of the ten implemented units runs in CI when lanes are
  idle; it does not gate core work.
- Commits use the repository's single-line imperative subjects without trailers.
