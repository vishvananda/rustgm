# Core census and transport plan


Latest budget checkpoint: [PA31 query ownership](pa31-declaration-query-budget.md)
clears cell.rs's cumulative unary allowance. Compiler291e6c36c/CLI16244e10 keeps
all limits and original722 inputs/375 opens/42310 passing declarations. Bodies
report71 failures before transmute-layout budget record72 at char/convert.rs
byte993. Repair that actual query owner next; no original artifact exists.

Adopted 2026-11-07. This replaces the serial first-error probe loop as the
operational path to original core, alloc and std. It changes execution order
and process only. Assignment requirements, the [unit graph](../assignment-plan.json),
`spec.md`, source pins and the frozen CPPGM backend boundary are unchanged.
The [blocker tracker](library-build-blockers.md) still owns concrete failures.

Latest single-worker checkpoint, 2026-11-10: [scoped method lookup](pa21-scoped-method-lookup.md)
clears all original-core declaration analysis and enters bodies. On unchanged722
inputs/profile/limits,42310 declarations pass with no failed/blocked item. The
census reports67 body failures before its unary-declaration budget stop in
cell.rs:1393; no body completion or original artifact is claimed. Attribute that
budget without raising it, then batch the repeated Unsize, inclusive-range and
receiver/projection causes. Keep the single existing worker and scratch tree.

Earlier single-worker checkpoint, 2026-11-10: the projected-layout, whole-slice and
numeric/query checkpoints clear the intervening records. Original core now stops
at Split::new's nonempty quantified lookup condition. The
[PA21 call consumer](pa21-universal-call-consumer.md) closes the retained f01
generic-semantic regression first, with independently checked ordinary reductions
and unchanged original inputs/profile/limits. Extend lookup using actual scoped
goals next. No original library artifact exists; census is analysis-only.

Earlier single-worker checkpoint, 2026-11-09: record variants are integrated on
`84baefb8c`; their focused join reuses 2,022 source-identical passing selections
and freshly passes 16 affected integration cases and five native groups.
[Census batch 22](parallel/census-batch22.md) then clears Intersperse's
unresolved-projection lookup with 13 independent controls and a 2,032-input
paired survey. Census 25's new record is an unprepared storage shape for
`AtomicPtr<()>`, reached by `from_mut`'s alignment assertion at
`sync/atomic.rs:1590`. All original inputs/profile/limits are unchanged;
42,310 declarations are checked, but body checking remains unreached. Follow
this observed lead before the provisional Split lead; f01 must close before
A+O widens bound-region lookup. Whole-assignment and artifact gates are unchanged.

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
paths that need them stay fenced and are recorded, never worked around. On
2026-11-08 the user lifted the freeze for inline assembly only; see the
assembly pin decision below.

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

- **Before Lane C's transparent record headers**, 2026-11-08:
  - **`repr(transparent)` uses the nominal header's representation byte.**
    It gains 2 = Transparent, read only under version 13 and only on struct
    headers. Each consumer re-proves the representation from its own fields.
    No layout or ABI fact travels.
    - Cost: rewriting byte 2 to 0 is not detected. That is a recorded
      integrity limit: the representation is authenticated by the packet's
      inventory identity, not re-proved. A later ABI digest may close it.
  - **Pattern types get type recipe tag 16**, `Pattern { base, constraint }`.
    - Lane C reports the exact constraint encoding before using it. It must
      cover `!null` over a raw-pointer base and the integer ranges core
      uses.
    - Each consumer re-proves the pattern against its own base.
    - Integer and reference `!null` bases stay rejected, a recorded
      deviation.

- **Main's full CI on `f98c5d7c8`**, 2026-11-08 (a manual qualification run):
  35 tests fail, 29 lib and 6 integration. That is the 17 lib tests from
  `4a9a3441f` plus regressions from this session's lane work, which
  lane-selected joins did not select.
  - **The nightly repair widens to all 35**, still Lane B's task 0, in two
    checkpoints. Most failures are record or layout size asserts after
    accepted growths, so each one either takes the growth with a recorded
    reason or gets a fix. None is blessed.
  - **`shape_constructors` is a real regression.** Lane B bisected it to
    `65a9eae6c` (census batch 5): `field_template_dependency` re-walks every
    storage root after each wait. That makes it quadratic, about 1.6M steps
    for a 512-field chain.
    - Lane B makes it resumable per nominal: each root is charged once and
      waits only once, with a linear scaling invariant. The tests' bounds stay
      unchanged.
    - Why Lane B: it holds the analysis, and Lane A stays on the frontier.
      Lane A avoids `field_storage.rs` until the fix joins.
  - **Joins probe the 35 by exact name** and fail on any failure outside that
    list. `verbatim_links` cannot be selected by fastcheck, so CI is its only
    check.
  - **The integrator runs full CI after each significant integration**, not
    only nightly.
    - Why: these regressions went unnoticed for a day.
    - Cost: one qualification run per integration.
  - **Accepting a record growth means updating every assert on an enclosing
    size.** The integrator greps for those asserts before accepting.
- **During Lane C's PhantomData Pointee role**, 2026-11-08:
  - **The shape is approved.**
    - It reuses version 13's spelled role row, admitted only under
      `body_nodes()` and only for `BindingProfile::Languages`.
    - Each consumer re-proves PhantomData:
      - a module-owned unit struct;
      - one type formal with no default;
      - `T: PointeeSized`.
  - **Integrity anchor: each consumer re-runs rustc's unused-parameter rule
    (E0392) over imported nominal headers.**
    - It computes each imported formal's variance itself. A type or lifetime
      formal that comes out bivariant fails the import as a transport
      integrity failure, because the exporter would have rejected the
      declaration. Const formals are exempt, as in rustc.
    - Only the declaration bound to the re-proved PhantomData role is exempt.
      Deleting that role row therefore fails the import.
    - Why: unlike the repr byte flip, a deleted role row makes `T` bivariant
      at the consumer, and a lifetime could then be extended through the
      imported type. That is a soundness hole, not a conservative loss, so it
      cannot be recorded as a known limit.
    - Rejected: recording it as a known limit and closing it later. That
      would join a known-unsound import.
    - Cost: a variance check over each imported header's field types,
      linear in their size, with a 6/24/96 scaling invariant. If it needs a
      new variance pass that consumers lack, Lane C reports the shape and
      cost before building it.

- **After Lane A's [census batch 10](parallel/census-batch10.md)**, 2026-11-08.
  Census 11 has no per-definition failure. Its one record is a budget record:
  core's crate-wide overlap pair checks take 2,404,914 steps against fixed
  one-million caps.
  - **Lane A owns the overlap budget, as the frontier. The fixed cap is not
    raised.**
    - First, Lane A shows whether the 2.4M steps are real per-pair work or
      repeated payment. It compares our pair enumeration with rustc's.
      rustc's specialization graph checks a new impl only against children
      with the same simplified self type, plus the blanket impls, after
      `DeepRejectCtxt` fast rejection.
    - If pairs cross simplified-type buckets, or a pair or its headers is
      paid more than once, the repair is that index plus fast rejection,
      charging each pair once. A 6/24/96 scaling invariant must show no
      cross-bucket pair work.
    - Only work that remains real gets a size-based cap, as the user
      approved. The cap is derived up front from the candidate pairs the
      index admits, times each pair's header size, with the old constant
      only as a floor.
    - Rejected: a larger fixed constant. It would hide repeated payment and
      break the spec's bounded-work rule.
    - Cost: a size-based cap grows with a crate's input. Within one bucket,
      rustc has the same worst case.
  - **Lane A repairs lookahead record 2** (`ConstParamTy_` under
    `#[unstable_feature_bound]`, `mem/transmutability.rs:91`) after the
    budget and after the next record behind it, "specialization implication
    still awaits a declaration dependency". It is genuine, in Lane A's
    parameter-kind code, and next on the frontier. No other lane is in
    `nominals/traits.rs`.
  - **Lane A also owns the `[const]` supertrait gap**, queued behind the
    frontier. That gap is `&F: [const] Fn` not proving its `[const]` FnMut
    and FnOnce supertraits.
    - Why Lane A: it continues the batch 9 `[const]` premise work.
    - Its three control cases stay listed as gaps in the invariant.

