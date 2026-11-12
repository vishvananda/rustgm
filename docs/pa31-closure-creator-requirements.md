# Closure creator requirements

With the [array length pattern change](pa31-array-length-patterns.md), core's
`is_sorted_by` 8380 (`slice/mod.rs:4782`) types:

```rust
self.array_windows().all(|[a, b]| compare(a, b))
```

Prepared lowering still rejected it with "required lifetime does not outlive
its target". The same failure occurs:

- with a known length (`core-form-known`);
- without any array pattern, for `take1(x, |a| compare(a, a))`
  (`captured-call`), on the base and on the earlier 22902d20.

rustc accepts every form.

## Diagnosis

`all`'s `F: FnMut(&'a [T; N])` instantiates `'a` with a creator inference
region `'?1`. The closure body passes `a: &'?1 T` to the captured
`compare: F`, whose bound is `F: FnMut(&'a T, &'a T)` for the creator's
universal `'a`. The body therefore needs two things:

- `'?1: 'a`, a relation between two regions it imports from its creator;
- `T: 'a`, from the formation of `&'a T`.

rustc's borrow checker makes every free region of a closure signature external
to the closure body. A requirement it cannot prove there becomes a closure
requirement for the creator. That includes a type-outlives subject such as
`T: 'a`, and the creator proves it with its own implied bounds.

RustGM linked a closure to its creator only for output ports, captured
formation and captured input conditions. Neither imported signature region was
a bridge, so the body proved `'?1: 'a` alone and failed. The closure also never
saw its creator's implied bounds. `x: &'a T` implies `T: 'a` in `h`, but not in
the closure. An explicit `T: 'a` (`declared-outlives`) and a concrete `u8`
(`concrete-u8`) needed only the first fix.

The debug pins `wipdbg` and `wipdbg2`, never committed, located both causes.
The closure's port observation had already recognized the imported input. The
borrow output had bridges, and the unsatisfied pair had a type parameter's
region as the longer region.

## Change

- **Imported signature regions.** A closure whose signature inputs mention
  regions imported from its creator links to the creator. It bridges every
  universal its creator owns. A requirement between them that the body cannot
  prove becomes a creator obligation, through the existing output proof. That
  proof already served output ports and captured inputs.
- **Creator implied bounds.** A closure contract assumes the lifetime
  requirements implied by its creator's signature: region pairs and parameter
  outlives. It already inherited its creator's explicit bounds and predicates.
  The closure's leading slots are its creator's universals in order, as those
  inherited bounds require, so the bounds apply only when the closure inherits
  every creator universal. Each creator's requirements are computed once.
- **Annotated elided inputs.** This also fixes a regression from the
  [closure input region change](pa31-closure-input-regions.md) (e3af1168c). An
  elided lifetime in a closure parameter annotation, as in `|x: &T|` or
  `|pair: (&'a T, &u8)|`, belongs to the closure's own binder: rustc makes it
  late-bound. The input port rule had made it a creator port, so instantiating
  a generic creator failed. The affected tests are
  `source_closures::specializations::generic_closure_programs_have_checked_specializations`
  (`inherited-and-elided`, `borrowed-copy`) and
  `inherited_lifetime_frames_keep_late_closure_inputs_distinct`. Annotated
  regions are no longer ports.

No new engine, backend, storage policy or compiler limit is added.

Tradeoffs:

- rustc propagates a type-outlives requirement to the creator. RustGM assumes
  the creator's implied bounds in the closure instead. The creator proves those
  bounds for every call, so both accept the same programs. A requirement the
  creator cannot prove is reported in the closure body instead of at the
  creator (`static-type`, E0310).
- Implied projection bounds of the creator are not assumed, so a closure still
  proves them itself. An async creator's body has its own signature and
  contributes none.
- Region requirements on imported inputs go to the creator, as in rustc
  (`static-region`, E0521).
- The recorded generic instantiation gap for unannotated closures with private
  input ports remains. `obj/parallel/closure-infer/native/generic-match` still
  fails with "closure signature lifetime interface differs from its body".

## Contract

