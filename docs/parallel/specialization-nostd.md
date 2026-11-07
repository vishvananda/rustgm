# Specialization selection, user callable outputs and no_std source clients

Lane B batch of 2026-11-07, on top of `e421547fa`. It covers three things:

- rustc's `min_specialization` selection rule;
- a user callable-trait impl's own `Self::Output`;
- `#![no_std]` and Rust 2021 source clients, as the [census plan's
  follow-up decisions](../core-census-plan.md) require.

It clears 19 of census 5's 23 failed "associated type requires an unambiguous
implementation" sites.

## 1. Specialization selection

Rustc's winnowing drops an applicable candidate that another applicable
candidate specializes, so the most specific impl is selected without a closed
graph. `assoc_def` then reads an associated type that the selected impl omits
from its nearest defining ancestor. Before this batch, RustGM ordered
candidates only by a sealed forest. During declarations every overlap was
still unordered, so all 14 specialization sites of core's `StepBy`, `Fuse` and
`Zip` failed.

- A crate that requests `min_specialization` or `specialization` now opens its
  graph before the declaration loop (`candidate_order::open`). This is called
  from `Checker::open_specialization`, after `prepare_outlives`. Staged
  consumers never seal a graph, so they skip it.
- `candidate_order::order` orders two proven candidates for one fully known
  goal. A sealed or conforming forest orders them by rank, as before. While
  the graph is building, an existing `Comparison::Ordered` gives the child.
  Otherwise one pairwise comparison runs (`graph_pairs::settle`). It runs only
  for a `Purpose::Use` query, in a crate with the request, and when both impls
  are positive and local.
- A comparison that waits on a declaration fact returns `Order::Await`. The
  selection frame rewinds its cursor and waits, so `Frame` keeps its asserted
  1312 bytes.
- Closure (`drain`) authenticates every recorded relation. Each child needs a
  source witness with its feature authority, or it fails with "specializing
  implementation requires its actual feature authority".
- `projections::defining` reads an omitted associated type from the nearest
  defining ancestor (`candidate_order::defining_type`). It translates the
  arguments through each relation (`selected_member::translate`), from the
  forest or, before closure, from the recorded relations. Associated-type
  conformance skips a missing type that the impl inherits
  (`inherited_paths::inherited_type`).
- Under `min_specialization`, a non-root vertex that defines a type alias
  fails with "specializing implementation overrides a final parent associated
  type". Other associated values stay rejected by name.

Controls: `pa31/tests/spec/specialization-selection/`, nine cases checked by
`dev/tests/invariants/candidate_winnowing.rs`. The three positive controls are
the `StepBy`, `Fuse` (unsafe marker) and three-level `Zip` shapes. The six
opponents are:

- an unordered overlap, E0119;
- an override of a final method, E0520;
- a child with an ordinary trait bound ("cannot specialize on trait");
- an override of the parent's final type, E0520;
- an inherited type used at the wrong type, E0308;
- `default type`, which needs full specialization, E0658.

Every outcome was declared before the rustc oracle. A second test confirms
that the unordered pair without a use still fails coherence. One amendment was
recorded before oracle v2. Oracle v1 rejected the positive fixtures themselves
("requires `copy` lang_item"), so every case's prefix became the exact pinned
core fragment. Bodies and outcomes were unchanged.

## 2. User callable-trait `Self::Output`

Rustc assembles impl candidates for `FnOnce` beside its closure and fn-item
evidence. RustGM answered a callable subject only from language evidence. A
struct (core `SyncView`) had no signature, and `&mut Drain` forwarded to a
referent that is not callable, so both were "absent".

- `start_callable` treats `Mismatch` (no language signature) as no language
  candidate, and selection falls through to impl candidates. `Occurs` stays
  absent.
- `complete_forwarding` falls back to `implementation_candidates` when the
  referent stopped as absent with no operation.