- **During Lane B's [nightly repair](parallel/nightly-private-repair.md)**,
  2026-11-08. CI on `71f4bc1e9` still had the same 35 failures, so the two
  earlier joins added none. Lane B bisected every one. 33 pass after its two
  checkpoints, and the other 2 are routed.
  - **The crate-wide `prepare` failure cache is restored** for `prepare` and
    `prepare_equality`. Census batch 5's `missing.is_none()` had stopped it
    caching misses caused by an unpublished template. That re-walked roots
    and let a later `prepare` succeed.
    - The non-caching storage path never cached, so batch 5's rule ("such a
      miss is never cached") holds without the condition.
    - Batch 5's invariant and its 27 PA31 controls pass with the cache
      restored. Lane A's census 12 on a main that contains the repair is the
      whole-core check, so no separate release-pin census is run.
  - **The storage walk resumes at the nominal it waited for** and charges each
    visited type once. That fixes `shape_constructors`, where batch 5 was
    quadratic: 1.6M steps for 512 constants, now 27n. The tests' bounds are
    unchanged.
  - **Amended controls, each with its cause commit:**
    - `ctfix` mirrors Lane A's reviewed `e2c0ead9e`, because `ctpop` gained
      a constant handler. Lane B took it rather than routing it back, since
      it is a one-test mirror.
    - Seven `Declarations` asserts take `c6b5642dc`'s 8-byte
      `pending_copy_selection`, found by grepping every spelling of the
      enclosing size.
    - The `Imports` assert takes `c5d056327`'s 32-byte `deferred` field.
  - **`path_compatibility` was an accounting regression, not growth.** Since
    `c5d056327`, every warm feature lookup charged its spelling's retained
    bytes (128 × 21 = 2,688). Now only the deferred use that keeps a spelling
    pays for it, and a new assertion checks that it still does.
  - **Routed to their owners:**
    - `template_demands` goes to Lane C. It fails since `2b0b5cf07`: late
      region free originals now transport, so the control's opponent no
      longer reaches its boundary. Lane C amends the opponent and keeps the
      cost assertions.
    - `abi::storage_relation` goes to Lane A's batch 11. Since
      `f9a2c96d4`, a missing-`Copy` opponent exhausts selection work instead
      of reporting the `Copy` error. It may share a cause with census 11's
      overlap budget.
  - **Lane B's next work:** the two quadratic hazards it found but did not
    fix, then the nested projection case, then the caller-region task. The
    hazards are `prepare_success_rows`' exact reservation and
    `lifetime_contract_dependency` re-scanning the unreached nominals after
    each wait.
    - Why first: the core census runs field storage at whole-crate width, so
      a quadratic walk there becomes a budget record later.

- **During Lane C's [PhantomData roles](parallel/phantom-roles.md)**,
  2026-11-08:
  - **The shape is approved.**
    - It reuses version 13's spelled role row, admitted only under
      `body_nodes()` and only for the languages binding profile.
    - Each consumer re-proves PhantomData:
      - a module-owned unit struct;
      - one type formal with no default;
      - `T: PointeeSized`.
  - **Integrity anchor: rustc's unused-parameter rule (E0392), re-run on
    imports.**
    - The consumer's existing variance pass already covers imported nominals
      and rejects bivariant formals. A bivariant formal on a source-free
      imported nominal now reports an integrity failure: the packet is
      forged or corrupted.
    - Only the declaration bound to the re-proved role is exempt, so deleting
      or moving the role row fails the import.
    - Why: unlike the repr byte flip, a deleted role row makes `T` bivariant
      at the consumer. A lifetime could then be extended through the
      imported type, which is a soundness hole, not a conservative loss.
    - Rejected: recording the deletion as a known limit, which would join a
      known-unsound import.
    - Cost: one error arm in the existing linear variance pass. No new pass,
      row or wire byte.
  - **The moved-row forgery targets a zero-formal `pub struct Unit;`.** An
    honest packet cannot carry a second unit struct with one formal: a type
    or lifetime formal would be bivariant and rejected by the provider, and
    const arguments do not transport.
  - **A false rejection on main is fixed in this slice.** A consumer could not
    lower a unit value of an imported type-generic nominal, such as
    `base::Opt::<u8>::None`, because imports set the nominal-requirement flag
    only for lifetime nominals.
    - The rule is now the local one: any lifetime or type formal, or an own
      clause.
    - 969 baseline-to-candidate runs over every PA29 provider and client
      differ only in the new phantom-roles packet. That is the evidence that
      the extra bound proofs reject nothing.
    - Work is linear: exactly 23 steps per enum at 6, 24 and 96.
  - **`template_demands` is amended, not weakened.** `2b0b5cf07`
    intentionally admits late-region free originals, and rustc accepts the
    old opponent. No source shape still reaches that catalog boundary: nine
    shapes were surveyed, and each is either accepted or rejected earlier.
    The test now checks the acceptance path's costs. Its failed-attempt and
    peak assertions are unchanged.

- **During Lane A's [census batch 11](parallel/census-batch11.md)**,
  2026-11-08.
  - **Census 11's overlap budget record was paid but unnecessary work, not
    real work. The fixed cap stays.** Read-only measurement found:
    - core has 22,895 pairs, the index matches rustc's buckets, and no pair
      is checked twice;
    - 41% of the 2,404,914 steps went to 20,986 pairs whose trait references
      cannot unify, because both headers were instantiated before a
      structural reject;
    - 54% were allocation bytes charged as steps, with the left side
      recounted on regrowth;
    - about 175k steps were real work.
  - **The fixes:**
    - a deep reject before header instantiation, as rustc's `DeepRejectCtxt`
      does. Params, inference variables, projections, aliases, opaques and
      placeholders match anything;
    - reservations charge one step per condition, and only their new bytes,
      in the byte ledger.
    - Result: 263,268 steps, under the unchanged one-million constant.
    - Rejected: a size-based cap, since no real work needs one.
  - **A specializing child's parent obligations are solved to a fixpoint**,
    and leftover ambiguity means "does not specialize", as rustc's
    `specializes` does. This repairs core's `SpecExtend` pair, whose `T`
    appears only in where clauses.
  - **A const parameter's type is proved under its owner's environment**, as
    rustc's `check_param_wf` does. Each const parameter gets its own proof,
    numbered after the last retained one.
    - The first pin reused one proof per owner. Its lifetime witnesses
      collided at core's `Arguments::new<const N, const M>`.
    - Five PA31 controls, declared before the oracle, agree with rustc.
  - **`f9a2c96d4`'s selection regression is fixed.** A pattern marker whose
    base lacks `Copy` relisted impl candidates until selection work ran out.
    Main's last known failing test, `abi::storage_relation`, passes.
  - **Census 12** has no per-definition failure. Its crate record is "minimal
    specialization introduces a non-marker predicate" at core's
    `SpecNextChunk` pair (`array/mod.rs:1044` and `1062`).
    - rustc stores a `[const] Tr` bound in `predicates_of` as the ordinary
      `Tr` clause; constness goes to `const_conditions`, which
      `check_predicates` does not compare.
    - Lane A repairs it next in `sema/specialization/predicate_basis.rs`,
      then the `[const]` supertrait gap, which may share the
      `const_conditions` split.

