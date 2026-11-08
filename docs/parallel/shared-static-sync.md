# Shared statics require Sync

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/static-sync-20261108`, based on pushed main `f98c5d7c8`. It fixes
a false acceptance. It makes no claim about an original-core artifact or a
whole assignment.

Main accepted a source-client `static` whose source core declares no `sync`
lang item, and rustc 1.98.0 rejects it. The const-method packet recorded the
divergence when it withdrew its `static-held` client. The integrator's
decision was: "a static requires the `sync` lang item and `T: Sync`, exactly
as rustc does".

## Rule

The rule is rustc's well-formedness check for statics. It applies to every
static that is not `mut`, not foreign and not thread-local:

- **With source bindings.** The `sync` lang item must exist. If it is absent,
  the static is rejected with rustc's own text: "requires `sync` lang_item".
  Otherwise the static's type must prove the source `Sync` trait's
  application, with the static's own declaration proof.
- **Built-in library.** The built-in library proves its structural `Sync`
  auto trait.
- **Failure.** A type that cannot be proved is rejected with "shared static
  type must implement Sync".

The proof is the ordinary auto-trait proof. Explicit positive and negative
impls replace the structural default. Without one, a source raw pointer
follows its pointee, as in rustc, so a core that lacks `impl !Sync for *const
T` has `Sync` raw pointers.

A `static mut` needs neither the item nor the proof. Foreign statics have no
initializer, so the check never sees them, and `#[thread_local]` is already
unsupported. Consts are unaffected.

The check is in `validate_signatures`
(`dev/src/sema/declarations/validation.rs`). It runs after the static's type
is proved well-formed and sized, and before the proof is retained. That file
is outside transport and needs owner review. Nothing in the wire, the
exporter or any consumer changes.

## Fixtures and outcomes

`pa29/tests/spec/static-sync/` holds one provider and four clients. The
outcomes were declared in `pa29/tests/static-sync-outcomes.json` at
2026-11-08T03:57:43Z, before any rustc or RustGM run of these files.

The provider is the shared lines of the earlier packets (lines 1-147 of
`lifetime-nominals/provider.rs`), which declare no `sync` item, plus
`answer`. The four clients behave as follows:

- `plain-static` and `local-static` are rejected with "requires `sync`
  lang_item", as rustc rejects them. Main accepted both.
- `mut-static` and `answer` exit 42.

Unsafe auto traits do not export yet, so a provider that declares `Sync`
cannot publish a packet. The positive proofs are seven single-crate source
cores, generated from the shared lines by
`crate_linking::synced_statics`:

| Control | rustc 1.98.0 | Candidate |
| --- | --- | --- |
| no `sync` item, `static N: u8` | "requires `sync` lang_item" | the same |
| `sync` item, `static N: u8` | accepted | accepted |
| `static R: Raw`, `Raw` holding `*const u8` | accepted | accepted |
| the same with `unsafe impl Sync for Raw {}` | accepted | accepted |
| the same as `static mut R` | accepted | accepted |
| `impl !Sync for Raw {}`, `static R` | E0277 | "shared static type must implement Sync" |
| the negative impl with `static mut R` | accepted | accepted |

Two controls were amended after their first rustc run and before any RustGM
run, as recorded in the outcome file:

- **The `get` name.** The first generated controls added `pub fn get() -> u8`,
  which the shared lines already define (E0428). They read `N` through
  `read_n` instead.
- **The raw-pointer control.** It was declared rejected. rustc accepts it,
  because the shared lines lack core's `impl<T: ?Sized> !Sync for *const T`
  and a raw pointer is then structurally `Sync`. The rejected `T: Sync`
  control moved to an explicit negative impl, declared at 04:02:16Z.

## pointer-bits

`pa31/tests/spec/source-scalar-patterns/pointer-bits.rs` declares a shared
`static WORD: u64` and no `sync` item. Its contract status is 1, and it stays
rejected. rustc 1.98.0's only error on it is "requires `sync` lang_item": the
missing item stops rustc before constant evaluation. So the fixture never had
a rustc witness for its intended rejection, the pointer-to-integer transmute
in `BAD`. The scalar-pattern README's claim that the qualification rejected
pointer-to-integer representations rested on status alone.

By integrator decision, the fixture keeps its exact bytes, and Lane A's
pattern-type invariant (`dev/tests/invariants/constrained_scalars.rs`) now
expects its `sync` diagnostic. A successor, `pointer-bits-sync.rs`, is
today's bytes followed by `#[lang = "sync"] pub unsafe auto trait Sync {}`.
rustc rejects it with E0080, "unable to turn pointer into integer", at the
`BAD` transmute. So it is the first rustc witness of the provenance
rejection. RustGM rejects it with "constant numeric read uses symbolic pointer
bytes", and the invariant's provenance case now uses it.

A scratch rustc run on the successor's exact bytes was observed before its
outcome was declared. Its contract entry records that exploration, then
declares status 1. The recorded oracle then ran on the committed bytes.

## Flip survey

Under the integrator's rule, any fixture that rustc accepts, or that an
accepted PA1-PA24 suite expects to pass, and that the new proof would reject,
stops this check. It is reported with its failing auto-trait goal, not
amended.

