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
- **After Lane A's [census batch 7](parallel/census-batch7.md)** (census 8: no
  per-definition failure, one crate record at `pin.rs:1105`), 2026-11-08:
  - **Lane A owns `pin.rs:1105` next**, because it is the census frontier.
    RustGM follows rustc's sizedness hierarchy:
    - a `?Sized` associated type's implicit bound is `MetaSized`, provable
      for its rigid projection from the associated type's own item bounds;
    - `PointeeSized` holds for every type.

    Lane A keeps its `sema::traits` edits minimal and lists them.
    - Cost: Lane A edits shared selection code. Lane B's next batch (the glob
      row, builtins injection and `self-where`) is crate loading, so the
      overlap is small.
  - **`reached_intrinsics_cannot_execute_their_authored_fallback` is stale
    since batch 5.** Batch 5 gave `ctpop` a validated constant handler, and the
    property under test, that an authored fallback never runs, still holds.
    Lane A updates it:
    - its `const` case expects the handler's result: `[u8; 3]` is accepted
      and `[u8; 7]` rejected for `ctpop(7)`;
    - an intrinsic with no handler keeps the original rejection.

    Joined checks missed the stale case because they select each lane's own
    tests. From now on they also select the tests that the latest nightly
    qualification failed, until they pass.
  - **Ordinary crates keep the recorded early-lifetime-contract gap.** The
    demand stays scoped to source libraries.
    - Why: extending it renumbers 11 accepted PA20 references, and that is a
      contract change needing an independent review.
    - Cost: one rule has two orders until it is extended.

    Lane A extends it after Milestone 2, as a reviewed reference update whose
    diff must show only renumbering.
  - **The census may keep going across crate-wide declaration passes**, as
    analysis only.
    - An independent pass still runs after a failed one.
    - A pass that consumes a failed pass's output is recorded as blocked.
    - Bodies are still not entered while any declaration record fails.

    Lane A adds this only if it fits in one checkpoint. Census 8 surfaced four
    crate records one at a time.
  - Tracker head entries stay newest first.
- **After Lane B's [scoped body proofs, variant re-exports and `--extern
  core=`](parallel/scoped-proofs-shim-globs.md) batch**, 2026-11-08. Lane B
  found that only `--source-core` builds publish a version-13 source
  descriptor. The shim and the `compiler_builtins` stand-in are source clients,
  which export the legacy core table, `PublicNames` (profile tag 6) and the
  tag-14 macro wrapper. So the earlier "version-13 glob row" and
  "version-13 descriptor flag" decisions are revised:
  - **The shim's glob is a tag-14 macro table version 5 row**, kind 4 `GLOB`:
    `[u8 4][placement][u32 owner][target module path]`.
    - `owner` is 1 plus the index in the publication's sorted alias
      dependencies, as for `REEXPORT` rows; 0 is rejected.
    - A table that needs no `GLOB` row keeps its exact version 2, 3 or 4
      bytes.
    - The consumer expands the row in `import_crates`, after macro re-exports
      and before `bind_crate_aliases`. It works per name, across the type,
      value and macro namespaces.
    - The semantics follow rustc:
      - only names visible outside the target crate are expanded;
      - an explicit shim binding shadows a glob name;
      - two globs giving one name different targets are an error only when
        that name is used;
      - each expanded name is charged once.
    - Older readers reject version 5 by name.
    - Forgeries:
      - owner 0;
      - an owner past the alias list;
      - a target module the owner does not export;
      - every strict prefix.
    - Why: one row carries all three namespaces, including core's
      `#[macro_export]` macros.
    - Rejected: splitting it across `PublicNames` and tag 14. That would be
      two expansions and two proofs for one `use`.
  - **`#![compiler_builtins]` is a crate-loader flags record**, profile tag
    15: `[u8 15][u8 flags]`, with bit 0 = `compiler_builtins`.
    - It is written only when a flag is set. Unknown bits are rejected by
      name, and older readers reject the tag by name.
    - Why: rustc keeps these crate-root attributes together, and std's graph
      needs more of them. `panic_abort` is `#![panic_runtime]` and std is
      `#![needs_panic_runtime]`; those bits join this record.
    - Cost: one more profile record kind.
    - Rejected: a flags byte in the tag-14 version-5 table. That would tie
      crate-loader state to the macro table.
  - **The struct-parent rejection in the variant re-export consumer is not
    reachable yet.** The enum header check and the version-13 enum gates
    catch those forgeries first. It stays as the branch Lane C extends for
    struct constructors.
  - **Re-exported variants used as types are a recorded deviation.**
    `Some { 0: x }` gives "unresolved type name" where rustc gives E0573.
  - **Lane B's next batch:**
    1. the `GLOB` row with the controls and forgeries above;
    2. `compiler_builtins` injection with the tag-15 record;
    3. `self-where`.

    The two caller-lifetime gaps (`caller-region`, `ordinary-region`) follow
    if they are cheap.
