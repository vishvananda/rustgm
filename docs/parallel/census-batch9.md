# Census batch 9: const-bound projections, the lookahead and the tenth census

Census 9 ([batch 8](census-batch8.md), pin `10c45c62`) checked 42,310
declarations of original core with none failed or blocked. Its one crate
record was "invalid canonical trait selection" at `option.rs:1386`
(`Option::as_deref`). This batch, on main `85dbaaff0`:

- lets a `const` or `[const]` trait premise form its parameter's projections,
  because rustc 1.98 elaborates it to the ordinary trait bound. This repairs
  `option.rs:1386` (section 1);
- adds an analysis-only lookahead: after a crate-wide declaration pass fails,
  the census still runs every later pass and records each one's first error,
  marked as following the failure (section 2);
- demands no static layout for a declaration ending in a trait object. This
  repairs the lookahead record at `error.rs:588` (section 3);
- runs census 10 (section 4).

Census 10 checks 42,310 declarations, with none failed or blocked. Its crate
record is "overlapping implementation headers have no specialization ordering"
at `ptr/mod.rs:2592` (`impl<F: FnPtr> PartialEq for F`), in pass
`validate_trait_implementations`. That is coherence, and the owner guess is
Lane B. Eight lookahead records follow it. Bodies are not entered, so
Milestone 2 ("original core passes declaration analysis") is not reached.

Out of scope:

- Lane B owns selection and normalization outside the `option.rs:1386` rule,
  re-exports, crate loading and the nightly repairs.
- Lane C owns body and item transport.
- LIB-061 (`implicit_provenance_casts`) stays deferred.
- The CPPGM backend is frozen.
- `!null` pattern types over integer or reference bases stay rejected by name.
- Ordinary crates keep batch 7's declaration-time lifetime contract gap.

## Diagnosis method

Each diagnostic was an analysis-only pin of the working tree with one census of
unchanged original core inputs, in `obj/parallel/census-batch9/diagnostics/dN/`.
Their binaries are retired.

| Run | Base | Checked | Failed | Blocked | End |
|---|---|---|---|---|---|
| Census 9 (`10c45c62`) | `97110f230` | 42,310 | 0 | 0 | crate record: invalid canonical trait selection (`option.rs:1386`) |
| d1 (`4e7828d8`), sections 1 and 2 | `85dbaaff0` | 42,310 | 0 | 0 | crate record: overlapping implementation headers have no specialization ordering (`ptr/mod.rs:2592`), then 8 lookahead records, one of them trait object has no fixed layout (`error.rs:588`) |
| d2 (`c9beca14`), section 3 | `514edca67` | 42,310 | 0 | 0 | the same crate record, then 8 lookahead records; the layout record now follows `complete_discriminants` (`ascii.rs:29`) |

Small drafts reduced each record. They are in
`obj/parallel/census-batch9/repro/` and were run with analysis-only builds
(`obj/parallel/census-batch9/dev/`, binaries retired) and, for exploration
only, with rustc 1.98.0. `base0` (`998c1cae`) is main `85dbaaff0` unchanged.
`dbg1` (`ca54ef0b`) is main plus a temporary backtrace print at the failing
region lookup, never committed.

## 1. A `[const]` premise forms its parameter's projections

Census 9's record is core's

```rust
pub const fn as_deref(&self) -> Option<&T::Target>
where
    T: [const] Deref,
```

Batch 8 (section 2) traced it to "selected type retains an unmapped lifetime"
in nominal conjunction selection. An instrumented analysis-only build of main
(`dbg1`, retired) gave the path:

- `validate_signatures` proves the formation of `Option<&'a <T as
  Deref>::Target>` (`sema/nominals.rs`, `Proof::check_inside`). The nominal
  conjunction asks the source sizing fallback whether `&'a <T as
  Deref>::Target: Sized`.
- The fallback's unscoped builtins proof walks the type's shape
  (`types/parameters/shape.rs`). A parameter's projection stays rigid only when
  `Environment::assumes_trait(T, Deref)` holds. That query looked for the exact
  ordinary premise `T: Deref`, and the environment holds `T: [const] Deref`.