- **During Lane B's [linear waits](parallel/linear-waits.md)**, 2026-11-08.
  Three quadratic walks it found during the nightly repair are now linear:
  - Storage completions' dense success rows grow geometrically. At 512
    completions they cost 4,288 steps, against 145,153 before.
  - A declaration-time body's lifetime-contract demand resumes at the
    nominal and closure owner it waited for: 54 steps per nominal, against
    about 20n² when restarted.
    - This also fixed a correctness bug. The rescan dropped nominals that
      completed before a later wait, so their outlives contracts were never
      demanded, and core-like code rustc accepts failed with "missing
      nominal outlives facts".
    - Three PA31 controls, declared before the oracle, agree with rustc.
  - **Fixed in the same checkpoint, not queued:** the explicit-bound index.
    Demanded outlives contracts regrouped every explicit bound in the crate
    per newly demanded nominal. Each bound is now indexed once, costing
    nominals plus bounds; the restarted control costs 1.3M at 512 × 512.
    - Why now: core has thousands of explicit bounds, and the census runs at
      whole-crate width.
    - Cost: `Declarations` grows by one pointer. The join found that its
      seven sum-spelled size asserts had been missed again, so every
      `size_of::<Declarations>()` test is now selected.
  - **Recorded known limit, queued after Lane B's task 3:** parked variance
    graphs are charged conservative growth estimates, 2.16 KB of the 3.77 KB
    a chained nominal attempts against a 2.94 KB allowance. A body with about
    200 or more waiting nominals then exceeds the input-graph bound.
    - The fix: charge the parked graph's actual retained bytes.
    - It moves ahead if a census or control hits it.

- **During Lane A's [census batch 12](parallel/census-batch12.md)**,
  2026-11-08.
  - **Census 12's record and the `[const]` supertrait gap share one cause:
    rustc's split between `predicates_of` and `const_conditions`.**
    - rustc stores a `[const] Tr` bound in `predicates_of` as the ordinary
      `Tr` clause, and its constness only in `const_conditions`.
    - min_specialization's predicate basis now compares clauses that way.
      A `const Tr` is kept as `Tr` plus the host-effect clause it stands for.
      Only an ordinary trait clause gets the marker allowance.
    - A const impl's supertraits and where clauses are proved as `[const]`
      goals, as rustc's wfcheck does. Before, they were `const` goals, which
      the impl's `[const]` premises cannot prove.
    - Constness is still checked where rustc checks it. Children that are
      not const, or only weakly const, are still rejected. A non-const
      supertrait impl under a const impl stays rejected.
    - 14 PA31 controls, declared before the oracle, agree with rustc. Batch
      10's three callable-sugar gaps now pass.
  - **Census 13** has no per-definition failure. Its crate record is core's
    `impl Copy for ! {}` (`marker.rs:484`): the Copy-target check has no arm
    for the never type, which rustc's `type_allowed_to_implement_copy`
    accepts.
    - Lane A repairs it next, checking rustc's whole accepted set rather than
      adding only `!`. Then it re-checks lookahead record 2 (`SpecFill for
      [MaybeUninit<T>]`) for derivation.

- **During Lane C's [NonNull and pattern types](parallel/nonnull-patterns.md)
  slice (item 4)**, 2026-11-08.
  - **Wire shape.** A pattern type is type recipe tag 16: a base recipe, a
    kind byte and its operands.
    - Kind 0 is `!null`, over a raw pointer base only. The exporter fences a
      `!null` over a type parameter by name; no current export reaches one,
      including original core's `pat.rs` `DispatchFromDyn` header.
    - Kind 1 is a closed inclusive interval of raw bits, as two `u128`s. A
      wrapped signed range is split into its connected alternatives and
      re-interned. Bits beyond the base width are rejected by name.
    - Every consumer re-proves the pattern through its own interning.
    - Rejected: endpoints sized to the base width. They save under 1 KB and
      need width dispatch.
  - **Integrity limit, same class as the transparent flip.** Rewriting a
    pattern to its bare base changes ABI (`Option<NonNull<T>>` is 8 bytes
    across crates, 16 without the pattern). Constant endpoints travel as
    provider-evaluated bits. Both are authenticated by the packet's identity,
    not re-derived.
  - **`NonNull`'s diagnostic identity** is role row 254, spelled from a
    closed table. Each new spelling needs its own approval.
    - Rejected: a separate diagnostics table, which would add a column to
      every version 13 packet.
    - Versions 10 and 12 fence the identity by name, as with `PhantomData`.
      The identity run (30 packets, 1,002 runs) flips no existing version 10
      or 12 outcome.
    - `rustc_nonnull_optimization_guaranteed` has no byte. Nothing consumes
      it, and the guarantee follows from the `!null` field, which runs
      natively.
  - **Declared-outcome amendments**, each recorded before its run:
    - Four range clients that rustc accepts (`nanoseconds`, `nonzero-char`,
      `code-point`, `percent`) hit the local fence "scalar pattern enum
      niches require their complete ABI profile" (`niches.rs:316`), the same
      way in one crate. They are pending cases; the LowIR equality is the
      transport claim.
    - `holder`: `Opt<H>` over a non-transparent `H(NonNull<u8>)` is 16x8 in
      RustGM and 8x8 in rustc. The nullable-carrier walk descends only
      `repr(transparent)`, rustc's guaranteed null-pointer optimization,
      while rustc also uses unguaranteed niches through ordinary struct
      fields. The oracle keeps 8x8; 16x8 is a separate divergence, pinned so
      a layout change surfaces. Rejected: field-niche placement inside this
      slice, since it changes every aggregate layout and needs its own
      declared oracle sweep. Also rejected: fencing `holder`, which would
      reject valid code.
    - Five layout witnesses check the range types through a function
      pointer, all at rustc's size. Their single-crate control waits on
      fn-item-to-fn-pointer reification, which has no version 13 body row.
      Closed by R1 (below).
  - **Open items.** "Niches: scalar ranges and placement through
    non-transparent struct fields" is frontend layout work (the backend
    stays frozen), with an owner assigned when bodies reach it.
    - Risk: `size_of` diverges silently wherever a runtime size is compared
      with rustc's. Core's const size asserts would fail at const-eval; the
      pinned core has none over such a struct.
    - Lane C's queue gains item 5b after auto traits (module const items,
      associated consts and the `RangePattern` roles), then fn-item
      reification. `Location` and the caller ABI move one slice later.