- **Main's private tests have failed since the 2026-11-07 nightly**, 2026-11-08.
  The last good nightly was `31c3a6077`, the first bad one `054b91950`.
  Re-running its failures by exact name on main `4a9a3441f` showed:
  - 17 lib tests still fail, on layout sizes and work budgets;
  - the integration failures are still being checked.

  Joined checks missed them because they select only each lane's own tests.
  - **Lane B repairs them first**, ahead of the `GLOB` row and builtins. Each
    test gets a cause commit and either a code fix or a recorded reason for a
    justified growth. Nothing is blessed to hide a failure. The record goes
    in `parallel/nightly-private-repair.md`.
    - Why Lane B: 11 of the 17 are resolve tests, and std crate loading is
      off core's critical path.
    - Rejected: a fourth worktree. Registering one needs an idle resource
      controller.
    - Cost: std crate loading slips one batch.
  - **Joined checks now also run the latest nightly's failing tests** as a
    probe. A join passes when the probe shows no failure outside that known
    list. The integrator reads the nightly result each day.
- **After Lane A's [census batch 8](parallel/census-batch8.md)** (census 9: no
  per-definition failure, one crate record at `option.rs:1386`), 2026-11-08:
  - **Lane A owns `option.rs:1386` next.** It is `Option::as_deref`,
    returning `Option<&T::Target>` under `T: [const] Deref`, and it fails with
    "selected type retains an unmapped lifetime" in nominal conjunction
    selection. rustc accepts it.
    - Why: it is the census frontier, Lane B is on the nightly repair, and
      the `[const]` conjunction is near Lane A's constant work.
    - Cost: Lane A edits shared selection and substitution code again. It
      lists every file.
  - **The full keep-going across crate-wide passes is deferred.** It needs a
    consume/publish audit of about 29 passes. Instead, the census may run a
    labelled, analysis-only lookahead:
    - after a failed crate-wide pass, the later passes still run on whatever
      facts exist;
    - every later record is marked as following that failure and possibly
      derived from it;
    - bodies are still not entered;
    - the lookahead is never a qualifying result.

    Lane A adds it only if it takes at most half a checkpoint.
    - Why: each census has shown only one crate record. A lookahead lets the
      remaining records be shared across lanes.
    - Cost: some listed records may be artefacts of the earlier failure.
      Each is re-proved when its turn comes.