No fixture flipped. The only changed outcome in the whole survey is
`pointer-bits.rs`, which stays rejected (above).

- **Fixture survey.** The baseline CLI was the previous checkpoint's pin
  (sha `f43c79d8`, main `b55c0dc49`'s `dev/src`). The candidate CLI was
  built from this checkpoint on `f98c5d7c8`. Both ran on the same inputs
  with the same flags, and their status and first diagnostic were compared.
  The inputs were:
  - every `pa*/tests` fixture that declares a static: 760 files, PA5 to
    PA40, with 423 accepted by both;
  - every inline test source in `dev/tests` and `dev/src` that declares a
    static, with a `check` helper prepended when it calls one: 224 sources,
    with 160 accepted by both.

  Of the 984 inputs, 583 are accepted by both binaries and 401 are rejected
  by both. Every rejection but one has the same first diagnostic. The
  exception is `pointer-bits.rs`, whose diagnostic changes.
- **Maintainer roots.** 109 integration roots run in full, 621 tests. They
  are the roots that hold an inline static or load a static-bearing fixture
  directory. Six such roots hold only native tests (`llvm_calls`,
  `option_abi`, `thin_references`, `wide_references`, `dynamic_patterns`
  and `tail_coercions`), and the inline survey covers their sources.
- **Private invariants.** 58 private tests run from the invariant files that
  declare a static, with the pattern-type provenance case.
- **PA1-PA24.** Every suite passes, with PA20 at 1,451/1,451. PA25 to PA34
  are planned assignments with no text contract, and the fixture survey
  covers them.

`shape_constructors::early_type_arguments_share_declaration_dependencies`
and `repeated_shape_defaults_reuse_completed_nominal_instances` fail on main
`b55c0dc49`. The first takes 1,601,553 field instantiation steps, where its
assertion allows fewer than 32,768, and the second exceeds its bound of 64.
Both fail the same way with this check set aside, and neither source has a
static. They passed in the 2026-11-07 nightly at `054b91950`, so they are a
new regression. The integrator routed the bisect to Lane B, and they are left
out of the batch.

## Evidence

All rows ran on the final code, on main `f98c5d7c8`.

| Check | Result | Receipt (`obj/qualification/fast/`) |
| --- | --- | --- |
| `synced_statics` (2), `crate_linking::exports` (8), private static and pattern-type selection (59) | 69/69 | `1794114838343017597` |
| every other non-native `crate_linking` test | 68 and 65 | `1794115107974058921`, `1794114975130877350` |
| native `--ignored`: `synced_statics` | 1/1 | `1794114757001516158` |
| static-bearing roots: declarations; closures; constants; frontend; frontend, linking and inference | 20, 93, 65, 70 and 20 | `1794113828320578897`, `1794113871277217631`, `1794113931945571789`, `1794113988911618602`, `1794114055144280757` |
| static-bearing roots: lowering and memory | 28, 28, 27 and 29 | `1794114101150706400`, `1794114155355762444`, `1794114220358960943`, `1794114283435274123` |
| static-bearing roots: ownership, patterns, projections, regions and traits | 68 and 173 | `1794114334327851762`, `1794114476569052041` |
| PA1-PA12 | pass | `1794113447323060800` |
| PA13-PA19, PA21-PA24 | pass | `1794114619984666071` |
| PA20 | 1,451/1,451 | `1794114699133606862` |
| `make test-upstream` | 127 fixtures, 126 references | `obj/parallel/static-sync/test-upstream.log` |
| `make test-harness` | 304 script tests; 464 roots in 13 binaries, 44 stages | `obj/parallel/static-sync/test-harness.log` |

The fixture survey is in `obj/parallel/static-sync/survey4/` (`all.txt` and
`diff.txt`). The candidate CLI (sha `dcc9556c`) is pinned at
`obj/parallel/static-sync/candidate/`.
The rustc observations are:

- `obj/parallel/static-sync/oracle1.log` for the packet;
- `obj/parallel/static-sync/oracle-controls.log` for the generated controls,
  with the first E0428 runs kept in `oracle-controls/first/`;
- `obj/parallel/static-sync/pointer-bits-oracle/oracle.log` for
  `pointer-bits.rs` and its successor on the committed bytes.

The baseline run of the two shape tests is
`obj/qualification/fast/1794113147148207089`. No wire, export or consumer
code changed, so byte identity and scaling do not apply.

## Files outside transport

For owner review:

- `dev/src/sema/declarations/validation.rs`: the check;
- `dev/tests/invariants/constrained_scalars.rs` (Lane A's pattern-type
  invariant): only the pointer-bits case changed;
- `pa31/tests/spec/source-scalar-patterns/`: the successor fixture, its
  contract entry and a README paragraph.

## Remaining gaps

- **Sync transport.** A provider cannot export `Sync`, because unsafe auto
  traits do not transport. So no source client can hold a shared static
  against a packet core. Auto traits, including negative impls, are item 5 of
  the integrator's order.
- **Thread-local statics.** `#[thread_local]` statics stay unsupported. When
  they arrive they must skip this check, as rustc's does.