- **During Lane A's [census batch 13](parallel/census-batch13.md)**,
  2026-11-08.
  - **Copy impl targets follow rustc's `type_allowed_to_implement_copy`.**
    - Accepted without fields: scalars, `!`, raw pointers, `&T` and arrays,
      with no element check. Core's `impl Copy for !` passes.
    - Rejected: `str` and `&mut T`.
    - Written Copy impls for tuples and fn pointers are recorded gaps.
      RustGM still accepts them; rustc rejects them with E0206, because it
      provides those impls itself.
    - All 13 controls were declared before the oracle and agree with rustc.
  - **Three superseded PA31 declarations recorded rustc outcomes that rustc
    never gave.** A rustc 1.98.0 run showed this for pointer-composition,
    source-format-macros/arguments and unadjusted-request: each records
    acceptance of a written tuple or fn-pointer Copy impl. No test reads
    them, and each packet's active successor was already witnessed.
    - The superseded bytes stay frozen. They are pre-registration history,
      and successors link to them.
    - Each packet gets an additive `reference-corrections.json` with the
      witnessed status, codes, shas and options, and its active successor.
    - Rejected: correcting the files in place. It would blur "frozen before
      observation" and risk the link hashes.
    - Unrelated errors in the same superseded bytes (E0204, E0508, E0600),
      and one classification that names only some of the observed errors,
      get no further action. The correction records state them, and the
      successors no longer have them.
  - **Lookahead record 2 (`SpecFill`) is derived.** Specializing children are
    ordered behind the impl that fails, and the owner pass reports the
    first unvalidated child. No repair is needed.
  - **Census 14** (pin `188a5350`): 42,310 declarations, none failed or
    blocked. Its crate record is core's `const impl<T> Try for
    NeverShortCircuit<T>` (`ops/try_trait.rs:433`).
    - The reduction is an ordinary `impl FromResidual for Short` that leaves
      the argument at its default, `<Self as Try>::Residual`. Writing the
      argument out passes. rustc substitutes the default and normalizes the
      impl's trait reference before matching.
    - Lane A repairs it next: a defaulted impl trait argument is checked as
      if it were written. It first locates the divergence.

- **During Lane C's [auto traits](parallel/auto-traits.md) slice (item 5,
  C1)**, 2026-11-08.
  - **Wire shape.** There is no new tag or bit.
    - The trait header's `AUTO` and `UNSAFE` flags are admitted only for an
      exact empty auto declaration. A role-bound marker keeps its local
      role's exact flags. `CONST` stays rejected.
    - `Send`'s diagnostic identity is role row 254's second closed spelling,
      installed in the consumer's own `Send` slot.
    - UnwindSafe and RefUnwindSafe get no identity (Q2).
    - Why a second 254 spelling rather than treating imported `Send` as a
      custom auto trait: one role path (object markers, the census
      classification, `source_auto_kind`) for a local and an imported core.
      Each new spelling of the closed table needs its own approval; this is
      `Send`'s.
    - A client's own `#[lang = "sync"]` or `Send` diagnostic beside an
      imported one is rejected by name before any catalog lookup (rustc's
      E0152), never by whichever catalog answers first.
  - **Item 5's scope**, decided before C1: three commits. C1 covers
    declarations and roles, C2 auto impls and the structural selection lift,
    and C3 non-auto negative impls and plain unsafe traits and impls.
    - Non-auto negatives stay in item 5, since whole-packet validation means
      core cannot export without them.
    - Auto impls of a trait from another artifact are fenced by name until
      item 9.
    - Specialized positive auto impls follow rustc's root-type rule: an impl
      for `Foo<u8>` leaves `Foo<u16>` without an auto candidate.
  - **Auto-marker accessor.** `Catalog::auto_marker` answers only the five
    markers, in providers and clients. `language()` is not widened.
    - It is used at four functions in `selection/search/freezing.rs`, at
      `validation.rs::require_shared_static_sync`, and at the body evidence
      consumer `roles/freezing.rs::predicate`. The last was found while the
      declared clients were being made to pass, and was approved.
    - Every remaining `language(...)` caller for a marker is either
      provider-only by design or deferred to C2 (`freeze_impls` in
      `validate_source_impl_policy`), as listed in the slice document.
  - **Payer.** A client's marker bound is paid by the installed header budget
    (`charge_source_headers`), as `classify_waiting_materialization` already
    splits `Source` and `Imported` origins. A source client never has the
    completed local header producer that `source_payments` requires.
  - **Flip survey.** Across 31 pa29 providers, 93 exports and 1,035 runs,
    only the new packet changes, all rejected→accepted as rustc accepts. The
    30 existing packets, each of which exports Freeze, are byte-identical.
  - **Recorded open items, unchanged:**
    - the same-packet same-head coherence pair limit (`contract_completion.rs`,
      Lane C);
    - `may_dangle` (frontend dropck; it blocks alloc, not core);
    - C3's `&T` negative-coherence local limit (selection);
    - `pub const` export, which returns with item 5b.