- **After Lane C's struct constructors, const inherent methods and
  raw-pointer and slice rows**, 2026-11-08:
  - **Another provider's region nominals (Lane C's old task 3) are blocked on
    multi-packet source export.** The exporter writes artifact 0 only, and a
    source core cannot import another version-13 packet. Clients of core
    already handle core's region nominals.
    - The export itself stays required. std consumes alloc's generic API, so
      alloc, as a source client, must publish a version-13 descriptor whose
      rows name core's artifacts. Core bodies and alloc's own build do not
      need it.
    - Lane C writes a design proposal
      (`parallel/multi-packet-export-proposal.md`), and the integrator
      decides the shape before any code is written.
    - Cost: std-path transport starts later. The design runs while core and
      alloc are in progress.
  - **Type recipe tags:**
    - 12 is `RawPointer`: `[u8 12][u8 mutable][id pointee]`.
    - 13 is `Slice`: `[u8 13][id element]`.
    - 14 is reserved for arrays, whose const length encoding needs its own
      proposal.
    - 15 is reserved for `dyn` trait objects.

    No sizedness travels: each consumer proves a slice element sized from its
    own facts.
    - Why now: raw pointers and slices run through core's whole API, which
      alloc consumes, not just the panic chain.
  - **The caller-location ABI and `panic_impl` wait on core's real
    `Location`**, which needs `repr(transparent)`, PhantomData's Pointee role
    and NonNull's diagnostic-item role. Lane C reports each exact shape
    before using it.
  - **Lane C's order:**
    1. the `sync` lang-item fix below;
    2. `repr(transparent)`;
    3. PhantomData's Pointee role;
    4. NonNull's diagnostic role;
    5. auto trait export (`Send` and `Sync`), which alloc's impls and every
       static need;
    6. `Location` and the caller ABI;
    7. `PanicInfo` and `panic_impl`;
    8. arrays (tag 14) with `CoerceUnsized`;
    9. the multi-packet proposal.

    Auto traits come before `Location`, because nearly every alloc type has
    an auto trait impl.
  - **Main accepts a source-client `static` without a `sync` lang item, and
    rustc rejects it.** This is a false acceptance, so it is fixed, not
    recorded: a static requires the `sync` lang item and `T: Sync`, as in
    rustc.
    - Lane C takes it first, because it holds the repro.
    - Cost: a small edit outside transport, listed for owner review.
  - **Writes through a call temporary's field stay fenced by name** (rustc
    admits them). This is a recorded conservative deviation, because core
    does not need them.
  - `#[non_exhaustive]` tuple and unit struct constructors and `struct T();`
    stay fenced by name.
- **After Lane A's [census batch 9](parallel/census-batch9.md)**, 2026-11-08.
  Census 10 has no per-definition failure. Its terminal crate record is
  `FnPtr` coherence at `ptr/mod.rs:2592`, followed by 8 lookahead records.
  - **The census frontier belongs to Lane A, whatever code it lands in.**
    Lane A therefore owns `ptr/mod.rs:2592`. Records off the frontier go to
    the lane whose area they are in.
    - Why: the frontier is the critical path, and each hand-off costs a
      batch.
    - Cost: Lane A edits shared selection and coherence code. It keeps those
      edits minimal and lists every file, and the other lanes are told.
  - **Explain why core's reference impls don't conflict.** rustc accepts core
    but rejects a reduction of its `&A` impl with E0119. The repair must
    follow rustc's actual coherence rule for `FnPtr`. A shortcut that
    happens to accept core is not enough.
  - **The nested projection case goes to Lane B's queue after the nightly
    repair.** That case is `Option<&'a <H::Inner as Deref>::Target>` with no
    `[const]` bound, and it is not on the frontier.
  - **Lane A reduces lookahead records 2, 3 and 5 now**, giving each a repro
    and an owner guess. It repairs only those that are its own, and only
    after record 1. The records are:
    - 2: `ConstParamTy`;
    - 3: conformance and selection semantics;
    - 5: a missing completed const argument.

    Records that follow record 1 wait for it.
    - Why: reduced records can be handed to idle lanes before the frontier
      reaches them.
- **During Lane C's shared-static Sync fix**, 2026-11-08:
  - **The `T: Sync` proof must not reject anything rustc accepts.** If a
    fixture that rustc accepted, or that an accepted PA1-PA24 suite expects
    to pass, flips to a rejection, the check does not land. The fixture is
    reported with its failing auto-trait goal, because the flip would be an
    incompleteness in RustGM's auto-trait proof.
  - **pa31's `pointer-bits.rs` keeps its bytes**, and its expected diagnostic
    becomes rustc's only error on it, "requires `sync` lang_item". A successor
    adds the `sync` item and witnesses the provenance rejection (rustc
    E0080). Lane A's pattern-type invariant changes only that case.

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