**PA31** (`pa31/tests/spec/closure-creator-requirements/`, eleven source-core
reductions). All declarations agree with rustc 1.98.0 (`outcomes.json`). Base
pin wip1, the array length change, rejects every accept in prepared lowering
except `captured-call-fn` and `declared-static`.

| Case | Form | Rust | RustGM |
| --- | --- | --- | --- |
| `captured-call` | `take1(x, \|a\| compare(a, a))` | accept | accept |
| `captured-call-fn` | a function item in place of `compare` | accept | accept |
| `tuple-pattern` | `take2(w, \|(a, b)\| compare(a, b))` | accept | accept |
| `declared-outlives` | `captured-call` with `T: 'a` | accept | accept |
| `concrete-u8` | `captured-call` with `u8` | accept | accept |
| `array-pattern` | `take(w, \|[a, b]\| compare(a, b))` | accept | accept |
| `nested-closures` | the call through two closures | accept | accept |
| `declared-static` | `needs::<U: 'static>` with `T: 'static` | accept | accept |
| `static-region` | `keep<'x: 'static>` on `&'a u8` | E0521 | lifetime proof |
| `static-type` | `needs::<U: 'static>` without `T: 'static` | E0310 | lifetime proof |
| `where-unimplied` | `F: FnMut(&'a T, &'a T)` with `x: &'b T` | E0309 | typing |

`where-unimplied` was declared to test a creator that cannot prove a closure's
region requirement. rustc and RustGM both reject its where clause first.

## Qualification

Five focused receipts pass. The first covers assignment metadata, formatting,
the architecture audit, strict Clippy for the closures and inference suites and
the CLI. All five use exact selections:

- 1794521272341714566 (141.6s), nine tests:
  - the two new tests in `dev/tests/creator_requirements.rs`. The first runs
    the packet's typing through the CLI and prepared lowering for every case
    that types, and requires the exact reason for each rejection. The second
    checks bounded work and lowers every width.
  - `array_length_patterns::array_patterns_fix_inferred_lengths_like_rustc`,
    which now requires core's form to lower;
  - `unannotated_closure_inputs::closure_input_regions_match_rustc`;
  - the `closure_lifetimes`, `polymorphic_outputs` and `captured_shapes`
    modules.
- 1794521443994023443 (81.5s): the `source_closures` and `nested_evaluation`
  modules, 70 tests. They include the two specialization tests that e3af1168c
  broke.
- 1794521537535739553 (60.5s): fifteen further closures-suite modules, 51
  tests.
- 1794521622366428261 (62.3s): the `environment_execution` module, 37 tests.
- Native receipt 1794521684751603032 (71.6s) runs four ignored tests with the
  CPPGM backend against rustc:
  - the new test's four ordinary-mode programs: a captured `FnMut` call, core's
    windows form, nested closures and two counted calls;
  - the array length and unannotated closure native tests;
  - `closure_lifetimes::contextual_closures_execute_their_proved_signatures`.

Earlier attempts:

- 1794519584215422363: Clippy rejected a `contains_key`/`insert` pair and a
  complex return type.
- 1794519719725130926: it exhausted its budget under load 140. It found the two
  `source_closures::specializations` failures, which pins dbeadba6 and wip1
  reproduce and 22902d20 does not.
- The first attempts at the last two receipts timed out at resource admission
  while a bounded-work run held the lease.

`make test-upstream` is not rerun: no upstream fixture changed. No graded case
or reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/array-pattern/scaling2-raw.txt`), for 8/32/128
creators, each passing `|a| compare(a, a)` to `take1`:

| Calls | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 40858 | 8165 | 512 |
| 32 | 75706 | 14549 | 920 |
| 128 | 215098 | 40649 | 2552 |

That is 1,452 inference steps, 271 canonical steps and 17 trait selections per
creator, linear across widths. Each creator's implied bounds are computed once
for all of its closures.

## Core progress

Exploratory pin wip3 (593be571, `obj/parallel/array-pattern/wip3-dev-fast`)
carries this change with the array length change. No core observation of it
has run. Core's census checks typing, so this lowering change matters for the
later core build. The annotation fix restores the instantiation behavior from
before e3af1168c.