- The walk then normalized the whole type in the unscoped proof
  (`sema/nominals/traits/normalization.rs`). Restoration maps only equality
  components, so the caller's free `'a` has no mapping. The fixed message hides
  this cause.

rustc 1.98 elaborates `T: [const] Tr` and `T: const Tr` to the ordinary
predicate `T: Tr` beside the effect predicate. Selection already agrees:
`alternatives::ordinary_consequence` lets an effect-qualified assumption prove
an ordinary goal. Only the shape walk's fast path did not.

The repair is in `Environment::assumes_trait`
(`types/parameters/environment.rs`). The ordinary query accepts an ordinary,
`const` or `[const]` premise for the same trait application. Its only callers
are the three shape-walk sites. `assumes_predicate`, which effect goals use,
is unchanged, so an effect goal still needs its qualified premise. The edit
changes no selection or substitution file.

The supertrait path holds too: `T: [const] DerefMut` elaborates to
`T: [const] Deref`, and `as-deref-mut` passes.

### Not repaired: a projection of a projection (Lane B)

The slow path is still reachable without any effect. This draft fails on main
(`dbg1`) and with the census 10 pin, with "invalid canonical trait selection"
and the same unmapped `'a`. rustc 1.98.0 accepts it (exploration, not a
declared control):

```rust
pub trait Holder { type Inner: Deref; }
pub fn g<'a, H: Holder>(x: &'a H) -> Option<&'a <H::Inner as Deref>::Target> {
    Option::None
}
```

`<H::Inner as Deref>::Target` has no environment premise for its subject, so
the shape walk normalizes `&'a ...` in the unscoped builtins proof
(`Search::new` uses `nominals::Proof::new`), whose restoration cannot map the
caller's free regions. This is normalization in selection, Lane B's. The draft
is `obj/parallel/census-batch9/repro/nested-item.rs`. Census 10 has no record
of it.

## 2. Lookahead after a failed crate-wide declaration pass

Batch 8 (section 4) did not add keep-going across the crate-wide declaration
passes, because they publish facts that later passes read. The approved
replacement is a labelled, analysis-only lookahead.

- The 29 passes after the per-definition loop now run from one named table
  (`sema/declarations/passes.rs`), in the same order. Ordinary checking is
  unchanged: the first error is returned.
- In a census, a pass that fails with an error that is not sticky budget
  exhaustion still ends the census. Before returning, the checker parks its
  census ledger, so each later pass runs exactly as in ordinary checking and
  records nothing, and keeps each later pass's first error.
- The terminal record's item now names its pass, for example
  ``pass `validate_trait_implementations` ``. Each kept error becomes a
  `lookahead` record after it, naming its own pass and putting the terminal
  record in `blocked_by`.
- Lookahead records change no count, so the summary and `has_failures` are
  unchanged, and bodies are never entered. A budget error ends the lookahead.
- The census file format and observer are unchanged: `lookahead` is a new
  category value.

A lookahead record may derive from the failure it follows: a later pass reads
whatever the failed pass published. Census 10 shows this. Five of its eight
lookahead records name state that the failed pass did not finish, and the
other three were not reduced (section 4).

`dev/tests/census_mode.rs` adds a crate with a duplicate inherent method
(`validate_inherent_overlaps`) and a recursive struct (`complete_layouts`). Its
census has the crate record for the first and a lookahead record for the
second, in both views; ordinary checking reports the first, and without the
duplicate it reports the second. `sema::census::tests` adds the record
invariant.

## 3. Trait object tails have no static layout

The first census 10 pin's lookahead record in `complete_layouts` was "trait
object has no fixed layout" at core's

```rust
#[repr(transparent)]
pub struct Request<'a>(Tagged<dyn Erased<'a> + 'a>);
```

(`error.rs:588`). It does not follow from the terminal record: drafts with
only a trait object tail fail the same way, with or without
`repr(transparent)` or a lifetime, and rustc 1.98.0 accepts them. A slice tail
already passed.

The layout pass computes a template for each declaration
(`abi/templates.rs`). A slice, str, parameter or projection makes the template
unknown, so no static layout is demanded; a trait object reached the concrete
layout, which has none. The repair adds `Kind::Dynamic` to the unknown arm
(one line). A trait object that is not the last field of a struct, or an enum
variant field, is still rejected earlier, with "only a struct's final field
may be unsized".

