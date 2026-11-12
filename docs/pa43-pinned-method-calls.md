# Pinned receiver method calls

Pinned core's `SyncView::poll` 6928 (`sync/sync_view.rs:296`) and `resume` 6932
(`sync/sync_view.rs:309`) failed with "no applicable inherent method for this
receiver". Both call `self.as_pin_mut()` with `self: Pin<&mut SyncView<T>>`.
That method belongs to `impl<T: ?Sized> SyncView<T>` and is declared with
`self: Pin<&mut Self>`.

## Diagnosis

`obj/parallel/sync-view/core/declared.json` records three declared source-core
batches. Each uses the prefix of `pa31/tests/spec/source-div/foundation.rs`
through its `Deref` trait, then a reduced `Pin` with core's `LegacyReceiver` and
`Deref` impls, and `SyncView`.

Batch 1 shows that every method-call form fails, generic or concrete, `?Sized`
or `Sized`. The path call `SyncView::as_pin_mut(p)` passes.

Without the receiver features, rustc assembles inherent members from each
autoderef step of the receiver and then matches each member's declared receiver
against each step, starting with the outer one. `as_pin_mut` comes from the
`SyncView` step and matches the outer `Pin<&mut SyncView<T>>` by value. RustGM
found inherent owners only on the receiver's own head (`Pin`). Its implicit
Deref search then moved to `SyncView`, where a `Pin<&mut Self>` receiver
cannot apply. The existing owner chain, which follows `Receiver` targets and
matches against the outer receiver, ran only under `arbitrary_self_types`.

Two exploratory pins show the steps:

- wip1 (e8653b43) let that chain run at the legacy level. Discovery succeeded,
  but every pinned call then failed with "missing lifetime argument". A
  temporary instrumented build (`wipdbg1`) located it: the chain's
  `LegacyReceiver` proof substituted the receiver into `impl<Ptr:
  LegacyReceiver> LegacyReceiver for Pin<Ptr>`, and the body's still-bound
  lifetime in `&mut` read as the impl's own.
- wip2 (697c5c17) opens those lifetimes first and agrees 12/12.

## Change

- **Legacy owner discovery.** At the legacy level, the existing receiver owner
  chain (`target_chain::collect`) now runs for source-mode method names that
  some inherent method declares with a validated smart receiver, such as
  `self: Pin<&mut Self>`. Each step requires genuine `LegacyReceiver` evidence
  and follows the actual `Deref::Target` (`arbitrary_self::next`), as
  declaration validity does. Found members are matched against the original
  outer receiver. Every other method name skips the chain.
- **Opened lifetimes.** The chain's proof opens the caller's bound lifetimes in
  the receiver into fresh regions beyond the body frontier
  (`Bound::open_regions`). The feature chains had the same latent defect for a
  receiver with a bound lifetime inside a smart pointer. Discovery keeps no
  region evidence, adjustment or loan; the selected call still proves its full
  signature.

The name set (`Methods::smart_names`) holds only names, filled when an inherent
method's receiver is validated. No new engine, backend, storage policy or
compiler limit is added.

Tradeoffs:

- A same-named method on another step makes the outer step ambiguous, which is
  what rustc reports (E0034, `two-owners`). Before this change RustGM picked
  `Pin`'s method and then failed on the result type.
- Trait methods with smart receivers keep their existing trait lookup. The
  reductions' trait `poll` calls on `Pin<&mut T>` already resolve that way.
- Ordinary mode has no source receiver catalog and keeps its existing
  behavior.
- The contract is type-level and prepared lowering. These library reductions
  have no `main`, so no native run is claimed.

## Contract

**PA43** (`pa43/tests/spec/pinned-method-calls/`, twelve source-core
reductions). They agree with rustc 1.98.0 12/12 (`outcomes.json`).

| Case | Form | Rust | a202701f |
| --- | --- | --- | --- |
| `core-poll` | core's `self.as_pin_mut().poll()` | accept | reject |
| `core-resume` | core's `G::resume(self.as_pin_mut(), arg)` | accept | reject |
| `sized-poll` | the same with a `Sized` impl | accept | reject |
| `free-generic` | `p.as_pin_mut()` in a free function | accept | reject |
| `free-concrete` | the same on `Pin<&mut SyncView<u8>>` | accept | reject |
| `pin-ref` | a `Pin<&Self>` method on `Pin<&SyncView<T>>` | accept | reject |
| `shared-pin-named` | a `&self` method through `Pin`'s `Deref`, a control | accept | accept |
| `path-call` | `SyncView::as_pin_mut(p)`, a control | accept | accept |
| `mut-receiver` | a `&mut self` method, a control | accept | accept |
| `missing` | no such method | E0599 | reject |
| `unpinned` | the pinned method on `&mut SyncView<T>` | E0599 | reject |
| `two-owners` | a same-named by-value method on `Pin` | E0034 | reject |

## Qualification

Focused receipt 1794504262206508206 passes in 136.9s. It covers assignment
metadata, formatting, the architecture audit, strict Clippy for the frontend
and inference suites and the CLI, plus nine exact tests:

- the two new tests (`dev/tests/smart_receivers/pinned_calls.rs`): the packet
  through the CLI with prepared lowering of every positive, and bounded work;
- PA43's pin receiver packet and its native-method lowering;
- two feature-level receiver chain contracts (`arbitrary_methods.rs`);
- implicit Deref method priority and chain-depth work
  (`deref_contracts/autocalls.rs`);
- the negative receiver packet.

`make test-upstream` verifies the pinned upstream fixtures. No graded case or
reference is added, and no planned assignment is activated.

Bounded work (`obj/parallel/sync-view/scaling-raw.txt`), for 8/32/128 free
functions calling `p.as_pin_mut()` on `Pin<&mut SyncView<T>>`:

| Calls | Inference | Canonical | Selections |
| --- | --- | --- | --- |
| 8 | 32139 | 10597 | 397 |
| 32 | 38499 | 22765 | 445 |
| 128 | 63939 | 71437 | 637 |

That is 265 inference steps, 507 canonical steps and two trait selections per
call over the prefix's fixed cost. Adding 64 unrelated nominals, each with its
own uncalled `Pin<&mut Self>` method, adds a constant 320 inference and 6,132
canonical steps at every width, and no trait selection.

## Core progress

**Exploratory observation.** Pin `obj/parallel/sync-view/wip2-dev-fast`
(697c5c17) carries this checkpoint's compiler tree before formatting. The
unchanged 120s observer ends at its deadline after 121.111s, with 7 partial
records and 13,875 attempts, under external load of 18–23. The last item is
8127, slice/iter.rs:2414. Against the
[parameterized storage](pa31-parameterized-storage.md) observation (a202701f,
11 records, 14,180 attempts, last item 8635) it clears `poll` 6928 and `resume`
6932. No new failure appears among previously passing reached bodies.

The first remaining failure is `small_sort_network` 7571 at
slice/sort/shared/smallsort.rs:355 ("can't compare `*const T` with `*mut T`").

Partial counts make no throughput claim, and this is neither a complete census
nor a core artifact.