- **During Lane A's [census batch 14](parallel/census-batch14.md)**,
  2026-11-08.
  - **A defaulted impl trait argument is normalized like a written one.**
    - Its normal form is kept on the pending impl trait reference, keyed by
      impl and argument index, beside the default, as written arguments keep
      `facts.unreduced`. One normalization point before
      `publish_trait_header` uses the routine written arguments use, with no
      defaulted-only branch. The keep-as-written rule for headers that name
      their own `Self` applies unchanged.
    - Rejected: a synthetic source node for the default.
    - Every cache keyed by source node was checked against the new key: an
      argument root never reaches a node-keyed table.
  - **A default naming another trait's member is a recorded gap.** rustc
    accepts it; RustGM rejects it by name, "associated type requires an
    unambiguous implementation". The default's alias requirement is
    instantiated from the path's arguments without the trait's `Self`.
    - Taken now as (b): the repair is committed with the gap declared.
    - Approved for batch 15 item 2 as (a): trait-path defaults are
      instantiated with `Self` set to the path's subject through one shared
      substitution, for impl trait references, bound trait paths and
      qualified projection prefixes, with controls for each caller and a
      locate report first. The report explains why core's `Try:
      FromResidual` supertrait hid the divergence.
  - **Census 15's Flatten record was proved uncovered before qualifying.**
    Census 14's pin over core with `FromResidual`'s argument written out gives
    exactly census 15's records.
  - **Census 15** (pin `6df12612`): 42,310 declarations, none failed or
    blocked. Its crate record is core's `unsafe impl<I> TrustedLen for
    Flatten<I>` (`iter/adapters/flatten.rs:313`), "invalid canonical trait
    selection". It is batch 15's first item, before (a).

- **During Lane C's [auto traits](parallel/auto-traits.md) slice (item 5,
  C2)**, 2026-11-08.
  - **Wire shape.** There is no new tag, bit or row. An impl header's
    `unsafe` and `negative` bytes are admitted only when its trait
    application names an exact empty `AUTO` trait of the same packet.
    `constant` stays rejected, and so do unsafe or negative impls of any
    other trait, until C3.
    - Why same-packet only: the multi-packet profile (item 9) owns impls of
      another artifact's traits. A foreign auto-trait impl is fenced by name,
      "imported foreign auto-trait impl requires its multi-packet profile".
    - Admission is not conformance. Each consumer re-proves safety
      (`unsafe == trait UNSAFE && !negative`), empty members, the auto Self
      rule, and a negative nominal impl's coverage, then completes the impl as
      `Validated` or `NegativeValidated` by its polarity.
  - **Consumer checks that admission keeps unreachable** stay as defense in
    depth. Each one is either tested by a direct call on an honest graph or
    listed in the slice document as unreachable and untested. Two checks were
    subsumed by earlier authentication and removed: a polarity comparison
    (the registry re-reads the same row) and an auto-flag comparison (owner
    authentication compares it).
  - **Non-ADT negative coverage is trusted to the provider.** A same-packet
    `AUTO` trait means the provider compiled with `auto_traits`, under which
    rustc skips it, and primitive impls are provider-only.
  - **New gap, item C2b, owned by Lane C.** It follows C3 and precedes 5b.
    `nominals::requirements` reads only installed predicates, so a client
    accepts `fn hold(_r: base::Rc<u8, NotAlloc>)`, where rustc gives E0277.
    C2b adds an imported nominal's transported where-clauses to its
    formation requirements, paid from the installed header budget as C1's
    marker bounds are. It removes C2's temporary
    `negative.rs::imported_nominal_inputs`. Its controls are rustc's E0277
    opponent, a positive with a satisfying argument, and an unchanged
    source-local nominal. Closed by C2b (below).
  - **Silent fallback removed.** `freezing.rs::object_marker` and
    `has_source` propagate an `auto_marker` integrity error. They never
    degrade it to "not a marker".
  - **A latent shared-folder race was checked for.** No two concurrent tests
    on main share a packet or helper output folder. The one race found was
    C2's own new labels, and it was fixed before commit.

- **During Lane A's [census batch 15](parallel/census-batch15.md)**,
  2026-11-08.
  - **Census 15's label was a fold.** Its cause is "type annotations
    needed": a proved equality whose normal form is a rigid projection left
    the impl's result-only parameter unbound.
  - **A proved equality's result is installed as a normalization installs
    one.** `equate_proved_predicate`'s projection branch marks the normal
    form's rigid roots through `apply_normalization`'s own
    `retain_opaque_results`, whose visibility widens. A second opaque-marking
    path was rejected, so that both installs stay identical.
    - Only proved roots are marked. A result projection with an unresolved
      input still blocks; no surface program reaches it, so a private
      equations regression guards it.
    - A later result for the same hole proves equality and never rebinds it.
    - The marking pays the counters `apply_normalization` pays. The
      trait-input and builtin-subject callers are unchanged.
  - **The fold keeps one text for distinct causes.** Decided: Lane A's
    follow-up after (a). The fold will keep ambiguity, internal region
    invariants and invalid canonical selections apart. Lane B found the second
    cause, "selected type retains an unmapped lifetime".
  - **Census 16** (pin `ed88ef69`): 42,310 declarations, none failed or
    blocked. Its crate record is core's `CoerceUnsized` impl for pattern types
    over raw pointers (`pat.rs:80`), "source CoerceUnsized implementation does
    not retain a permitted pointer coercion". Next: (a)'s locate report.
  - **(a): a trait path's omitted defaults take its subject for `Self`**
    (`d97f986eb`, on main `078d3f238`).
    - One substitution is shared by the default's normal and unreduced folds
      and by its alias contract expansion. A nested default's dependency maps
      its subject as it maps its arguments.
    - `applied_trait` only interns, so a written `Self` stays the enclosing
      trait's own. Decided with the coordinator: both effects go in one
      commit, because either half alone leaves the tree wrong in a new way.
    - The caller audit found no caller that needs `Self` mapped in a
      non-default argument.
    - Subject-bearing folds are not cached, and each use pays the
      substitution account in proportion to its fold.
    - Recorded, not changed: when a trait header demands a default first,
      the header phase gives it no contract. This is why core's 1,448 default
      uses, `Try: FromResidual` among them, never showed the gap.
  - **Census 17** (pin `c020dad2`) is byte-identical to census 16. Next: the
    fold follow-up.
  - **The fold keeps its cause's kind** (on main `347373f7f`).
    - Ambiguity and the unmapped-lifetime invariant keep their own texts.
      Exhaustion keeps its sticky wording, and every other cause keeps the
      caller's invalid text.
    - Decided with the coordinator: messages are classified by equality with
      a shared constant that their single producer reads. A typed `Failure`
      kind would touch every selection producer, for two kinds no surface
      program reaches today.
    - The fixed-message folds that are not `fold_cause` sites are recorded,
      not changed.
  - **Census 18** (pin `b5bcfc0f`) is byte-identical to census 17.

- **During Lane C's [auto traits](parallel/auto-traits.md) slice (item 5,
  C3)**, 2026-11-08.
  - **Wire shape.** There is no new tag, bit or row. Version 13 admits
    `UNSAFE` on a trait that is not auto, and an impl's `unsafe` and
    `negative` bytes for every trait impl, on one fact,
    `Bindings::languages`. `constant` stays rejected. The ledger's
    zero-member rule for negative impls applies in every version.
    - Why: core's negative impls of traits with members (`!Clone for &mut T`,
      `!DerefMut for &T`, `!Iterator for [T]`) and its unsafe traits
      (`TrustedLen`, `TrustedFused`, `Searcher`, `SliceIndex`) need it.
    - Admission is not conformance. Each consumer re-proves every imported
      impl's safety (`required_unsafe = trait.UNSAFE && !negative`), and
      completes a non-auto negative as `NegativeValidated` with no further
      proof, as rustc does.
  - **Four subsumed checks were removed**, among them the Q5 foreign
    auto-impl fence in admission and consumer: the ledger rejects a foreign
    trait application first.
  - **C3b follows** (Lane C, before C2b): absence and disjointness relative
    to the crate whose coherence is proved, so a consumer re-proves an
    imported same-head pair from its packet's perspective. This replaces the
    recorded "same-packet same-head coherence pair limit", which named the
    wrong path (`contract_completion.rs` is the Provided-contracts consumer
    only).
  - **Recorded open items:**
    - a trait-impl method under an impl that captures a lifetime fails in the
      exporter (core's `DerefMut for &mut T`, `Error for &E`): a core
      transport blocker, unowned, after C3b;
    - `may_dangle` (frontend dropck; it blocks alloc, not core): attribute
      resolution, the unsafe-Drop safety rule and the dropck opt-out;
    - C3's `&T` negative-coherence local limit is narrowed: `pay_negative`
      pays only Sized premises, so a `?Sized` negative impl's MetaSized
      premise is never paid (Lane B, after chain-bytes; not a core blocker);
    - `pub const` export, which returns with item 5b.

- **The user directed a backend pin refresh for assembly**, 2026-11-08: "cppgm
  extended now supports assembly so if you update the pin it should unblock
  assembly".
  - **Scope.** This lifts the 2026-11-07 freeze for inline assembly only.
    f16/f128 and every other backend extension stay frozen until the user
    raises them.
  - **Owner.** Lane D (`rustgm-backend-work`) refreshes the pin, then lands the
    first Rust slice: stage B of cppgm-extended's inline-assembly plan, the
    original `_mm_stream_ps`. Each is its own commit, and the slice's shape is
    approved by the integrator before its code is written.
  - **Pin.** cppgm-extended `17b024f69` is the newest main commit whose Tests
    run is green. The new pin gets a new lock and a new `backend-pins-v3`
    distribution. The v1 and v2 releases and their pins are unchanged.
  - **Order.** The refresh is qualified before any frontend change. Its
    regression criterion is native output equality with the old pin on the
    existing native suites. Any changed native outcome stops the lane.
  - **Rejected: pinning the active checkout's tools.** The `dev/` binaries
    in `~/cppgm-extended` predate the assembly commits, and that checkout
    must not be built in. The tools are built in separate clones instead.
  - **Rejected: a frontend-only change.** Accepting assembly without real
    native emission would advertise a capability the pin does not provide.
    An ordinary vector store is not a substitute for `movntps`.

- **During Lane A's [census batch 16](parallel/census-batch16.md)** (item
  C2c, moved from Lane C), 2026-11-08, on main `bfbf43049`.
  - **A pair of pattern types coerces as rustc decides it.** One identical
    pattern, then an ordinary `base_a: CoerceUnsized<base_b>` obligation in
    the impl's environment, the goal a nominal impl's changed field proves.
    Core's `pat.rs:80` impl passes, and so does core's `NonNull` shape in
    its control.
    - Decided with the coordinator: no `DispatchFromDyn` change. Core's
      pattern `DispatchFromDyn` impl (`pat.rs:85`, a type parameter base)
      and `NonNull`'s have passed since batch 6, under rustc's rule of one
      identical pattern and no base obligation.
    - Fenced, because no core declaration needs them: general arrays and
      `CoerceUnsized`, and executing a pattern-type coercion.
    - Stricter than rustc by name, accepted for now: a non-null pattern over
      a reference or over a pattern type, and a body that executes the
      coercion.
  - **16 controls** in PA31, which has no graded manifest, were declared
    before the rustc 1.98.0 oracle; all statuses agree.
  - **Recorded, not changed:** `Head::Pattern` keys by constraint only, so
    each pair of a pattern impl and a consumer goal costs about 20 steps.
    Core has one pattern `CoerceUnsized` impl.
  - **Census 19** (pin `480f4169`): 42,310 declarations, none failed or
    blocked. Records 1 and 2, `ZipImpl` among them, are replaced by core's
    `impl<'a, P> FusedIterator for RSplit<'a, P> where P: Pattern<Searcher<'a>: ReverseSearcher<'a>>`
    (`str/iter.rs:569`), "trait implementation does not satisfy the trait's
    requirements". Records 3 to 8 are unchanged.

- **During Lane C's [auto traits](parallel/auto-traits.md) slice (item 5,
  C3b)**, 2026-11-08.
  - **Rule.** Coherence proves absence for the crate whose coherence it
    proves. This is rustc 1.98's `trait_ref_is_knowable`, and there is no
    wire change.
    - A pair imported from one packet is proved from that packet, over what
      it could see. The consumer's own impls are never candidates there.
    - Every other pair is proved for the consumer, including a pair across
      two packets, until multi-packet.
    - Absence is knowable when no downstream crate could implement the goal
      and that crate owns the trait or an input type, after `&`, `&mut` and
      `#[fundamental]` nominals.
    - Why: core's `impl<T: Error + ?Sized> Error for &T` beside
      `impl !Error for &str` exports but was rejected by every consumer.
      Clients' impls of an imported trait for their own types were rejected
      too: `impl base::Error for &Local`, or a blanket over `T: base::Copy`
      beside a local Self.
  - **The C3 pair control flips to accepted**, and the C3 entry's C3b item
    closes. Opponents stay rejected, each at its own impl.
  - **Correction found by the candidate.** Coherence never read an imported
    impl's where-clauses: the consumer re-proved imported pairs with no
    conditions at all, not, as C3's Correction A said, with conditions it
    could not close. Coherence now reads the same imported original inputs
    selection does. Generated controls check a condition that holds, and
    `?Sized` conditions under negative coherence.
  - **Recorded open items:**
    - conservative limits: cross-packet pairs are proved for the consumer;
      `dyn` and foreign types are never owned inputs; an imported
      `#[fundamental]` nominal is never peeled, since a packet cannot carry
      one;
    - an imported pair's failure is unlocated (`input:`), so a dishonest pair
      in a forged packet would still be reported that way: a
      diagnostic-quality item, not fixed in C3b;
    - Lane B's MetaSized premise item follows in `overlap/inputs.rs`, rebased
      on C3b; `negative-metasized-param` is its parametric repro with
      imported conditions.