Controls: `pa31/tests/spec/user-callable-output/`, four cases checked by
`dev/tests/invariants/user_callable_output.rs`, all matching rustc. The
sugar control `const-call-adapter` moved from the expected gaps to the passing
cases.

## 3. `no_std` and Rust 2021 source clients

Rustc 1.98.0 was observed with an empty sysroot. A `#![no_std]` crate without
`#![no_core]` gets an injected `extern crate core` with no `macro_use`, and a
glob of `core::prelude::rust_20xx` for its own edition. Without that module
rustc reports "cannot resolve a prelude import". Rust 2015 also names the
injected crate as a crate-root item. Rustc also injects
`extern crate compiler_builtins`.

- The source-client gate (`resolve/bindings/client.rs`) admits every edition.
  A client is either `#![no_core]` with its feature or `#![no_std]` over a
  provider named `core`. Other clients get "source client requires no_core
  with its source feature or no_std over core". A `no_std` client of any
  other provider gets "no_std source client requires its core provider".
- `resolve/standard_imports.rs` runs after `import_crates`.
  - It binds `core` in the extern scope to the provider's imported root.
    `Dependency::new` already rejects a `core` alias, so this is the only
    entry, and the provider arrives through the dependency inventory.
  - It selects the provider's public, unambiguous `prelude::rust_20xx`.
  - It records an `Injected` row (core root, prelude module, scope) in the
    cold `ItemScopes` box, paid for like a `prelude_import` claim.
- The source prelude lookups return the injected module and recheck its row
  each time. A `#[prelude_import]` beside it is a second selector and is
  rejected. The extern-prelude lookup admits `core` only as the injected
  root. The Rust 2015 crate-root rule now covers source clients that have an
  injected `core`.
- No hot record grows: `Resolution` stays 2032 bytes and `Package` 256.

Controls: `pa29/tests/no-std-client-outcomes.json` has 13 cases under
`pa29/tests/spec/no-std-clients/`. Outcomes were declared (plan
`7a4254da`) before the rustc oracle and before any RustGM run of the
fixtures; exploratory probes ran first in `obj/lane-b/nostd`.

- The provider is a no_core `core` with core's prelude shape. Only `rust_2024`
  adds `IntoFuture`.
- Native object clients in 2015, 2018, 2021 and 2024 each print `42 40` under
  both compilers. The 2015 client names `core` through a crate-relative `use`
  and `crate::core`.
- The opponents are `IntoFuture` in 2021 (E0405), `crate::core` in 2018
  (E0433), `std::` in a `no_std` crate (E0433), and a `core` without prelude
  modules ("cannot resolve a prelude import").
- `builtins.rs` is rustc-only: it stands in for `compiler_builtins`.

The harness change (`scripts/check_macro_metadata.py`) is limited:

- `rustc_sysroot: "empty"` adds `--sysroot` and `-L crate=`;
- `rustc_deps` holds rustc-only externs;
- `rustc_only` marks a case the frontend never compiles.

The other 75 cases are unchanged and publish byte-identical metadata: all 27
`.rgm` hashes match `obj/parallel/definition-lineage/macro/native-t4`.
`transported_rows` now expects 88 cases.

Amended control: `client_selection_policy` previously asserted the rejection
of 2021 clients and one message for every non-no_core header. It now asserts:

- 2021 reaches dependency admission, as 2015, 2018 and 2024 do;
- a bare `#![no_core]` keeps the old message;
- `""` and `#![feature(no_core)]` get the new message;
- a `no_std` client of `minicore` is rejected.

This follows the integrator's 2021/no_std decision; it is not a blessing.

## 4. Census

The pins were built from committed trees, with census 4's manifest, flags and
inputs.