d2 confirms it: the `complete_layouts` lookahead record became "enum layout
requires completed discriminants", which follows from the failed
`complete_discriminants` pass.

## 4. Census 10

The pin `1375fd2b` (`obj/parallel/census-batch9/qualified-2/pin/rustgm`) was
built from `b8e58f062`, which is main `85dbaaff0` plus this batch's three code
commits. Nothing was uncommitted: the build's retained patch is empty, and this
record was moved out of the tree during the build.

- Its manifest is census 9's, with only `expected_cli_sha256` and
  `predecessor_manifest_sha256` changed.
- No configuration source pin changed, and nothing was rebound.
- The ordinary first error moved from `option.rs:1386` to `ptr/mod.rs:2592`:
  "overlapping implementation headers have no specialization ordering". It
  matches the terminal record.

A first pin, `5be88e5e` from `514edca67` (sections 1 and 2), produced the same
census as d1. Its layout lookahead record led to section 3, so that census is
superseded; it stays in
`obj/parallel/census-batch9/superseded/core-census-10-pin1/` (`census.tsv`
SHA-256 `66d597ffc720cc06`) and its pin is retired.

The census is analysis-only and ran over unchanged original core inputs: 375
opens in 59 s. Its records are in
`rustgm-pa29-work/obj/parallel/core-census-10/` (`census.tsv` SHA-256
`5ad6a17d3667c252`).

| Census | Checked | Failed | Blocked | End |
|---|---|---|---|---|
| Census 9 (`10c45c62`) | 42,310 | 0 | 0 | crate record: invalid canonical trait selection (`option.rs:1386`) |
| Census 10 (`1375fd2b`) | 42,310 | 0 | 0 | crate record in `validate_trait_implementations`: overlapping implementation headers have no specialization ordering (`ptr/mod.rs:2592`), then 8 lookahead records |

A crate-wide declaration pass still fails, so bodies are not entered: 0
bodies are checked, and there are no body causes and no checked-of-total body
count.

### The crate record (Lane B)

`ptr/mod.rs:2592` is core's `impl<F: FnPtr> PartialEq for F`. The record does
not name the competing impl. `FnPtr` is a builtin-only lang trait
(`#[rustc_deny_explicit_impl]`), so for a non-fundamental self type, such as a
tuple, rustc's coherence knows no downstream crate can implement it, and the
blanket impl does not overlap. Here, coherence cannot rule it out.

- Reduction (`obj/parallel/census-batch9/repro/fnptr-tuple-spec.rs`): the PA31
  `function-pointers` prefix with `min_specialization`, a `PartialEq` lang
  item, `impl<F: Pointer> PartialEq for F` and
  `impl<T: PartialEq> PartialEq for (T,)`. It fails with this exact message,
  and rustc 1.98.0 accepts it (exploration, not a declared control). Without
  `min_specialization` it fails with "overlapping trait implementation headers
  require coherence proof". With `impl PartialEq for u8` it passes.