- **During Lane C's [auto traits](parallel/auto-traits.md) slice (item 5,
  C2b)**, 2026-11-09.
  - **Rule.** A client forms each imported nominal it names under that
    nominal's transported where-clauses. There is no wire change.
    - The requirements are its original inputs, declared and implied,
      substituted at the instance's arguments, with outlives omitted. They
      are reported with the source twin's text and location.
    - Why: `fn hold(_r: base::Rc<u8, NotAlloc>)` was accepted against
      rustc's E0277 (found in C2).
    - Payment: an uncharged walk pays the installed header budget exactly
      its attempted substitution, once per instance. A charged walk pays
      only its own closure. Cache hits pay as source hits on both paths,
      whichever path filled them.
  - **C2's workaround is removed.** Negative coverage reads
    `nominals::requirements` for every nominal, and the C2 entry's C2b item
    closes.
  - **Survey.** Only the new packet's seven imported opponents flip, from
    accepted to rejected. Every other run is byte-identical, including C2's
    `auto-impls` packet.
  - **Recorded:**
    - pa29 has no `manifest.json` yet, and the packet's outcomes file lists
      every input with its sha256;
    - the baseline already enforced an imported nominal's Sized edges
      (`slot-unsized`), so only where-clauses were missing.

- **During Lane C's [fn-item reification](parallel/fn-reification.md) (R1)**,
  2026-11-09.
  - **Wire shape.** Value code 23 is `(23, [Value(item)])`, a preceding
    code-16 Function value. Its own type is the target `Function` row.
    - No coercion decision travels. Each consumer re-proves the item's
      applied signature against the pointer, as a client's own `let f: fn(..)
      = item;` does: equal convention and variadicness, and `unsafe` added
      but never removed.
    - A pointer over a type parameter requests its formation.
    - Existing packets keep their bytes.
  - **Scope, accepted before code.** Closed Rust fn pointers only:
    - higher-ranked and C targets keep their type-row text;
    - closures stop at construction first, and their defensive fence stays;
    - tracked functions wait for the caller-location ABI;
    - constructors take Q3's Branch B. The declared probe's single-crate
      controls stopped at R2's indirect-call fence, so R2 re-probes them.
      R2's re-probe passed all four steps and retired the fence.
  - **Amended before its run:** the nonnull-patterns layout witness's
    single-crate control now builds, and its declared LowIR equality is
    checked.
  - **Survey.** Against the C2b pin, only the new packet differs.
  - **Recorded:**
    - the fmt chain's next dependencies, each still fenced:
      - the transmute intrinsic transport;
      - higher-ranked anonymous callables, whose witness is `argument_new!`'s
        `fn(&T, &mut Formatter<'_>) -> Result`;
      - R2, for calls through a pointer, which R2 now exports;
    - the target-feature note for Lane D's F;
    - explicit `item as fn(..)` casts stay fenced by the scalar-cast text;
    - order after R1: 5b-1, developed on K's join commit and qualified once
      K is on origin/main, then R2. If K needs an amendment, R2 goes first.

- **During Lane A's [census batch 17](parallel/census-batch17.md)**,
  2026-11-08 to 2026-11-09, declared on main `952781b14`, qualified on
  `dcb956cf8` and censused on `28e2b19fb`.
  - **A certified premise relates a rigid projection's inputs invariantly**,
    the approved option B. The relation applies only when the premise's own
    environment proved it normal: its owner's declared bounds after their
    normalization completed. It applies only to a goal side that a complete
    input rewrite marked rigid, and only under four layout gates. The
    lifetimes become region equalities for the region proof. Every gate
    failure keeps the exact compare's result. Core's `RSplit`
    `FusedIterator` impl (`str/iter.rs:569`) now passes.
  - **Certificates.** They are keyed by the clause value and carried only
    through lifetime-only constructions: the regions-only fold, a body's
    defining clauses, and the body's trait assumptions. Any type
    substitution and every other site drops them. Three sites are
    conservative-uncertified by name.
  - **19 PA21 controls** were declared before the rustc 1.98.0 oracle; all
    statuses agree. Eight flip to accept as declared. `hrtb-required`'s
    text differs from rustc's E0277 (amendment 2), and
    `raw-environment-premise` stays stricter than rustc.
  - **A wrong explanation, recorded.** Record 6's change was first
    attributed to rigid marks leaking through the opaque map. E0, a declared
    diagnostic pin that skipped only census 19's record 1 impl on the clean
    base, disproved that. The `rigid` map stays, justified only by gate 1's
    reads.
  - **Census 20** (pin `6a699887`): 42,310 declarations, none failed or
    blocked. `str/iter.rs:569` passes. Record 1 moves to the same
    expansion's `DoubleEndedIterator for RSplit` (`str/iter.rs:595`), and
    record 2 follows it. Record 6 changes text and is classified as a
    lookahead over a partial validated set. Records 3, 4, 5, 7 and 8 are
    unchanged.
  - **Batch 18's leads:** certify elaborated supertrait premises (record 1).
    In census mode, a lookahead selection over a partial validated impl set
    is reported `blocked_by` the crate record, option (i) (record 6).
  - **Open:** option A, the general body relation of rigid projections; a
    type-parameter GAT member; the named open risk that region substitution
    cannot make a rigid projection normalizable.

- **During Lane C's [named constants](parallel/named-constants.md) (5b-1)**,
  2026-11-09, developed on `28e2b19fb`, qualified on `306b82377` and rebased
  onto Lane D's A (`f62db7da9`).
  - **Wire shape.** Declaration tag 13 `Constant`; one header row per
    constant after the enum columns, `ty | value` with K's term in place (2a);
    value code 17 `(17, [Declaration(const)])`. Existing packets keep their
    bytes, and the unnamed-constant packet equals its twin's (2b).
  - **Scope, accepted before code.** Primitive and unit values, at module
    scope and inside function bodies (2d):
    - aggregate, reference and pointer values keep their texts (2c);
    - associated constants, code 18 and tag-1 values are 5b-2 (2e);
    - statics keep the declaration-kind text.
  - **Sites the notice did not list**, found by candidate runs 1 and 2:
    module bindings admit a constant in the value namespace, and the lazy
    ordinary graph demands each constant's header type.
  - **Amended before their runs:**
    - two forged-row declarations the frozen inputs could not reach as
      worded (amendment 4);
    - `const-call`, a failed prediction at an existing client fence
      (amendment 5);
    - the forged rows' file, renamed for the architecture audit's
      filename-stem limit (amendment 6);
    - the type-parameter parent, which the existing lexical forest rejects
      first, a failed prediction kept with its text (amendment 7). The
      free-owner text's witness is a generated unit struct parent, with
      `INNER` renumbered as its next lexical child (amendment 8).
  - **Survey.** Against the base pin, only the new packet differs.
  - **Recorded:**
    - "implementation validation has no source scheduling context": a client's
      const evaluation of an imported const fn fails whenever the provider
      declares primitive Copy impls, with or without constants. Core declares
      exactly those, so the fence blocks alloc and std clients that
      const-evaluate anything imported from core;
    - a provider body reading another crate's constant waits for the
      multi-packet item;
    - order after 5b-1 (integrator, 2026-11-09): the scheduling-context fence,
      starting with a read-only diagnosis and notice, then R2, then 5b-2 and
      5b-3. R2's next `value_rows.rs` arm needs a real factoring of `emit()`,
      which is at Clippy's line limit.

- **During Lane A's [census batch 18](parallel/census-batch18.md)**,
  2026-11-09, started on main `044455065` and qualified and censused on
  `7e8c5cbbf`, under the approved notice `45dad0be` (D1 to D5).
  - **Part 1, option (i).** A census lookahead never reads a partial
    validated impl set as complete. After unfinished impl validation, a
    `Validated` selection whose goal still has unvalidated owners fails with
    "trait selection requires the completed immutable impl epoch". The
    census records it `blocked`, `blocked_by` the crate record, and counts
    it: the single exception to "a lookahead changes no count", matched by
    the exact `PARTIAL_EPOCH` constant (D2). The guard is in
    `implementation_candidates` only, and the `Registry` API is unchanged
    (D3). Lane B read-reviewed it. Ordinary mode cannot change.
  - **P1** (analysis-only, retired) is census 20 with record 6 alone
    changed to the `blocked` partial record and declarations `blocked 1`,
    as previewed.
  - **Part 2, the subject-shared rule (D4).** An elaborated supertrait
    premise inherits a certified declared bound's certificate when both
    have the same binder shape and interned subject and its trait arguments
    carry no projection. The scan is O(1) per premise on interned
    properties, paid before it runs, and fails closed.
    `expanded_trait_bounds` is untouched. Nine PA21 controls were declared
    before the rustc 1.98.0 oracle; all statuses agree. Five flip to accept
    as declared, and `elaborated-projection-argument` stays stricter than
    rustc by name.
  - **Census 21** (pin `30d6eb8a`): 42,310 declarations, none failed and
    one blocked. The `:595` header passes in the Split/RSplit expansion,
    and record 1 moves into the same impl, to `fn next_back`
    (`str/iter.rs:600`): "associated type requires an unambiguous validated
    implementation". Declared probes show it needs the `Iterator` impl's
    requirement on the rigid GAT projection while `Self::Item` normalizes.
    A written premise fails the same way, on census 20's pin too, so it
    predates this batch. Record 6 is the `blocked` partial record. Records
    2 to 5, 7 and 8 are unchanged.
  - **Batch 19's lead:** census 21's record 1, the projection path's
    selection of an impl whose where clause is on a rigid GAT projection.
  - **Open:** projection-bearing supertrait arguments,
    binder-changing supertraits and other subjects stay uncertified (D4).

- **During Lane C's [scheduling-context fence](parallel/scheduling-context.md)**,
  2026-11-09, developed on `1aa288ad1`. A read-only diagnosis, notice,
  pre-addendum and addendum came before code; the integrator approved
  amendments A1-A4 and rulings F1, F2 and N1.
  - **Rule.** A source client with no source core, outside selected mode,
    validates a source-free imported header on an executable demand before
    the eager walk, through the walk's own conformance. The walk then skips
    only that header's conformance and still checks every overlap pair first
    (A3).
  - **Rulings:**
    - selected mode keeps the base path and text, because it has no constant
      executor (N1, option γ);
    - a client source header keeps a named fence, a known rejects-valid (A4);
    - an imported endpoint of the client's specialization relations stops by
      name, a gate no real input reaches, with a forged private test (F2).
  - **Amended before its run:** `generic-usize`, a failed prediction at the
    existing cleanup fence "defining cleanup requires its actual language
    declaration". The blanket provider was split so that the overlap control
    reaches the pair check (amendment 1).
  - **Survey.** Against the base pin, only named-constants `const-call`
    flips, which its amendment 9 records.
  - **Recorded:**
    - the cleanup fence: the branch provider's 57 lines declare no Destruct
      language item, and core does;
    - the efficiency item, with its untraced hypothesis that an imported
      body's Copy goal reads the coarse `heads` bucket;
    - Lane C's next item is R2, with `emit()` factored first.

- **During Lane A's [census batch 19](parallel/census-batch19.md)**,
  2026-11-09, on main `4ac1361c3`, under the approved notice `8bd84f0a`
  (D1 to D5).
  - **D0** (`cfg(test)` rows only, reverted) showed one signature, H2. In
    the projection query the condition's premise is uncertified, so the
    exact compare defers. The environment is the member-conformance
    contract's, which `mapping.rs` forms and nothing certified, so option
    A's carry through the opening would change nothing.
  - **A′** (approved with six conditions). A member's conformance contract
    certifies its copies of the owner's certified premises. The gate is
    structural: the trait method contributes no clause of its own after
    elaboration, and a type parameter's implicit `Sized` is its parameter
    kind (`sema/declarations/generics.rs:231`). Every contract trait clause
    must be one of the owner's own by canonical value, and only the
    owner's certified ones are certified. The carry is 3N + 1 canonical
    steps, and the opening's summary cache is untouched. Thirteen PA21
    controls were declared before the oracle; all statuses agree, with
    seven declared flips, and `cond-method-bound` stays stricter than
    rustc by name.
  - **Census 22** (pin `7df1f865`): 42,310 declarations, none failed or
    blocked, one record. `validate_trait_implementations` passes for all
    of core, and batch 18's 3.7 claim holds for all four expansions. The
    crate record is `local_annotations` at `ptr/metadata.rs:251`,
    `<*const VTable>::cmp(..)`: "no applicable trait method for this
    prefix". Census 21's records 2 to 8 are gone: every pass after the
    record still runs as a lookahead, now over a complete impl epoch, and
    passes. Census 22 does not cover the local annotations after the record
    or the bodies phase.
  - **Batch 20's lead:** census 22's record 1. Declared probes reduce it to
    a call of a const trait's method whose supertraits are `[const]`
    bounds, through a non-const impl. It fails in both the type-qualified
    and the method-call form, and rustc accepts both. It is not an
    impl-condition failure in the projection path, so option C's trigger
    does not fire.
  - **Open:** option C stays the recorded item. Its trigger also covers
    raw or imported premises, batch 18's uncertified shapes and methods
    with their own bounds.

- **During Lane C's [calls through fn pointers](parallel/pointer-calls.md)
  (R2)**, 2026-11-09, rebased onto `645df0a92`. Checkpoint 1 factors
  `emit()` byte-identically; checkpoint 2 is the item. The integrator
  approved the notice, rulings 1-10, the option-(a) scope ruling and Q3's
  Branch A, and Lane D reviewed the codec.
  - **Wire shape.** Value code 30's second shape, `(30, [Type(signature),
    Value(callee), Range(arguments), Flag(safety)])`, read as
    `PointerCall`. The read matches the whole operand slice, the flag is the
    request codec's, and a pointer call owns no request row.
    - Each consumer re-proves the signature against the callee's pointer
      type node by node, and the safety with direct Call's matrix, both at
      validation and over the rebuilt body. A safe callee records Safe, in an
      unsafe block or not.
    - Existing packets keep their bytes.
  - **Scope (option a).** R2 is body-scoped. A pointer stored in a record
    variant or a struct field keeps its pre-existing header fence; both are
    generated opponents and known rejects-valid.
  - **Q3, Branch A.** All four steps passed, so the constructor fence is
    retired: each consumer defines its own constructor instance.
  - **Amended before their runs:** R1's `indirect` (amendment 1, observed
    first by fc2, which is not evidence) and its three constructor
    opponents (amendment 2) now export.
  - **Survey.** Against base2, among existing inputs only those four
    change; R2's own new inputs differ as declared.
  - **Recorded:**
    - the fmt chain's blockers ahead of HR/C: record variants, and nominal
      callable fields. core's `ArgumentType` needs both, and R2 alone does
      not unblock it;
    - the associated unsafe call fence widens for direct and pointer calls
      together, as its own item before HR/C;
    - rustc's const-fn diagnostic is unnumbered, a failed prediction of the
      declared E0015;
    - forged row 9 stopped at the formation check before the re-proof, a
      failed prediction; a consistent forgery with its formation input
      reaches the re-proof (outcomes amendment 1).

- **During Lane A's [census batch 20](parallel/census-batch20.md)**,
  2026-11-09, on main `4df942467`, under the approved notice `2db17f41` and
  its addendum `79dc69ab`.
  - **D0** (code reading and eight declared probe rounds): method
    applicability pushed a `[const]` supertrait as a conditional goal, which
    an ordinary impl or an ordinary premise cannot prove, so lookup dropped
    the candidate in both the type-qualified and the method-call form.
  - **Fix** (option A, gated by grep plus comment): method lookup ignores
    constness, as rustc 1.98's probe does. `declaration_conditions` maps a
    `[const]` condition to its ordinary bound; the chosen callable still
    proves its complete clause under the use's effect. **A-dedupe**
    (amendment 2, after the declared work invariant failed in `fc-1`): of a
    `[const]` condition and its equal elaborated ordinary bound, lookup
    keeps the first, at most P·M canonical steps. A `[const]` hierarchy then
    pays exactly its plain twin's probes and steps, and with no `[const]`
    condition all 26 work counters equal the base's.
  - **Controls:** 22 declared PA31 controls agree with rustc 1.98.0, with
    four flips to accept and two E0034 accepts-invalid repairs. The flip
    survey over 3,299 committed `[const]` inputs changed none of them.
  - **Census 23** (pin `93fb6b89`): 42,310 declarations, none failed or
    blocked, one record. Record 1 at `ptr/metadata.rs:251` is gone. The
    crate record is `local_annotations` at `array/mod.rs:643`, in
    `<[T; N]>::try_map`: `drain::Drain::new(&mut me, &mut f)` reports
    "quantified clause requires a scoped consumer". It is batch 21's lead.
  - **Tracked:** the std-mode E0034 accepts-invalid, the std-mode raw
    pointer comparison rejects-valid, and b1's `[const]` placement
    accepts-invalid (library-build-blockers.md).

- **During Lane C's [zero-slot nominal callable fields](parallel/callable-fields.md)
  ((ii))**, 2026-11-09, from `c8c2a8eda`. The integrator approved the notice,
  its request_ports trace and two deviations: the constructor-payload arm and
  R2's associated shape.
  - **Rule, no wire change.** Version 13 admits a nominal-owned callable-type
    binder with no parent and no slots (gate a). The port plan forms a field
    in its nominal's header frame and a called constructor's payload in its
    own nominal's, with the exact owner (gate b). A call signature's frame
    must belong to a function.
  - **Amended before their runs:** R2's `field` opponent now exports
    (pointer-call amendment 2); the `apply` and `pair` clients avoid a rustc
    ICE on an i32 match (amendment 1); the scaling control's client bound
    compares against `u8` twins (amendment 2, a failed prediction).
  - **Survey.** Against the base pin, among existing inputs only `field`
    changes, and the 58 compared packets are byte-identical.
  - **Recorded:**
    - the HR item must cover nominal-owned callable frames with bound slots,
      not just pointer-call frames;
    - an existing header-mapping cost, not scheduled: an empty client's
      header work over functions on distinct nominals adds about 2.1 times
      its previous increment per doubling, close to linear. It is the same
      for `u8` twins, before (ii) and without fn fields
      (library-build-blockers.md has the numbers);
    - a pre-existing exporter fence: a field's fn type never prepares the
      canonical type arena, so the source-nominals provider plus a fn field
      rejects with "source anonymous callable requires its actual prepared
      type arena", on the base pin too;
    - the associated unsafe widening covers unsafe associated declarations,
      as Lane D found in F.

- **During Lane A's [census batch 21](parallel/census-batch21.md)**,
  2026-11-09, on main `0bcba9e3f`, under the approved notice `ab59bd13`.
  - **D0** (code reading and four declared probe rounds): callable sugar
    always opens a binder, so `F: FnMut(T) -> U` is published as zero-region
    quantified clauses. Method applicability read every quantified clause
    through `Clause::ordinary`, which rejects it, on the path core's
    `ZipImpl::new` forces for `Drain::new`.
  - **Fix A:** lookup reads an empty binder, one that quantifies no lifetime
    and captures nothing outside it, as its body. **D1** (after the declared
    twin invariant failed in `fc-1`): elaboration lifts a sugar bound's
    `MetaSized` and `PointeeSized` into its binder, so a condition read this
    way that equals a flat clause of the same scope is skipped. The
    lookup-local reader pays P + M canonical steps per lookup that reads an
    empty binder and nothing otherwise. A sugar bound's lookup then has
    exactly its flat twin's conditions and method selection work, and with no
    empty binder every counter equals the base's.
  - **Controls:** 22 declared PA31 controls agree with rustc 1.98.0, with
    nine flips to accept and six bound-region known rejects-valid. The flip
    survey over 1,086 committed sugar and `for<` inputs changed none of them.
  - **Census 24** (pin `88273b11`): 42,310 declarations, none failed or
    blocked, one record. Record 1 at `array/mod.rs:643` is gone. The crate
    record is `local_annotations` at `iter/traits/iterator.rs:698`, in
    `Iterator::intersperse`: `Intersperse::new(self, separator)` reports
    "candidate properties require associated type normalization".
  - **Tracked:** f01's higher-ranked accepts-invalid (batch 22's lead-in,
    before A+O for the bound-region shapes), f03a's `[const]` sugar
    supertrait rejects-valid, the inherent-impl `[const]` bound
    accepts-invalid, `methods/overlap.rs:111` and D3
    (library-build-blockers.md, census-batch21.md).

- **During Lane C's [unsafe associated functions](parallel/guarded-methods.md)**,
  2026-11-09, from `a6be80084`. The integrator approved the notice, its
  two-file source footprint and the movers M1 to M4 by name.
  - **Rule, no wire change.** Under version 13 an associated function may
    be `unsafe` (gate A); its original and public safety agree; an impl
    member's safety is its trait member's; and calls inside or to it
    follow direct Call's matrix (gate B). Constness, convention, variadic
    and default-body fences stay.
  - **Amended before their runs:** const-methods' unsafe control, F's
    `Lane::get` (amendment 6, with `inherent.rs` returned), R2's and
    (ii)'s associated opponents (amendment 3 each) and M1's client, whose
    declared match ICEd rustc (amendment 1).
  - **Survey.** Against the base pin, among existing inputs only M1 to M4
    change, and the 78 compared packets are byte-identical.
  - **Recorded:** `SliceIndex` also needs const trait impls; primitive-impl
    unsafe methods are untested and not claimed.

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