| census | base | pin | checked | failed | blocked | associated type f/b |
|---|---|---|---|---|---|---|
| Lane B census 5 | `e84a85783` | `4cb211bb` | 42,122 | 62 | 126 | 23/25 |
| census-2 | `61c2a0cb8` | `4f1dff29` | 42,245 | 36 | 30 | 9/1 |
| census-3 | `67c01c493` | `bab534e4` | 42,250 | 31 | 30 | 4/1 |

- Census-2 cleared all 14 specialization sites: `fuse.rs` ×4, `zip.rs` ×3 and
  `step_by.rs` ×7. Every other cause equals Lane A's census 5.
- Census-3 cleared the 5 callable sites: `drain.rs` ×2 and `sync_view.rs` ×3.
  No other cause changed.
- Four associated-type sites remain:
  - `control_flow.rs:427` (a self type);
  - `map_windows.rs:245` and `scan.rs:60`, region-binding sugar;
  - `iter/mod.rs:376`, the `impl_fold_via_try_fold!` repeated-`Self::Item`
    where-clause.

The records are in `obj/parallel/specialization-nostd/census-{2,3}`, with
`compare.txt`. Census-3 is the pinned baseline. The census-2 and Lane B
census-5 pins are retired.

## 5. Diagnosed, not repaired

- **Region-binding sugar** (`scan.rs`, `map_windows.rs`). The sugar clause
  binds an elided region, so it stays quantified (`vacuous` handles only zero
  bound regions). `conditions` waits on `resolved_parent`, and `B` is never
  inferred.
- **Override conformance of a repeated `Self::Assoc` where-clause**
  (`iter/mod.rs:376`). Probes in which the impl repeats
  `G: Tr<Self::Item>` or writes `G: Tr<T>` all fail with "implementation
  method imposes a stronger generic requirement", while rustc accepts them.
  `traits/conformance/mapping.rs::contract` folds the trait's method bounds
  into assumptions without normalizing `<Impl as Trait>::Item`. The fix
  normalizes the assumption clauses through the impl's checked `Definitions`,
  as `Definitions::signature` does, and rebuilds the environment. Quantified
  clauses need their binder depth. This is `sema::traits` code, so it is left
  to the lane that owns it.
- **Method calls with inferred trait arguments on a specialized impl.** A call
  like `s.spec_next()` needs a canonical goal before candidates can be ordered,
  so it reports "trait obligation requires an unambiguous matching
  assumption". The qualified call works. No census site needs it yet.
- **Enum variant re-exports in a source-core export.** The variant re-exports
  in core's `prelude::v1` (`pub use crate::option::Option::{self, None,
  Some}`) fail module-bodies-v13 publication with "source header names an
  unreserved actual declaration". The type-namespace variant alias has no
  declaration row. The fixtures re-export `Option` and spell its variants.
  This is item transport, Lane C's area.

## 6. Checks

| check | receipt | result |
|---|---|---|
| specialization lib cases (13) | `obj/qualification/fast/1794084272984424481` | pass |
| callable lib cases (3) | `obj/qualification/fast/1794085559380758429` | pass |
| no_std lib cases (15), `legacy_editions` | `obj/qualification/fast/1794088394474150520` | pass |
| `transported_rows` macro outcomes (88) | `obj/qualification/fast/1794089000831846215` | pass |
| macro-metadata harness `--native` (88) | `obj/parallel/specialization-nostd/macro-native-1/complete.json` | pass |
| no_std rustc oracle (13, native) | `obj/parallel/specialization-nostd/no-std-oracle/oracle.json` | pass |
| PA1–PA12 | `obj/qualification/fast/1794088779027656133` | pass |
| PA13–PA19, PA21–PA24 | `obj/qualification/fast/1794088910996490681` | pass |
| PA20 | `obj/qualification/fast/1794088694471006687` | 1451/1451 |
| `make test-upstream`, `make test-harness` | `obj/parallel/specialization-nostd/test-harness-1.log` | pass |

The no_std lib cases include both resolve layout invariants. Full release,
all-target Clippy and the independent suites are still pending in scheduled
CI.