- With core's `&A` impl instead (`fnptr-ref-spec.rs`), rustc 1.98.0 rejects
  the reduction too (E0119: "downstream crates may implement trait `Pointer`
  for type `&_`"), although it accepts core. Why core's reference impls do not
  conflict is not identified here.

This is coherence's negative reasoning through selection
(`sema/traits/implementations/overlap.rs`, `sema::coherence_policy`). Owner
guess: Lane B.

### Lookahead records

Each record follows record 1 (`blocked_by` 1).

| Record | Pass | Site | Message | Reading |
|---|---|---|---|---|
| 2 | `validate_const_parameter_types` | `mem/transmutability.rs:91`, `TransmuteFrom<Src, const ASSUME: Assume = ...>` | const parameter type must implement actual ConstParamTy | not reduced; `Assume`'s `ConstParamTy_` impl is among those left unvalidated |
| 3 | `validate_trait_impl_owners` | `num/imp/dec2flt/common.rs:20`, `impl ByteSlice for [u8]` | trait implementation requires conformance and selection semantics | not reduced; names unfinished impl conformance |
| 4 | `local_annotations` | `num/niche_types.rs:122`, `NonZeroI8Inner(i8 is ..0 \| 1..)` | specialization graph validation previously failed | derived |
| 5 | `complete_alias_normalizations` | the same site | missing completed const argument | not reduced; same site as 4 |
| 6 | `validate_objects` | `any.rs:152`, `impl fmt::Debug for dyn Any` | source receiver compatibility requires the completed immutable impl epoch | derived |
| 7 | `complete_discriminants` | `mem/alignment.rs:440`, `AlignmentEnum` | associated type requires an unambiguous validated implementation | derived |
| 8 | `complete_layouts` | `ascii.rs:29`, `pub struct EscapeDefault(EscapeIterInner<4, AlwaysEscaped>)` | enum layout requires completed discriminants | derived from 7 |
| 9 | `require_specialization_execution` | unlocated | specialization graph validation previously failed | derived |

"Derived" means the message names state that the failed pass did not finish.
Records 2, 3 and 5 were not reduced. They should be rechecked once record 1 is
repaired.

### Failures by cause

| Cause | Census 9 | Census 10 | Sites |
|---|---|---|---|
| invalid canonical trait selection | crate record | none | `option.rs:1386`, repaired (section 1) |
| overlapping implementation headers have no specialization ordering | not reached | crate record | `ptr/mod.rs:2592` (Lane B) |
| trait object has no fixed layout | not reached | none | `error.rs:588`, repaired (section 3; first pin's lookahead) |

Neither census has a per-definition failure or blocked record, so no cause has
a failed or blocked count.

### Largest blocking roots

None. The one crate record ends the census. It blocks bodies and makes every
later crate-wide pass a lookahead, which the census does not count.

## Controls

These cases were declared
(`obj/parallel/census-batch9/oracle/*.declared.json`) before the frozen rustc
1.98.0 oracle ran on their bytes and before any candidate run, and all agree.
Both are recorded in each directory's `outcomes.json`.

| Directory | Cases | Accept | Reject | rustc |
|---|---|---|---|---|
| `const-bound-projections` | 14 | 12 | 2 (E0277) | 14/14 agree |
| `dynamic-tail-layouts` | 8 | 6 | 2 (E0277) | 8/8 agree |

### Amended controls

Each amendment was declared before the next oracle run and before any
candidate run, with the earlier oracle output as its independent observation.
No signature, declaration body or declared outcome changed.

- `const-bound-projections`, once: rustc reported `requires freeze lang_item`
  for twelve controls, rejecting ten declared to be accepted. Every file gained
  `auto_traits` and the `Freeze` lang item (the PA28 source-transmute
  declaration). The amendment's reason first said eleven controls; it was
  recounted after the amended oracle ran
  (`const-bound-projections.amended-1.first-reason.*` keeps the first text).
- `dynamic-tail-layouts`, twice: rustc rejected three controls with
  `requires copy lang_item`, so `Clone` and `Copy` were added. Then all six
  positive controls failed with `requires legacy_receiver lang_item`, so
  `LegacyReceiver` and its reference impls were added. Lines 3 to 54 now equal
  the `const-bound-projections` prefix.

### Base comparison

- `const-bound-projections` on base `85dbaaff0` (`base0`): the eight new
  positive shapes fail with "invalid canonical trait selection"; the four
  guards pass; the opponents are rejected as here.
- `dynamic-tail-layouts` on the first census 10 pin (`5be88e5e`, before
  section 3): the five trait object tails fail with "trait object has no fixed
  layout"; `slice-tail` passes; the opponents are rejected as here.
- The opponents' messages here are "nominal type argument must be sized"
  (`unsized-target`), "associated type requires an unambiguous implementation"
  (`missing-bound`) and "only a struct's final field may be unsized" (both
  layout opponents).

`dev/tests/invariants/elaborated_effect_bounds.rs` (attached to
`types/parameters/environment.rs`) and
`dev/tests/invariants/trailing_objects.rs` (attached to `abi/templates.rs`)
check every case. Their filename stems,
`elaborated` and `trailing`, each have 1 file. The new module
`sema/declarations/passes.rs` has stem `passes`, 1 file.

## Checks

The final pin's sources are those of `b8e58f062`, docs aside, which are its
source fingerprint. The three code commits were checked as two trees:
`514edca67` (sections 1 and 2, the first pin) and `b8e58f062`.

| Check | Tree | Result | Receipt |
|---|---|---|---|
| Private: 23 lib cases, strict private Clippy, format, architecture | `514edca67` | pass | `obj/qualification/fast/1794108005719593244` |
| `census_mode` and PA1 to PA12 | `514edca67` | pass, 10 tests, every PA full | `obj/qualification/fast/1794108701086082401` |
| PA13 to PA19 and PA21 to PA24 | `514edca67` | pass, every PA full | `obj/qualification/fast/1794108952388266689` |
| PA20 alone | `514edca67` | pass, 1451/1451 | `obj/qualification/fast/1794109386772153903` |
| Private: 24 lib cases, strict private Clippy, format, architecture | `b8e58f062` | pass | `obj/qualification/fast/1794110772644074634` |
| `census_mode` and PA1 to PA12 | `b8e58f062` | pass, 10 tests, every PA full | `obj/qualification/fast/1794111222085360338` |
| PA13 to PA19 and PA21 to PA24 | `b8e58f062` | pass, every PA full | `obj/qualification/fast/1794111306171825506` |
| PA20 alone | `b8e58f062` | pass, 1451/1451 | `obj/qualification/fast/1794111377755547117` |
| `make test-upstream` | both | pass, 127 fixtures, 126 Reference files | `obj/parallel/census-batch9/tmp/test-upstream.log`, `test-upstream2.log` |
| rustc 1.98.0 oracle | | 22/22 agree | `obj/parallel/census-batch9/oracle/` |
| Pin build and ordinary observation | `b8e58f062` | pass | `obj/parallel/census-batch9/qualified-2/receipt-build.json` |
| Census 10 | `b8e58f062` | completed, analysis-only | `obj/parallel/census-batch9/qualified-2/receipt-census.json` |

The 24 private lib cases are:

- the two new invariants (const-bound projections, trailing objects) and the
  new census record invariant, with the three existing census record cases;
- the shape walk's normalized-shape case;
- the stability transport hot-record and derive-owner layout invariants;
- batch 8's sizing set: the sizedness hierarchy, the executed-intrinsic test,
  the hidden sized projection's lifetime debt, the two size wait cases, the
  three completed sizing proof reuse cases and the two sizing default cases;
- batch 7's five invariants.

No record changed layout: the lookahead state is one boxed field of the boxed
census ledger, and `Checker` is unchanged. `make test-harness` was not needed,
because no integration root was added.

Pending for scheduled CI: full release regular and native tests, uncached
audits, all-target strict Clippy, harness and course text.

## Storage

Only the census 10 pin (`qualified-2`) is kept. The census 9 pin
(`obj/parallel/census-batch8/qualified-1/pin`), the superseded first census 10
pin (`qualified-1`), the d1 and d2 diagnostic pins and the analysis-only builds
in `obj/parallel/census-batch9/dev/` were retired and pruned; their hashes stay
in their receipts. `obj` reached 13 GiB at the first candidate build, so
`make tidy-build` ran once, before it.

## Shared files touched

- `types/parameters/environment.rs`: `assumes_trait` and the test attachment
  (section 1). No `sema::traits` or `types::substitute` file changed.
- `sema/census.rs`, `sema/declarations.rs` and
  `sema/declarations/isolation.rs` (section 2).
- `abi/templates.rs`: one match arm and the test attachment (section 3).
- `dev/tests/census_mode.rs` and `dev/tests/invariants/census_records.rs`.
- `docs/parallel/core-census-mode.md`: the lookahead paragraph.

## Decisions needed

- Section 4: Lane B's ownership of `ptr/mod.rs:2592`, the next crate record:
  coherence must know that a builtin-only lang trait such as `FnPtr` has no
  impl for a non-fundamental self type. Core's reference impls need their own
  explanation, because rustc rejects the reference reduction.
- Section 1: Lane B's ownership of a projection of a projection inside a
  nominal argument, which census 10 does not reach.
- Whether lookahead records 2, 3 and 5 need reduction now, or after record 1 is
  repaired.
